use futures::StreamExt;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, interval};

use crate::{
    action::Action,
    artwork,
    client::Backend,
    events,
    messages::{DataMessage, LoopMode, PlayerCommand, PlayerState},
    models::{SearchResults, TrackId},
    player::PlayerHandle,
    playlist::PlaylistStore,
    ui,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Searching,
}

pub struct App {
    pub backend: Backend,
    pub player: PlayerHandle,
    pub mode: InputMode,
    pub query: String,
    pub results: SearchResults,
    pub selected: usize,
    pub player_state: PlayerState,
    pub artwork: Option<StatefulProtocol>,
    pub picker: Option<Picker>,
    pub playlist_store: PlaylistStore,
    pub should_quit: bool,
    data_tx: mpsc::Sender<DataMessage>,
    data_rx: mpsc::Receiver<DataMessage>,
}

impl App {
    pub fn new(
        backend: Backend,
        player: PlayerHandle,
        picker: Option<Picker>,
        playlist_store: PlaylistStore,
    ) -> Self {
        let (data_tx, data_rx) = mpsc::channel(64);
        Self {
            backend,
            player,
            mode: InputMode::Normal,
            query: String::new(),
            results: SearchResults::default(),
            selected: 0,
            player_state: PlayerState::default(),
            artwork: None,
            picker,
            playlist_store,
            should_quit: false,
            data_tx,
            data_rx,
        }
    }
}

pub async fn run(mut app: App, mut terminal: ratatui::DefaultTerminal) -> anyhow::Result<()> {
    let mut events = crossterm::event::EventStream::new();
    let mut ticker = interval(Duration::from_millis(100));
    let mut state_rx: broadcast::Receiver<PlayerState> = app.player.subscribe();

    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        tokio::select! {
            biased;

            Some(Ok(ev)) = events.next() => {
                if let crossterm::event::Event::Key(key) = ev {
                    if key.kind == crossterm::event::KeyEventKind::Press {
                        if let Some(action) = events::translate(key, &app.mode) {
                            handle_action(&mut app, action).await;
                        }
                    }
                }
            }

            Ok(state) = state_rx.recv() => {
                let track_changed = app.player_state.current.as_ref().map(|t| &t.id)
                    != state.current.as_ref().map(|t| &t.id);
                app.player_state = state;
                if track_changed {
                    app.artwork = None;
                    if let Some(track) = &app.player_state.current {
                        if let Some(thumb) = &track.thumbnail {
                            spawn_artwork_fetch(
                                app.backend.clone(),
                                track.id.clone(),
                                thumb.url.clone(),
                                app.data_tx.clone(),
                            );
                        }
                    }
                }
            }

            Some(msg) = app.data_rx.recv() => {
                handle_data(&mut app, msg);
            }

            _ = ticker.tick() => {}
        }
    }

    Ok(())
}

async fn handle_action(app: &mut App, action: Action) {
    match action {
        Action::Quit => app.should_quit = true,

        Action::StartSearch => {
            app.mode = InputMode::Searching;
            app.query.clear();
        }
        Action::CancelSearch => {
            app.mode = InputMode::Normal;
        }
        Action::SubmitSearch => {
            app.mode = InputMode::Normal;
            if !app.query.is_empty() {
                spawn_search(app.backend.clone(), app.query.clone(), app.data_tx.clone());
            }
        }
        Action::SearchInput(c) => app.query.push(c),
        Action::SearchBackspace => { app.query.pop(); }

        Action::SelectNext => {
            if app.selected + 1 < app.results.tracks.len() {
                app.selected += 1;
            }
        }
        Action::SelectPrev => {
            app.selected = app.selected.saturating_sub(1);
        }

        Action::PlaySelected => {
            if let Some(track) = app.results.tracks.get(app.selected).cloned() {
                let _ = app.player.send(PlayerCommand::Play(track)).await;
            }
        }
        Action::TogglePause => {
            let _ = app.player.send(PlayerCommand::TogglePause).await;
        }
        Action::ToggleLoop => {
            let next = match app.player_state.loop_mode {
                LoopMode::Off => LoopMode::One,
                LoopMode::One => LoopMode::Off,
            };
            let _ = app.player.send(PlayerCommand::SetLoop(next)).await;
        }
    }
}

fn handle_data(app: &mut App, msg: DataMessage) {
    match msg {
        DataMessage::SearchCompleted(results) => {
            app.results = results;
            app.selected = 0;
        }
        DataMessage::SearchFailed { error, .. } => {
            app.player_state.error = Some(error);
        }
        DataMessage::ArtworkReady { image, .. } => {
            if let Some(picker) = &app.picker {
                app.artwork = Some(picker.new_resize_protocol((*image).clone()));
            }
        }
        DataMessage::ArtworkFailed { .. } => {
            app.artwork = None;
        }
    }
}

fn spawn_search(backend: Backend, query: String, tx: mpsc::Sender<DataMessage>) {
    tokio::spawn(async move {
        let msg = match backend.search(&query).await {
            Ok(results) => DataMessage::SearchCompleted(results),
            Err(e) => DataMessage::SearchFailed { query, error: e.to_string() },
        };
        let _ = tx.send(msg).await;
    });
}

fn spawn_artwork_fetch(
    backend: Backend,
    track_id: TrackId,
    url: String,
    tx: mpsc::Sender<DataMessage>,
) {
    tokio::spawn(async move {
        let result = artwork::fetch_artwork(&backend, &url).await;
        let msg = match result {
            Ok(img) => DataMessage::ArtworkReady {
                track_id,
                image: std::sync::Arc::new(img),
            },
            Err(e) => DataMessage::ArtworkFailed {
                track_id,
                error: e.to_string(),
            },
        };
        let _ = tx.send(msg).await;
    });
}
