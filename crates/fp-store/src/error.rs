use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not serialise: {0}")]
    Serialize(String),
}
