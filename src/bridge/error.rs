use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("bridge config error: {0}")]
    Config(String),

    #[error("bridge request failed: {0}")]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, BridgeError>;
