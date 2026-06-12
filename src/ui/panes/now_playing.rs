use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::{app::App, messages::{LoopMode, PlaybackStatus}, ui::theme};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::new().borders(Borders::ALL).title(" Now Playing ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 4 {
        return;
    }

    let [title_area, meta_area, progress_area, status_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner);

    match &app.player_state.current {
        None => {
            frame.render_widget(
                Paragraph::new("Nothing playing").style(theme::dimmed()),
                title_area,
            );
        }
        Some(track) => {
            frame.render_widget(
                Paragraph::new(track.title.clone()).style(theme::bold()),
                title_area,
            );

            let meta = match &track.album {
                Some(album) => format!("{} · {}", track.artist, album),
                None => track.artist.clone(),
            };
            frame.render_widget(Paragraph::new(meta).style(theme::dimmed()), meta_area);

            let (ratio, label) = match track.duration {
                Some(dur) if !dur.is_zero() => {
                    let pos = app.player_state.position;
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
                _ => (0.0, String::from("--:-- / --:--")),
            };

            frame.render_widget(
                Gauge::default()
                    .ratio(ratio)
                    .label(label)
                    .style(theme::normal())
                    .gauge_style(theme::reversed()),
                progress_area,
            );
        }
    }

    let status_str = match app.player_state.status {
        PlaybackStatus::Playing => ">> Playing",
        PlaybackStatus::Paused => "|| Paused",
        PlaybackStatus::Loading => ".. Loading",
        PlaybackStatus::Idle => "   Idle",
    };
    let loop_str = match app.player_state.loop_mode {
        LoopMode::One => "  [loop: on]",
        LoopMode::Off => "",
    };
    frame.render_widget(
        Paragraph::new(format!("{status_str}{loop_str}")).style(theme::dimmed()),
        status_area,
    );
}
