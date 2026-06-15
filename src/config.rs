use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{result::Result, storage::paths};

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LauncherConfig {
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub vim_mode: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigManager {
    root: PathBuf,
}

impl ConfigManager {
    pub fn with_default_storage() -> Result<Self> {
        Ok(Self::new(paths::config_dir()?))
    }

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn load(&self) -> Result<LauncherConfig> {
        let path = self.config_path();

        if !path.exists() {
            return Ok(LauncherConfig::default());
        }

        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }

    pub fn save(&self, config: &LauncherConfig) -> Result<()> {
        fs::create_dir_all(&self.root)?;
        fs::write(self.config_path(), toml::to_string_pretty(config)?)?;

        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn config_path(&self) -> PathBuf {
        self.root.join(CONFIG_FILE)
    }
}
