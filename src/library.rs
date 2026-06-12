use std::path::Path;

use crate::{
    models::Track,
    playlist::{Playlist, PlaylistEntry},
};

/// Scan `music_dir` for `<id>.json` metadata sidecars (written on each download)
/// and return them as a "Downloads" playlist.
pub fn load_downloads(music_dir: &Path) -> Playlist {
    let Ok(dir) = std::fs::read_dir(music_dir) else {
        return Playlist { name: "Downloads".to_owned(), tracks: vec![] };
    };

    let mut tracks: Vec<PlaylistEntry> = dir
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| {
            let text = std::fs::read_to_string(e.path()).ok()?;
            let track: Track = serde_json::from_str(&text).ok()?;
            // Only include it if the audio file actually exists alongside the sidecar
            let audio = e.path().with_extension("mp3");
            let audio_m4a = e.path().with_extension("m4a");
            if !audio.exists() && !audio_m4a.exists() {
                return None;
            }
            Some(PlaylistEntry {
                id: track.id,
                title: track.title,
                artist: track.artist,
                duration_ms: track.duration.map(|d| d.as_millis() as u64),
            })
        })
        .collect();

    tracks.sort_by(|a, b| a.artist.cmp(&b.artist).then(a.title.cmp(&b.title)));

    Playlist { name: "Downloads".to_owned(), tracks }
}

/// Write a metadata sidecar `<stem>.json` next to the audio file so
/// future startups can reconstruct the Downloads playlist.
pub fn save_sidecar(audio_path: &Path, track: &Track) {
    let sidecar = audio_path.with_extension("json");
    if let Ok(json) = serde_json::to_string(track) {
        let _ = std::fs::write(sidecar, json);
    }
}
