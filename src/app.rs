use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{config, http, instances, result::Result, tui};

pub async fn run() -> Result<()> {
    let client = Arc::new(http::HttpClient::new()?);
    let instance_manager = instances::InstanceManager::with_default_storage()?;
    let config_manager = config::ConfigManager::with_default_storage()?;
    let launcher_config = config_manager.load()?;
    let mut state =
        tui::state::TuiState::create(client, instance_manager, config_manager, launcher_config)?;

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
