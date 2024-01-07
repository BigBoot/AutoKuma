use thiserror::Error;

use crate::kuma;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Kuma(#[from] kuma::Error),

    #[error(transparent)]
    Docker(#[from] bollard::errors::Error),

    #[error("Error while trying to parse labels: {0}")]
    LabelParseError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
