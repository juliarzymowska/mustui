pub mod error;
pub mod messages;
pub mod models;

mod artwork;
mod client;
mod player;
mod search;
mod ytdlp;

pub use artwork::fetch_artwork;
pub use client::Backend;
pub use error::{CoreError, Result};
pub use messages::{DataMessage, LoopMode, PlaybackStatus, PlayerCommand, PlayerState};
pub use models::{SearchResults, ThumbnailUrl, Track, TrackId};
pub use player::PlayerHandle;

pub async fn init() -> Result<(Backend, PlayerHandle)> {
    let backend = Backend::new().await?;
    let player = player::spawn(backend.clone());
    Ok((backend, player))
}
