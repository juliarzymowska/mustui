use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::action::Action;
use crate::app::InputMode;

pub fn translate(key: KeyEvent, mode: &InputMode) -> Option<Action> {
    // Ctrl-C always quits
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(Action::Quit);
    }

    match mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char('/') => Some(Action::StartSearch),
            KeyCode::Char(' ') => Some(Action::TogglePause),
            KeyCode::Char('l') => Some(Action::ToggleLoop),
            KeyCode::Char('j') | KeyCode::Down => Some(Action::SelectNext),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::SelectPrev),
            KeyCode::Enter => Some(Action::PlaySelected),
            _ => None,
        },
        InputMode::Searching => match key.code {
            KeyCode::Esc => Some(Action::CancelSearch),
            KeyCode::Enter => Some(Action::SubmitSearch),
            KeyCode::Backspace => Some(Action::SearchBackspace),
            KeyCode::Char(c) => Some(Action::SearchInput(c)),
            _ => None,
        },
    }
}
