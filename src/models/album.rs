//! Album response types.

use serde::{Deserialize, Serialize};

/// Album response from the Immich API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumResponse {
    /// Unique album identifier
    pub id: String,

    /// Album name
    pub album_name: String,

    /// Album description
    #[serde(default)]
    pub description: Option<String>,

    /// Album thumbnail asset ID
    #[serde(default)]
    pub album_thumbnail_asset_id: Option<String>,

    /// Owner user ID
    pub owner_id: String,

    /// Asset IDs in this album
    #[serde(default)]
    pub asset_ids: Vec<String>,
}

/// Request body for adding assets to an album.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddAssetsRequest {
    /// Asset IDs to add
    pub ids: Vec<String>,
}

/// Request body for removing assets from an album.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveAssetsRequest {
    /// Asset IDs to remove
    pub ids: Vec<String>,
}
