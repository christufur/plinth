use std::{path::{Path, PathBuf}, sync::{mpsc::Sender, Mutex}};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::{io::{AsyncBufReadExt, AsyncReadExt, BufReader}, process::Command};

mod player;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Track {
    path: String,
    title: String,
    artist: Option<String>,
    album_artist: Option<String>,
    album: Option<String>,
    track_no: Option<u32>,
    disc: Option<u32>,
    year: Option<u32>,
    secs: f64,
}

fn read_track(path: &Path) -> Track {
    use lofty::{prelude::*, tag::ItemKey};
    let stem = path.file_stem().map_or(String::new(), |s| s.to_string_lossy().into_owned());
    let mut t = Track {
        path: path.to_string_lossy().into_owned(),
        title: stem,
        artist: None,
        album_artist: None,
        album: None,
        track_no: None,
        disc: None,
        year: None,
        secs: 0.0,
    };
    // Truncated/mislabeled audio fails the strict read but its tags are often intact: retry without
    // parsing audio properties (duration stays 0). Still unreadable: keep it listed by filename.
    let file = lofty::read_from_path(path).or_else(|_| -> Result<_, Box<dyn std::error::Error>> {
        Ok(lofty::probe::Probe::open(path)?
            .options(lofty::config::ParseOptions::new().read_properties(false))
            .read()?)
    });
    let mut tagged_title = false;
    if let Ok(f) = &file {
        t.secs = f.properties().duration().as_secs_f64();
    }
    if let Some(tag) = file.as_ref().ok().and_then(|f| f.primary_tag().or_else(|| f.first_tag())) {
        let own = |s: Option<std::borrow::Cow<str>>| s.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
        if let Some(title) = own(tag.title()) {
            t.title = title;
            tagged_title = true;
        }
        t.artist = own(tag.artist());
        t.album = own(tag.album());
        t.album_artist = own(tag.get_string(ItemKey::AlbumArtist).map(Into::into));
        t.track_no = tag.track();
        t.disc = tag.disk();
        t.year = tag.date().map(|d| d.year as u32);
    }
    // mixtapes often carry the order only in filenames: "01 - Song.mp3"
    if t.track_no.is_none() {
        t.track_no = filename_number(path);
    }
    // an untagged title is the filename; drop its "08 - " prefix once that became the track number
    if !tagged_title && t.track_no.is_some() {
        let rest = t.title.trim_start_matches(|c: char| c.is_ascii_digit());
        t.title = rest.trim_start_matches(|c: char| matches!(c, '-' | '.' | '_' | ' ')).to_string();
    }
    t
}

fn filename_number(path: &Path) -> Option<u32> {
    let stem = path.file_stem()?.to_str()?;
    let digits: String = stem.chars().take_while(|c| c.is_ascii_digit()).collect();
    let rest = &stem[digits.len()..];
    // require a separator so "2 Chainz.mp3" or "1999.mp3" aren't read as track numbers
    let sep = rest.starts_with(|c: char| matches!(c, '-' | '.' | '_' | ' ') ) && rest.len() > 1;
    (!digits.is_empty() && digits.len() <= 3 && sep).then(|| digits.parse().ok()).flatten()
}

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

const PROGRESS_TAG: &str = "PLINTH_PROGRESS";

/// Parses "PLINTH_PROGRESS <downloaded> <total> <estimate>" into 0..=1. "NA" fields are unknown.
fn progress_frac(line: &str) -> Option<f64> {
    let mut it = line.strip_prefix(PROGRESS_TAG)?.split_whitespace().map(|v| v.parse::<f64>().ok());
    let (done, total, est) = (it.next()??, it.next().flatten(), it.next().flatten());
    let total = total.or(est).filter(|t| *t > 0.0)?;
    Some((done / total).clamp(0.0, 1.0))
}

#[tauri::command]
async fn download_audio(
    app: AppHandle,
    url: String,
    dir: Option<String>,
) -> Result<Track, String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("Not a valid URL".into());
    }

    let binary = ensure_binaries(&app).await?;
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let libs_dir = base.join("libs");
    // save into the user's chosen library folder, else the app-data fallback
    let downloads_dir = dir.map(PathBuf::from).unwrap_or_else(|| base.join("downloads"));

    std::fs::create_dir_all(&downloads_dir).map_err(|e| e.to_string())?;

    let output_template = downloads_dir.join("%(title)s.%(ext)s");

    // TODO: Figure out how to use yt_dlp without timeout issues instead of using a system call
    // to access yt_dlp directly.
    let mut child = Command::new(&binary)
        .arg("-x")
        .arg("--audio-format")
        .arg("m4a")
        .arg("--audio-quality")
        .arg("0")
        .arg("--embed-metadata")
        .arg("--embed-thumbnail")
        .arg("--convert-thumbnails")
        .arg("jpg")
        .arg("--ffmpeg-location")
        .arg(&libs_dir)
        .arg("-o")
        .arg(&output_template)
        .arg("--no-playlist")
        .arg("--force-ipv4")
        // --print implies quiet, so progress needs asking for explicitly
        .arg("--newline")
        .arg("--progress")
        .arg("--progress-template")
        .arg(format!("download:{PROGRESS_TAG} %(progress.downloaded_bytes)s %(progress.total_bytes)s %(progress.total_bytes_estimate)s"))
        .arg("--print")
        .arg("after_move:filepath")
        .arg(&url)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute process: {e}"))?;

    // drain stderr concurrently so a full pipe can't stall yt-dlp
    let mut stderr = child.stderr.take().unwrap();
    let err_task = tauri::async_runtime::spawn(async move {
        let mut buf = String::new();
        let _ = stderr.read_to_string(&mut buf).await;
        buf
    });

    // progress lines carry the tag; the last other non-empty line is the final file path
    let mut final_path = String::new();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(f) = progress_frac(&line) {
            let _ = app.emit("download_progress", f);
        } else if !line.trim().is_empty() {
            final_path = line.trim().to_string();
        }
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;
    if !status.success() {
        let err_msg = err_task.await.unwrap_or_default();
        return Err(format!("yt-dlp failed: {err_msg}"));
    }

    if final_path.is_empty() {
        return Err("Download completed but no file path was returned.".into());
    }

    Ok(read_track(Path::new(&final_path)))
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
async fn scan_folder(app: AppHandle, path: String) -> Result<Vec<Track>, String> {
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let roots = vec![PathBuf::from(path)];
    let cache = base.join("tags-v3.json");
    // sync commands run on the main thread; tag reads would freeze the UI
    tauri::async_runtime::spawn_blocking(move || scan(&roots, Some(&cache)))
        .await
        .map_err(|e| e.to_string())
}

/// Cache entry is valid while the file's mtime and size are unchanged.
type Cache = std::collections::HashMap<String, ((u64, u64), Track)>;

fn stamp(p: &Path) -> (u64, u64) {
    let m = std::fs::metadata(p).ok();
    let mtime = m
        .as_ref()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs());
    (mtime, m.map_or(0, |m| m.len()))
}

fn scan(roots: &[PathBuf], cache_file: Option<&Path>) -> Vec<Track> {
    let mut files = Vec::new();
    for r in roots {
        walk(r, &mut files);
    }
    files.sort();
    files.dedup(); // roots may overlap (library folder inside app data)

    let old: Cache = cache_file
        .and_then(|f| std::fs::read(f).ok())
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let mut fresh = Cache::new();
    let mut out = Vec::with_capacity(files.len());
    for p in &files {
        let key = p.to_string_lossy().into_owned();
        let st = stamp(p);
        let t = match old.get(&key) {
            Some((s, t)) if *s == st => t.clone(),
            _ => read_track(p),
        };
        fresh.insert(key, (st, t.clone())); // only seen files survive: deleted ones are pruned
        out.push(t);
    }
    if let (Some(f), Ok(b)) = (cache_file, serde_json::to_vec(&fresh)) {
        let _ = std::fs::write(f, b);
    }

    out.sort_by_cached_key(|t| {
        (
            t.album_artist.as_ref().or(t.artist.as_ref()).map(|s| s.to_lowercase()),
            t.album.as_ref().map(|s| s.to_lowercase()),
            t.disc,
            t.track_no,
            t.path.to_lowercase(),
        )
    });
    out
}

/// Cover as a data URL: embedded picture first, else a cover/folder/front image beside the file.
#[tauri::command]
async fn cover_art(path: String) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || find_cover(Path::new(&path)))
        .await
        .ok()
        .flatten()
}

fn find_cover(path: &Path) -> Option<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use lofty::{picture::PictureType, prelude::*};
    let data_url = |mime: &str, bytes: &[u8]| format!("data:{mime};base64,{}", STANDARD.encode(bytes));

    if let Ok(f) = lofty::read_from_path(path) {
        if let Some(tag) = f.primary_tag().or_else(|| f.first_tag()) {
            let pics = tag.pictures();
            let pic = pics.iter().find(|p| p.pic_type() == PictureType::CoverFront).or(pics.first());
            if let Some(p) = pic {
                let mime = p.mime_type().map_or("image/jpeg", |m| m.as_str());
                return Some(data_url(mime, p.data()));
            }
        }
    }
    for entry in std::fs::read_dir(path.parent()?).ok()?.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        let mime = match name.rsplit_once('.') {
            Some((_, "jpg" | "jpeg")) => "image/jpeg",
            Some((_, "png")) => "image/png",
            _ => continue,
        };
        if ["cover", "folder", "front"].iter().any(|k| name.starts_with(k)) {
            return std::fs::read(entry.path()).ok().map(|b| data_url(mime, &b));
        }
    }
    None
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
struct Playlist {
    id: String,
    name: String,
    tracks: Vec<String>,
}

// missing file = first run = return no playlists; a damaged file is
//an error so a save can't overwrite it
fn read_playlists(path: &Path) -> Result<Vec<Playlist>, String> {
    match std::fs::read(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("Failed to read playlists.json: {e}")),
        Ok(b) => serde_json::from_slice(&b).map_err(|e| format!("Failed to parse playlists.json: {e}")),
    }
}

//write a .tmp file then rename, so in case of a crash, it can't leave
//half a file on disk
fn write_playlists(path: &Path, playlists: &[Playlist]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec(playlists).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, bytes).map_err(|e| format!("Failed to write playlists.json.tmp: {e}"))?;
    std::fs::rename(tmp, path).map_err(|e| format!("Failed to rename playlists.json.tmp to playlists.json: {e}"))
}

fn playlists_file(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(|e| e.to_string())?.join("playlists.json"))
}

//load playlists.json from the app data directory
#[tauri::command]
fn load_playlists(app: AppHandle) -> Result<Vec<Playlist>, String> {
    read_playlists(&playlists_file(&app)?)
}

//save playlists.json to the app data directory
#[tauri::command]
fn save_playlists(app: AppHandle, playlists: Vec<Playlist>) -> Result<(), String> {
   write_playlists(&playlists_file(&app)?, &playlists)
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
            cover_art,
            play,
            pause,
            resume,
            stop,
            seek,
            set_volume,
            download_audio,
            load_playlists,
            save_playlists
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
        let got = scan(&[dir.clone()], None);
        assert_eq!(got.len(), 2);
        // unreadable files stay listed: filename title, no tags, zero duration
        assert_eq!(got[0].title, "a");
        assert!(got[0].artist.is_none() && got[0].secs == 0.0);
        assert!(got[1].path.ends_with("sub/b.flac"));
    }

    #[test]
    fn reads_tags() {
        use lofty::{config::WriteOptions, prelude::*, tag::{Tag, TagType}};
        // minimal valid 44-byte-header WAV (0.1s mono 8-bit silence) with an ID3v2 tag
        let mut wav = Vec::new();
        let data = vec![128u8; 800];
        wav.extend(b"RIFF");
        wav.extend((36 + data.len() as u32).to_le_bytes());
        wav.extend(b"WAVEfmt ");
        wav.extend(16u32.to_le_bytes());
        wav.extend([1, 0, 1, 0]);
        wav.extend(8000u32.to_le_bytes());
        wav.extend(8000u32.to_le_bytes());
        wav.extend([1, 0, 8, 0]);
        wav.extend(b"data");
        wav.extend((data.len() as u32).to_le_bytes());
        wav.extend(data);
        let p = std::env::temp_dir().join("plinth_tag_test.wav");
        std::fs::write(&p, wav).unwrap();
        let mut tag = Tag::new(TagType::Id3v2);
        tag.set_title("Song".into());
        tag.set_artist("Band".into());
        tag.set_album("Record".into());
        tag.set_track(3);
        tag.save_to_path(&p, WriteOptions::default()).unwrap();
        let t = read_track(&p);
        assert_eq!((t.title.as_str(), t.artist.as_deref(), t.album.as_deref(), t.track_no),
                   ("Song", Some("Band"), Some("Record"), Some(3)));
        assert!((t.secs - 0.1).abs() < 0.01);
    }

    #[test]
    fn cache_reuses_unchanged_and_prunes_deleted() {
        let dir = std::env::temp_dir().join("plinth_cache_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (a, b) = (dir.join("a.mp3"), dir.join("b.mp3"));
        std::fs::write(&a, b"x").unwrap();
        std::fs::write(&b, b"y").unwrap();
        let cf = dir.join("tags.json");
        // seed a poisoned entry for a.mp3: if the cache is used, the fake title survives
        let mut seeded = scan(&[dir.clone()], Some(&cf));
        assert_eq!(seeded.len(), 2);
        let mut c: Cache = serde_json::from_slice(&std::fs::read(&cf).unwrap()).unwrap();
        let key = a.to_string_lossy().into_owned();
        c.get_mut(&key).unwrap().1.title = "cached".into();
        std::fs::write(&cf, serde_json::to_vec(&c).unwrap()).unwrap();
        seeded = scan(&[dir.clone()], Some(&cf));
        assert!(seeded.iter().any(|t| t.title == "cached"));
        // changing the file invalidates its entry
        std::fs::write(&a, b"longer content").unwrap();
        assert!(scan(&[dir.clone()], Some(&cf)).iter().all(|t| t.title != "cached"));
        // deleting a file prunes it from the cache
        std::fs::remove_file(&b).unwrap();
        scan(&[dir.clone()], Some(&cf));
        let c: Cache = serde_json::from_slice(&std::fs::read(&cf).unwrap()).unwrap();
        assert!(!c.contains_key(&b.to_string_lossy().into_owned()));
    }

    #[test]
    fn parses_progress() {
        assert_eq!(progress_frac("PLINTH_PROGRESS 50 200 NA"), Some(0.25));
        assert_eq!(progress_frac("PLINTH_PROGRESS 50 NA 100"), Some(0.5)); // falls back to estimate
        assert_eq!(progress_frac("PLINTH_PROGRESS 5 NA NA"), None); // total unknown
        assert_eq!(progress_frac("PLINTH_PROGRESS 300 200 NA"), Some(1.0)); // clamped
        assert_eq!(progress_frac("/music/PLINTH_PROGRESS.m4a"), None); // a path is not progress
    }

    #[test]
    fn filename_numbers() {
        let n = |s: &str| filename_number(Path::new(s));
        assert_eq!(n("01 - Monster.mp3"), Some(1));
        assert_eq!(n("12 - 3Hunna (Remix).mp3"), Some(12));
        assert_eq!(n("7. Song.mp3"), Some(7));
        assert_eq!(n("03_Song.mp3"), Some(3));
        assert_eq!(n("2 Chainz.mp3"), Some(2)); // ponytail: ambiguous with a title, accepted
        assert_eq!(n("1999.mp3"), None);
        assert_eq!(n("Song.mp3"), None);
        assert_eq!(n("2024 Mix.mp3"), None); // 4 digits = a year, not a track
    }

    #[test]
    fn finds_folder_cover() {
        let dir = std::env::temp_dir().join("plinth_cover_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.mp3"), b"").unwrap();
        assert!(find_cover(&dir.join("a.mp3")).is_none());
        std::fs::write(dir.join("Folder.JPG"), b"img").unwrap();
        assert!(find_cover(&dir.join("a.mp3")).unwrap().starts_with("data:image/jpeg;base64,"));
    }

    #[test]
    fn unreadable_file_keeps_order_and_clean_title() {
        let dir = std::env::temp_dir().join("plinth_broken_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("08 - Broke Boi.mp3");
        std::fs::write(&p, b"not audio").unwrap();
        let t = read_track(&p);
        assert_eq!((t.title.as_str(), t.track_no, t.secs), ("Broke Boi", Some(8), 0.0));
    }

    #[test]
    fn playlists_round_trip() {
        let dir = std::env::temp_dir().join("plinth_playlists_round_trip");
        let _ = std::fs::remove_dir_all(&dir);
        let p = dir.join("playlists.json"); // dir doesn't exist yet: write must create it
        assert_eq!(read_playlists(&p).unwrap(), vec![]); // missing file = empty
        let want = vec![Playlist {
            id: "1".into(),
            name: "Late Night Study".into(),
            tracks: vec!["a.mp3".into(), "a.mp3".into()], // duplicates are allowed
        }];
        write_playlists(&p, &want).unwrap();
        assert_eq!(read_playlists(&p).unwrap(), want);
        std::fs::write(&p, b"not json").unwrap();
        assert!(read_playlists(&p).is_err()); // damaged file must not read as empty
    }
}
