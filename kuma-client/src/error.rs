use thiserror::Error;

/// Custom error type for handling various errors in the kuma_client library.
#[derive(Error, Debug)]
pub enum Error {
    /// The config contains an invalid url.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Connection timeout error while trying to connect to the Uptime Kuma server.
    #[error("Timeout while trying to connect to Uptime Kuma server")]
    ConnectionTimeout,

    /// Timeout error while trying to call a specific function.
    #[error("Timeout while trying to call '{0}'.")]
    CallTimeout(String),

    /// Attempt to access Uptime Kuma state before it was ready.
    #[error("Tried to access Uptime Kuma state before it was ready...")]
    NotReady,

    /// The login details were rejected by the server.
    #[error("The server rejected the login: {0}")]
    LoginError(String),

    /// Error when the server expects a username/password, but none was provided.
    #[error("It looks like the server is expecting a username/password, but none was provided")]
    NotAuthenticated,

    /// Connection loss to Uptime Kuma.
    #[error("Connection to Uptime Kuma was lost")]
    Disconnected,

    /// Invalid response from the server with a missing key.
    #[error("Received invalid response from server (missing key '{1}'): {0:?}")]
    InvalidResponse(Vec<serde_json::Value>, String),

    /// The server responded with an unexpected error.
    #[error("Server responded with an error: {0}")]
    ServerError(String),

    /// Unsupported message received from the server.
    #[error("Received unsupported message from server")]
    UnsupportedResponse,

    /// Communication error.
    #[error("Error during communication: {0}")]
    CommunicationError(String),

    /// Validation error with a field name and a list of validation errors.
    #[error("Encountered errors trying to validate '{0}': {1:?}")]
    ValidationError(String, Vec<String>),

    /// Error when a group with a specific name is not found.
    #[error("No group named {0} could be found")]
    GroupNotFound(String),

    /// Error when an entity with a specific ID is not found.
    #[error("No {0} with ID {1} could be found")]
    IdNotFound(String, i32),

    /// Error when an entity with a specific slug is not found.
    #[error("No {0} with slug {1} could be found")]
    SlugNotFound(String, String),

    /// Wrapper for an underlying reqwest error.
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

/// Custom result type for handling various errors in the kuma_client library.
pub type Result<T> = std::result::Result<T, Error>;
