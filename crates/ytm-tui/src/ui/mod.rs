pub mod layout;
pub mod panes;
pub mod theme;

use ratatui::{Frame, layout::Alignment, widgets::Paragraph};

use crate::app::{App, InputMode};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let layout = layout::compute(frame.area());

    panes::playlists::draw(frame, layout.left);
    panes::center::draw(frame, layout.center, app);
    panes::now_playing::draw(frame, layout.right, app);
    draw_status(frame, layout.status, app);
}

fn draw_status(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let text = match &app.mode {
        InputMode::Searching => format!("Search: {}█", app.query),
        InputMode::Normal => {
            if let Some(err) = &app.player_state.error {
                format!("Error: {err}")
            } else {
                " [/] search   [Space] pause   [l] loop   [q] quit".to_string()
            }
        }
    };

    let style = match &app.mode {
        InputMode::Searching => theme::bold(),
        InputMode::Normal if app.player_state.error.is_some() => theme::normal(),
        _ => theme::dimmed(),
    };

    frame.render_widget(
        Paragraph::new(text).style(style).alignment(Alignment::Left),
        area,
    );
}
