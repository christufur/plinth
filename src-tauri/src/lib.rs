use std::{path::{Path, PathBuf}, sync::{mpsc::Sender, Mutex}};
use tauri::{AppHandle, Manager, State};
use tokio::process::Command;

mod player;

async fn ensure_binaries(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let libs_dir = base.join("libs");
    let downloads_dir = base.join("downloads");

    // Download binaries using yt-dlp crate helper if not already present
    yt_dlp::Downloader::with_new_binaries(&libs_dir, &downloads_dir)
        .await
        .map_err(|e| format!("Installing yt-dlp/ffmpeg failed: {e}"))?;

    #[cfg(target_os = "windows")]
    let binary = libs_dir.join("yt-dlp.exe");
    #[cfg(not(target_os = "windows"))]
    let binary = libs_dir.join("yt-dlp");

    Ok(binary)
}

#[tauri::command]
async fn download_audio(
    app: AppHandle,
    url: String,
) -> Result<String, String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("Not a valid URL".into());
    }

    let binary = ensure_binaries(&app).await?;
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let libs_dir = base.join("libs");
    let downloads_dir = base.join("downloads");

    std::fs::create_dir_all(&downloads_dir).map_err(|e| e.to_string())?;

    let output_template = downloads_dir.join("%(title)s.%(ext)s");

    // TODO: Figure out how to use yt_dlp without timeout issues instead of using a system call
    // to access yt_dlp directly.
    let output = Command::new(&binary)
        .arg("-x")
        .arg("--audio-format")
        .arg("m4a")
        .arg("--audio-quality")
        .arg("0")
        .arg("--ffmpeg-location")
        .arg(&libs_dir)
        .arg("-o")
        .arg(&output_template)
        .arg("--no-playlist")
        .arg("--force-ipv4")
        .arg("--print")
        .arg("after_move:filepath")
        .arg(&url)
        .output()
        .await
        .map_err(|e| format!("Failed to execute process: {e}"))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp failed: {err_msg}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let final_path = stdout.trim().to_string();

    if final_path.is_empty() {
        return Err("Download completed but no file path was returned.".into());
    }

    Ok(final_path)
}

struct Audio(Mutex<Sender<player::Cmd>>);

fn send(a: &State<Audio>, c: player::Cmd) {
    let _ = a.0.lock().unwrap().send(c);
}

const AUDIO_EXT: &[&str] = &["mp3", "flac", "wav", "ogg", "aac", "m4a"];

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| AUDIO_EXT.contains(&e.to_ascii_lowercase().as_str()))
        {
            out.push(path);
        }
    }
}

#[tauri::command]
fn scan_folder(path: String) -> Vec<String> {
    let mut out = Vec::new();
    walk(Path::new(&path), &mut out);
    out.sort();
    out.into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}

#[tauri::command]
fn play(path: String, a: State<Audio>) {
    send(&a, player::Cmd::Play(path))
}

#[tauri::command]
fn pause(a: State<Audio>) {
    send(&a, player::Cmd::Pause)
}

#[tauri::command]
fn resume(a: State<Audio>) {
    send(&a, player::Cmd::Resume)
}

#[tauri::command]
fn stop(a: State<Audio>) {
    send(&a, player::Cmd::Stop)
}

#[tauri::command]
fn seek(secs: f64, a: State<Audio>) {
    send(&a, player::Cmd::Seek(secs))
}

#[tauri::command]
fn set_volume(vol: f32, a: State<Audio>) {
    send(&a, player::Cmd::Volume(vol))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(Audio(Mutex::new(player::spawn(app.handle().clone()))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_folder,
            play,
            pause,
            resume,
            stop,
            seek,
            set_volume,
            download_audio
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_audio_recursively() {
        let dir = std::env::temp_dir().join("plinth_scan_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.MP3"), b"").unwrap();
        std::fs::write(dir.join("sub/b.flac"), b"").unwrap();
        std::fs::write(dir.join("c.txt"), b"").unwrap();
        let got = scan_folder(dir.to_string_lossy().into_owned());
        assert_eq!(got.len(), 2);
        assert!(got[0].ends_with("a.MP3"));
        assert!(got[1].ends_with("sub/b.flac"));
    }
}
