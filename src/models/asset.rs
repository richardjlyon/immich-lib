//! Asset response types.

use serde::Deserialize;

use super::exif::ExifInfo;

/// Type of asset (image or video).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AssetType {
    /// Image file (JPEG, PNG, HEIC, etc.)
    Image,

    /// Video file (MP4, MOV, etc.)
    Video,
}

/// Asset response from the Immich API.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetResponse {
    /// Unique asset identifier
    pub id: String,

    /// Original filename when uploaded
    pub original_file_name: String,

    /// File creation timestamp (UTC)
    pub file_created_at: String,

    /// Local date/time (timezone-aware)
    pub local_date_time: String,

    /// Asset type (image or video)
    #[serde(rename = "type")]
    pub asset_type: AssetType,

    /// EXIF metadata (may be absent)
    pub exif_info: Option<ExifInfo>,

    /// SHA-1 checksum (base64 encoded)
    pub checksum: String,

    /// Whether the asset is in trash
    pub is_trashed: bool,

    /// Whether the asset is marked as favorite
    pub is_favorite: bool,

    /// Whether the asset is archived
    pub is_archived: bool,
}
