use std::path::PathBuf;
use std::time::Duration;

use crate::domain::{PlaylistEntry, SearchResults, Track, TrackId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchStatus {
    Fetching,
    Done,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PlayerFocus {
    #[default]
    Library,
    Queue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum View {
    Search,
    #[default]
    Player,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SearchFocus {
    #[default]
    Input,
    Results,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopMode {
    #[default]
    Off,
    One,
    Playlist,
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
    pub pending_id: Option<TrackId>,
    pub position: Duration,
    pub loop_mode: LoopMode,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct Model {
    pub view: View,

    // ── Search view ─────────────────────────────
    pub search_focus: SearchFocus,
    pub query: String,
    pub results: SearchResults,
    pub results_selected: usize,
    pub fetching_id: Option<TrackId>,
    pub fetch_status: Option<FetchStatus>,

    // ── Player view ─────────────────────────────
    pub player_focus: PlayerFocus,
    pub library: Vec<PlaylistEntry>,
    pub library_selected: usize,
    pub queue: Vec<Track>,
    pub queue_selected: usize,

    // ── Shared ──────────────────────────────────
    pub playback: PlaybackState,
    pub should_quit: bool,
}
