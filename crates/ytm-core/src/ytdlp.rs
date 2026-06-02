use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use serde::Deserialize;
use tokio::process::Command;
use tracing::{debug, warn};

use crate::{
    models::{SearchResults, ThumbnailUrl, Track, TrackId},
    CoreError, Result,
};

pub(crate) async fn search(query: &str, limit: u32) -> Result<SearchResults> {
    let search_spec = format!("ytsearch{limit}:{query}");
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
        .await
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

    let mut tracks = Vec::with_capacity(limit as usize);
    for line in stdout.lines().filter(|l| !l.trim().is_empty()) {
        match serde_json::from_str::<FlatEntry>(line) {
            Ok(entry) => tracks.push(entry.into_track()),
            Err(e) => warn!(error = %e, "skipping malformed yt-dlp entry"),
        }
    }

    Ok(SearchResults { query: query.to_owned(), tracks })
}

pub(crate) async fn ensure_local_audio(music_dir: &Path, id: &TrackId) -> Result<PathBuf> {
    let target = music_dir.join(format!("{}.mp3", id.0));
    if tokio::fs::try_exists(&target).await.unwrap_or(false) {
        debug!(path = %target.display(), "audio cache hit");
        return Ok(target);
    }

    tokio::fs::create_dir_all(music_dir).await?;

    let url = format!("https://www.youtube.com/watch?v={}", id.0);
    let template = music_dir.join("%(id)s.%(ext)s");

    let output = Command::new("yt-dlp")
        .args([
            "-x",
            "--audio-format",
            "mp3",
            "--audio-quality",
            "0",
            "--no-playlist",
            "--no-warnings",
            "--ignore-config",
            "-o",
        ])
        .arg(&template)
        .arg(&url)
        .stdin(Stdio::null())
        .output()
        .await
        .map_err(|e| CoreError::YtDlpFailed(format!("spawn yt-dlp: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(CoreError::YtDlpFailed(format!(
            "yt-dlp download exited {:?}: {}",
            output.status.code(),
            stderr.trim()
        )));
    }

    if !tokio::fs::try_exists(&target).await.unwrap_or(false) {
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
    #[serde(default)]
    uploader: Option<String>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    artists: Option<Vec<String>>,
    #[serde(default)]
    album: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    thumbnails: Vec<RawThumb>,
}

#[derive(Debug, Deserialize)]
struct RawThumb {
    url: String,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
}

impl FlatEntry {
    fn into_track(self) -> Track {
        let artist = self
            .artists
            .as_ref()
            .filter(|v| !v.is_empty())
            .map(|v| v.join(", "))
            .or(self.uploader)
            .or(self.channel)
            .unwrap_or_default();

        let thumbnail = self
            .thumbnails
            .into_iter()
            .filter(|t| t.width.is_some())
            .max_by_key(|t| t.width.unwrap_or(0))
            .map(|t| ThumbnailUrl {
                url: t.url,
                width: t.width.unwrap_or(0),
                height: t.height.unwrap_or(0),
            });

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
