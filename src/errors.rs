//! Typed errors returned by the Supabase client.

use thiserror::Error;

/// A PostgREST or Supabase API error response.
#[derive(Debug, Error)]
#[error("Supabase API error ({status}): {message}")]
pub struct ApiError {
    /// HTTP status returned by the API.
    pub status: u16,
    /// PostgREST or PostgreSQL error code, when supplied.
    pub code: Option<String>,
    /// Provider error message, or a status-based fallback.
    pub message: String,
    /// Additional provider details.
    pub details: Option<String>,
    /// Provider hint for resolving the error.
    pub hint: Option<String>,
    /// Up to 8 KiB of the original response body for malformed provider errors.
    pub raw_response: Option<String>,
}

/// Errors returned by public Supabase SDK operations.
#[derive(Debug, Error)]
pub enum Error {
    /// The HTTP request failed before a response was received.
    #[error("HTTP transport error: {0}")]
    Transport(#[from] reqwest::Error),
    /// A request or response could not be serialized as JSON.
    #[error("JSON error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// A local filesystem operation failed.
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    /// Supabase returned a non-success HTTP response.
    #[error(transparent)]
    Api(Box<ApiError>),
    /// A caller-provided value is invalid.
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// Client configuration is invalid.
    #[error("invalid configuration: {0}")]
    Configuration(String),
    /// A successful HTTP response violated the expected response contract.
    #[error("unexpected response: {message}")]
    UnexpectedResponse {
        /// Explanation of the response mismatch.
        message: String,
        /// Status code, when available.
        status: Option<u16>,
        /// Bounded response body, when available.
        body: Option<String>,
    },
}

/// Result type used by public SDK operations.
pub type Result<T> = std::result::Result<T, Error>;

impl From<ApiError> for Error {
    fn from(error: ApiError) -> Self {
        Self::Api(Box::new(error))
    }
}

impl From<std::env::VarError> for Error {
    fn from(error: std::env::VarError) -> Self {
        Self::Configuration(error.to_string())
    }
}
