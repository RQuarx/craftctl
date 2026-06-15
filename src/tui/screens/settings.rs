use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use crate::tui::{state::TuiState, theme};

pub fn draw(frame: &mut Frame<'_>, state: &TuiState) {
    let area = frame.area();

    frame.render_widget(Clear, area);
    frame.render_widget(Block::default().style(theme::background()), area);

    let block = Block::default()
        .title(" launcher settings ")
        .title_alignment(Alignment::Center)
        .title_style(theme::primary())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::primary())
        .style(theme::panel());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(5)])
        .split(inner);

    draw_settings(frame, rows[0], state);
    draw_keys(frame, rows[1], state);
}

fn draw_settings(frame: &mut Frame<'_>, area: ratatui::layout::Rect, state: &TuiState) {
    let checkbox = if state.config.ui.vim_mode {
        "[x]"
    } else {
        "[ ]"
    };
    let status = if state.config.ui.vim_mode {
        "enabled"
    } else {
        "disabled"
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(">", theme::accent()),
            Span::raw(" "),
            Span::styled(checkbox, theme::primary()),
            Span::raw(" "),
            Span::styled(
                "Vim mode (hjkl)",
                theme::primary().add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {status}"), theme::muted()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Config ", theme::muted()),
            Span::raw(state.settings.config_root.clone()),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_keys(frame: &mut Frame<'_>, area: ratatui::layout::Rect, state: &TuiState) {
    let mut lines = vec![Line::from(vec![
        Span::styled("Space/Enter", theme::accent()),
        Span::raw(" toggle   "),
        Span::styled("Esc/Q", theme::accent()),
        Span::raw(" back"),
    ])];

    if let Some(status) = state.settings.status() {
        lines.push(Line::from(Span::styled(status.to_string(), theme::muted())));
    }

    let block = Block::default()
        .title(" Keys ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::muted())
        .style(theme::panel());

    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}
