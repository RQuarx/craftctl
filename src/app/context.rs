use std::sync::Arc;

use crate::{
    config::{ConfigManager, LauncherConfig},
    http::HttpClient,
    instances::InstanceManager,
    result::Result,
};

pub(super) struct AppContext {
    pub(super) client: Arc<HttpClient>,
    pub(super) instance_manager: InstanceManager,
    pub(super) config_manager: ConfigManager,
    pub(super) launcher_config: LauncherConfig,
}

impl AppContext {
    pub(super) fn load() -> Result<Self> {
        let client = Arc::new(HttpClient::new()?);
        let instance_manager = InstanceManager::with_default_storage()?;
        let config_manager = ConfigManager::with_default_storage()?;
        let launcher_config = config_manager.load()?;

        Ok(Self {
            client,
            instance_manager,
            config_manager,
            launcher_config,
        })
    }
}
