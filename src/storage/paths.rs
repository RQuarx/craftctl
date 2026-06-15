use std::path::PathBuf;

use directories::ProjectDirs;

use crate::{error::LauncherError, result::Result};

const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "craftctl";
const APPLICATION: &str = "craftctl";

pub fn config_dir() -> Result<PathBuf> {
    Ok(project_dirs()?.config_dir().to_path_buf())
}

pub fn data_local_dir() -> Result<PathBuf> {
    Ok(project_dirs()?.data_local_dir().to_path_buf())
}

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| LauncherError::Storage("could not resolve project directories".to_string()))
}
