use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("yt-dlp: {0}")]
    YtDlpFailed(String),
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("player actor disconnected")]
    PlayerDisconnected,
    #[error("image: {0}")]
    Image(#[from] image::ImageError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
