use std::path::PathBuf;
use std::time::Duration;

use ratatui_image::protocol::StatefulProtocol;

use crate::models::{SearchResults, Track};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Searching,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopMode {
    #[default]
    Off,
    One,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioStatus {
    #[default]
    Idle,
    Loading,
    Playing,
    Paused,
}

#[derive(Debug, Default)]
pub struct PlaybackState {
    pub status: AudioStatus,
    pub current: Option<Track>,
    pub current_path: Option<PathBuf>,
    pub position: Duration,
    pub loop_mode: LoopMode,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct Model {
    pub mode: InputMode,
    pub query: String,
    pub results: SearchResults,
    pub selected: usize,
    pub playback: PlaybackState,
    pub artwork: Option<StatefulProtocol>,
    pub should_quit: bool,
}
