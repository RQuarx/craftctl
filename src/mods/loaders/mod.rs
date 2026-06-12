use std::{io, path::Path};

use crate::mods::ModMetadata;

#[derive(Debug)]
pub enum ModLoaderError {
    Io(io::Error),
    InvalidFormat(String),
}

pub trait ModLoader {
    fn name() -> &'static str;
    fn get_mod_metadata(jar: &Path) -> Result<ModMetadata, ModLoaderError>;
}

mod fabric;
mod forge;
mod neoforge;
pub use fabric::Fabric;
pub use forge::Forge;
pub use neoforge::NeoForge;