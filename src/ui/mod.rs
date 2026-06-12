pub mod player;
pub mod search;
pub mod theme;

use ratatui::Frame;

use crate::model::{Model, View};

pub fn draw(frame: &mut Frame, model: &mut Model) {
    match model.view {
        View::Search => search::draw(frame, frame.area(), model),
        View::Player => player::draw(frame, frame.area(), model),
    }
}
