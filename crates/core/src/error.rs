use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("{0}")]
    Message(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("template error: {0}")]
    Template(#[from] minijinja::Error),
}

pub type CoreResult<T> = Result<T, CoreError>;

impl CoreError {
    pub fn msg(s: impl Into<String>) -> Self {
        Self::Message(s.into())
    }
}
