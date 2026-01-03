use ratatui::style::{Color, Style};

pub fn style_focus() -> Style {
    Style::default().fg(Color::Yellow)
}

pub fn style_normal() -> Style {
    Style::default()
}
