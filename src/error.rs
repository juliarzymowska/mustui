use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("yt-dlp: {0}")]
    YtDlpFailed(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
