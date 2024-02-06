pub use kuma_client::error::Error as KumaError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Kuma(#[from] KumaError),

    #[error(transparent)]
    Docker(#[from] bollard::errors::Error),

    #[error("Error while trying to parse labels: {0}")]
    LabelParseError(String),

    #[error("Unable to deserialize: {0}")]
    DeserializeError(String),

    #[error("Found invalid config '{0}': {1}")]
    InvalidConfig(String, String),

    #[error("IO error: {0}")]
    IO(String),
}

pub type Result<T> = std::result::Result<T, Error>;
