//! Metadata scoring and duplicate analysis.
//!
//! This module provides scoring algorithms for ranking assets by metadata completeness
//! and detecting conflicts between duplicate assets.

use serde::Serialize;

use crate::models::{AssetResponse, DuplicateGroup};

/// Weight values for metadata categories.
/// Higher weights indicate more valuable metadata that's harder to recover.
mod weights {
    pub const GPS: u32 = 30; // Most valuable, irreplaceable location data
    pub const TIMEZONE: u32 = 20; // Important for correct timestamps
    pub const CAMERA_INFO: u32 = 15; // Useful provenance information
    pub const CAPTURE_TIME: u32 = 15; // Original timestamp
    pub const LENS_INFO: u32 = 10; // Nice to have
    pub const LOCATION: u32 = 10; // Reverse-geocoded, derivable from GPS
}

/// GPS coordinate threshold for conflict detection.
/// Approximately 11 meters at the equator.
const GPS_THRESHOLD: f64 = 0.0001;

/// Metadata completeness score for an asset.
///
/// Each category contributes a weighted score based on presence of metadata.
/// Higher total scores indicate more complete metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct MetadataScore {
    /// GPS coordinate score (0 or 30)
    pub gps: u32,

    /// Timezone score (0 or 20)
    pub timezone: u32,

    /// Camera make/model score (0 or 15)
    pub camera_info: u32,

    /// Original capture time score (0 or 15)
    pub capture_time: u32,

    /// Lens info score (0 or 10)
    pub lens_info: u32,

    /// Location (city/country) score (0 or 10)
    pub location: u32,

    /// Total weighted score (sum of all categories)
    pub total: u32,
}

impl PartialOrd for MetadataScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MetadataScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.total.cmp(&other.total)
    }
}

impl MetadataScore {
    /// Score an asset based on its metadata completeness.
    ///
    /// Uses the `has_*()` helper methods on `ExifInfo` to determine
    /// which metadata categories are present.
    pub fn from_asset(asset: &AssetResponse) -> Self {
        let Some(exif) = &asset.exif_info else {
            return Self::default();
        };

        let gps = if exif.has_gps() { weights::GPS } else { 0 };
        let timezone = if exif.has_timezone() {
            weights::TIMEZONE
        } else {
            0
        };
        let camera_info = if exif.has_camera_info() {
            weights::CAMERA_INFO
        } else {
            0
        };
        let capture_time = if exif.has_capture_time() {
            weights::CAPTURE_TIME
        } else {
            0
        };
        let lens_info = if exif.has_lens_info() {
            weights::LENS_INFO
        } else {
            0
        };
        let location = if exif.has_location() {
            weights::LOCATION
        } else {
            0
        };

        let total = gps + timezone + camera_info + capture_time + lens_info + location;

        Self {
            gps,
            timezone,
            camera_info,
            capture_time,
            lens_info,
            location,
            total,
        }
    }
}

/// Detected conflict between duplicate assets.
///
/// A conflict occurs when multiple assets have different values
/// for the same metadata field.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MetadataConflict {
    /// Different GPS coordinates across duplicates
    Gps {
        /// List of unique coordinate pairs (latitude, longitude)
        values: Vec<(f64, f64)>,
    },

    /// Different timezones across duplicates
    Timezone {
        /// List of unique timezone values
        values: Vec<String>,
    },

    /// Different camera make/model combinations across duplicates
    CameraInfo {
        /// List of unique camera identifiers
        values: Vec<String>,
    },

    /// Different original capture times across duplicates
    CaptureTime {
        /// List of unique capture timestamps
        values: Vec<String>,
    },
}

/// Detect metadata conflicts across a set of assets.
///
/// A conflict is detected when multiple assets have different values
/// for the same metadata field. This helps identify cases where
/// automatic selection may lose important information.
///
/// # Arguments
///
/// * `assets` - Slice of assets to check for conflicts
///
/// # Returns
///
/// A vector of detected conflicts (empty if no conflicts found)
pub fn detect_conflicts(assets: &[AssetResponse]) -> Vec<MetadataConflict> {
    let mut conflicts = Vec::new();

    // Check GPS conflicts
    let gps_values: Vec<(f64, f64)> = assets
        .iter()
        .filter_map(|a| a.exif_info.as_ref())
        .filter_map(|e| match (e.latitude, e.longitude) {
            (Some(lat), Some(lon)) => Some((lat, lon)),
            _ => None,
        })
        .collect();

    if has_gps_conflict(&gps_values) {
        let unique_gps = dedupe_gps(&gps_values);
        conflicts.push(MetadataConflict::Gps { values: unique_gps });
    }

    // Check timezone conflicts
    let timezone_values: Vec<String> = assets
        .iter()
        .filter_map(|a| a.exif_info.as_ref())
        .filter_map(|e| e.time_zone.clone())
        .collect();

    if let Some(unique) = find_unique_strings(&timezone_values) {
        conflicts.push(MetadataConflict::Timezone { values: unique });
    }

    // Check camera info conflicts
    let camera_values: Vec<String> = assets
        .iter()
        .filter_map(|a| a.exif_info.as_ref())
        .filter_map(|e| {
            let make = e.make.as_deref().unwrap_or("");
            let model = e.model.as_deref().unwrap_or("");
            if make.is_empty() && model.is_empty() {
                None
            } else {
                Some(format!("{} {}", make, model).trim().to_string())
            }
        })
        .collect();

    if let Some(unique) = find_unique_strings(&camera_values) {
        conflicts.push(MetadataConflict::CameraInfo { values: unique });
    }

    // Check capture time conflicts
    let capture_time_values: Vec<String> = assets
        .iter()
        .filter_map(|a| a.exif_info.as_ref())
        .filter_map(|e| e.date_time_original.clone())
        .collect();

    if let Some(unique) = find_unique_strings(&capture_time_values) {
        conflicts.push(MetadataConflict::CaptureTime { values: unique });
    }

    conflicts
}

/// Check if GPS coordinates have conflicts beyond the threshold.
fn has_gps_conflict(coords: &[(f64, f64)]) -> bool {
    if coords.len() < 2 {
        return false;
    }

    for i in 0..coords.len() {
        for j in (i + 1)..coords.len() {
            let (lat1, lon1) = coords[i];
            let (lat2, lon2) = coords[j];
            if (lat1 - lat2).abs() > GPS_THRESHOLD || (lon1 - lon2).abs() > GPS_THRESHOLD {
                return true;
            }
        }
    }

    false
}

/// Deduplicate GPS coordinates within threshold.
fn dedupe_gps(coords: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut unique: Vec<(f64, f64)> = Vec::new();

    for &(lat, lon) in coords {
        let is_duplicate = unique.iter().any(|&(ulat, ulon)| {
            (lat - ulat).abs() <= GPS_THRESHOLD && (lon - ulon).abs() <= GPS_THRESHOLD
        });

        if !is_duplicate {
            unique.push((lat, lon));
        }
    }

    unique
}

/// Find unique string values (case-insensitive, trimmed).
/// Returns None if there are 0 or 1 unique values.
fn find_unique_strings(values: &[String]) -> Option<Vec<String>> {
    if values.is_empty() {
        return None;
    }

    let mut seen: Vec<String> = Vec::new();
    let mut unique_original: Vec<String> = Vec::new();

    for value in values {
        let normalized = value.trim().to_lowercase();
        if !normalized.is_empty() && !seen.contains(&normalized) {
            seen.push(normalized);
            unique_original.push(value.trim().to_string());
        }
    }

    if unique_original.len() > 1 {
        Some(unique_original)
    } else {
        None
    }
}

/// A scored asset with metadata score and file information.
#[derive(Debug, Clone, Serialize)]
pub struct ScoredAsset {
    /// Asset unique identifier
    pub asset_id: String,

    /// Original filename
    pub filename: String,

    /// Metadata completeness score
    pub score: MetadataScore,

    /// File size in bytes (for tiebreaking)
    pub file_size: Option<u64>,
}

/// Analysis result for a duplicate group.
///
/// Contains the selected winner, losers, detected conflicts,
/// and whether manual review is recommended.
#[derive(Debug, Clone, Serialize)]
pub struct DuplicateAnalysis {
    /// Duplicate group identifier
    pub duplicate_id: String,

    /// The asset selected as the winner (highest metadata score)
    pub winner: ScoredAsset,

    /// Assets that should be deleted (lower metadata scores)
    pub losers: Vec<ScoredAsset>,

    /// Detected metadata conflicts
    pub conflicts: Vec<MetadataConflict>,

    /// Whether manual review is recommended due to conflicts
    pub needs_review: bool,
}

impl DuplicateAnalysis {
    /// Analyze a duplicate group and select a winner.
    ///
    /// The winner is selected based on:
    /// 1. Highest metadata score
    /// 2. Largest file size (tiebreaker)
    /// 3. First in list (stable sort, final tiebreaker)
    ///
    /// # Arguments
    ///
    /// * `group` - The duplicate group to analyze
    ///
    /// # Returns
    ///
    /// Analysis result with winner, losers, and conflict information
    pub fn from_group(group: &DuplicateGroup) -> Self {
        // Score all assets
        let mut scored: Vec<ScoredAsset> = group
            .assets
            .iter()
            .map(|asset| ScoredAsset {
                asset_id: asset.id.clone(),
                filename: asset.original_file_name.clone(),
                score: MetadataScore::from_asset(asset),
                file_size: asset.exif_info.as_ref().and_then(|e| e.file_size_in_byte),
            })
            .collect();

        // Sort by score descending, then by file size descending (stable sort)
        scored.sort_by(|a, b| {
            match b.score.total.cmp(&a.score.total) {
                std::cmp::Ordering::Equal => {
                    // Tiebreaker: larger file size wins
                    let size_a = a.file_size.unwrap_or(0);
                    let size_b = b.file_size.unwrap_or(0);
                    size_b.cmp(&size_a)
                }
                other => other,
            }
        });

        // Detect conflicts
        let conflicts = detect_conflicts(&group.assets);
        let needs_review = !conflicts.is_empty();

        // Split into winner and losers
        let winner = scored.remove(0);
        let losers = scored;

        Self {
            duplicate_id: group.duplicate_id.clone(),
            winner,
            losers,
            conflicts,
            needs_review,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_score_default() {
        let score = MetadataScore::default();
        assert_eq!(score.total, 0);
    }

    #[test]
    fn test_gps_conflict_detection() {
        // Same coordinates within threshold
        let coords = vec![(51.5074, -0.1278), (51.5074, -0.1278)];
        assert!(!has_gps_conflict(&coords));

        // Different coordinates beyond threshold
        let coords = vec![(51.5074, -0.1278), (52.0, -0.5)];
        assert!(has_gps_conflict(&coords));
    }

    #[test]
    fn test_find_unique_strings() {
        // Single value
        let values = vec!["America/New_York".to_string()];
        assert!(find_unique_strings(&values).is_none());

        // Same values (case-insensitive)
        let values = vec!["America/New_York".to_string(), "america/new_york".to_string()];
        assert!(find_unique_strings(&values).is_none());

        // Different values
        let values = vec!["America/New_York".to_string(), "Europe/London".to_string()];
        let unique = find_unique_strings(&values).unwrap();
        assert_eq!(unique.len(), 2);
    }
}
