use std::path::{Path, PathBuf};

const AUDIO_EXT: &[&str] = &["mp3", "flac", "wav", "ogg", "aac", "m4a"];

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
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
    out.into_iter().map(|p| p.to_string_lossy().into_owned()).collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![scan_folder])
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
