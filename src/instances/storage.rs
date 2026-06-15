use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{error::LauncherError, result::Result, storage::paths};

use super::{Instance, instance::is_safe_instance_id};

const INSTANCE_FILE: &str = "instance.toml";

#[derive(Debug, Clone)]
pub struct InstanceStorage {
    root: PathBuf,
}

impl InstanceStorage {
    pub fn default() -> Result<Self> {
        Ok(Self::new(paths::data_local_dir()?))
    }

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn instances_dir(&self) -> PathBuf {
        self.root.join("instances")
    }

    pub fn instance_dir(&self, id: &str) -> PathBuf {
        self.instances_dir().join(id)
    }

    pub fn exists(&self, id: &str) -> bool {
        if !is_safe_instance_id(id) {
            return false;
        }

        self.instance_config_path(id).exists()
    }

    pub fn load_all(&self) -> Result<Vec<Instance>> {
        let instances_dir = self.instances_dir();

        if !instances_dir.exists() {
            return Ok(Vec::new());
        }

        let mut instances: Vec<Instance> = Vec::new();

        for entry in fs::read_dir(instances_dir)? {
            let entry = entry?;

            if !entry.file_type()?.is_dir() {
                continue;
            }

            let path = entry.path().join(INSTANCE_FILE);

            if path.exists() {
                let contents = fs::read_to_string(path)?;
                instances.push(toml::from_str(&contents)?);
            }
        }

        instances.sort_by_cached_key(|instance| instance.name.to_ascii_lowercase());
        Ok(instances)
    }

    pub fn save(&self, instance: &Instance) -> Result<()> {
        let instance_dir = self.checked_instance_dir(&instance.id)?;

        fs::create_dir_all(instance_dir.join("minecraft").join("saves"))?;
        fs::create_dir_all(instance_dir.join("minecraft").join("resourcepacks"))?;
        fs::create_dir_all(instance_dir.join("minecraft").join("shaderpacks"))?;
        fs::create_dir_all(instance_dir.join("mods"))?;
        fs::create_dir_all(instance_dir.join("logs"))?;

        let contents = toml::to_string_pretty(instance)?;
        fs::write(instance_dir.join(INSTANCE_FILE), contents)?;

        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let instance_dir = self.checked_instance_dir(id)?;

        if instance_dir.exists() {
            fs::remove_dir_all(instance_dir)?;
        }

        Ok(())
    }

    fn instance_config_path(&self, id: &str) -> PathBuf {
        self.instance_dir(id).join(INSTANCE_FILE)
    }

    fn checked_instance_dir(&self, id: &str) -> Result<PathBuf> {
        if !is_safe_instance_id(id) {
            return Err(LauncherError::InvalidInstance(
                "id can only contain letters, numbers, hyphens, and underscores".to_string(),
            ));
        }

        Ok(self.instance_dir(id))
    }
}
