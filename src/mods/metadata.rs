use std::path::{Path, PathBuf};

pub struct ModMetadata {
    path: PathBuf,
    id: String,
    name: String,
    version: String,
}

impl ModMetadata {
    pub fn create(path: &Path, id: String, name: String, version: String) -> ModMetadata {
        ModMetadata {
            path: PathBuf::from(path),
            id: id.to_owned(),
            name: name.to_owned(),
            version: version.to_owned(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}
