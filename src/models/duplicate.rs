//! Duplicate group response types.

use serde::Deserialize;

use super::asset::AssetResponse;

/// A group of duplicate assets identified by Immich.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    /// Unique identifier for this duplicate group
    pub duplicate_id: String,

    /// Assets in this duplicate group
    pub assets: Vec<AssetResponse>,
}
