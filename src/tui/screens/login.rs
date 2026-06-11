use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    auth::{AccountKind, LoginClickTarget, LoginMode, LoginPhase},
    tui::{state::TuiState, theme},
};

pub fn draw(frame: &mut Frame<'_>, state: &TuiState) {
    let area = frame.area();
    frame.render_widget(Clear, area);
    frame.render_widget(Block::default().style(theme::background()), area);

    let panel = centered_rect(72, 22, area);
    let block = Block::default()
        .title(" craftctl ")
        .title_alignment(Alignment::Center)
        .title_style(theme::primary())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::primary())
        .style(theme::panel());

    let inner = block.inner(panel);
    frame.render_widget(block, panel);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(inner);

    draw_title(frame, rows[0]);
    draw_modes(frame, rows[1], state.login.mode());

    match state.login.phase() {
        LoginPhase::Choosing => draw_choice(frame, rows[2], state),
        LoginPhase::WaitingForMicrosoft => draw_waiting(frame, rows[2], state),
        LoginPhase::Complete => draw_complete(frame, rows[2], state),
    }

    draw_status(frame, rows[3], state);
}

pub fn hit_test(area: Rect, column: u16, row: u16) -> Option<LoginClickTarget> {
    let layout = login_layout(area);

    if contains(layout.microsoft_button, column, row) {
        Some(LoginClickTarget::Microsoft)
    } else if contains(layout.offline_button, column, row) {
        Some(LoginClickTarget::Offline)
    } else {
        None
    }
}

fn draw_title(frame: &mut Frame<'_>, area: Rect) {
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Minecraft Java login",
            theme::muted(),
        )))
        .alignment(Alignment::Center),
        area,
    );
}

fn draw_modes(frame: &mut Frame<'_>, area: Rect, mode: LoginMode) {
    let columns = mode_buttons(area);

    draw_mode(frame, columns[0], "Microsoft", mode == LoginMode::Microsoft);
    draw_mode(frame, columns[1], "Offline", mode == LoginMode::Offline);
}

fn draw_mode(frame: &mut Frame<'_>, area: Rect, label: &str, selected: bool) {
    let style = if selected {
        theme::primary()
    } else {
        theme::muted()
    };

    let marker = if selected { ">" } else { " " };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(style)
        .style(theme::panel());

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(marker, style),
            Span::raw(" "),
            Span::styled(label, style.add_modifier(Modifier::BOLD)),
        ]))
        .block(block)
        .alignment(Alignment::Center),
        area,
    );
}

fn draw_choice(frame: &mut Frame<'_>, area: Rect, state: &TuiState) {
    match state.login.mode() {
        LoginMode::Microsoft => draw_center_message(
            frame,
            area,
            &[
                Line::from(Span::styled("Sign in with Microsoft", theme::primary())),
                Line::from(""),
                Line::from(Span::styled("Press Enter to get a code", theme::muted())),
            ],
        ),
        LoginMode::Offline => draw_offline(frame, area, state),
    }
}

fn draw_offline(frame: &mut Frame<'_>, area: Rect, state: &TuiState) {
    let username = state.login.username();
    let input = if username.is_empty() {
        "username".to_string()
    } else {
        format!("{username}_")
    };

    let input_style = if username.is_empty() {
        theme::muted()
    } else {
        theme::primary()
    };

    let box_area = centered_rect(34, 5, area);
    let block = Block::default()
        .title(" offline name ")
        .title_alignment(Alignment::Center)
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::accent())
        .style(theme::panel());

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(input, input_style)))
            .block(block)
            .alignment(Alignment::Center),
        box_area,
    );
}

fn draw_waiting(frame: &mut Frame<'_>, area: Rect, state: &TuiState) {
    let Some(login) = state.login.microsoft_login() else {
        return;
    };

    draw_center_message(
        frame,
        area,
        &[
            Line::from(Span::styled(
                login.device_code.user_code.as_str(),
                theme::primary().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                login.device_code.verification_uri.as_str(),
                theme::accent(),
            )),
            Line::from(""),
            Line::from(Span::styled("Waiting for approval...", theme::muted())),
        ],
    );
}

fn draw_complete(frame: &mut Frame<'_>, area: Rect, state: &TuiState) {
    let Some(account) = state.login.account() else {
        return;
    };

    let kind = match account.kind {
        AccountKind::Microsoft => "Microsoft",
        AccountKind::Offline => "Offline",
    };

    draw_center_message(
        frame,
        area,
        &[
            Line::from(Span::styled("Ready", theme::primary())),
            Line::from(""),
            Line::from(Span::styled(account.username.as_str(), theme::accent())),
            Line::from(Span::styled(kind, theme::muted())),
        ],
    );
}

fn draw_center_message(frame: &mut Frame<'_>, area: Rect, lines: &[Line<'_>]) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(lines.len() as u16),
            Constraint::Min(0),
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(lines.to_vec())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        vertical[1],
    );
}

fn draw_status(frame: &mut Frame<'_>, area: Rect, state: &TuiState) {
    let line = if let Some(error) = state.login.error() {
        Line::from(Span::styled(short_error(error), theme::error()))
    } else {
        Line::from(vec![
            Span::styled("Tab", theme::accent()),
            Span::raw(" switch   "),
            Span::styled("Enter", theme::accent()),
            Span::raw(" continue   "),
            Span::styled("Esc", theme::accent()),
            Span::raw(" quit"),
        ])
    };

    frame.render_widget(
        Paragraph::new(line)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn short_error(error: &str) -> String {
    const MAX_LEN: usize = 92;

    if error.chars().count() <= MAX_LEN {
        error.to_string()
    } else {
        format!("{}...", error.chars().take(MAX_LEN).collect::<String>())
    }
}

struct LoginLayout {
    microsoft_button: Rect,
    offline_button: Rect,
}

fn login_layout(area: Rect) -> LoginLayout {
    let panel = centered_rect(72, 22, area);
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(panel);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(inner);
    let buttons = mode_buttons(rows[1]);

    LoginLayout {
        microsoft_button: buttons[0],
        offline_button: buttons[1],
    }
}

fn mode_buttons(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area)
}

fn contains(area: Rect, column: u16, row: u16) -> bool {
    column >= area.x
        && column < area.x.saturating_add(area.width)
        && row >= area.y
        && row < area.y.saturating_add(area.height)
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height.min(area.height)),
            Constraint::Min(0),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(width.min(area.width)),
            Constraint::Min(0),
        ])
        .split(vertical[1]);

    horizontal[1]
}
