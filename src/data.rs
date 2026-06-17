//! Data layer: filesystem paths, on-disk library, yt-dlp subprocess driver.
//!
//! Everything in here touches the outside world (disk, network via yt-dlp).
//! The `update` reducer is the only caller; nothing in `ui` or `state` should
//! reach in here directly.

pub mod client;
pub mod library;
pub mod ytdlp;
