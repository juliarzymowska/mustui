use std::path::PathBuf;
use std::sync::Arc;

use image::DynamicImage;

use crate::models::{SearchResults, TrackId};

#[derive(Debug)]
pub enum Message {
    // Input
    Quit,
    Tick,
    None,
    EnterSearch,
    CancelSearch,
    SubmitSearch,
    SearchChar(char),
    SearchBackspace,
    SelectNext,
    SelectPrev,
    PlaySelected,
    TogglePause,
    ToggleLoop,

    // Async results
    SearchDone(Result<SearchResults, String>),
    DownloadReady(TrackId, PathBuf),
    DownloadFailed(TrackId, String),
    ArtworkReady(TrackId, Arc<DynamicImage>),
    ArtworkFailed(TrackId),
}
