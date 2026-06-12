use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{result::Result, tui};

pub async fn run() -> Result<()> {
    let mut terminal = tui::terminal::init()?;
    let should_quit = Arc::new(AtomicBool::new(false));
    install_ctrlc_handler(Arc::clone(&should_quit));

    let mut state = tui::state::TuiState::new();
    let result = tui::events::run(&mut terminal, &mut state, should_quit).await;

    tui::terminal::restore(&mut terminal)?;
    result
}

fn install_ctrlc_handler(should_quit: Arc<AtomicBool>) {
    let _ = ctrlc::set_handler(move || {
        should_quit.store(true, Ordering::SeqCst);
    });
}
