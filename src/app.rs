use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui_image::picker::Picker;

use crate::{
    audio::Audio,
    client::Backend,
    model::{InputMode, Model},
    msg::Message,
    playlist::PlaylistStore,
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
    picker: Option<Picker>,
    pub playlist_store: PlaylistStore,
}

impl App {
    pub fn new(
        backend: Backend,
        audio: Audio,
        picker: Option<Picker>,
        playlist_store: PlaylistStore,
    ) -> Self {
        let (task_tx, task_rx) = mpsc::channel();
        Self {
            model: Model::default(),
            audio,
            backend,
            task: Task::new(task_tx),
            task_rx,
            picker,
            playlist_store,
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> anyhow::Result<()> {
        let tick_rate = Duration::from_millis(40);
        let mut last_tick = Instant::now();

        while !self.model.should_quit {
            terminal.draw(|f| ui::draw(f, &self.model))?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        let msg = translate_key(key, &self.model.mode);
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
        let mut next = update(&mut self.model, msg, &mut self.audio, &self.backend, &self.task, &self.picker);
        while !matches!(next, Message::None) {
            next = update(&mut self.model, next, &mut self.audio, &self.backend, &self.task, &self.picker);
        }
    }
}

fn translate_key(key: event::KeyEvent, mode: &InputMode) -> Message {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Message::Quit;
    }
    match mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => Message::Quit,
            KeyCode::Char('/') => Message::EnterSearch,
            KeyCode::Char(' ') => Message::TogglePause,
            KeyCode::Char('l') => Message::ToggleLoop,
            KeyCode::Char('j') | KeyCode::Down => Message::SelectNext,
            KeyCode::Char('k') | KeyCode::Up => Message::SelectPrev,
            KeyCode::Enter => Message::PlaySelected,
            _ => Message::None,
        },
        InputMode::Searching => match key.code {
            KeyCode::Esc => Message::CancelSearch,
            KeyCode::Enter => Message::SubmitSearch,
            KeyCode::Backspace => Message::SearchBackspace,
            KeyCode::Char(c) => Message::SearchChar(c),
            _ => Message::None,
        },
    }
}
