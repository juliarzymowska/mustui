mod action;
mod app;
mod artwork;
mod client;
mod error;
mod events;
mod logging;
mod messages;
mod models;
mod player;
mod playlist;
mod search;
mod terminal;
mod ui;
mod ytdlp;

use std::path::PathBuf;
use app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _log_guard = logging::init();

    let picker: Option<ratatui_image::picker::Picker> = None;

    let backend = client::Backend::new().await?;
    let player = player::spawn(backend.clone());

    let data_dir = directories::ProjectDirs::from("", "", "ytm-tui")
        .map(|d| d.data_local_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let playlist_store = playlist::PlaylistStore::new(data_dir.join("playlists"))?;

    let term = terminal::init();
    let result = app::run(App::new(backend, player, picker, playlist_store), term).await;
    terminal::restore();

    result
}
