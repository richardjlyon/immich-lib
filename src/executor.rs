//! Execution pipeline for duplicate processing.
//!
//! This module provides the `Executor` struct which handles rate-limited,
//! concurrent execution of duplicate processing operations including
//! downloading backups and deleting duplicates.

use std::num::NonZeroU32;
use std::sync::Arc;

use governor::{Quota, RateLimiter};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use nonzero_ext::nonzero;
use tokio::sync::Semaphore;

use crate::client::ImmichClient;
use crate::error::Result;
use crate::models::{ExecutionConfig, ExecutionReport, GroupResult, OperationResult};
use crate::scoring::DuplicateAnalysis;

/// Type alias for the governor rate limiter.
type DirectRateLimiter = RateLimiter<
    governor::state::NotKeyed,
    governor::state::InMemoryState,
    governor::clock::DefaultClock,
>;

/// Executor for duplicate processing operations.
///
/// Handles rate-limited, concurrent execution of the duplicate processing pipeline:
/// 1. Download backup copies of loser assets
/// 2. Delete successfully downloaded assets
///
/// # Example
///
/// ```no_run
/// use immich_lib::{ImmichClient, Executor};
/// use immich_lib::models::ExecutionConfig;
///
/// # async fn example() -> immich_lib::Result<()> {
/// let client = ImmichClient::new("https://immich.example.com", "api-key")?;
/// let config = ExecutionConfig::default();
/// let executor = Executor::new(client, config);
///
/// // Execute analysis results
/// // let report = executor.execute_all(&analyses).await;
/// # Ok(())
/// # }
/// ```
pub struct Executor {
    /// The Immich API client
    client: ImmichClient,

    /// Rate limiter for API requests
    rate_limiter: DirectRateLimiter,

    /// Semaphore for concurrent operation control
    concurrency: Arc<Semaphore>,

    /// Execution configuration
    config: ExecutionConfig,
}

impl Executor {
    /// Create a new executor with the given client and configuration.
    ///
    /// # Arguments
    ///
    /// * `client` - The Immich API client to use for operations
    /// * `config` - Execution configuration (rate limits, concurrency, backup dir)
    pub fn new(client: ImmichClient, config: ExecutionConfig) -> Self {
        // Create rate limiter with configured requests per second
        let quota = Quota::per_second(
            NonZeroU32::new(config.requests_per_sec).unwrap_or(nonzero!(10u32)),
        );
        let rate_limiter = RateLimiter::direct(quota);

        // Create semaphore for concurrency control
        let concurrency = Arc::new(Semaphore::new(config.max_concurrent));

        Self {
            client,
            rate_limiter,
            concurrency,
            config,
        }
    }

    /// Wait for rate limit and acquire concurrency permit before executing an operation.
    ///
    /// This helper ensures all API operations respect rate limits and concurrency bounds.
    async fn rate_limited<F, T>(&self, op: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // Wait for rate limit allowance
        self.rate_limiter.until_ready().await;

        // Acquire concurrency permit (automatically released when dropped)
        let _permit = self.concurrency.acquire().await.expect("semaphore closed");

        // Execute the operation
        op.await
    }

    /// Execute processing for all duplicate groups.
    ///
    /// Iterates through all groups, downloading backups and deleting duplicates
    /// for each. Shows progress via console progress bars.
    ///
    /// # Arguments
    ///
    /// * `groups` - Slice of duplicate analysis results to process
    ///
    /// # Returns
    ///
    /// An execution report summarizing all operations and their outcomes.
    pub async fn execute_all(&self, groups: &[DuplicateAnalysis]) -> ExecutionReport {
        let mut report = ExecutionReport::new();

        if groups.is_empty() {
            return report;
        }

        // Create multi-progress container
        let multi_progress = MultiProgress::new();

        // Create overall progress bar
        let overall_style = ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} groups ({eta})")
            .expect("valid template")
            .progress_chars("##-");

        let overall_pb = multi_progress.add(ProgressBar::new(groups.len() as u64));
        overall_pb.set_style(overall_style);

        // Create progress bar for current group operations
        let group_style = ProgressStyle::default_bar()
            .template("  {spinner:.green} {msg}")
            .expect("valid template");

        let group_pb = multi_progress.add(ProgressBar::new_spinner());
        group_pb.set_style(group_style);

        // Ensure backup directory exists
        if let Err(e) = tokio::fs::create_dir_all(&self.config.backup_dir).await {
            overall_pb.finish_with_message(format!("Failed to create backup directory: {}", e));
            return report;
        }

        // Process each group
        for analysis in groups {
            group_pb.set_message(format!(
                "Processing group {} ({} losers)",
                analysis.duplicate_id,
                analysis.losers.len()
            ));

            let result = self.execute_group(analysis, &group_pb).await;
            report.add_group_result(result);

            overall_pb.inc(1);
        }

        overall_pb.finish_with_message("Complete");
        group_pb.finish_and_clear();

        report
    }

    /// Execute processing for a single duplicate group.
    ///
    /// Downloads backup copies of all loser assets, then deletes only those
    /// that were successfully downloaded.
    ///
    /// # Arguments
    ///
    /// * `analysis` - The duplicate analysis for this group
    /// * `pb` - Progress bar to update with status messages
    ///
    /// # Returns
    ///
    /// A group result detailing the outcome of each operation.
    pub async fn execute_group(
        &self,
        analysis: &DuplicateAnalysis,
        pb: &ProgressBar,
    ) -> GroupResult {
        let mut download_results = Vec::new();

        // TODO: Metadata consolidation requires re-fetching full asset data.
        // For now, we skip consolidation. Future enhancement: add exif fields
        // to ScoredAsset during analysis, or fetch asset data here.
        let consolidation_result: Option<String> = None;

        // Download each loser asset
        for loser in &analysis.losers {
            pb.set_message(format!("Downloading {}", loser.filename));

            let result = self.download_loser(&loser.asset_id, &loser.filename).await;
            download_results.push(result);
        }

        // Collect successfully downloaded asset IDs for deletion
        let downloaded_ids: Vec<String> = download_results
            .iter()
            .filter_map(|r| match r {
                OperationResult::Success { id, .. } => Some(id.clone()),
                _ => None,
            })
            .collect();

        // Only delete if we have successfully downloaded assets
        let delete_result = if downloaded_ids.is_empty() {
            Some(OperationResult::Skipped {
                id: analysis.duplicate_id.clone(),
                reason: "No assets were successfully downloaded".to_string(),
            })
        } else {
            pb.set_message(format!("Deleting {} assets", downloaded_ids.len()));

            match self.delete_assets(&downloaded_ids).await {
                Ok(()) => Some(OperationResult::Success {
                    id: analysis.duplicate_id.clone(),
                    path: None,
                }),
                Err(e) => Some(OperationResult::Failed {
                    id: analysis.duplicate_id.clone(),
                    error: e.to_string(),
                }),
            }
        };

        GroupResult {
            duplicate_id: analysis.duplicate_id.clone(),
            winner_id: analysis.winner.asset_id.clone(),
            consolidation_result,
            download_results,
            delete_result,
        }
    }

    /// Download a loser asset to the backup directory.
    ///
    /// Files are named as `{asset_id}_{filename}` to avoid collisions.
    async fn download_loser(&self, asset_id: &str, filename: &str) -> OperationResult {
        // Build path with asset ID prefix to avoid collisions
        let safe_filename = format!("{}_{}", asset_id, filename);
        let path = self.config.backup_dir.join(&safe_filename);

        let download_result = self
            .rate_limited(async { self.client.download_asset(asset_id, &path).await })
            .await;

        match download_result {
            Ok(_bytes) => OperationResult::Success {
                id: asset_id.to_string(),
                path: Some(path),
            },
            Err(e) => OperationResult::Failed {
                id: asset_id.to_string(),
                error: e.to_string(),
            },
        }
    }

    /// Delete assets using the API.
    async fn delete_assets(&self, asset_ids: &[String]) -> Result<()> {
        self.rate_limited(async {
            self.client
                .delete_assets(asset_ids, self.config.force_delete)
                .await
        })
        .await
    }
}
