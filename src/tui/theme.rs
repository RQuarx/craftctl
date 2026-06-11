use ratatui::style::{Color, Modifier, Style};

pub fn background() -> Style {
    Style::default()
        .bg(Color::Rgb(9, 12, 16))
        .fg(Color::Rgb(226, 232, 240))
}

pub fn panel() -> Style {
    Style::default()
        .bg(Color::Rgb(15, 23, 32))
        .fg(Color::Rgb(226, 232, 240))
}

pub fn muted() -> Style {
    Style::default().fg(Color::Rgb(148, 163, 184))
}

pub fn primary() -> Style {
    Style::default()
        .fg(Color::Rgb(122, 255, 178))
        .add_modifier(Modifier::BOLD)
}

pub fn accent() -> Style {
    Style::default()
        .fg(Color::Rgb(93, 214, 255))
        .add_modifier(Modifier::BOLD)
}

pub fn error() -> Style {
    Style::default()
        .fg(Color::Rgb(255, 117, 140))
        .add_modifier(Modifier::BOLD)
}
