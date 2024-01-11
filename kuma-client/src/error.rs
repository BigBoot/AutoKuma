use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Timeout while trying to connect to Uptime Kuma server")]
    ConnectionTimeout,

    #[error("Timeout while trying to call '{0}'.")]
    CallTimeout(String),

    #[error("Tried to access Uptime Kuma state before it was ready...")]
    NotReady,

    #[error("The server rejected the login: {0}")]
    LoginError(String),

    #[error("It looks like the server is expecting a username/password, but none was provided")]
    NotAuthenticated,

    #[error("Connection to Uptime Kuma was lost")]
    Disconnected,

    #[error("Received invalid response from server (missing key '{1}'): {0:?}")]
    InvalidResponse(Vec<serde_json::Value>, String),

    #[error("Server responded with an error: {0}")]
    ServerError(String),

    #[error("Received unsupported message from server")]
    UnsupportedResponse,

    #[error("Error during communication: {0}")]
    CommunicationError(String),

    #[error("Encountered Errors trying to validate '{0}': {1:?}")]
    ValidationError(String, Vec<String>),

    #[error("No group named {0} could be found")]
    GroupNotFound(String),

    #[error("No {0} with id {1} could be found")]
    IdNotFound(String, i32),
}

pub type Result<T> = std::result::Result<T, Error>;
