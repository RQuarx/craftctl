use std::{io, path::Path};

use crate::mods::ModMetadata;

pub enum ModLoaderError {
    Io(io::Error),
    InvalidFormat(String),
}

pub trait ModLoader {
    fn name(&self) -> &str;
    fn get_mod_metadata(&self, jar: &Path) -> Result<ModMetadata, ModLoaderError>;
}

mod fabric;
pub use fabric::Fabric;