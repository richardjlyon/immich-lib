//! Scenario detection logic for duplicate groups.

use chrono::{Datelike, Utc};

use crate::models::{AssetType, DuplicateGroup};
use crate::scoring::{detect_conflicts, MetadataConflict};

use super::scenarios::{ScenarioMatch, TestScenario};

/// GPS coordinate threshold for conflict detection (~11 meters).
const GPS_THRESHOLD: f64 = 0.0001;

/// Large file threshold in bytes (50MB).
const LARGE_FILE_THRESHOLD: u64 = 50 * 1024 * 1024;

/// Detect all matching test scenarios for a duplicate group.
///
/// Analyzes the group and returns matches for all applicable scenarios.
pub fn detect_scenarios(group: &DuplicateGroup) -> Vec<ScenarioMatch> {
    let mut matches = Vec::new();
    let dup_id = &group.duplicate_id;

    // Group size checks
    detect_group_size_scenarios(group, &mut matches, dup_id);

    // Dimension-based winner selection
    detect_dimension_scenarios(group, &mut matches, dup_id);

    // Consolidation scenarios (winner vs loser metadata)
    detect_consolidation_scenarios(group, &mut matches, dup_id);

    // Conflict scenarios
    detect_conflict_scenarios(group, &mut matches, dup_id);

    // Edge cases
    detect_edge_case_scenarios(group, &mut matches, dup_id);

    matches
}

/// Detect group size scenarios (W7, X1, X2).
fn detect_group_size_scenarios(
    group: &DuplicateGroup,
    matches: &mut Vec<ScenarioMatch>,
    dup_id: &str,
) {
    let count = group.assets.len();

    if count == 1 {
        matches.push(ScenarioMatch {
            scenario: TestScenario::X1SingleAssetGroup,
            duplicate_id: dup_id.to_string(),
            details: "Only 1 asset in group".to_string(),
        });
    }

    if count >= 3 {
        matches.push(ScenarioMatch {
            scenario: TestScenario::W7ThreePlusDuplicates,
            duplicate_id: dup_id.to_string(),
            details: format!("{} assets in group", count),
        });
    }

    if count >= 10 {
        matches.push(ScenarioMatch {
            scenario: TestScenario::X2LargeGroup,
            duplicate_id: dup_id.to_string(),
            details: format!("{} assets in group", count),
        });
    }
}

/// Detect dimension-based winner selection scenarios (W1-W6, W8).
fn detect_dimension_scenarios(
    group: &DuplicateGroup,
    matches: &mut Vec<ScenarioMatch>,
    dup_id: &str,
) {
    // Collect dimensions for each asset
    let dims: Vec<Option<(u32, u32)>> = group
        .assets
        .iter()
        .map(|a| {
            a.exif_info.as_ref().and_then(|e| {
                match (e.exif_image_width, e.exif_image_height) {
                    (Some(w), Some(h)) => Some((w, h)),
                    _ => None,
                }
            })
        })
        .collect();

    let has_dims: Vec<(u32, u32)> = dims.iter().filter_map(|d| *d).collect();
    let with_dims_count = has_dims.len();
    let without_dims_count = dims.len() - with_dims_count;

    // W6: All missing dimensions
    if with_dims_count == 0 && dims.len() > 1 {
        matches.push(ScenarioMatch {
            scenario: TestScenario::W6AllMissingDimensions,
            duplicate_id: dup_id.to_string(),
            details: format!("None of {} assets have dimensions", dims.len()),
        });
        return; // Can't check other dimension scenarios
    }

    // W5: Only one has dimensions
    if with_dims_count == 1 && without_dims_count > 0 {
        matches.push(ScenarioMatch {
            scenario: TestScenario::W5OnlyOneHasDimensions,
            duplicate_id: dup_id.to_string(),
            details: format!(
                "1 asset has dimensions, {} missing",
                without_dims_count
            ),
        });
    }

    // W4: Some missing dimensions (but not all, not just one)
    if with_dims_count > 1 && without_dims_count > 0 {
        matches.push(ScenarioMatch {
            scenario: TestScenario::W4SomeMissingDimensions,
            duplicate_id: dup_id.to_string(),
            details: format!(
                "{} have dimensions, {} missing",
                with_dims_count, without_dims_count
            ),
        });
    }

    // Now analyze dimension differences (only if we have multiple with dims)
    if has_dims.len() >= 2 {
        let pixels: Vec<u64> = has_dims.iter().map(|(w, h)| u64::from(*w) * u64::from(*h)).collect();
        let all_same_pixels = pixels.iter().all(|&p| p == pixels[0]);

        // Check if all dimensions are exactly the same
        let all_same_dims = has_dims.iter().all(|d| *d == has_dims[0]);

        if all_same_dims {
            // W2 or W3: Same dimensions, check file sizes
            let sizes: Vec<Option<u64>> = group
                .assets
                .iter()
                .filter_map(|a| a.exif_info.as_ref())
                .map(|e| e.file_size_in_byte)
                .collect();

            let valid_sizes: Vec<u64> = sizes.iter().filter_map(|s| *s).collect();
            if valid_sizes.len() >= 2 {
                let all_same_size = valid_sizes.iter().all(|&s| s == valid_sizes[0]);
                if all_same_size {
                    matches.push(ScenarioMatch {
                        scenario: TestScenario::W3SameDimensionsSameSize,
                        duplicate_id: dup_id.to_string(),
                        details: format!(
                            "{}x{}, all {} bytes",
                            has_dims[0].0, has_dims[0].1, valid_sizes[0]
                        ),
                    });
                } else {
                    matches.push(ScenarioMatch {
                        scenario: TestScenario::W2SameDimensionsDifferentSize,
                        duplicate_id: dup_id.to_string(),
                        details: format!(
                            "{}x{}, sizes: {:?}",
                            has_dims[0].0, has_dims[0].1, valid_sizes
                        ),
                    });
                }
            }
        } else if all_same_pixels {
            // W8: Same pixel count, different aspect ratio
            matches.push(ScenarioMatch {
                scenario: TestScenario::W8SamePixelsDifferentAspect,
                duplicate_id: dup_id.to_string(),
                details: format!(
                    "Same {} pixels, dims: {:?}",
                    pixels[0], has_dims
                ),
            });
        } else {
            // W1: Clear dimension winner
            matches.push(ScenarioMatch {
                scenario: TestScenario::W1ClearDimensionWinner,
                duplicate_id: dup_id.to_string(),
                details: format!("Dimensions: {:?}", has_dims),
            });
        }
    }
}

/// Detect consolidation scenarios (C1-C8).
fn detect_consolidation_scenarios(
    group: &DuplicateGroup,
    matches: &mut Vec<ScenarioMatch>,
    dup_id: &str,
) {
    if group.assets.len() < 2 {
        return;
    }

    // Determine winner based on dimensions (same logic as DuplicateAnalysis)
    let mut sorted = group.assets.clone();
    sorted.sort_by(|a, b| {
        let pixels_a = a
            .exif_info
            .as_ref()
            .and_then(|e| match (e.exif_image_width, e.exif_image_height) {
                (Some(w), Some(h)) => Some(u64::from(w) * u64::from(h)),
                _ => None,
            })
            .unwrap_or(0);
        let pixels_b = b
            .exif_info
            .as_ref()
            .and_then(|e| match (e.exif_image_width, e.exif_image_height) {
                (Some(w), Some(h)) => Some(u64::from(w) * u64::from(h)),
                _ => None,
            })
            .unwrap_or(0);

        match pixels_b.cmp(&pixels_a) {
            std::cmp::Ordering::Equal => {
                let size_a = a
                    .exif_info
                    .as_ref()
                    .and_then(|e| e.file_size_in_byte)
                    .unwrap_or(0);
                let size_b = b
                    .exif_info
                    .as_ref()
                    .and_then(|e| e.file_size_in_byte)
                    .unwrap_or(0);
                size_b.cmp(&size_a)
            }
            other => other,
        }
    });

    let winner = &sorted[0];
    let losers = &sorted[1..];

    // Check winner metadata
    let winner_exif = winner.exif_info.as_ref();
    let winner_has_gps = winner_exif.is_some_and(|e| e.has_gps());
    let winner_has_datetime = winner_exif.is_some_and(|e| e.date_time_original.is_some());
    let winner_has_description = winner_exif
        .is_some_and(|e| e.description.as_ref().is_some_and(|d| !d.is_empty()));

    // Check loser metadata
    let any_loser_has_gps = losers.iter().any(|l| {
        l.exif_info.as_ref().is_some_and(|e| e.has_gps())
    });
    let any_loser_has_datetime = losers.iter().any(|l| {
        l.exif_info.as_ref().is_some_and(|e| e.date_time_original.is_some())
    });
    let any_loser_has_description = losers.iter().any(|l| {
        l.exif_info.as_ref()
            .is_some_and(|e| e.description.as_ref().is_some_and(|d| !d.is_empty()))
    });

    // C8: Winner has everything
    if winner_has_gps && winner_has_datetime && winner_has_description {
        matches.push(ScenarioMatch {
            scenario: TestScenario::C8WinnerHasEverything,
            duplicate_id: dup_id.to_string(),
            details: "Winner has GPS, datetime, description".to_string(),
        });
    }

    // C5: Both have GPS
    if winner_has_gps && any_loser_has_gps {
        matches.push(ScenarioMatch {
            scenario: TestScenario::C5BothHaveGps,
            duplicate_id: dup_id.to_string(),
            details: "Winner and loser(s) have GPS".to_string(),
        });
    }

    // C1: Winner lacks GPS, loser has GPS
    if !winner_has_gps && any_loser_has_gps {
        matches.push(ScenarioMatch {
            scenario: TestScenario::C1WinnerLacksGpsLoserHas,
            duplicate_id: dup_id.to_string(),
            details: "Winner missing GPS, loser has it".to_string(),
        });
    }

    // C2: Winner lacks datetime, loser has datetime
    if !winner_has_datetime && any_loser_has_datetime {
        matches.push(ScenarioMatch {
            scenario: TestScenario::C2WinnerLacksDatetimeLoserHas,
            duplicate_id: dup_id.to_string(),
            details: "Winner missing datetime, loser has it".to_string(),
        });
    }

    // C3: Winner lacks description, loser has description
    if !winner_has_description && any_loser_has_description {
        matches.push(ScenarioMatch {
            scenario: TestScenario::C3WinnerLacksDescriptionLoserHas,
            duplicate_id: dup_id.to_string(),
            details: "Winner missing description, loser has it".to_string(),
        });
    }

    // C4: Winner lacks all, loser has all
    if !winner_has_gps && !winner_has_datetime && !winner_has_description {
        let loser_has_all = losers.iter().any(|l| {
            let e = l.exif_info.as_ref();
            let has_gps = e.is_some_and(|e| e.has_gps());
            let has_dt = e.is_some_and(|e| e.date_time_original.is_some());
            let has_desc = e.is_some_and(|e| e.description.as_ref().is_some_and(|d| !d.is_empty()));
            has_gps && has_dt && has_desc
        });
        if loser_has_all {
            matches.push(ScenarioMatch {
                scenario: TestScenario::C4WinnerLacksAllLoserHasAll,
                duplicate_id: dup_id.to_string(),
                details: "Winner lacks GPS/datetime/description, loser has all".to_string(),
            });
        }
    }

    // C6: Multiple losers contribute different fields
    if losers.len() >= 2 {
        let loser_gps: Vec<bool> = losers
            .iter()
            .map(|l| l.exif_info.as_ref().is_some_and(|e| e.has_gps()))
            .collect();
        let loser_dt: Vec<bool> = losers
            .iter()
            .map(|l| l.exif_info.as_ref().is_some_and(|e| e.date_time_original.is_some()))
            .collect();
        let loser_desc: Vec<bool> = losers
            .iter()
            .map(|l| {
                l.exif_info.as_ref()
                    .is_some_and(|e| e.description.as_ref().is_some_and(|d| !d.is_empty()))
            })
            .collect();

        // Check if different losers contribute different things
        let gps_sources: Vec<usize> = loser_gps.iter().enumerate().filter_map(|(i, &v)| if v { Some(i) } else { None }).collect();
        let dt_sources: Vec<usize> = loser_dt.iter().enumerate().filter_map(|(i, &v)| if v { Some(i) } else { None }).collect();
        let desc_sources: Vec<usize> = loser_desc.iter().enumerate().filter_map(|(i, &v)| if v { Some(i) } else { None }).collect();

        // If different losers are the source for different fields
        let contributions = [gps_sources.first(), dt_sources.first(), desc_sources.first()];
        let unique_contributors: std::collections::HashSet<_> = contributions.iter().filter_map(|&o| o).collect();
        if unique_contributors.len() >= 2 {
            matches.push(ScenarioMatch {
                scenario: TestScenario::C6MultipleLosersContribute,
                duplicate_id: dup_id.to_string(),
                details: "Different losers contribute different metadata".to_string(),
            });
        }
    }

    // C7: No loser has what winner lacks
    let winner_needs_gps = !winner_has_gps;
    let winner_needs_datetime = !winner_has_datetime;
    let winner_needs_description = !winner_has_description;

    if (winner_needs_gps || winner_needs_datetime || winner_needs_description)
        && !any_loser_has_gps
        && !any_loser_has_datetime
        && !any_loser_has_description
    {
        matches.push(ScenarioMatch {
            scenario: TestScenario::C7NoLoserHasNeeded,
            duplicate_id: dup_id.to_string(),
            details: "Winner missing metadata, no loser has it".to_string(),
        });
    }
}

/// Detect conflict scenarios (F1-F7).
fn detect_conflict_scenarios(
    group: &DuplicateGroup,
    matches: &mut Vec<ScenarioMatch>,
    dup_id: &str,
) {
    let conflicts = detect_conflicts(&group.assets);

    if conflicts.is_empty() {
        matches.push(ScenarioMatch {
            scenario: TestScenario::F7NoConflicts,
            duplicate_id: dup_id.to_string(),
            details: "No metadata conflicts".to_string(),
        });
        return;
    }

    let mut has_gps_conflict = false;
    let mut has_timezone_conflict = false;
    let mut has_camera_conflict = false;
    let mut has_capture_time_conflict = false;

    for conflict in &conflicts {
        match conflict {
            MetadataConflict::Gps { values } => {
                has_gps_conflict = true;
                matches.push(ScenarioMatch {
                    scenario: TestScenario::F1GpsConflict,
                    duplicate_id: dup_id.to_string(),
                    details: format!("{} different locations", values.len()),
                });
            }
            MetadataConflict::Timezone { values } => {
                has_timezone_conflict = true;
                matches.push(ScenarioMatch {
                    scenario: TestScenario::F3TimezoneConflict,
                    duplicate_id: dup_id.to_string(),
                    details: format!("Timezones: {:?}", values),
                });
            }
            MetadataConflict::CameraInfo { values } => {
                has_camera_conflict = true;
                matches.push(ScenarioMatch {
                    scenario: TestScenario::F4CameraConflict,
                    duplicate_id: dup_id.to_string(),
                    details: format!("Cameras: {:?}", values),
                });
            }
            MetadataConflict::CaptureTime { values } => {
                has_capture_time_conflict = true;
                matches.push(ScenarioMatch {
                    scenario: TestScenario::F5CaptureTimeConflict,
                    duplicate_id: dup_id.to_string(),
                    details: format!("Times: {:?}", values),
                });
            }
        }
    }

    // F6: Multiple conflicts
    let conflict_count = [has_gps_conflict, has_timezone_conflict, has_camera_conflict, has_capture_time_conflict]
        .iter()
        .filter(|&&v| v)
        .count();
    if conflict_count >= 2 {
        matches.push(ScenarioMatch {
            scenario: TestScenario::F6MultipleConflicts,
            duplicate_id: dup_id.to_string(),
            details: format!("{} different conflict types", conflict_count),
        });
    }

    // F2: GPS within threshold (no conflict detected but multiple GPS values exist)
    if !has_gps_conflict {
        let gps_values: Vec<(f64, f64)> = group
            .assets
            .iter()
            .filter_map(|a| a.exif_info.as_ref())
            .filter_map(|e| match (e.latitude, e.longitude) {
                (Some(lat), Some(lon)) => Some((lat, lon)),
                _ => None,
            })
            .collect();

        if gps_values.len() >= 2 {
            // Check if they're all within threshold
            let mut all_within = true;
            for i in 0..gps_values.len() {
                for j in (i + 1)..gps_values.len() {
                    let (lat1, lon1) = gps_values[i];
                    let (lat2, lon2) = gps_values[j];
                    if (lat1 - lat2).abs() > GPS_THRESHOLD || (lon1 - lon2).abs() > GPS_THRESHOLD {
                        all_within = false;
                        break;
                    }
                }
            }
            if all_within {
                matches.push(ScenarioMatch {
                    scenario: TestScenario::F2GpsWithinThreshold,
                    duplicate_id: dup_id.to_string(),
                    details: format!("{} GPS values within threshold", gps_values.len()),
                });
            }
        }
    }
}

/// Detect edge case scenarios (X3-X11).
fn detect_edge_case_scenarios(
    group: &DuplicateGroup,
    matches: &mut Vec<ScenarioMatch>,
    dup_id: &str,
) {
    for asset in &group.assets {
        let filename = &asset.original_file_name;
        let lowercase = filename.to_lowercase();

        // X3: Large file (>50MB)
        if let Some(size) = asset.exif_info.as_ref().and_then(|e| e.file_size_in_byte)
            && size > LARGE_FILE_THRESHOLD
        {
            matches.push(ScenarioMatch {
                scenario: TestScenario::X3LargeFile,
                duplicate_id: dup_id.to_string(),
                details: format!("{}: {} bytes", filename, size),
            });
        }

        // X4: Special characters in filename
        if filename.chars().any(|c| "!@#$%^&*()[]{}|;'\"<>?".contains(c)) {
            matches.push(ScenarioMatch {
                scenario: TestScenario::X4SpecialCharsFilename,
                duplicate_id: dup_id.to_string(),
                details: format!("Filename: {}", filename),
            });
        }

        // X5: Video
        if asset.asset_type == AssetType::Video {
            matches.push(ScenarioMatch {
                scenario: TestScenario::X5Video,
                duplicate_id: dup_id.to_string(),
                details: format!("Video: {}", filename),
            });
        }

        // X6: HEIC
        if lowercase.ends_with(".heic") || lowercase.ends_with(".heif") {
            matches.push(ScenarioMatch {
                scenario: TestScenario::X6Heic,
                duplicate_id: dup_id.to_string(),
                details: format!("HEIC: {}", filename),
            });
        }

        // X7: PNG
        if lowercase.ends_with(".png") {
            matches.push(ScenarioMatch {
                scenario: TestScenario::X7Png,
                duplicate_id: dup_id.to_string(),
                details: format!("PNG: {}", filename),
            });
        }

        // X8: RAW files
        let raw_extensions = [
            ".raw", ".cr2", ".cr3", ".nef", ".arw", ".orf", ".rw2", ".dng", ".raf", ".pef",
        ];
        if raw_extensions.iter().any(|ext| lowercase.ends_with(ext)) {
            matches.push(ScenarioMatch {
                scenario: TestScenario::X8Raw,
                duplicate_id: dup_id.to_string(),
                details: format!("RAW: {}", filename),
            });
        }

        // X9: Unicode in description
        if let Some(desc) = asset.exif_info.as_ref().and_then(|e| e.description.as_ref())
            && !desc.is_ascii()
        {
            matches.push(ScenarioMatch {
                scenario: TestScenario::X9UnicodeDescription,
                duplicate_id: dup_id.to_string(),
                details: format!("Description: {}", desc),
            });
        }

        // X10: Very old date (<1990) and X11: Future date
        if let Some(dt) = asset.exif_info.as_ref().and_then(|e| e.date_time_original.as_ref())
            && let Some(year) = extract_year(dt)
        {
            if year < 1990 {
                matches.push(ScenarioMatch {
                    scenario: TestScenario::X10VeryOldDate,
                    duplicate_id: dup_id.to_string(),
                    details: format!("Date: {}", dt),
                });
            }

            let current_year = Utc::now().year();
            if year > current_year {
                matches.push(ScenarioMatch {
                    scenario: TestScenario::X11FutureDate,
                    duplicate_id: dup_id.to_string(),
                    details: format!("Date: {} (future)", dt),
                });
            }
        }
    }
}

/// Extract year from a date string (various formats).
fn extract_year(date_str: &str) -> Option<i32> {
    // Try common formats: "2023:01:15 12:00:00", "2023-01-15T12:00:00Z"
    let cleaned = date_str.replace(':', "-").replace('T', " ");
    let year_str = cleaned.split(['-', ' ', '/']).next()?;
    let year = year_str.parse::<i32>().ok()?;
    if (1800..=2100).contains(&year) {
        Some(year)
    } else {
        None
    }
}
