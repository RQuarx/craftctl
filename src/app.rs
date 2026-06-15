use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

mod context;

use context::AppContext;

use crate::{result::Result, tui};

pub async fn run() -> Result<()> {
    let context = AppContext::load()?;
    let mut state = tui::state::TuiState::create(
        context.client,
        context.instance_manager,
        context.config_manager,
        context.launcher_config,
    )?;

    let mut terminal = tui::terminal::init()?;
    let should_quit = Arc::new(AtomicBool::new(false));
    install_ctrlc_handler(Arc::clone(&should_quit));

    let result = tui::events::run(&mut terminal, &mut state, should_quit).await;

    tui::terminal::restore(&mut terminal)?;
    result
}

fn install_ctrlc_handler(should_quit: Arc<AtomicBool>) {
    let _ = ctrlc::set_handler(move || {
        should_quit.store(true, Ordering::SeqCst);
    });
}
