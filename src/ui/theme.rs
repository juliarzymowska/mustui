use ratatui::style::{Color, Modifier, Style};

pub fn normal() -> Style {
    Style::default()
}

pub fn bold() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}

pub fn dimmed() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

pub fn reversed() -> Style {
    Style::default().add_modifier(Modifier::REVERSED)
}

/// Accent colour used for the active/focused panel border.
pub fn accent() -> Style {
    Style::default().fg(Color::Cyan)
}
