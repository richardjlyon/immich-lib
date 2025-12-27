//! HTTP client wrapper for the Immich API.

use chrono::{DateTime, Utc};
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};
use reqwest::multipart::{Form, Part};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::error::{ImmichError, Result};
use crate::models::{AssetResponse, DuplicateGroup};

/// Response from the Immich upload endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResponse {
    /// The ID of the newly created asset
    pub id: String,
    /// Whether this was a duplicate of an existing asset
    #[serde(default)]
    pub duplicate: bool,
}

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

    /// Fetches a single asset by ID.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The ID of the asset to fetch
    ///
    /// # Returns
    ///
    /// The asset with its EXIF metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP request fails (network error, timeout)
    /// - The server returns an error response (401 unauthorized, 404 not found)
    /// - The response cannot be parsed as JSON
    pub async fn get_asset(&self, asset_id: &str) -> Result<AssetResponse> {
        let url = self.base_url.join(&format!("/api/assets/{}", asset_id))?;
        let response = self.client.get(url).send().await?;
        self.handle_response(response).await
    }

    /// Downloads an asset's original file to the specified path.
    ///
    /// Uses streaming to avoid buffering the entire file in memory,
    /// making it suitable for large files.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The ID of the asset to download
    /// * `path` - The destination path to save the file
    ///
    /// # Returns
    ///
    /// The total number of bytes written to the file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP request fails
    /// - The server returns an error response
    /// - The file cannot be created or written to
    pub async fn download_asset(&self, asset_id: &str, path: &Path) -> Result<u64> {
        let url = self
            .base_url
            .join(&format!("/api/assets/{}/original", asset_id))?;
        let response = self.client.get(url).send().await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ImmichError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        let mut file = tokio::fs::File::create(path).await?;
        let mut stream = response.bytes_stream();
        let mut bytes_written: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            bytes_written += chunk.len() as u64;
        }

        file.flush().await?;
        Ok(bytes_written)
    }

    /// Deletes multiple assets in a single API call.
    ///
    /// # Arguments
    ///
    /// * `asset_ids` - The IDs of the assets to delete
    /// * `force` - If true, permanently delete; if false, move to trash
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP request fails
    /// - The server returns an error response
    pub async fn delete_assets(&self, asset_ids: &[String], force: bool) -> Result<()> {
        #[derive(Serialize)]
        struct DeleteRequest<'a> {
            ids: &'a [String],
            force: bool,
        }

        let url = self.base_url.join("/api/assets")?;
        let body = DeleteRequest {
            ids: asset_ids,
            force,
        };

        let response = self.client.delete(url).json(&body).send().await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ImmichError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        Ok(())
    }

    /// Updates an asset's metadata fields.
    ///
    /// This method allows updating GPS coordinates, date/time, and description
    /// for an asset. Only non-None fields will be sent in the update request.
    ///
    /// # Arguments
    ///
    /// * `asset_id` - The ID of the asset to update
    /// * `latitude` - New GPS latitude (optional)
    /// * `longitude` - New GPS longitude (optional)
    /// * `date_time_original` - New original date/time as ISO 8601 string (optional)
    /// * `description` - New description (optional)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP request fails
    /// - The server returns an error response
    pub async fn update_asset_metadata(
        &self,
        asset_id: &str,
        latitude: Option<f64>,
        longitude: Option<f64>,
        date_time_original: Option<&str>,
        description: Option<&str>,
    ) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct UpdateRequest<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            latitude: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            longitude: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            date_time_original: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
        }

        let url = self.base_url.join(&format!("/api/assets/{}", asset_id))?;
        let body = UpdateRequest {
            latitude,
            longitude,
            date_time_original,
            description,
        };

        let response = self.client.put(url).json(&body).send().await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ImmichError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        Ok(())
    }

    /// Uploads a file to Immich as a new asset.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to upload
    ///
    /// # Returns
    ///
    /// Information about the uploaded asset including its new ID.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The HTTP request fails
    /// - The server returns an error response
    pub async fn upload_asset(&self, file_path: &Path) -> Result<UploadResponse> {
        // Read file content
        let file_content = tokio::fs::read(file_path).await?;

        // Extract filename - strip asset ID prefix if present (format: {uuid}_{original})
        let original_filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|name| {
                // Check if filename starts with UUID pattern (8-4-4-4-12 chars + underscore)
                if name.len() > 37 && name.chars().nth(36) == Some('_') {
                    // Check if first 36 chars look like a UUID
                    let prefix = &name[..36];
                    if prefix.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
                        return name[37..].to_string();
                    }
                }
                name.to_string()
            })
            .unwrap_or_else(|| "unknown".to_string());

        // Get file modification time for timestamps
        let file_time = tokio::fs::metadata(file_path)
            .await
            .ok()
            .and_then(|m| m.modified().ok())
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(Utc::now);

        let file_time_str = file_time.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        // Determine MIME type from extension
        let mime_type = match file_path.extension().and_then(|e| e.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            Some("heic") | Some("heif") => "image/heic",
            Some("mp4") => "video/mp4",
            Some("mov") => "video/quicktime",
            Some("avi") => "video/x-msvideo",
            Some("webm") => "video/webm",
            _ => "application/octet-stream",
        };

        // Build multipart form
        let file_part = Part::bytes(file_content)
            .file_name(original_filename.clone())
            .mime_str(mime_type)?;

        let form = Form::new()
            .part("assetData", file_part)
            .text("deviceAssetId", format!("restore-{}", uuid::Uuid::new_v4()))
            .text("deviceId", "immich-dupes-restore")
            .text("fileCreatedAt", file_time_str.clone())
            .text("fileModifiedAt", file_time_str);

        let url = self.base_url.join("/api/assets")?;
        let response = self.client.post(url).multipart(form).send().await?;

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
