use std::path::PathBuf;

use directories::UserDirs;

#[derive(Clone)]
pub struct Backend {
    pub(crate) music_dir: PathBuf,
}

impl Backend {
    pub fn new() -> crate::error::Result<Self> {
        let music_dir = UserDirs::new()
            .and_then(|d| d.audio_dir().map(|p| p.join("mustui")))
            .unwrap_or_else(|| {
                std::env::var("HOME")
                    .map(|h| PathBuf::from(h).join("Music").join("mustui"))
                    .unwrap_or_else(|_| PathBuf::from("Music/mustui"))
            });
        std::fs::create_dir_all(&music_dir)?;
        Ok(Self { music_dir })
    }
}
