use thiserror::Error;

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("authentication error: {0}")]
    Auth(String),
}
