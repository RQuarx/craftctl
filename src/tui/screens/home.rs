use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::{
    auth::AccountKind,
    tui::{
        state::{CreateField, HomeMode, HomeState, TuiState},
        theme,
    },
};

pub fn draw(frame: &mut Frame<'_>, state: &TuiState) {
    let area = frame.area();

    frame.render_widget(Clear, area);
    frame.render_widget(Block::default().style(theme::background()), area);

    let block = Block::default()
        .title(" craftctl ")
        .title_alignment(Alignment::Center)
        .title_style(theme::primary())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::primary())
        .style(theme::panel());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(inner);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(6)])
        .split(columns[0]);

    draw_instances(frame, left[0], &state.home);
    draw_keybinds(frame, left[1], &state.home, state.config.ui.vim_mode);
    draw_instance_info(frame, columns[1], state);

    match state.home.mode {
        HomeMode::Creating => draw_instance_form_dialog(frame, area, &state.home, " New instance "),
        HomeMode::Editing => draw_instance_form_dialog(frame, area, &state.home, " Edit instance "),
        HomeMode::ConfirmingDelete => draw_delete_dialog(frame, area, &state.home),
        HomeMode::Browsing => {}
    }
}

fn draw_instances(frame: &mut Frame<'_>, area: Rect, home: &HomeState) {
    let items = if home.instances.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "No instances yet",
            theme::muted(),
        )))]
    } else {
        home.instances
            .iter()
            .enumerate()
            .map(|(index, instance)| {
                let selected = index == home.selected;
                let style = if selected {
                    theme::primary()
                } else {
                    theme::muted()
                };
                let marker = if selected { "#" } else { " " };

                ListItem::new(Line::from(vec![
                    Span::styled(marker, style),
                    Span::raw(" "),
                    Span::styled(instance.name.as_str(), style.add_modifier(Modifier::BOLD)),
                    Span::raw("  "),
                    Span::styled(instance.summary(), theme::muted()),
                ]))
            })
            .collect()
    };

    let block = Block::default()
        .title(" Local instances ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::muted())
        .style(theme::panel());

    frame.render_widget(List::new(items).block(block), area);
}

fn draw_keybinds(frame: &mut Frame<'_>, area: Rect, home: &HomeState, vim_mode: bool) {
    let keybinds = match home.mode {
        HomeMode::Browsing => browsing_keybinds(vim_mode),
        HomeMode::Creating => vec![
            keybind_line(&[
                ("Tab", "field"),
                ("Left/Right", "loader"),
                ("Enter", "next/create"),
            ]),
            keybind_line(&[("Esc", "cancel")]),
        ],
        HomeMode::Editing => vec![
            keybind_line(&[
                ("Tab", "field"),
                ("Left/Right", "loader"),
                ("Enter", "next/save"),
            ]),
            keybind_line(&[("Esc", "cancel")]),
        ],
        HomeMode::ConfirmingDelete => vec![keybind_line(&[("Y", "delete"), ("N/Esc", "cancel")])],
    };

    let status = home.status().map(|status| {
        if home.status_is_error() {
            Line::from(Span::styled(status.to_string(), theme::error()))
        } else {
            Line::from(Span::styled(status.to_string(), theme::muted()))
        }
    });

    let mut lines = keybinds;
    if let Some(status) = status {
        lines.push(status);
    }

    let block = Block::default()
        .title(" Keys ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::muted())
        .style(theme::panel());

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn browsing_keybinds(vim_mode: bool) -> Vec<Line<'static>> {
    if vim_mode {
        vec![
            keybind_line(&[
                ("Up/Down j/k", "select"),
                ("Enter/l", "launch"),
                ("N", "new"),
            ]),
            keybind_line(&[
                ("E", "edit"),
                ("R", "refresh"),
                ("D", "delete"),
                ("S", "settings"),
                ("Q/Esc", "quit"),
            ]),
        ]
    } else {
        vec![
            keybind_line(&[
                ("Up/Down", "select"),
                ("Enter", "launch"),
                ("N", "new"),
                ("E", "edit"),
            ]),
            keybind_line(&[
                ("R", "refresh"),
                ("D", "delete"),
                ("S", "settings"),
                ("Q/Esc", "quit"),
            ]),
        ]
    }
}

fn draw_instance_info(frame: &mut Frame<'_>, area: Rect, state: &TuiState) {
    let mut lines = Vec::new();

    if let Some(account) = state.login.account() {
        let kind = match account.kind {
            AccountKind::Microsoft => "Microsoft",
            AccountKind::Offline => "Offline",
        };

        lines.push(Line::from(vec![
            Span::styled("Account ", theme::muted()),
            Span::styled(account.username.as_str(), theme::accent()),
            Span::styled(format!(" ({kind})"), theme::muted()),
        ]));
        lines.push(Line::from(""));
    }

    if let Some(instance) = state.home.selected_instance() {
        lines.extend([
            Line::from(Span::styled(instance.name.as_str(), theme::primary())),
            Line::from(""),
            info_line("ID", &instance.id),
            info_line("Minecraft", &instance.minecraft_version),
            info_line("Loader", instance.loader.display_name()),
            info_line("Loader version", &instance.loader_version),
            info_line("Java", &instance.profile.java_runtime),
            info_line(
                "Memory",
                &format!(
                    "{}-{} MB",
                    instance.profile.memory_min_mb, instance.profile.memory_max_mb
                ),
            ),
        ]);
    } else {
        lines.extend([
            Line::from(Span::styled("No instance selected", theme::primary())),
            Line::from(""),
            Line::from(Span::styled(
                "Press N to create your first instance.",
                theme::muted(),
            )),
        ]);
    }

    lines.push(Line::from(""));
    lines.push(info_line(
        "Data",
        &state.home.data_root().display().to_string(),
    ));

    let block = Block::default()
        .title(" Instance info ")
        .title_style(theme::accent())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::muted())
        .style(theme::panel());

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_instance_form_dialog(
    frame: &mut Frame<'_>,
    area: Rect,
    home: &HomeState,
    title: &'static str,
) {
    let dialog = centered_rect(68, 16, area);
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .title_style(theme::primary())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::primary())
        .style(theme::panel());

    let mut lines = vec![
        form_line(
            "Name",
            value_or_placeholder(&home.instance_form.name, "My instance"),
            home.instance_form.field == CreateField::Name,
            home.instance_form.name.is_empty(),
        ),
        form_line(
            "Minecraft",
            value_or_placeholder(&home.instance_form.minecraft_version, "latest-release"),
            home.instance_form.field == CreateField::MinecraftVersion,
            home.instance_form.minecraft_version.is_empty(),
        ),
        form_line(
            "Loader",
            home.instance_form.loader.display_name().to_string(),
            home.instance_form.field == CreateField::Loader,
            false,
        ),
        form_line(
            "Loader version",
            value_or_placeholder(&home.instance_form.loader_version, "latest"),
            home.instance_form.field == CreateField::LoaderVersion,
            home.instance_form.loader_version.is_empty(),
        ),
        Line::from(""),
        Line::from(Span::styled(
            "Enter advances fields. Submit from loader version.",
            theme::muted(),
        )),
    ];

    if let Some(status) = home.status() {
        lines.push(Line::from(""));
        let style = if home.status_is_error() {
            theme::error()
        } else {
            theme::muted()
        };
        lines.push(Line::from(Span::styled(status.to_string(), style)));
    }

    frame.render_widget(Clear, dialog);
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        dialog,
    );
}

fn draw_delete_dialog(frame: &mut Frame<'_>, area: Rect, home: &HomeState) {
    let dialog = centered_rect(58, 9, area);
    let block = Block::default()
        .title(" Delete instance ")
        .title_alignment(Alignment::Center)
        .title_style(theme::error())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::error())
        .style(theme::panel());

    let name = home
        .pending_delete
        .as_ref()
        .map(|instance| instance.name.as_str())
        .unwrap_or("selected instance");

    let mut lines = vec![
        Line::from(vec![
            Span::raw("Delete "),
            Span::styled(name.to_string(), theme::primary()),
            Span::raw("?"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "This removes the instance directory.",
            theme::muted(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Y", theme::error()),
            Span::raw(" delete   "),
            Span::styled("N/Esc", theme::accent()),
            Span::raw(" cancel"),
        ]),
    ];

    push_status(&mut lines, home);

    frame.render_widget(Clear, dialog);
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        dialog,
    );
}

fn keybind_line(bindings: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();

    for (index, (key, label)) in bindings.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw("   "));
        }

        spans.push(Span::styled((*key).to_string(), theme::accent()));
        spans.push(Span::raw(format!(" {label}")));
    }

    Line::from(spans)
}

fn push_status(lines: &mut Vec<Line<'static>>, home: &HomeState) {
    let Some(status) = home.status() else {
        return;
    };

    let style = if home.status_is_error() {
        theme::error()
    } else {
        theme::muted()
    };

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(status.to_string(), style)));
}

fn info_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme::muted()),
        Span::raw(value.to_string()),
    ])
}

fn form_line(label: &str, value: String, selected: bool, placeholder: bool) -> Line<'static> {
    let marker = if selected { ">" } else { " " };
    let label_style = if selected {
        theme::accent()
    } else {
        theme::muted()
    };
    let value_style = if placeholder {
        theme::muted()
    } else if selected {
        theme::primary()
    } else {
        theme::panel()
    };

    Line::from(vec![
        Span::styled(marker.to_string(), label_style),
        Span::raw(" "),
        Span::styled(format!("{label:<16}"), label_style),
        Span::styled(value, value_style),
    ])
}

fn value_or_placeholder(value: &str, placeholder: &str) -> String {
    if value.is_empty() {
        placeholder.to_string()
    } else {
        value.to_string()
    }
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
