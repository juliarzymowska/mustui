pub mod layout;
pub mod panes;
pub mod theme;

use ratatui::{Frame, layout::Alignment, widgets::Paragraph};

use crate::model::{InputMode, Model};

pub fn draw(frame: &mut Frame, model: &Model) {
    let layout = layout::compute(frame.area());

    panes::playlists::draw(frame, layout.left);
    panes::center::draw(frame, layout.center, model);
    panes::now_playing::draw(frame, layout.right, model);
    draw_status(frame, layout.status, model);
}

fn draw_status(frame: &mut Frame, area: ratatui::layout::Rect, model: &Model) {
    let text = match &model.mode {
        InputMode::Searching => format!("Search: {}█", model.query),
        InputMode::Normal => {
            if let Some(err) = &model.playback.error {
                format!("Error: {err}")
            } else {
                " [/] search   [Space] pause   [l] loop   [q] quit".to_string()
            }
        }
    };

    let style = match &model.mode {
        InputMode::Searching => theme::bold(),
        InputMode::Normal if model.playback.error.is_some() => theme::normal(),
        _ => theme::dimmed(),
    };

    frame.render_widget(
        Paragraph::new(text).style(style).alignment(Alignment::Left),
        area,
    );
}
