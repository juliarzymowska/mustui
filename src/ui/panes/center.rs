use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::{model::Model, ui::theme};

pub fn draw(frame: &mut Frame, area: Rect, model: &Model) {
    if !model.results.tracks.is_empty() {
        draw_results(frame, area, model);
    } else {
        draw_empty(frame, area);
    }
}

fn draw_results(frame: &mut Frame, area: Rect, model: &Model) {
    let items: Vec<ListItem> = model
        .results
        .tracks
        .iter()
        .map(|t| {
            let label = format!(
                "{} — {}{}",
                t.artist,
                t.title,
                t.duration
                    .map(|d| format!("  [{:02}:{:02}]", d.as_secs() / 60, d.as_secs() % 60))
                    .unwrap_or_default()
            );
            ListItem::new(label)
        })
        .collect();

    let title = format!(" Results: {} ", model.results.query);
    let list = List::new(items)
        .block(Block::new().borders(Borders::ALL).title(title))
        .highlight_style(theme::reversed())
        .highlight_symbol("> ");

    let mut state = ListState::default().with_selected(Some(model.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_empty(frame: &mut Frame, area: Rect) {
    let block = Block::new().borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new("type / to search")
            .style(theme::dimmed())
            .alignment(Alignment::Center),
        Rect { y: inner.y + inner.height / 2, height: 1, ..inner },
    );
}
