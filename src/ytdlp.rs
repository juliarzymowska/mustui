use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::Deserialize;
use tracing::{debug, warn};

use crate::{
    error::{CoreError, Result},
    models::{SearchResults, ThumbnailUrl, Track, TrackId},
};

pub(crate) fn search(query: &str, limit: u32) -> Result<SearchResults> {
    let fetch = limit * 2;
    let search_spec = format!("ytsearch{fetch}:{query}");
    let output = Command::new("yt-dlp")
        .args([
            "--dump-json",
            "--flat-playlist",
            "--no-warnings",
            "--ignore-config",
            &search_spec,
        ])
        .stdin(Stdio::null())
        .output()
        .map_err(|e| CoreError::YtDlpFailed(format!("spawn yt-dlp: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(CoreError::YtDlpFailed(format!(
            "yt-dlp search exited {:?}: {}",
            output.status.code(),
            stderr.trim()
        )));
    }

    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|e| CoreError::YtDlpFailed(format!("non-utf8 stdout: {e}")))?;

    let mut entries: Vec<(i32, FlatEntry)> = Vec::with_capacity(fetch as usize);
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        match serde_json::from_str::<FlatEntry>(line) {
            Ok(entry) => entries.push((audio_score(&entry), entry)),
            Err(e) => warn!(error = %e, "skipping malformed yt-dlp entry"),
        }
    }
    entries.sort_by(|a, b| b.0.cmp(&a.0));

    let tracks = entries.into_iter().take(limit as usize).map(|(_, e)| e.into_track()).collect();
    Ok(SearchResults { query: query.to_owned(), tracks })
}

/// Score a result toward official album audio and away from videos/live/covers.
fn audio_score(e: &FlatEntry) -> i32 {
    let mut score = 0i32;
    let t = e.title.to_lowercase();

    let channel = e.channel.as_deref().or(e.uploader.as_deref()).unwrap_or("");
    if channel.ends_with("- Topic") { score += 12; }
    if t.contains("official audio") || t.contains("audio only") { score += 8; }
    if t.contains("provided to youtube") { score += 6; }
    if e.album.is_some() { score += 6; }

    if t.contains("official video") || t.contains("official music video")
        || t.contains("(mv)") || t.contains("music video") { score -= 8; }
    if t.contains("lyric video") || t.contains("(lyrics)") || t.ends_with(" lyrics") { score -= 5; }
    if t.contains("(live)") || t.contains("live at ") || t.contains("live from ")
        || t.contains("live performance") || t.contains("live version") { score -= 7; }
    if t.contains("(cover)") || t.contains("cover version") || t.contains("fan cover") { score -= 6; }
    if t.contains("karaoke") || t.contains("instrumental") || t.contains("backing track") { score -= 10; }
    if t.contains("remix") || t.contains("re-mix") { score -= 3; }
    if let Some(dur) = e.duration {
        if dur < 90.0 || dur > 600.0 { score -= 4; }
    }
    score
}

pub(crate) fn ensure_local_audio(music_dir: &Path, id: &TrackId) -> Result<PathBuf> {
    let target = music_dir.join(format!("{}.mp3", id.0));
    if target.exists() {
        debug!(path = %target.display(), "audio cache hit");
        return Ok(target);
    }

    std::fs::create_dir_all(music_dir)?;

    let url = format!("https://www.youtube.com/watch?v={}", id.0);
    let template = music_dir.join("%(id)s.%(ext)s");

    let output = Command::new("yt-dlp")
        .args(["-x", "--audio-format", "mp3", "--audio-quality", "0",
               "--no-playlist", "--no-warnings", "--ignore-config", "-o"])
        .arg(&template)
        .arg(&url)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| CoreError::YtDlpFailed(format!("spawn yt-dlp: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(CoreError::YtDlpFailed(format!(
            "yt-dlp download exited {:?}: {}",
            output.status.code(),
            stderr.trim()
        )));
    }

    if !target.exists() {
        return Err(CoreError::YtDlpFailed(format!(
            "yt-dlp succeeded but {} is missing",
            target.display()
        )));
    }

    Ok(target)
}

#[derive(Debug, Deserialize)]
struct FlatEntry {
    id: String,
    title: String,
    #[serde(default)] uploader: Option<String>,
    #[serde(default)] channel: Option<String>,
    #[serde(default)] artists: Option<Vec<String>>,
    #[serde(default)] album: Option<String>,
    #[serde(default)] duration: Option<f64>,
    #[serde(default)] thumbnails: Vec<RawThumb>,
}

#[derive(Debug, Deserialize)]
struct RawThumb {
    url: String,
    #[serde(default)] width: Option<u32>,
    #[serde(default)] height: Option<u32>,
}

impl FlatEntry {
    fn into_track(self) -> Track {
        let artist = self.artists.as_ref().filter(|v| !v.is_empty())
            .map(|v| v.join(", "))
            .or(self.uploader)
            .or(self.channel)
            .unwrap_or_default();

        let thumbnail = self.thumbnails.into_iter()
            .filter(|t| t.width.is_some())
            .max_by_key(|t| t.width.unwrap_or(0))
            .map(|t| ThumbnailUrl { url: t.url, width: t.width.unwrap_or(0), height: t.height.unwrap_or(0) });

        Track {
            id: TrackId(self.id),
            title: self.title,
            artist,
            album: self.album,
            duration: self.duration.map(|s| Duration::from_secs_f64(s)),
            thumbnail,
        }
    }
}
