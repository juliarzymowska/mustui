use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph},
};

use crate::{
    model::{AudioStatus, LoopMode, Model},
    ui::theme,
};

pub fn draw(frame: &mut Frame, area: Rect, model: &mut Model) {
    let [main_area, now_playing_area, shortcuts_area] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(5),
        Constraint::Length(1),
    ])
    .areas(area);

    let [songs_area, queue_area] = Layout::horizontal([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ])
    .areas(main_area);

    draw_songs(frame, songs_area, model);
    draw_queue(frame, queue_area, model);
    draw_now_playing(frame, now_playing_area, model);
    draw_shortcuts(frame, shortcuts_area, model);
}

fn draw_songs(frame: &mut Frame, area: Rect, model: &Model) {
    let block = Block::new()
        .borders(Borders::ALL)
        .title(" Library ")
        .border_style(theme::accent());

    if model.library.is_empty() {
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new("no downloads yet — search and play a track")
                .style(theme::dimmed())
                .alignment(Alignment::Center),
            Rect { y: inner.y + inner.height / 2, height: 1, ..inner },
        );
        return;
    }

    let items: Vec<ListItem> = model
        .library
        .iter()
        .map(|t| {
            let dur = t
                .duration_ms
                .map(|ms| {
                    let s = ms / 1000;
                    format!(" [{:02}:{:02}]", s / 60, s % 60)
                })
                .unwrap_or_default();
            ListItem::new(format!("{} — {}{}", t.artist, t.title, dur))
        })
        .collect();

    let sel = model.library_selected.min(model.library.len().saturating_sub(1));

    let list = List::new(items)
        .block(block)
        .highlight_style(theme::reversed())
        .highlight_symbol("> ");

    let mut state = ListState::default().with_selected(Some(sel));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_queue(frame: &mut Frame, area: Rect, model: &Model) {
    let block = Block::new().borders(Borders::ALL).title(" Queue ");

    if model.queue.is_empty() {
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new("queue is empty")
                .style(theme::dimmed())
                .alignment(Alignment::Center),
            Rect { y: inner.y + inner.height / 2, height: 1, ..inner },
        );
        return;
    }

    let playing_id = model.playback.current.as_ref().map(|t| &t.id);
    let items: Vec<ListItem> = model
        .queue
        .iter()
        .map(|t| {
            let label = format!("{} — {}", t.artist, t.title);
            if Some(&t.id) == playing_id {
                ListItem::new(format!("▶ {}", label)).style(theme::bold())
            } else {
                ListItem::new(format!("  {}", label))
            }
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_now_playing(frame: &mut Frame, area: Rect, model: &Model) {
    let block = Block::new().borders(Borders::ALL).title(" Now Playing ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 3 {
        return;
    }

    let [track_area, progress_area, status_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    match &model.playback.current {
        None => {
            frame.render_widget(
                Paragraph::new("nothing playing").style(theme::dimmed()),
                track_area,
            );
        }
        Some(track) => {
            let meta = match &track.album {
                Some(a) => format!("{} — {} · {}", track.artist, track.title, a),
                None => format!("{} — {}", track.artist, track.title),
            };
            frame.render_widget(Paragraph::new(meta).style(theme::bold()), track_area);

            let (ratio, label) = match track.duration {
                Some(dur) if !dur.is_zero() => {
                    let pos = model.playback.position;
                    let r = (pos.as_secs_f64() / dur.as_secs_f64()).clamp(0.0, 1.0);
                    let label = format!(
                        "{:02}:{:02} / {:02}:{:02}",
                        pos.as_secs() / 60,
                        pos.as_secs() % 60,
                        dur.as_secs() / 60,
                        dur.as_secs() % 60,
                    );
                    (r, label)
                }
                _ => (0.0, "--:-- / --:--".to_owned()),
            };
            frame.render_widget(
                Gauge::default()
                    .ratio(ratio)
                    .label(label)
                    .style(theme::normal())
                    .gauge_style(theme::reversed()),
                progress_area,
            );

            let status = match model.playback.status {
                AudioStatus::Playing => "▶  Playing",
                AudioStatus::Paused => "⏸  Paused",
                AudioStatus::Loading => "⏳ Loading",
                AudioStatus::Idle => "   Idle",
            };
            let loop_str = match model.playback.loop_mode {
                LoopMode::One => "  [↺ one]",
                LoopMode::Playlist => "  [⟳ all]",
                LoopMode::Off => "",
            };
            let err = model
                .playback
                .error
                .as_deref()
                .map(|e| format!("  ⚠ {e}"))
                .unwrap_or_default();
            frame.render_widget(
                Paragraph::new(format!("{status}{loop_str}{err}")).style(theme::dimmed()),
                status_area,
            );
        }
    }
}

fn draw_shortcuts(frame: &mut Frame, area: Rect, _model: &Model) {
    let text =
        " [Space] pause  [r] loop  [H/L] skip  [j/k] nav  [↵] play  [Tab] search  [q] quit";
    frame.render_widget(Paragraph::new(text).style(theme::dimmed()), area);
}
