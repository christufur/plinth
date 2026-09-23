use std::sync::mpsc::{self, Sender};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub enum Cmd {
    Play(String),
    Pause,
    Resume,
    Stop,
    Seek(f64),
    Volume(f32)
}

pub fn spawn(app: AppHandle) -> Sender<Cmd> {
    let (tx, rx) = mpsc::channel::<Cmd>();
    std::thread::spawn(move || {
        let stream = rodio::DeviceSinkBuilder::open_default_sink().expect("No audio device");
        let player = rodio::Player::connect_new(stream.mixer());
        let mut duration = 0.0;
        let mut was_playing = false;
        loop {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(Cmd::Play(path)) => {
                    let Ok(file) = std::fs::File::open(&path) else {continue};
                    let Ok(src) = rodio::Decoder::try_from(file) else {continue};
                    duration = rodio::Source::total_duration(&src).map_or(0.0, |d| d.as_secs_f64());
                    player.clear();
                    player.append(src);
                    player.play();
                    let _ = app.emit("track_changed", &path);
                    let _ = app.emit("state_changed", "playing");
                }
                Ok(Cmd::Pause) => { player.pause(); let _ = app.emit("state_changed", "paused"); }
                Ok(Cmd::Resume) => { player.play(); let _ = app.emit("state_changed", "playing"); }
                Ok(Cmd::Stop) => { player.clear(); let _ = app.emit("state_changed", "stopped"); }
                Ok(Cmd::Seek(s)) => { let _ = player.try_seek(Duration::from_secs_f64(s)); }
                Ok(Cmd::Volume(v)) => player.set_volume(v.clamp(0.0, 1.0)),
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            // tick
            let playing = !player.empty() && !player.is_paused();
            if playing {
                let _ = app.emit("position", (player.get_pos().as_secs_f64(), duration));
            }
            if was_playing && player.empty() {
                let _ = app.emit("track_ended", ());
            }
            was_playing = playing;
        }
    });
    tx
}
