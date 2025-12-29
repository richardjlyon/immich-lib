//! Execution types for duplicate processing pipeline.
//!
//! These types capture configuration, results, and outcomes for
//! the duplicate execution workflow.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Configuration for the execution pipeline.
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum requests per second to the Immich API
    pub requests_per_sec: u32,

    /// Maximum concurrent operations
    pub max_concurrent: usize,

    /// Directory to save backup downloads before deletion
    pub backup_dir: PathBuf,

    /// If true, permanently delete assets; if false, move to trash
    pub force_delete: bool,

    /// If true, preserve album memberships by transferring to winner
    pub preserve_albums: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            requests_per_sec: 10,
            max_concurrent: 5,
            backup_dir: PathBuf::from("./backups"),
            force_delete: false,
            preserve_albums: true,
        }
    }
}

/// Result of a single operation (download or delete).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum OperationResult {
    /// Operation completed successfully
    Success {
        /// Asset ID that was processed
        id: String,
        /// Path where file was saved (for downloads)
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<PathBuf>,
    },

    /// Operation failed with an error
    Failed {
        /// Asset ID that failed
        id: String,
        /// Error message describing the failure
        error: String,
    },

    /// Operation was skipped
    Skipped {
        /// Asset ID that was skipped
        id: String,
        /// Reason for skipping
        reason: String,
    },
}

/// Result of metadata consolidation from loser assets to winner.
///
/// Tracks which metadata fields were transferred and from which asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    /// Whether GPS coordinates were transferred
    pub gps_transferred: bool,

    /// Whether date/time was transferred
    pub datetime_transferred: bool,

    /// Whether description was transferred
    pub description_transferred: bool,

    /// Asset ID that provided the consolidated metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_asset_id: Option<String>,
}

impl ConsolidationResult {
    /// Check if any consolidation was performed.
    pub fn any_transferred(&self) -> bool {
        self.gps_transferred || self.datetime_transferred || self.description_transferred
    }
}

/// Result of transferring album memberships from losers to winner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumTransferResult {
    /// Number of albums successfully transferred
    pub albums_transferred: usize,

    /// Album IDs that were transferred
    pub album_ids: Vec<String>,

    /// Album names for readability
    pub album_names: Vec<String>,

    /// Whether any transfer failed
    pub had_failures: bool,

    /// Error message if transfers failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl AlbumTransferResult {
    /// Check if any albums were transferred.
    pub fn any_transferred(&self) -> bool {
        self.albums_transferred > 0
    }

    /// Check if all transfers succeeded.
    pub fn all_succeeded(&self) -> bool {
        !self.had_failures
    }
}

/// Result of processing a single duplicate group.
#[derive(Debug, Clone, Serialize)]
pub struct GroupResult {
    /// The duplicate group identifier
    pub duplicate_id: String,

    /// The winner asset ID
    pub winner_id: String,

    /// Result of metadata consolidation (if attempted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consolidation_result: Option<ConsolidationResult>,

    /// Result of album membership transfer (if attempted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_transfer_result: Option<AlbumTransferResult>,

    /// Results of downloading each loser asset
    pub download_results: Vec<OperationResult>,

    /// Result of deleting assets (if downloads succeeded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_result: Option<OperationResult>,
}

/// Summary report of the entire execution.
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionReport {
    /// Total number of duplicate groups processed
    pub total_groups: usize,

    /// Number of assets successfully downloaded
    pub downloaded: usize,

    /// Number of assets deleted
    pub deleted: usize,

    /// Number of operations that failed
    pub failed: usize,

    /// Number of operations that were skipped
    pub skipped: usize,

    /// Total number of albums transferred across all groups
    pub albums_transferred: usize,

    /// Number of album transfer operations that had failures
    pub album_transfer_failures: usize,

    /// Detailed results for each group
    pub results: Vec<GroupResult>,
}

impl ExecutionReport {
    /// Create an empty execution report.
    pub fn new() -> Self {
        Self {
            total_groups: 0,
            downloaded: 0,
            deleted: 0,
            failed: 0,
            skipped: 0,
            albums_transferred: 0,
            album_transfer_failures: 0,
            results: Vec::new(),
        }
    }

    /// Add a group result and update counters.
    pub fn add_group_result(&mut self, result: GroupResult) {
        self.total_groups += 1;

        // Count download outcomes
        for download in &result.download_results {
            match download {
                OperationResult::Success { .. } => self.downloaded += 1,
                OperationResult::Failed { .. } => self.failed += 1,
                OperationResult::Skipped { .. } => self.skipped += 1,
            }
        }

        // Count delete outcomes
        if let Some(ref delete) = result.delete_result {
            match delete {
                OperationResult::Success { .. } => {
                    // Count deleted losers (download successes that were deleted)
                    self.deleted += result
                        .download_results
                        .iter()
                        .filter(|r| matches!(r, OperationResult::Success { .. }))
                        .count();
                }
                OperationResult::Failed { .. } => self.failed += 1,
                OperationResult::Skipped { .. } => self.skipped += 1,
            }
        }

        // Count album transfer outcomes
        if let Some(ref album_transfer) = result.album_transfer_result {
            self.albums_transferred += album_transfer.albums_transferred;
            if album_transfer.had_failures {
                self.album_transfer_failures += 1;
            }
        }

        self.results.push(result);
    }
}

impl Default for ExecutionReport {
    fn default() -> Self {
        Self::new()
    }
}
