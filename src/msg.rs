use std::path::PathBuf;

use crate::models::{SearchResults, TrackId};

#[derive(Debug)]
pub enum Message {
    // ── Global ──────────────────────────────────
    Quit,
    Tick,
    None,
    ToggleView,
    TogglePause,
    ToggleLoop,

    // ── Navigation (context-routed in update) ───
    NavUp,
    NavDown,
    FocusLeft,
    FocusRight,
    Confirm,
    Back,

    // ── Search view ─────────────────────────────
    EnterSearch,
    SearchChar(char),
    SearchBackspace,
    SubmitSearch,

    // ── Async results ────────────────────────────
    SearchDone(Result<SearchResults, String>),
    DownloadReady(TrackId, PathBuf),
    DownloadFailed(TrackId, String),

    // ── Internal auto-advance ────────────────────
    PlayNext,

    // ── User-triggered queue skip (always wraps) ─
    SkipNext,
    SkipPrev,

    // ── Queue management ─────────────────────────
    AddToQueue,
    RemoveFromQueue,
    DeleteFromLibrary,
}
