use std::path::PathBuf;

use directories::UserDirs;

#[derive(Clone)]
pub struct Backend {
    pub(crate) http: reqwest::Client,
    pub(crate) music_dir: PathBuf,
}

impl Backend {
    pub async fn new() -> crate::Result<Self> {
        let music_dir = UserDirs::new()
            .and_then(|d| d.audio_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| {
                std::env::var("HOME")
                    .map(|h| PathBuf::from(h).join("Music"))
                    .unwrap_or_else(|_| PathBuf::from("Music"))
            });

        let http = reqwest::Client::builder().build()?;

        Ok(Self { http, music_dir })
    }

    pub(crate) fn music_dir(&self) -> &std::path::Path {
        &self.music_dir
    }
}
