use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::theme;

pub fn draw(frame: &mut Frame, area: Rect) {
    let block = Block::new().borders(Borders::ALL).title(" Playlists ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new("(coming in v2)").style(theme::dimmed()),
        inner,
    );
}
