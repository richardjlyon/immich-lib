//! Error types for the Immich API client.

use thiserror::Error;

/// Errors that can occur when interacting with the Immich API.
#[derive(Error, Debug)]
pub enum ImmichError {
    /// HTTP request failed (network error, timeout, etc.)
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response
    #[error("API error {status}: {message}")]
    Api {
        /// HTTP status code
        status: u16,
        /// Error message from the API
        message: String,
    },

    /// Invalid URL format
    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),

    /// API key is malformed or invalid
    #[error("Invalid API key format")]
    InvalidApiKey,

    /// Requested asset was not found
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
}

/// Convenience type alias for Results using ImmichError.
pub type Result<T> = std::result::Result<T, ImmichError>;
