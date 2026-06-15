use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crossterm::event::{self, Event, KeyEventKind, MouseButton, MouseEventKind};

use crate::{result::Result, tui, tui::state::TuiState};

use super::terminal::AppTerminal;

pub async fn run(
    terminal: &mut AppTerminal,
    state: &mut TuiState,
    should_quit: Arc<AtomicBool>,
) -> Result<()> {
    drain_startup_events()?;

    loop {
        if should_quit.load(Ordering::SeqCst) {
            break;
        }

        terminal.draw(|frame| match state.screen() {
            tui::state::TuiScreen::Login => tui::screens::login::draw(frame, state),
            tui::state::TuiScreen::Home => tui::screens::home::draw(frame, state),
            tui::state::TuiScreen::Settings => tui::screens::settings::draw(frame, state),
        })?;

        if event::poll(Duration::from_millis(80))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if state.handle_key(key).await? {
                        break;
                    }
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                    if !matches!(state.screen(), tui::state::TuiScreen::Login) {
                        continue;
                    }

                    let area = terminal.size()?.into();
                    if let Some(target) =
                        tui::screens::login::hit_test(area, mouse.column, mouse.row)
                    {
                        state.login.handle_click(target).await;
                    }
                }
                _ => {}
            }
        }

        state.tick().await;
    }

    Ok(())
}

fn drain_startup_events() -> Result<()> {
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read()?;
    }

    Ok(())
}
