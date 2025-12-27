//! Test fixture specifications for all 32 test scenarios.
//!
//! Each fixture defines the images, metadata, and expected outcomes
//! for integration testing. All images are created by transforming
//! real base photos to ensure CLIP-based duplicate detection works.

use chrono::{TimeZone, Utc};

use super::generator::{ExifSpec, TestImage, TransformSpec};
use super::scenarios::TestScenario;

/// A complete test fixture for a scenario.
#[derive(Debug, Clone)]
pub struct ScenarioFixture {
    /// The scenario this fixture tests
    pub scenario: TestScenario,
    /// Images in the duplicate group
    pub images: Vec<TestImage>,
    /// Index of expected winner (0-based)
    pub expected_winner_index: usize,
    /// Description of what this tests
    pub description: String,
}

/// Returns fixture definitions for all 32 test scenarios.
pub fn all_fixtures() -> Vec<ScenarioFixture> {
    vec![
        // ===== Winner Selection Scenarios (W) =====
        w1_clear_dimension_winner(),
        w2_same_dimensions_different_size(),
        w3_same_dimensions_same_size(),
        w4_some_missing_dimensions(),
        w5_only_one_has_dimensions(),
        w6_all_missing_dimensions(),
        w7_three_plus_duplicates(),
        w8_same_pixels_different_aspect(),
        // ===== Consolidation Scenarios (C) =====
        c1_winner_lacks_gps_loser_has(),
        c2_winner_lacks_datetime_loser_has(),
        c3_winner_lacks_description_loser_has(),
        c4_winner_lacks_all_loser_has_all(),
        c5_both_have_gps(),
        c6_multiple_losers_contribute(),
        c7_no_loser_has_needed(),
        c8_winner_has_everything(),
        // ===== Conflict Scenarios (F) =====
        f1_gps_conflict(),
        f2_gps_within_threshold(),
        f3_timezone_conflict(),
        f4_camera_conflict(),
        f5_capture_time_conflict(),
        f6_multiple_conflicts(),
        f7_no_conflicts(),
        // ===== Edge Case Scenarios (X) =====
        x1_single_asset_group(),
        x2_large_group(),
        x3_large_file(),
        x4_special_chars_filename(),
        x5_video(),
        x7_png(),
        x9_unicode_description(),
        x10_very_old_date(),
        x11_future_date(),
    ]
}

// ===== Winner Selection Scenarios =====
// Each uses its own unique base image

fn w1_clear_dimension_winner() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W1ClearDimensionWinner,
        images: vec![
            TestImage::new(
                "w1_large.jpg",
                TransformSpec::new("base_w1.jpg").with_scale(100),
            ),
            TestImage::new(
                "w1_small.jpg",
                TransformSpec::new("base_w1.jpg").with_scale(50),
            ),
        ],
        expected_winner_index: 0,
        description: "Larger dimensions should win (100% vs 50% scale)".into(),
    }
}

fn w2_same_dimensions_different_size() -> ScenarioFixture {
    // Same dimensions but different file size via quality
    ScenarioFixture {
        scenario: TestScenario::W2SameDimensionsDifferentSize,
        images: vec![
            TestImage::new(
                "w2_a.jpg",
                TransformSpec::new("base_w2.jpg")
                    .with_scale(75)
                    .with_quality(95),
            ),
            TestImage::new(
                "w2_b.jpg",
                TransformSpec::new("base_w2.jpg")
                    .with_scale(75)
                    .with_quality(60),
            ),
        ],
        expected_winner_index: 0, // first when dimensions tied
        description: "Same dimensions - first in list wins on tie".into(),
    }
}

fn w3_same_dimensions_same_size() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W3SameDimensionsSameSize,
        images: vec![
            TestImage::new(
                "w3_a.jpg",
                TransformSpec::new("base_w3.jpg")
                    .with_scale(60)
                    .with_quality(85),
            ),
            TestImage::new(
                "w3_b.jpg",
                TransformSpec::new("base_w3.jpg")
                    .with_scale(60)
                    .with_quality(85),
            ),
        ],
        expected_winner_index: 0,
        description: "Identical dimensions and size - first wins".into(),
    }
}

fn w4_some_missing_dimensions() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W4SomeMissingDimensions,
        images: vec![
            TestImage::new(
                "w4_with_dims.jpg",
                TransformSpec::new("base_w4.jpg").with_scale(80),
            ),
            TestImage::new(
                "w4_no_dims.jpg",
                TransformSpec::new("base_w4.jpg")
                    .with_scale(80)
                    .without_dimensions(),
            ),
        ],
        expected_winner_index: 0,
        description: "Asset with dimensions beats asset without".into(),
    }
}

fn w5_only_one_has_dimensions() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W5OnlyOneHasDimensions,
        images: vec![
            TestImage::new(
                "w5_no_dims.jpg",
                TransformSpec::new("base_w5.jpg")
                    .with_scale(70)
                    .without_dimensions(),
            ),
            TestImage::new(
                "w5_with_dims.jpg",
                TransformSpec::new("base_w5.jpg").with_scale(50),
            ),
        ],
        expected_winner_index: 1, // second has dimensions
        description: "Only second asset has dimensions - it wins".into(),
    }
}

fn w6_all_missing_dimensions() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W6AllMissingDimensions,
        images: vec![
            TestImage::new(
                "w6_a.jpg",
                TransformSpec::new("base_w6.jpg")
                    .with_scale(60)
                    .without_dimensions(),
            ),
            TestImage::new(
                "w6_b.jpg",
                TransformSpec::new("base_w6.jpg")
                    .with_scale(60)
                    .without_dimensions(),
            ),
        ],
        expected_winner_index: 0,
        description: "No dimensions on any - first wins".into(),
    }
}

fn w7_three_plus_duplicates() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W7ThreePlusDuplicates,
        images: vec![
            TestImage::new(
                "w7_small.jpg",
                TransformSpec::new("base_w7.jpg").with_scale(30),
            ),
            TestImage::new(
                "w7_large.jpg",
                TransformSpec::new("base_w7.jpg").with_scale(100),
            ),
            TestImage::new(
                "w7_medium.jpg",
                TransformSpec::new("base_w7.jpg").with_scale(60),
            ),
        ],
        expected_winner_index: 1, // largest
        description: "3 duplicates - largest dimensions wins".into(),
    }
}

fn w8_same_pixels_different_aspect() -> ScenarioFixture {
    // Use explicit dimensions to control aspect ratio
    ScenarioFixture {
        scenario: TestScenario::W8SamePixelsDifferentAspect,
        images: vec![
            TestImage::new(
                "w8_wide.jpg",
                TransformSpec::new("base_w8.jpg").with_size(2000, 1000),
            ),
            TestImage::new(
                "w8_tall.jpg",
                TransformSpec::new("base_w8.jpg").with_size(1000, 2000),
            ),
        ],
        expected_winner_index: 0, // first on tie
        description: "Same pixel count, different aspect - first wins".into(),
    }
}

// ===== Consolidation Scenarios =====

fn c1_winner_lacks_gps_loser_has() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C1WinnerLacksGpsLoserHas,
        images: vec![
            TestImage::new(
                "c1_winner_no_gps.jpg",
                TransformSpec::new("base_c1.jpg").with_scale(100),
            ),
            TestImage::new(
                "c1_loser_has_gps.jpg",
                TransformSpec::new("base_c1.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                gps: Some((51.5074, -0.1278)), // London
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "Winner lacks GPS, loser has it - consolidate GPS".into(),
    }
}

fn c2_winner_lacks_datetime_loser_has() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C2WinnerLacksDatetimeLoserHas,
        images: vec![
            TestImage::new(
                "c2_winner_no_dt.jpg",
                TransformSpec::new("base_c2.jpg").with_scale(100),
            ),
            TestImage::new(
                "c2_loser_has_dt.jpg",
                TransformSpec::new("base_c2.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                datetime: Some(Utc.with_ymd_and_hms(2024, 6, 15, 14, 30, 0).unwrap()),
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "Winner lacks datetime, loser has it".into(),
    }
}

fn c3_winner_lacks_description_loser_has() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C3WinnerLacksDescriptionLoserHas,
        images: vec![
            TestImage::new(
                "c3_winner_no_desc.jpg",
                TransformSpec::new("base_c3.jpg").with_scale(100),
            ),
            TestImage::new(
                "c3_loser_has_desc.jpg",
                TransformSpec::new("base_c3.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                description: Some("Delicious salad".into()),
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "Winner lacks description, loser has it".into(),
    }
}

fn c4_winner_lacks_all_loser_has_all() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C4WinnerLacksAllLoserHasAll,
        images: vec![
            TestImage::new(
                "c4_winner_bare.jpg",
                TransformSpec::new("base_c4.jpg").with_scale(100),
            ),
            TestImage::new(
                "c4_loser_rich.jpg",
                TransformSpec::new("base_c4.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                gps: Some((40.7128, -74.0060)), // NYC
                datetime: Some(Utc.with_ymd_and_hms(2023, 12, 25, 10, 0, 0).unwrap()),
                timezone: Some("-05:00".into()),
                camera_make: Some("Canon".into()),
                camera_model: Some("EOS R5".into()),
                description: Some("Lion at the zoo".into()),
            }),
        ],
        expected_winner_index: 0,
        description: "Winner has no metadata, loser has everything".into(),
    }
}

fn c5_both_have_gps() -> ScenarioFixture {
    let gps = Some((48.8566, 2.3522)); // Paris
    ScenarioFixture {
        scenario: TestScenario::C5BothHaveGps,
        images: vec![
            TestImage::new(
                "c5_a_gps.jpg",
                TransformSpec::new("base_c5.jpg").with_scale(100),
            )
            .with_exif(ExifSpec {
                gps,
                ..Default::default()
            }),
            TestImage::new(
                "c5_b_gps.jpg",
                TransformSpec::new("base_c5.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                gps,
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "Both have same GPS - no consolidation needed".into(),
    }
}

fn c6_multiple_losers_contribute() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C6MultipleLosersContribute,
        images: vec![
            TestImage::new(
                "c6_winner.jpg",
                TransformSpec::new("base_c6.jpg").with_scale(100),
            ),
            TestImage::new(
                "c6_loser_gps.jpg",
                TransformSpec::new("base_c6.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                gps: Some((35.6762, 139.6503)), // Tokyo
                ..Default::default()
            }),
            TestImage::new(
                "c6_loser_dt.jpg",
                TransformSpec::new("base_c6.jpg").with_scale(30),
            )
            .with_exif(ExifSpec {
                datetime: Some(Utc.with_ymd_and_hms(2024, 3, 20, 9, 0, 0).unwrap()),
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "Multiple losers contribute different metadata".into(),
    }
}

fn c7_no_loser_has_needed() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C7NoLoserHasNeeded,
        images: vec![
            TestImage::new(
                "c7_winner.jpg",
                TransformSpec::new("base_c7.jpg").with_scale(100),
            ),
            TestImage::new(
                "c7_loser.jpg",
                TransformSpec::new("base_c7.jpg").with_scale(50),
            ),
        ],
        expected_winner_index: 0,
        description: "Winner lacks GPS but no loser has it either".into(),
    }
}

fn c8_winner_has_everything() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C8WinnerHasEverything,
        images: vec![
            TestImage::new(
                "c8_winner_full.jpg",
                TransformSpec::new("base_c8.jpg").with_scale(100),
            )
            .with_exif(ExifSpec {
                gps: Some((37.7749, -122.4194)), // SF
                datetime: Some(Utc.with_ymd_and_hms(2024, 7, 4, 12, 0, 0).unwrap()),
                timezone: Some("-07:00".into()),
                camera_make: Some("Sony".into()),
                camera_model: Some("A7R IV".into()),
                description: Some("Golden Gate at noon".into()),
            }),
            TestImage::new(
                "c8_loser_bare.jpg",
                TransformSpec::new("base_c8.jpg").with_scale(50),
            ),
        ],
        expected_winner_index: 0,
        description: "Winner already has all metadata - nothing to consolidate".into(),
    }
}

// ===== Conflict Scenarios =====

fn f1_gps_conflict() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F1GpsConflict,
        images: vec![
            TestImage::new(
                "f1_london.jpg",
                TransformSpec::new("base_f1.jpg").with_scale(100),
            )
            .with_exif(ExifSpec {
                gps: Some((51.5074, -0.1278)), // London
                ..Default::default()
            }),
            TestImage::new(
                "f1_paris.jpg",
                TransformSpec::new("base_f1.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                gps: Some((48.8566, 2.3522)), // Paris
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "GPS conflict - London vs Paris (should flag conflict)".into(),
    }
}

fn f2_gps_within_threshold() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F2GpsWithinThreshold,
        images: vec![
            TestImage::new(
                "f2_pos_a.jpg",
                TransformSpec::new("base_f2.jpg").with_scale(100),
            )
            .with_exif(ExifSpec {
                gps: Some((51.50740, -0.12780)),
                ..Default::default()
            }),
            TestImage::new(
                "f2_pos_b.jpg",
                TransformSpec::new("base_f2.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                gps: Some((51.50745, -0.12785)), // ~5m away
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "GPS within threshold - should NOT conflict".into(),
    }
}

fn f3_timezone_conflict() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F3TimezoneConflict,
        images: vec![
            TestImage::new(
                "f3_tz_a.jpg",
                TransformSpec::new("base_f3.jpg").with_scale(100),
            )
            .with_exif(ExifSpec {
                timezone: Some("+00:00".into()), // UTC
                ..Default::default()
            }),
            TestImage::new(
                "f3_tz_b.jpg",
                TransformSpec::new("base_f3.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                timezone: Some("-08:00".into()), // PST
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "Timezone conflict - UTC vs PST".into(),
    }
}

fn f4_camera_conflict() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F4CameraConflict,
        images: vec![
            TestImage::new(
                "f4_canon.jpg",
                TransformSpec::new("base_f4.jpg").with_scale(100),
            )
            .with_exif(ExifSpec {
                camera_make: Some("Canon".into()),
                camera_model: Some("EOS R5".into()),
                ..Default::default()
            }),
            TestImage::new(
                "f4_nikon.jpg",
                TransformSpec::new("base_f4.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                camera_make: Some("Nikon".into()),
                camera_model: Some("Z6 II".into()),
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "Camera conflict - Canon vs Nikon".into(),
    }
}

fn f5_capture_time_conflict() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F5CaptureTimeConflict,
        images: vec![
            TestImage::new(
                "f5_morning.jpg",
                TransformSpec::new("base_f5.jpg").with_scale(100),
            )
            .with_exif(ExifSpec {
                datetime: Some(Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap()),
                ..Default::default()
            }),
            TestImage::new(
                "f5_evening.jpg",
                TransformSpec::new("base_f5.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                datetime: Some(Utc.with_ymd_and_hms(2024, 1, 15, 20, 0, 0).unwrap()),
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "Capture time conflict - morning vs evening".into(),
    }
}

fn f6_multiple_conflicts() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F6MultipleConflicts,
        images: vec![
            TestImage::new(
                "f6_a.jpg",
                TransformSpec::new("base_f6.jpg").with_scale(100),
            )
            .with_exif(ExifSpec {
                gps: Some((51.5074, -0.1278)), // London
                camera_make: Some("Canon".into()),
                timezone: Some("+00:00".into()),
                ..Default::default()
            }),
            TestImage::new(
                "f6_b.jpg",
                TransformSpec::new("base_f6.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                gps: Some((40.7128, -74.0060)), // NYC
                camera_make: Some("Sony".into()),
                timezone: Some("-05:00".into()),
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "Multiple conflicts - GPS, camera, timezone all differ".into(),
    }
}

fn f7_no_conflicts() -> ScenarioFixture {
    let exif = ExifSpec {
        gps: Some((51.5074, -0.1278)),
        camera_make: Some("Canon".into()),
        ..Default::default()
    };
    ScenarioFixture {
        scenario: TestScenario::F7NoConflicts,
        images: vec![
            TestImage::new(
                "f7_a.jpg",
                TransformSpec::new("base_f7.jpg").with_scale(100),
            )
            .with_exif(exif.clone()),
            TestImage::new(
                "f7_b.jpg",
                TransformSpec::new("base_f7.jpg").with_scale(50),
            )
            .with_exif(exif),
        ],
        expected_winner_index: 0,
        description: "No conflicts - metadata matches".into(),
    }
}

// ===== Edge Case Scenarios =====

fn x1_single_asset_group() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X1SingleAssetGroup,
        images: vec![TestImage::new(
            "x1_single.jpg",
            TransformSpec::new("base_x1.jpg").with_scale(80),
        )],
        expected_winner_index: 0,
        description: "Single asset group - trivial case".into(),
    }
}

fn x2_large_group() -> ScenarioFixture {
    let images: Vec<TestImage> = (0..12)
        .map(|i| {
            TestImage::new(
                format!("x2_dup_{:02}.jpg", i),
                TransformSpec::new("base_x2.jpg").with_scale(30 + i * 5),
            )
        })
        .collect();

    ScenarioFixture {
        scenario: TestScenario::X2LargeGroup,
        images,
        expected_winner_index: 11, // largest (30 + 11*5 = 85%)
        description: "12 duplicates - largest should win".into(),
    }
}

fn x3_large_file() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X3LargeFile,
        images: vec![
            TestImage::new(
                "x3_large.jpg",
                TransformSpec::new("base_x3.jpg")
                    .with_scale(100)
                    .with_quality(100),
            ),
            TestImage::new(
                "x3_small.jpg",
                TransformSpec::new("base_x3.jpg").with_scale(25),
            ),
        ],
        expected_winner_index: 0,
        description: "Large file handling (full size, max quality)".into(),
    }
}

fn x4_special_chars_filename() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X4SpecialCharsFilename,
        images: vec![
            TestImage::new(
                "x4_photo (1).jpg",
                TransformSpec::new("base_x4.jpg").with_scale(100),
            ),
            TestImage::new(
                "x4_photo-copy_2024.jpg",
                TransformSpec::new("base_x4.jpg").with_scale(50),
            ),
        ],
        expected_winner_index: 0,
        description: "Filenames with spaces, parens, hyphens".into(),
    }
}

fn x5_video() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X5Video,
        images: vec![
            TestImage::new(
                "x5_video_hd.mp4",
                TransformSpec::new("base_x5.jpg").with_size(1920, 1080),
            ),
            TestImage::new(
                "x5_video_sd.mp4",
                TransformSpec::new("base_x5.jpg").with_size(640, 480),
            ),
        ],
        expected_winner_index: 0,
        description: "Video duplicates - HD vs SD".into(),
    }
}

fn x7_png() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X7Png,
        images: vec![
            TestImage::new(
                "x7_image.png",
                TransformSpec::new("base_x7.jpg").with_scale(100),
            ),
            TestImage::new(
                "x7_image.jpg",
                TransformSpec::new("base_x7.jpg").with_scale(50),
            ),
        ],
        expected_winner_index: 0,
        description: "PNG vs JPEG - PNG larger".into(),
    }
}

fn x9_unicode_description() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X9UnicodeDescription,
        images: vec![
            TestImage::new(
                "x9_unicode.jpg",
                TransformSpec::new("base_x9.jpg").with_scale(100),
            )
            .with_exif(ExifSpec {
                description: Some("日本の桜 🌸 Cherry blossoms".into()),
                ..Default::default()
            }),
            TestImage::new(
                "x9_plain.jpg",
                TransformSpec::new("base_x9.jpg").with_scale(50),
            ),
        ],
        expected_winner_index: 0,
        description: "Unicode in description - Japanese, emoji".into(),
    }
}

fn x10_very_old_date() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X10VeryOldDate,
        images: vec![
            TestImage::new(
                "x10_old.jpg",
                TransformSpec::new("base_x10.jpg").with_scale(80),
            )
            .with_exif(ExifSpec {
                datetime: Some(Utc.with_ymd_and_hms(1985, 7, 20, 12, 0, 0).unwrap()),
                ..Default::default()
            }),
            TestImage::new(
                "x10_scan.jpg",
                TransformSpec::new("base_x10.jpg").with_scale(40),
            ),
        ],
        expected_winner_index: 0,
        description: "Very old date (1985) - film scan scenario".into(),
    }
}

fn x11_future_date() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X11FutureDate,
        images: vec![
            TestImage::new(
                "x11_future.jpg",
                TransformSpec::new("base_x11.jpg").with_scale(100),
            )
            .with_exif(ExifSpec {
                datetime: Some(Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()),
                ..Default::default()
            }),
            TestImage::new(
                "x11_normal.jpg",
                TransformSpec::new("base_x11.jpg").with_scale(50),
            )
            .with_exif(ExifSpec {
                datetime: Some(Utc.with_ymd_and_hms(2024, 6, 15, 14, 0, 0).unwrap()),
                ..Default::default()
            }),
        ],
        expected_winner_index: 0,
        description: "Future date (2030) - camera clock error scenario".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_fixtures_count() {
        let fixtures = all_fixtures();
        assert_eq!(fixtures.len(), 32, "Should have exactly 32 fixtures");
    }

    #[test]
    fn test_all_scenarios_covered() {
        let fixtures = all_fixtures();
        let all_scenarios = TestScenario::all();

        for scenario in &all_scenarios {
            let found = fixtures.iter().any(|f| f.scenario == *scenario);
            assert!(found, "Missing fixture for scenario: {:?}", scenario);
        }
    }

    #[test]
    fn test_each_fixture_has_images() {
        let fixtures = all_fixtures();
        for fixture in &fixtures {
            assert!(
                !fixture.images.is_empty(),
                "Fixture {:?} has no images",
                fixture.scenario
            );
        }
    }

    #[test]
    fn test_winner_index_valid() {
        let fixtures = all_fixtures();
        for fixture in &fixtures {
            assert!(
                fixture.expected_winner_index < fixture.images.len(),
                "Fixture {:?} has invalid winner index {} (only {} images)",
                fixture.scenario,
                fixture.expected_winner_index,
                fixture.images.len()
            );
        }
    }
}
