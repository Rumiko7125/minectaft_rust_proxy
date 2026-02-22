use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Varint too large")]
    VarintTooLarge,

    #[error("Invalid packet id: {0}")]
    InvalidPacketId(i32),

    #[error("Connection rejected: {0}")]
    Rejected(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("2FA error: {0}")]
    TwoFactor(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ProxyError>;
