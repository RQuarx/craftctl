use thiserror::Error;

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("authentication error: {0}")]
    Auth(String),

    #[error("config decode error: {0}")]
    TomlDecode(#[from] toml::de::Error),

    #[error("config encode error: {0}")]
    TomlEncode(#[from] toml::ser::Error),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("invalid instance: {0}")]
    InvalidInstance(String),
}
