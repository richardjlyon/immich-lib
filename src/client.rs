//! HTTP client wrapper for the Immich API.

use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};
use serde::de::DeserializeOwned;
use std::time::Duration;
use url::Url;

use crate::error::{ImmichError, Result};
use crate::models::DuplicateGroup;

/// Client for interacting with the Immich REST API.
///
/// Handles authentication via API key and provides typed methods for API endpoints.
///
/// # Example
///
/// ```no_run
/// use immich_lib::ImmichClient;
///
/// # async fn example() -> immich_lib::Result<()> {
/// let client = ImmichClient::new("https://immich.example.com", "your-api-key")?;
/// let duplicates = client.get_duplicates().await?;
/// println!("Found {} duplicate groups", duplicates.len());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ImmichClient {
    /// HTTP client with default headers (API key) configured
    client: reqwest::Client,
    /// Base URL of the Immich server
    base_url: Url,
}

impl ImmichClient {
    /// Creates a new ImmichClient with the given base URL and API key.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the Immich server (e.g., `https://immich.example.com`)
    /// * `api_key` - The API key for authentication (created in Immich web UI)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The base_url is not a valid URL
    /// - The api_key is empty or contains invalid characters
    /// - The HTTP client cannot be built
    pub fn new(base_url: &str, api_key: &str) -> Result<Self> {
        // Validate API key
        if api_key.is_empty() {
            return Err(ImmichError::InvalidApiKey);
        }

        // Parse base URL
        let base_url = Url::parse(base_url)?;

        // Build default headers with API key
        let mut headers = HeaderMap::new();
        let header_value = HeaderValue::from_str(api_key).map_err(|_: InvalidHeaderValue| {
            ImmichError::InvalidApiKey
        })?;
        headers.insert("x-api-key", header_value);

        // Build HTTP client with defaults
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self { client, base_url })
    }

    /// Fetches all duplicate groups from the Immich server.
    ///
    /// # Returns
    ///
    /// A vector of duplicate groups, each containing assets that Immich has
    /// identified as duplicates. Returns an empty vector if no duplicates exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP request fails (network error, timeout)
    /// - The server returns an error response (401 unauthorized, etc.)
    /// - The response cannot be parsed as JSON
    pub async fn get_duplicates(&self) -> Result<Vec<DuplicateGroup>> {
        let url = self.base_url.join("/api/duplicates")?;
        let response = self.client.get(url).send().await?;
        self.handle_response(response).await
    }

    /// Handles an HTTP response, parsing success responses or extracting error details.
    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(ImmichError::Api {
                status: status.as_u16(),
                message: body,
            })
        }
    }
}
