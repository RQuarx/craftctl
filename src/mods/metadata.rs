use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ModMetadata {
    path: PathBuf,
    id: String,
    name: String,
    version: String,
    description: Option<String>,
}

impl ModMetadata {
    pub fn create(
        path: &Path,
        id: String,
        name: String,
        version: String,
        description: Option<String>,
    ) -> ModMetadata {
        ModMetadata {
            path: PathBuf::from(path),
            id,
            name,
            version,
            description,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn version(&self) -> &String {
        &self.version
    }

    pub fn description(&self) -> &Option<String> {
        &self.description
    }
}
