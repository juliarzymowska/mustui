mod app;
mod audio;
mod data;
mod domain;
mod error;
mod logging;
mod msg;
mod state;
mod task;
mod terminal;
mod ui;
mod update;

fn main() -> anyhow::Result<()> {
    let _log_guard = logging::init();

    let backend = data::client::Backend::new()?;
    let audio = audio::Audio::new()?;

    let mut app = app::App::new(backend, audio);

    let mut term = terminal::init();
    let result = app.run(&mut term);
    terminal::restore();

    result
}
