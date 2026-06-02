use std::time::Duration;
use crate::models::Track;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopMode {
    #[default]
    Off,
    One,
}

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Play(Track),
    TogglePause,
    Stop,
    SetLoop(LoopMode),
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Idle,
    Loading,
    Playing,
    Paused,
}

impl Default for PlaybackStatus {
    fn default() -> Self { Self::Idle }
}

#[derive(Debug, Clone, Default)]
pub struct PlayerState {
    pub status: PlaybackStatus,
    pub current: Option<Track>,
    pub position: Duration,
    pub loop_mode: LoopMode,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DataMessage {
    SearchCompleted(crate::models::SearchResults),
    SearchFailed { query: String, error: String },
    ArtworkReady {
        track_id: crate::models::TrackId,
        image: std::sync::Arc<image::DynamicImage>,
    },
    ArtworkFailed {
        track_id: crate::models::TrackId,
        error: String,
    },
}
