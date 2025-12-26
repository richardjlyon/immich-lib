//! Execution types for duplicate processing pipeline.
//!
//! These types capture configuration, results, and outcomes for
//! the duplicate execution workflow.

use std::path::PathBuf;

use serde::Serialize;

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
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            requests_per_sec: 10,
            max_concurrent: 5,
            backup_dir: PathBuf::from("./backups"),
            force_delete: false,
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

/// Result of processing a single duplicate group.
#[derive(Debug, Clone, Serialize)]
pub struct GroupResult {
    /// The duplicate group identifier
    pub duplicate_id: String,

    /// The winner asset ID
    pub winner_id: String,

    /// Result of metadata consolidation (if attempted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consolidation_result: Option<String>,

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

        self.results.push(result);
    }
}

impl Default for ExecutionReport {
    fn default() -> Self {
        Self::new()
    }
}
