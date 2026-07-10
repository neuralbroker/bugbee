use thiserror::Error;

pub type Result<T> = std::result::Result<T, BugbeeError>;

#[derive(Debug, Error)]
pub enum BugbeeError {
    #[error("config error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("database error: {0}")]
    Db(String),

    #[error("provider error: {0}")]
    Provider(String),

    #[error("engine error: {0}")]
    Engine(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

impl From<rusqlite::Error> for BugbeeError {
    fn from(e: rusqlite::Error) -> Self {
        BugbeeError::Db(e.to_string())
    }
}

impl From<anyhow::Error> for BugbeeError {
    fn from(e: anyhow::Error) -> Self {
        BugbeeError::Other(e.to_string())
    }
}
