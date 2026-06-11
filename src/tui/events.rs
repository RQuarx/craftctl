use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use crossterm::event::{self, Event, KeyEventKind, MouseButton, MouseEventKind};

use crate::{app::App, result::Result, tui};

use super::terminal::AppTerminal;

pub async fn run(
    terminal: &mut AppTerminal,
    app: &mut App,
    should_quit: Arc<AtomicBool>,
) -> Result<()> {
    drain_startup_events()?;

    loop {
        if should_quit.load(Ordering::SeqCst) {
            break;
        }

        terminal.draw(|frame| tui::screens::login::draw(frame, app))?;

        if event::poll(Duration::from_millis(80))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if app.handle_key(key).await {
                        break;
                    }
                }
                Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
                    let area = terminal.size()?.into();
                    if let Some(target) =
                        tui::screens::login::hit_test(area, mouse.column, mouse.row)
                    {
                        app.handle_click(target).await;
                    }
                }
                _ => {}
            }
        }

        app.tick().await;
    }

    Ok(())
}

fn drain_startup_events() -> Result<()> {
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read()?;
    }

    Ok(())
}
