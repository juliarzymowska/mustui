use std::path::PathBuf;

use crate::{models::TrackId, ytdlp, Backend, Result};

pub(crate) async fn ensure_local(backend: &Backend, id: &TrackId) -> Result<PathBuf> {
    ytdlp::ensure_local_audio(backend.music_dir(), id).await
}
