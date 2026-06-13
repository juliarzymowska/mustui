use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};

use crate::{
    audio::Audio,
    client::Backend,
    model::{Model, SearchFocus, View},
    msg::Message,
    task::Task,
    ui,
    update::update,
};

pub struct App {
    pub model: Model,
    audio: Audio,
    backend: Backend,
    task: Task<Message>,
    task_rx: mpsc::Receiver<Message>,
}

impl App {
    pub fn new(backend: Backend, audio: Audio) -> Self {
        let (task_tx, task_rx) = mpsc::channel();
        Self {
            model: Model::default(),
            audio,
            backend,
            task: Task::new(task_tx),
            task_rx,
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> anyhow::Result<()> {
        self.model.library = crate::library::load_downloads(&self.backend.music_dir);

        let tick_rate = Duration::from_millis(40);
        let mut last_tick = Instant::now();

        while !self.model.should_quit {
            terminal.draw(|f| ui::draw(f, &mut self.model))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        let msg = translate_key(key, &self.model);
                        self.dispatch(msg);
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.dispatch(Message::Tick);
                last_tick = Instant::now();
            }

            while let Ok(msg) = self.task_rx.try_recv() {
                self.dispatch(msg);
            }
        }

        Ok(())
    }

    fn dispatch(&mut self, msg: Message) {
        let mut next =
            update(&mut self.model, msg, &mut self.audio, &self.backend, &self.task);
        while !matches!(next, Message::None) {
            next = update(&mut self.model, next, &mut self.audio, &self.backend, &self.task);
        }
    }
}

fn translate_key(key: event::KeyEvent, model: &Model) -> Message {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Message::Quit;
    }
    match model.view {
        View::Search => translate_search(key, &model.search_focus),
        View::Player => translate_player(key),
    }
}

fn translate_search(key: event::KeyEvent, focus: &SearchFocus) -> Message {
    match focus {
        SearchFocus::Input => match key.code {
            KeyCode::Esc => Message::Back,
            KeyCode::Enter => Message::SubmitSearch,
            KeyCode::Backspace => Message::SearchBackspace,
            KeyCode::Char(c) => Message::SearchChar(c),
            KeyCode::Tab => Message::ToggleView,
            _ => Message::None,
        },
        SearchFocus::Results => match key.code {
            KeyCode::Char('/') | KeyCode::Esc => Message::EnterSearch,
            KeyCode::Char('j') | KeyCode::Down => Message::NavDown,
            KeyCode::Char('k') | KeyCode::Up => Message::NavUp,
            KeyCode::Enter => Message::Confirm,
            KeyCode::Char('a') => Message::AddToQueue,
            KeyCode::Char('H') => Message::SkipPrev,
            KeyCode::Char('L') => Message::SkipNext,
            KeyCode::Tab => Message::ToggleView,
            KeyCode::Char('q') => Message::Quit,
            _ => Message::None,
        },
    }
}

fn translate_player(key: event::KeyEvent) -> Message {
    match key.code {
        KeyCode::Char(' ') => Message::TogglePause,
        KeyCode::Char('r') => Message::ToggleLoop,
        KeyCode::Tab => Message::ToggleView,
        KeyCode::Char('q') => Message::Quit,
        KeyCode::Char('H') => Message::SkipPrev,
        KeyCode::Char('L') => Message::SkipNext,
        KeyCode::Char('h') | KeyCode::Left => Message::FocusLeft,
        KeyCode::Char('l') | KeyCode::Right => Message::FocusRight,
        KeyCode::Char('j') | KeyCode::Down => Message::NavDown,
        KeyCode::Char('k') | KeyCode::Up => Message::NavUp,
        KeyCode::Enter => Message::Confirm,
        KeyCode::Esc => Message::Back,
        KeyCode::Char('d') => Message::RemoveFromQueue,
        KeyCode::Char('D') => Message::DeleteFromLibrary,
        _ => Message::None,
    }
}
