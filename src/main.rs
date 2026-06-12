mod app;
mod audio;
mod client;
mod error;
mod library;
mod logging;
mod model;
mod models;
mod msg;
mod playlist;
mod task;
mod terminal;
mod ui;
mod update;
mod ytdlp;

use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let _log_guard = logging::init();

    let backend = client::Backend::new()?;
    let audio = audio::Audio::new()?;

    let data_dir = directories::ProjectDirs::from("", "", "ytm-tui")
        .map(|d| d.data_local_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let playlist_store = playlist::PlaylistStore::new(data_dir.join("playlists"))?;

    let mut app = app::App::new(backend, audio, playlist_store);

    let mut term = terminal::init();
    let result = app.run(&mut term);
    terminal::restore();

    result
}
