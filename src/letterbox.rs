//! Letterbox detection and pairing for iPhone 4:3/16:9 crop duplicates.
//!
//! This module identifies duplicate pairs where iPhone photos exist as both:
//! - 4:3 aspect ratio (full sensor, more pixels)
//! - 16:9 aspect ratio (cropped version)
//!
//! The 4:3 version is always preferred as the "keeper" since it contains the full scene.

use std::collections::HashMap;

use serde::Serialize;

use crate::models::AssetResponse;

/// Aspect ratio classification for iPhone photos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectRatio {
    /// 4:3 ratio (1.333) - Full sensor capture
    FourThree,
    /// 16:9 ratio (1.778) - Cropped version
    SixteenNine,
}

/// Tolerance for aspect ratio matching.
const RATIO_TOLERANCE: f64 = 0.01;

/// 4:3 aspect ratio value.
const RATIO_4_3: f64 = 4.0 / 3.0; // 1.333...

/// 16:9 aspect ratio value.
const RATIO_16_9: f64 = 16.0 / 9.0; // 1.777...

/// Detect the aspect ratio of an image from its dimensions.
///
/// Uses orientation-agnostic calculation (max/min) to handle both
/// portrait and landscape orientations correctly.
///
/// # Arguments
///
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
///
/// # Returns
///
/// * `Some(AspectRatio::FourThree)` if ratio is 4:3 (within tolerance)
/// * `Some(AspectRatio::SixteenNine)` if ratio is 16:9 (within tolerance)
/// * `None` for other aspect ratios
///
/// # Examples
///
/// ```
/// use immich_lib::letterbox::detect_aspect_ratio;
///
/// // Landscape 4:3
/// assert!(detect_aspect_ratio(5712, 4284).is_some());
///
/// // Portrait 4:3 (same ratio, just rotated)
/// assert!(detect_aspect_ratio(4284, 5712).is_some());
/// ```
pub fn detect_aspect_ratio(width: u32, height: u32) -> Option<AspectRatio> {
    if width == 0 || height == 0 {
        return None;
    }

    // Use max/min for orientation-agnostic ratio
    let max_dim = width.max(height) as f64;
    let min_dim = width.min(height) as f64;
    let ratio = max_dim / min_dim;

    if (ratio - RATIO_4_3).abs() < RATIO_TOLERANCE {
        Some(AspectRatio::FourThree)
    } else if (ratio - RATIO_16_9).abs() < RATIO_TOLERANCE {
        Some(AspectRatio::SixteenNine)
    } else {
        None
    }
}

/// A detected letterbox pair (4:3 original + 16:9 crop).
#[derive(Debug, Clone, Serialize)]
pub struct LetterboxPair {
    /// The 4:3 version to keep (more pixels, full scene)
    pub keeper: AssetResponse,
    /// The 16:9 version to delete (cropped)
    pub delete: AssetResponse,
    /// Shared capture timestamp
    pub timestamp: String,
    /// Camera identifier (e.g., "Apple iPhone 15 Pro Max")
    pub camera: String,
}

/// Internal key for grouping assets by capture moment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PairingKey {
    /// dateTimeOriginal truncated to second
    timestamp_second: String,
    /// Camera manufacturer (e.g., "Apple")
    make: String,
    /// Camera model (e.g., "iPhone 15 Pro Max")
    model: String,
    /// GPS coordinates rounded to 4 decimals, if available
    gps_key: Option<String>,
}

impl PairingKey {
    /// Create a pairing key from an asset.
    ///
    /// Returns None if required fields are missing.
    fn from_asset(asset: &AssetResponse) -> Option<Self> {
        let exif = asset.exif_info.as_ref()?;

        // Require timestamp
        let timestamp = exif.date_time_original.as_ref()?;

        // Truncate to second (remove sub-second precision)
        // Format: "2024-12-23T10:30:45.123Z" -> "2024-12-23T10:30:45"
        let timestamp_second = if let Some(dot_pos) = timestamp.find('.') {
            timestamp[..dot_pos].to_string()
        } else if let Some(z_pos) = timestamp.find('Z') {
            timestamp[..z_pos].to_string()
        } else {
            timestamp.clone()
        };

        // Require make and model
        let make = exif.make.clone()?;
        let model = exif.model.clone()?;

        // Optional GPS key for disambiguation
        let gps_key = match (exif.latitude, exif.longitude) {
            (Some(lat), Some(lon)) => {
                // Round to 4 decimal places (~11 meters precision)
                Some(format!("{:.4},{:.4}", lat, lon))
            }
            _ => None,
        };

        Some(Self {
            timestamp_second,
            make,
            model,
            gps_key,
        })
    }
}

/// Check if an asset is from an iPhone.
fn is_iphone_asset(asset: &AssetResponse) -> bool {
    let Some(exif) = &asset.exif_info else {
        return false;
    };

    let is_apple = exif
        .make
        .as_ref()
        .is_some_and(|make| make.to_lowercase().contains("apple"));

    let is_iphone = exif
        .model
        .as_ref()
        .is_some_and(|model| model.to_lowercase().contains("iphone"));

    is_apple && is_iphone
}

/// Get aspect ratio from asset dimensions.
fn get_asset_aspect_ratio(asset: &AssetResponse) -> Option<AspectRatio> {
    let exif = asset.exif_info.as_ref()?;
    let width = exif.exif_image_width?;
    let height = exif.exif_image_height?;
    detect_aspect_ratio(width, height)
}

/// Find letterbox pairs in a collection of assets.
///
/// Identifies pairs of iPhone photos where one is 4:3 (full sensor)
/// and the other is 16:9 (cropped). These pairs are created when
/// iPhone users take photos in certain modes.
///
/// # Algorithm
///
/// 1. Filter to iPhone images only (make="Apple", model contains "iPhone")
/// 2. Group by pairing key (timestamp + make + model + GPS)
/// 3. For each group with exactly one 4:3 and one 16:9, create a pair
/// 4. Skip ambiguous groups (multiple images of same ratio)
///
/// # Arguments
///
/// * `assets` - Slice of assets to analyze
///
/// # Returns
///
/// Vector of detected letterbox pairs, with 4:3 as keeper and 16:9 as delete.
pub fn find_letterbox_pairs(assets: &[AssetResponse]) -> Vec<LetterboxPair> {
    // Group assets by pairing key
    let mut groups: HashMap<PairingKey, Vec<&AssetResponse>> = HashMap::new();

    for asset in assets {
        // Skip non-iPhone assets
        if !is_iphone_asset(asset) {
            continue;
        }

        // Skip trashed assets
        if asset.is_trashed {
            continue;
        }

        // Skip assets without valid aspect ratio
        if get_asset_aspect_ratio(asset).is_none() {
            continue;
        }

        // Group by pairing key
        if let Some(key) = PairingKey::from_asset(asset) {
            groups.entry(key).or_default().push(asset);
        }
    }

    // Find pairs within each group
    let mut pairs = Vec::new();

    for (key, group_assets) in groups {
        // Separate by aspect ratio
        let mut four_three: Vec<&AssetResponse> = Vec::new();
        let mut sixteen_nine: Vec<&AssetResponse> = Vec::new();

        for asset in group_assets {
            match get_asset_aspect_ratio(asset) {
                Some(AspectRatio::FourThree) => four_three.push(asset),
                Some(AspectRatio::SixteenNine) => sixteen_nine.push(asset),
                None => {}
            }
        }

        // Only create pair if exactly one of each
        if four_three.len() == 1 && sixteen_nine.len() == 1 {
            let keeper = four_three[0];
            let delete = sixteen_nine[0];

            pairs.push(LetterboxPair {
                keeper: keeper.clone(),
                delete: delete.clone(),
                timestamp: key.timestamp_second.clone(),
                camera: format!("{} {}", key.make, key.model),
            });
        }
        // Skip ambiguous groups (multiple of same ratio at same timestamp)
    }

    pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetType, ExifInfo};

    /// Helper to create a mock asset with configurable EXIF data.
    fn mock_asset(
        id: &str,
        width: Option<u32>,
        height: Option<u32>,
        make: Option<&str>,
        model: Option<&str>,
        timestamp: Option<&str>,
        lat: Option<f64>,
        lon: Option<f64>,
    ) -> AssetResponse {
        let exif = ExifInfo {
            exif_image_width: width,
            exif_image_height: height,
            make: make.map(String::from),
            model: model.map(String::from),
            date_time_original: timestamp.map(String::from),
            latitude: lat,
            longitude: lon,
            // Required fields with defaults
            city: None,
            state: None,
            country: None,
            time_zone: None,
            lens_model: None,
            exposure_time: None,
            f_number: None,
            focal_length: None,
            iso: None,
            file_size_in_byte: None,
            description: None,
            rating: None,
            orientation: None,
            modify_date: None,
            projection_type: None,
        };

        AssetResponse {
            id: id.to_string(),
            original_file_name: format!("{}.HEIC", id),
            file_created_at: "2024-12-23T10:30:45Z".to_string(),
            local_date_time: "2024-12-23T10:30:45".to_string(),
            asset_type: AssetType::Image,
            exif_info: Some(exif),
            checksum: "abc123".to_string(),
            is_trashed: false,
            is_favorite: false,
            is_archived: false,
            has_metadata: true,
            duration: "0:00:00.000000".to_string(),
            owner_id: "owner-1".to_string(),
            original_mime_type: Some("image/heic".to_string()),
            duplicate_id: None,
            thumbhash: None,
        }
    }

    // ============ Aspect Ratio Detection Tests ============

    #[test]
    fn test_detect_4_3_landscape() {
        // iPhone 15 Pro Max 4:3 dimensions
        assert_eq!(
            detect_aspect_ratio(5712, 4284),
            Some(AspectRatio::FourThree)
        );
    }

    #[test]
    fn test_detect_4_3_portrait() {
        // Portrait orientation (rotated 90 degrees)
        assert_eq!(
            detect_aspect_ratio(4284, 5712),
            Some(AspectRatio::FourThree)
        );
    }

    #[test]
    fn test_detect_16_9_landscape() {
        // iPhone 15 Pro Max 16:9 dimensions
        assert_eq!(
            detect_aspect_ratio(5712, 3213),
            Some(AspectRatio::SixteenNine)
        );
    }

    #[test]
    fn test_detect_16_9_portrait() {
        // Portrait orientation
        assert_eq!(
            detect_aspect_ratio(3213, 5712),
            Some(AspectRatio::SixteenNine)
        );
    }

    #[test]
    fn test_detect_other_ratio_1_1() {
        // Square image - not 4:3 or 16:9
        assert_eq!(detect_aspect_ratio(1000, 1000), None);
    }

    #[test]
    fn test_detect_other_ratio_3_2() {
        // 3:2 ratio (common in DSLRs) - not 4:3 or 16:9
        assert_eq!(detect_aspect_ratio(3000, 2000), None);
    }

    #[test]
    fn test_detect_with_tolerance_4_3_edge() {
        // Edge case near 4:3 boundary
        // 4:3 = 1.333..., tolerance = 0.01
        // 1.333 + 0.009 = 1.342 should still match
        // 1000 / 745 = 1.342
        assert_eq!(
            detect_aspect_ratio(1000, 745),
            Some(AspectRatio::FourThree)
        );
    }

    #[test]
    fn test_detect_with_tolerance_16_9_edge() {
        // Edge case near 16:9 boundary
        // 16:9 = 1.778..., tolerance = 0.01
        // 1778 / 1000 = 1.778 should match
        assert_eq!(
            detect_aspect_ratio(1778, 1000),
            Some(AspectRatio::SixteenNine)
        );
    }

    #[test]
    fn test_detect_zero_dimension() {
        assert_eq!(detect_aspect_ratio(0, 100), None);
        assert_eq!(detect_aspect_ratio(100, 0), None);
        assert_eq!(detect_aspect_ratio(0, 0), None);
    }

    #[test]
    fn test_detect_hd_16_9() {
        // Standard HD 16:9 dimensions
        assert_eq!(
            detect_aspect_ratio(1920, 1080),
            Some(AspectRatio::SixteenNine)
        );
    }

    #[test]
    fn test_detect_4k_16_9() {
        // 4K 16:9 dimensions
        assert_eq!(
            detect_aspect_ratio(3840, 2160),
            Some(AspectRatio::SixteenNine)
        );
    }

    // ============ Pairing Tests ============

    #[test]
    fn test_find_pair_basic() {
        // One 4:3 + one 16:9 at same timestamp from iPhone
        let assets = vec![
            mock_asset(
                "asset-4-3",
                Some(5712),
                Some(4284),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45.123Z"),
                Some(51.5074),
                Some(-0.1278),
            ),
            mock_asset(
                "asset-16-9",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45.456Z"),
                Some(51.5074),
                Some(-0.1278),
            ),
        ];

        let pairs = find_letterbox_pairs(&assets);

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].keeper.id, "asset-4-3");
        assert_eq!(pairs[0].delete.id, "asset-16-9");
        assert_eq!(pairs[0].camera, "Apple iPhone 15 Pro Max");
    }

    #[test]
    fn test_skip_non_iphone() {
        // Android assets should be ignored
        let assets = vec![
            mock_asset(
                "asset-4-3",
                Some(4000),
                Some(3000),
                Some("Samsung"),
                Some("Galaxy S23"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
            mock_asset(
                "asset-16-9",
                Some(4000),
                Some(2250),
                Some("Samsung"),
                Some("Galaxy S23"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
        ];

        let pairs = find_letterbox_pairs(&assets);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_skip_missing_timestamp() {
        // Assets without dateTimeOriginal should be skipped
        let assets = vec![
            mock_asset(
                "asset-4-3",
                Some(5712),
                Some(4284),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                None, // No timestamp
                None,
                None,
            ),
            mock_asset(
                "asset-16-9",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                None, // No timestamp
                None,
                None,
            ),
        ];

        let pairs = find_letterbox_pairs(&assets);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_skip_ambiguous_two_4_3() {
        // Two 4:3 at same timestamp = ambiguous, skip
        let assets = vec![
            mock_asset(
                "asset-4-3-a",
                Some(5712),
                Some(4284),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
            mock_asset(
                "asset-4-3-b",
                Some(5712),
                Some(4284),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
            mock_asset(
                "asset-16-9",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
        ];

        let pairs = find_letterbox_pairs(&assets);
        assert!(pairs.is_empty()); // Ambiguous, so no pair
    }

    #[test]
    fn test_skip_ambiguous_two_16_9() {
        // Two 16:9 at same timestamp = ambiguous, skip
        let assets = vec![
            mock_asset(
                "asset-4-3",
                Some(5712),
                Some(4284),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
            mock_asset(
                "asset-16-9-a",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
            mock_asset(
                "asset-16-9-b",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
        ];

        let pairs = find_letterbox_pairs(&assets);
        assert!(pairs.is_empty()); // Ambiguous, so no pair
    }

    #[test]
    fn test_multiple_pairs_different_timestamps() {
        // Two separate pairs at different timestamps
        let assets = vec![
            // Pair 1 at 10:30:45
            mock_asset(
                "pair1-4-3",
                Some(5712),
                Some(4284),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
            mock_asset(
                "pair1-16-9",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
            // Pair 2 at 11:00:00
            mock_asset(
                "pair2-4-3",
                Some(5712),
                Some(4284),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T11:00:00Z"),
                None,
                None,
            ),
            mock_asset(
                "pair2-16-9",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T11:00:00Z"),
                None,
                None,
            ),
        ];

        let pairs = find_letterbox_pairs(&assets);
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn test_gps_disambiguation() {
        // Same timestamp but different GPS = separate groups (no pairs formed)
        let assets = vec![
            mock_asset(
                "loc1-4-3",
                Some(5712),
                Some(4284),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                Some(51.5074), // London
                Some(-0.1278),
            ),
            mock_asset(
                "loc2-16-9",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                Some(40.7128), // New York
                Some(-74.0060),
            ),
        ];

        let pairs = find_letterbox_pairs(&assets);
        // Different GPS means different groups, so no pair
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_gps_same_location_pairs() {
        // Same timestamp AND same GPS = should pair
        let assets = vec![
            mock_asset(
                "asset-4-3",
                Some(5712),
                Some(4284),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                Some(51.5074),
                Some(-0.1278),
            ),
            mock_asset(
                "asset-16-9",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                Some(51.5074), // Same GPS
                Some(-0.1278),
            ),
        ];

        let pairs = find_letterbox_pairs(&assets);
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn test_skip_trashed_assets() {
        let mut asset_4_3 = mock_asset(
            "asset-4-3",
            Some(5712),
            Some(4284),
            Some("Apple"),
            Some("iPhone 15 Pro Max"),
            Some("2024-12-23T10:30:45Z"),
            None,
            None,
        );
        asset_4_3.is_trashed = true;

        let asset_16_9 = mock_asset(
            "asset-16-9",
            Some(5712),
            Some(3213),
            Some("Apple"),
            Some("iPhone 15 Pro Max"),
            Some("2024-12-23T10:30:45Z"),
            None,
            None,
        );

        let assets = vec![asset_4_3, asset_16_9];
        let pairs = find_letterbox_pairs(&assets);
        assert!(pairs.is_empty()); // 4:3 is trashed, no pair
    }

    #[test]
    fn test_skip_missing_dimensions() {
        // Assets without dimensions should be skipped
        let assets = vec![
            mock_asset(
                "asset-4-3",
                None, // No width
                None, // No height
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
            mock_asset(
                "asset-16-9",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
        ];

        let pairs = find_letterbox_pairs(&assets);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_different_iphone_models_no_pair() {
        // Same timestamp but different iPhone models = different groups
        let assets = vec![
            mock_asset(
                "asset-4-3",
                Some(5712),
                Some(4284),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
            mock_asset(
                "asset-16-9",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 14 Pro"), // Different model
                Some("2024-12-23T10:30:45Z"),
                None,
                None,
            ),
        ];

        let pairs = find_letterbox_pairs(&assets);
        assert!(pairs.is_empty()); // Different models, no pair
    }

    #[test]
    fn test_only_4_3_no_pair() {
        // Only 4:3 images, no 16:9 = no pair
        let assets = vec![mock_asset(
            "asset-4-3",
            Some(5712),
            Some(4284),
            Some("Apple"),
            Some("iPhone 15 Pro Max"),
            Some("2024-12-23T10:30:45Z"),
            None,
            None,
        )];

        let pairs = find_letterbox_pairs(&assets);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_only_16_9_no_pair() {
        // Only 16:9 images, no 4:3 = no pair
        let assets = vec![mock_asset(
            "asset-16-9",
            Some(5712),
            Some(3213),
            Some("Apple"),
            Some("iPhone 15 Pro Max"),
            Some("2024-12-23T10:30:45Z"),
            None,
            None,
        )];

        let pairs = find_letterbox_pairs(&assets);
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_subsecond_timestamp_handling() {
        // Different sub-second precision should still match (truncated to second)
        let assets = vec![
            mock_asset(
                "asset-4-3",
                Some(5712),
                Some(4284),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45.123Z"),
                None,
                None,
            ),
            mock_asset(
                "asset-16-9",
                Some(5712),
                Some(3213),
                Some("Apple"),
                Some("iPhone 15 Pro Max"),
                Some("2024-12-23T10:30:45.999Z"), // Different sub-second
                None,
                None,
            ),
        ];

        let pairs = find_letterbox_pairs(&assets);
        assert_eq!(pairs.len(), 1); // Should pair (same second)
    }
}
