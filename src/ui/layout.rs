use ratatui::{
    layout::{Constraint, Layout, Rect},
};

pub struct AppLayout {
    pub left: Rect,
    pub center: Rect,
    pub right: Rect,
    pub status: Rect,
}

pub fn compute(area: Rect) -> AppLayout {
    let [content, status] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

    let [left, center, right] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Percentage(45),
        Constraint::Percentage(35),
    ])
    .areas(content);

    AppLayout { left, center, right, status }
}
