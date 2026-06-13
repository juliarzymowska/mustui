use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    model::{Model, SearchFocus},
    ui::theme,
};

pub fn draw(frame: &mut Frame, area: Rect, model: &Model) {
    let [search_area, main_area, shortcuts_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);

    draw_search_bar(frame, search_area, model);

    let [results_area, meta_area] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
            .areas(main_area);

    draw_results(frame, results_area, model);
    draw_metadata(frame, meta_area, model);
    draw_shortcuts(frame, shortcuts_area, model);
}

fn draw_shortcuts(frame: &mut Frame, area: Rect, model: &Model) {
    use crate::model::SearchFocus;
    let text = match model.search_focus {
        SearchFocus::Input => {
            " [Enter] search  [Esc] go to results  [Tab] player  [Ctrl-C] quit"
        }
        SearchFocus::Results => {
            " [/][Esc] edit search  [j/k] nav  [↵] play now  [a] add to queue  [Tab] player  [q] quit"
        }
    };
    frame.render_widget(Paragraph::new(text).style(theme::dimmed()), area);
}

fn draw_search_bar(frame: &mut Frame, area: Rect, model: &Model) {
    let focused = model.search_focus == SearchFocus::Input;
    let border_style = if focused { theme::accent() } else { Style::default() };
    let block = Block::new()
        .borders(Borders::ALL)
        .title(" Search YouTube ")
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cursor = if focused { "█" } else { "" };
    let text = format!("{}{}", model.query, cursor);
    let hint = if model.query.is_empty() && !focused {
        "  press / to search..."
    } else {
        ""
    };
    let display = if model.query.is_empty() && !focused {
        hint.to_owned()
    } else {
        text
    };

    let style = if model.query.is_empty() && !focused {
        theme::dimmed()
    } else {
        theme::normal()
    };

    frame.render_widget(Paragraph::new(display).style(style), inner);
}

fn draw_results(frame: &mut Frame, area: Rect, model: &Model) {
    let focused = model.search_focus == SearchFocus::Results;
    let border_style = if focused { theme::accent() } else { Style::default() };

    if model.results.tracks.is_empty() {
        let block = Block::new()
            .borders(Borders::ALL)
            .title(" Results ")
            .border_style(border_style);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new("no results yet")
                .style(theme::dimmed())
                .alignment(Alignment::Center),
            Rect { y: inner.y + inner.height / 2, height: 1, ..inner },
        );
        return;
    }

    let items: Vec<ListItem> = model
        .results
        .tracks
        .iter()
        .map(|t| {
            let dur = t
                .duration
                .map(|d| format!(" [{:02}:{:02}]", d.as_secs() / 60, d.as_secs() % 60))
                .unwrap_or_default();
            let item = ListItem::new(format!("{} — {}{}", t.artist, t.title, dur));
            if model.queue.iter().any(|q| q.id == t.id) {
                item.style(Style::default().add_modifier(Modifier::DIM))
            } else {
                item
            }
        })
        .collect();

    let title = if model.results.query.is_empty() {
        " Results ".to_owned()
    } else {
        format!(" Results: {} ", model.results.query)
    };

    let list = List::new(items)
        .block(Block::new().borders(Borders::ALL).title(title).border_style(border_style))
        .highlight_style(theme::reversed())
        .highlight_symbol("> ");

    let mut state = ListState::default().with_selected(Some(model.results_selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_metadata(frame: &mut Frame, area: Rect, model: &Model) {
    let block = Block::new().borders(Borders::ALL).title(" Track Info ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(track) = model.results.tracks.get(model.results_selected) else {
        frame.render_widget(
            Paragraph::new("select a track").style(theme::dimmed()),
            inner,
        );
        return;
    };

    let already_queued = model.queue.iter().any(|q| q.id == track.id);

    let dur_str = track
        .duration
        .map(|d| format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60))
        .unwrap_or_else(|| "—".to_owned());

    let status_line = if let Some(ref err) = model.playback.error {
        format!("⚠ {err}")
    } else {
        String::new()
    };

    let lines = [
        format!("Title:    {}", track.title),
        format!("Artist:   {}", track.artist),
        format!("Album:    {}", track.album.as_deref().unwrap_or("—")),
        format!("Duration: {}", dur_str),
        String::new(),
        if already_queued {
            "[↵] play now  ✓ already in queue".to_owned()
        } else {
            "[↵] play now  [a] add to queue".to_owned()
        },
        status_line,
    ];

    frame.render_widget(
        Paragraph::new(lines.join("\n")).wrap(Wrap { trim: false }).style(theme::normal()),
        inner,
    );
}
