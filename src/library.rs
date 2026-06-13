use std::path::Path;

use crate::models::{PlaylistEntry, Track, TrackId};

/// Scan `music_dir` for `<id>.json` metadata sidecars (written on each download)
/// and return them as a sorted list of tracks.
pub fn load_downloads(music_dir: &Path) -> Vec<PlaylistEntry> {
    let Ok(dir) = std::fs::read_dir(music_dir) else {
        return vec![];
    };

    let mut tracks: Vec<PlaylistEntry> = dir
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| {
            let text = std::fs::read_to_string(e.path()).ok()?;
            let track: Track = serde_json::from_str(&text).ok()?;
            // Only include it if the audio file actually exists alongside the sidecar
            let has_audio = ["mp3", "m4a", "webm", "opus"]
                .iter()
                .any(|ext| e.path().with_extension(ext).exists());
            if !has_audio {
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
    tracks
}

/// Delete all audio files and the JSON sidecar for `id` from `music_dir`.
pub fn delete_track(music_dir: &Path, id: &TrackId) {
    for ext in ["m4a", "mp3", "webm", "opus", "json"] {
        let p = music_dir.join(format!("{}.{ext}", id.0));
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    }
}

/// Write a metadata sidecar `<stem>.json` next to the audio file so
/// future startups can reconstruct the Downloads playlist.
pub fn save_sidecar(audio_path: &Path, track: &Track) {
    let sidecar = audio_path.with_extension("json");
    if let Ok(json) = serde_json::to_string(track) {
        let _ = std::fs::write(sidecar, json);
    }
}
