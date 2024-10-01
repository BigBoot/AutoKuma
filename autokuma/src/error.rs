pub use kuma_client::error::Error as KumaError;
use thiserror::Error;

use crate::name::Name;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Kuma(#[from] KumaError),

    #[error(transparent)]
    Docker(#[from] bollard::errors::Error),

    #[error(transparent)]
    Database(#[from] sled::Error),

    #[error("Error while trying to parse labels: {0}")]
    LabelParseError(String),

    #[error("Unable to deserialize: {0}")]
    DeserializeError(String),

    #[error("Found invalid config '{0}': {1}")]
    InvalidConfig(String, String),

    #[error("IO error: {0}")]
    IO(String),

    #[error("No {} named {} could be found", .0.type_name(), .0.name())]
    NameNotFound(Name),
}

pub type Result<T> = std::result::Result<T, Error>;
