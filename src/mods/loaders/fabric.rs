use std::{fs::File, io::Read, path::Path};

use serde::Deserialize;
use zip::ZipArchive;

use crate::mods::{ModMetadata, loaders::ModLoaderError};

#[derive(Debug, Deserialize)]
struct FabricModJson {
    id: String,
    version: String,
    name: Option<String>,
}


/// A struct representing the Fabric modloader, used to load fabric mods metadata.
pub struct Fabric;

impl super::ModLoader for Fabric {
    fn name(&self) -> &str {
        "fabric"
    }

    fn get_mod_metadata(&self, jar: &Path) -> Result<ModMetadata, ModLoaderError> {
        let file = File::open(jar).map_err(ModLoaderError::Io)?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| ModLoaderError::InvalidFormat(e.to_string()))?;

        let mut mod_json = archive
            .by_name("fabric.mod.json")
            .map_err(|_| ModLoaderError::InvalidFormat("fabric.mod.json not found".into()))?;

        let mut contents = String::new();
        mod_json
            .read_to_string(&mut contents)
            .map_err(|e| ModLoaderError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let parsed: FabricModJson = serde_json::from_str(&contents)
            .map_err(|e| ModLoaderError::InvalidFormat(e.to_string()))?;

        Ok(ModMetadata::create(
            jar,
            parsed.id.clone(),
            parsed.name.unwrap_or(parsed.id),
            parsed.version,
        ))
    }
}
