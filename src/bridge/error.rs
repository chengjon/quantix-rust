use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("bridge config error: {0}")]
    Config(String),

    #[error("bridge request timed out: {0}")]
    Timeout(String),

    #[error("bridge unavailable: {0}")]
    Unavailable(String),

    #[error("bridge authorization failed: {0}")]
    Unauthorized(String),

    #[error("bridge unsupported contract version: {0}")]
    UnsupportedContractVersion(String),

    #[error("bridge unsupported method: {0}")]
    UnsupportedMethod(String),

    #[error("bridge invalid result: {0}")]
    InvalidResult(String),

    #[error("bridge protocol error: {0}")]
    Protocol(String),

    #[error("bridge request failed: {0}")]
    Http(String),
}

impl From<reqwest::Error> for BridgeError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            return Self::Timeout(error.to_string());
        }

        if error.is_connect() {
            return Self::Unavailable(error.to_string());
        }

        if let Some(status) = error.status() {
            if matches!(
                status,
                reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN
            ) {
                return Self::Unauthorized(error.to_string());
            }

            if status.is_server_error() {
                return Self::Unavailable(error.to_string());
            }
        }

        Self::Http(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, BridgeError>;
