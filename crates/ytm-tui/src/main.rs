mod action;
mod app;
mod events;
mod logging;
mod terminal;
mod ui;

use app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _log_guard = logging::init();

    // Image protocol detection deferred to Phase F (picker query interferes with EventStream)
    let picker: Option<ratatui_image::picker::Picker> = None;

    let (backend, player) = ytm_core::init().await?;

    let term = terminal::init();
    let result = app::run(App::new(backend, player, picker), term).await;
    terminal::restore();

    result
}
