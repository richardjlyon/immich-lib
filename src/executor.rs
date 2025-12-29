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
use crate::models::{
    AlbumTransferResult, ConsolidationResult, ExecutionConfig, ExecutionReport, GroupResult,
    OperationResult,
};
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
    /// 1. Consolidates metadata from losers to winner (GPS, datetime, description)
    /// 2. Downloads backup copies of all loser assets
    /// 3. Deletes only those that were successfully downloaded
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

        // Step 1: Consolidate metadata from losers to winner
        pb.set_message("Checking metadata consolidation");
        let consolidation_result = self.consolidate_metadata(analysis).await;

        // Step 2: Transfer album memberships (if enabled)
        let album_transfer_result = if self.config.preserve_albums {
            pb.set_message("Transferring album memberships");
            Some(self.transfer_albums(analysis).await)
        } else {
            None
        };

        // Step 3: Decide whether to proceed with deletion
        // If album transfer failed after retry, skip deletion for this group
        let should_skip_deletion = if let Some(ref transfer) = album_transfer_result {
            transfer.had_failures
        } else {
            false
        };

        if should_skip_deletion {
            // Return early - don't download or delete
            return GroupResult {
                duplicate_id: analysis.duplicate_id.clone(),
                winner_id: analysis.winner.asset_id.clone(),
                consolidation_result,
                album_transfer_result,
                download_results: vec![],
                delete_result: Some(OperationResult::Skipped {
                    id: analysis.duplicate_id.clone(),
                    reason: "Album transfer failed after retry, skipping deletion to preserve album integrity".to_string(),
                }),
            };
        }

        // Step 4: Download each loser asset
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

        // Step 5: Only delete if we have successfully downloaded assets
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
            album_transfer_result,
            download_results,
            delete_result,
        }
    }

    /// Consolidate metadata from loser assets to the winner.
    ///
    /// Checks if the winner lacks GPS, datetime, or description that any loser has,
    /// and transfers the metadata to preserve it before deletion.
    async fn consolidate_metadata(
        &self,
        analysis: &DuplicateAnalysis,
    ) -> Option<ConsolidationResult> {
        // Fetch winner asset to check what metadata it already has
        let winner_asset = match self
            .rate_limited(async { self.client.get_asset(&analysis.winner.asset_id).await })
            .await
        {
            Ok(asset) => asset,
            Err(_) => return None, // Can't consolidate if we can't fetch winner
        };

        let winner_exif = winner_asset.exif_info.as_ref();
        let winner_has_gps = winner_exif.map(|e| e.has_gps()).unwrap_or(false);
        let winner_has_datetime = winner_exif
            .and_then(|e| e.date_time_original.as_ref())
            .is_some();
        let winner_has_description = winner_exif.and_then(|e| e.description.as_ref()).is_some();

        // If winner has all metadata, no consolidation needed
        if winner_has_gps && winner_has_datetime && winner_has_description {
            return None;
        }

        // Find best source for each missing field from losers (owned values)
        let mut best_gps: Option<(f64, f64, String)> = None;
        let mut best_datetime: Option<(String, String)> = None;
        let mut best_description: Option<(String, String)> = None;

        for loser in &analysis.losers {
            let loser_asset = match self
                .rate_limited(async { self.client.get_asset(&loser.asset_id).await })
                .await
            {
                Ok(asset) => asset,
                Err(_) => continue, // Skip losers we can't fetch
            };

            if let Some(exif) = &loser_asset.exif_info {
                // Check GPS
                if !winner_has_gps
                    && best_gps.is_none()
                    && exif.has_gps()
                    && let (Some(lat), Some(lon)) = (exif.latitude, exif.longitude)
                {
                    best_gps = Some((lat, lon, loser.asset_id.clone()));
                }

                // Check datetime
                if !winner_has_datetime
                    && best_datetime.is_none()
                    && let Some(dt) = &exif.date_time_original
                {
                    best_datetime = Some((dt.clone(), loser.asset_id.clone()));
                }

                // Check description
                if !winner_has_description
                    && best_description.is_none()
                    && let Some(desc) = &exif.description
                {
                    best_description = Some((desc.clone(), loser.asset_id.clone()));
                }
            }

            // If we've found all we need, stop searching
            if (winner_has_gps || best_gps.is_some())
                && (winner_has_datetime || best_datetime.is_some())
                && (winner_has_description || best_description.is_some())
            {
                break;
            }
        }

        // Nothing to consolidate
        if best_gps.is_none() && best_datetime.is_none() && best_description.is_none() {
            return None;
        }

        // Prepare update parameters
        let (latitude, longitude) = match &best_gps {
            Some((lat, lon, _)) => (Some(*lat), Some(*lon)),
            None => (None, None),
        };
        let date_time_original = best_datetime.as_ref().map(|(dt, _)| dt.as_str());
        let description = best_description.as_ref().map(|(desc, _)| desc.as_str());

        // Determine source asset ID (prefer GPS source, then datetime, then description)
        let source_asset_id = best_gps
            .as_ref()
            .map(|(_, _, id)| id.clone())
            .or_else(|| best_datetime.as_ref().map(|(_, id)| id.clone()))
            .or_else(|| best_description.as_ref().map(|(_, id)| id.clone()));

        // Update winner with consolidated metadata
        let update_result = self
            .rate_limited(async {
                self.client
                    .update_asset_metadata(
                        &analysis.winner.asset_id,
                        latitude,
                        longitude,
                        date_time_original,
                        description,
                    )
                    .await
            })
            .await;

        if update_result.is_ok() {
            Some(ConsolidationResult {
                gps_transferred: best_gps.is_some(),
                datetime_transferred: best_datetime.is_some(),
                description_transferred: best_description.is_some(),
                source_asset_id,
            })
        } else {
            None // Consolidation failed, but we can still proceed with download/delete
        }
    }

    /// Transfer album memberships from loser assets to the winner.
    ///
    /// For each loser asset that belongs to albums, adds the winner to those albums
    /// and removes the loser. Implements retry logic: attempt once, retry on failure,
    /// skip deletion if still failing.
    async fn transfer_albums(&self, analysis: &DuplicateAnalysis) -> AlbumTransferResult {
        let mut transferred_albums = Vec::new();
        let mut album_names = Vec::new();
        let mut had_failures = false;
        let mut error_messages = Vec::new();

        // Collect all unique albums from all losers
        let mut albums_to_transfer = std::collections::HashMap::new();

        for loser in &analysis.losers {
            let albums_result = self
                .rate_limited(async { self.client.get_albums_for_asset(&loser.asset_id).await })
                .await;

            match albums_result {
                Ok(albums) => {
                    for album in albums {
                        albums_to_transfer
                            .entry(album.id.clone())
                            .or_insert_with(|| album);
                    }
                }
                Err(e) => {
                    had_failures = true;
                    error_messages.push(format!(
                        "Failed to get albums for {}: {}",
                        loser.asset_id, e
                    ));
                }
            }
        }

        // If no albums found, return success with 0 transfers
        if albums_to_transfer.is_empty() {
            return AlbumTransferResult {
                albums_transferred: 0,
                album_ids: vec![],
                album_names: vec![],
                had_failures: false,
                error_message: None,
            };
        }

        // Transfer each album (with retry logic)
        for (album_id, album) in albums_to_transfer {
            let transfer_succeeded = self
                .transfer_single_album(
                    &album_id,
                    &analysis.winner.asset_id,
                    &analysis
                        .losers
                        .iter()
                        .map(|l| l.asset_id.clone())
                        .collect::<Vec<_>>(),
                )
                .await;

            if transfer_succeeded {
                transferred_albums.push(album_id);
                album_names.push(album.album_name);
            } else {
                had_failures = true;
                error_messages.push(format!(
                    "Failed to transfer album '{}' after retry",
                    album.album_name
                ));
            }
        }

        AlbumTransferResult {
            albums_transferred: transferred_albums.len(),
            album_ids: transferred_albums,
            album_names,
            had_failures,
            error_message: if error_messages.is_empty() {
                None
            } else {
                Some(error_messages.join("; "))
            },
        }
    }

    /// Transfer a single album with exponential backoff retry logic.
    ///
    /// Retries with exponential backoff (250ms, 500ms, 1s, 2s, 4s, 8s, 16s, 32s)
    /// for up to 60 seconds total. Returns true if transfer succeeded, false if
    /// all retries exhausted.
    async fn transfer_single_album(
        &self,
        album_id: &str,
        winner_id: &str,
        loser_ids: &[String],
    ) -> bool {
        const MAX_DURATION: std::time::Duration = std::time::Duration::from_secs(60);
        const INITIAL_DELAY: std::time::Duration = std::time::Duration::from_millis(250);

        let start = tokio::time::Instant::now();
        let mut delay = INITIAL_DELAY;

        loop {
            // Attempt transfer
            match self
                .try_transfer_album(album_id, winner_id, loser_ids)
                .await
            {
                Ok(()) => return true,
                Err(_) => {
                    // Check if we've exceeded the maximum duration
                    if start.elapsed() >= MAX_DURATION {
                        return false;
                    }

                    // Wait with exponential backoff, but don't exceed remaining time
                    let remaining = MAX_DURATION.saturating_sub(start.elapsed());
                    let sleep_duration = delay.min(remaining);

                    if sleep_duration.is_zero() {
                        return false;
                    }

                    tokio::time::sleep(sleep_duration).await;

                    // Double the delay for next attempt (exponential backoff)
                    delay = delay.saturating_mul(2);
                }
            }
        }
    }

    /// Attempt to transfer an album once.
    async fn try_transfer_album(
        &self,
        album_id: &str,
        winner_id: &str,
        loser_ids: &[String],
    ) -> Result<()> {
        // Add winner to album (skip if already in album)
        let add_result = self
            .rate_limited(async {
                self.client
                    .add_assets_to_album(album_id, &[winner_id.to_string()])
                    .await
            })
            .await;

        // Even if add fails because winner is already in album, continue to remove losers
        // Check if error is "already in album" type error
        if let Err(e) = add_result {
            // If it's not a "duplicate" type error, propagate it
            match e {
                crate::error::ImmichError::Api { status, ref message } => {
                    // 400 with "duplicate" or "already" in message is OK
                    if status != 400
                        || (!message.to_lowercase().contains("duplicate")
                            && !message.to_lowercase().contains("already"))
                    {
                        return Err(e);
                    }
                }
                _ => return Err(e),
            }
        }

        // Remove losers from album
        self.rate_limited(async {
            self.client
                .remove_assets_from_album(album_id, loser_ids)
                .await
        })
        .await?;

        Ok(())
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
