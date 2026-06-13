mod app;
mod audio;
mod client;
mod error;
mod library;
mod logging;
mod model;
mod models;
mod msg;
mod task;
mod terminal;
mod ui;
mod update;
mod ytdlp;

fn main() -> anyhow::Result<()> {
    let _log_guard = logging::init();

    let backend = client::Backend::new()?;
    let audio = audio::Audio::new()?;

    let mut app = app::App::new(backend, audio);

    let mut term = terminal::init();
    let result = app.run(&mut term);
    terminal::restore();

    result
}
