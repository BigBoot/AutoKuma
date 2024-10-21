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

    #[cfg(feature = "kubernetes")]
    #[error(transparent)]
    K8S(#[from] K8SError),

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

#[cfg(feature = "kubernetes")]
#[derive(Error, Debug)]
pub enum K8SError {
    #[error("Finalizer Error: {0}")]
    FinalizerError(#[source] Box<kube::runtime::finalizer::Error<Error>>),
}

pub type Result<T> = std::result::Result<T, Error>;
