use std::{collections::HashMap, fs::File, io::Read};

use serde::Deserialize;
use zip::ZipArchive;

use crate::mods::{
    ModMetadata,
    loader::{ModLoader, ModLoaderError},
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NeoForgeModTomlEntry {
    mod_id: String,
    display_name: Option<String>,
    version: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NeoForgeModToml {
    mods: Vec<NeoForgeModTomlEntry>,
}

/// A struct representing the NeoForge modloader, used to load NeoForge mod's metadata.
pub struct NeoForge;

impl ModLoader for NeoForge {
    fn name() -> &'static str {
        "NeoForge"
    }

    fn get_mod_metadata(jar: &std::path::Path) -> Result<ModMetadata, ModLoaderError> {
        let file = File::open(jar).map_err(ModLoaderError::Io)?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| ModLoaderError::InvalidFormat(e.to_string()))?;

        let manifest = read_manifest(&mut archive);

        // NeoForge 1.13+
        let mut mods_toml = archive
            .by_name("META-INF/mods.toml")
            .map_err(|_| ModLoaderError::InvalidFormat("META-INF/mods.toml not found".into()))?;

        let mut contents = String::new();
        mods_toml
            .read_to_string(&mut contents)
            .map_err(|e| ModLoaderError::Io(std::io::Error::other(e)))?;

        let toml: NeoForgeModTomlEntry = parse_mods_toml(&contents, &manifest)?;
        Ok(ModMetadata::create(
            jar,
            toml.mod_id.clone(),
            toml.display_name.unwrap_or(toml.mod_id),
            toml.version.unwrap(),
            toml.description,
        ))
    }
}

fn parse_mods_toml(
    contents: &str,
    manifest: &HashMap<String, String>,
) -> Result<NeoForgeModTomlEntry, ModLoaderError> {
    let parsed: NeoForgeModToml =
        toml::from_str(contents).map_err(|e| ModLoaderError::InvalidFormat(e.to_string()))?;

    // pick first mod that isn't "minecraft"
    let mut entry = parsed
        .mods
        .into_iter()
        .find(|m| m.mod_id != "minecraft")
        .ok_or_else(|| ModLoaderError::InvalidFormat("mods.toml contains no valid mods".into()))?;

    entry.version = Some(resolve_version(entry.version, manifest));
    Ok(entry)
}

fn parse_manifest(contents: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in contents.lines() {
        if let Some((key, value)) = line.split_once(':') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    map
}

fn read_manifest(archive: &mut ZipArchive<File>) -> HashMap<String, String> {
    let Ok(mut file) = archive.by_name("META-INF/MANIFEST.MF") else {
        return HashMap::new();
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_ok() {
        parse_manifest(&contents)
    } else {
        HashMap::new()
    }
}

fn manifest_version(manifest: &HashMap<String, String>) -> Option<String> {
    [
        "Implementation-Version",
        "Specification-Version",
        "ImplementationVersion",
    ]
    .into_iter()
    .find_map(|k| manifest.get(k).cloned())
}

fn resolve_version(version: Option<String>, manifest: &HashMap<String, String>) -> String {
    let Some(version) = version else {
        return manifest_version(manifest).unwrap_or_else(|| "unknown".into());
    };

    if let Some(inner) = version.strip_prefix("${").and_then(|s| s.strip_suffix('}')) {
        match inner {
            "file.jarVersion" | "version" => manifest_version(manifest).unwrap_or(version),
            _ => manifest.get(inner).cloned().unwrap_or(version),
        }
    } else {
        version
    }
}
