//! Test fixture specifications for all 34 test scenarios.
//!
//! Each fixture defines the exact images, metadata, and expected outcomes
//! for reproducible integration testing.

use chrono::{TimeZone, Utc};

use super::generator::{ExifSpec, ImageSpec, TestImage};
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

/// Returns fixture definitions for all 34 test scenarios.
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
        x6_heic(),
        x7_png(),
        x8_raw(),
        x9_unicode_description(),
        x10_very_old_date(),
        x11_future_date(),
    ]
}

// ===== Winner Selection Scenarios =====

fn w1_clear_dimension_winner() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W1ClearDimensionWinner,
        images: vec![
            TestImage {
                filename: "w1_large.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [255, 0, 0], // red
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "w1_small.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(750),
                    color: [255, 0, 0],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0,
        description: "Larger dimensions should win (2000x1500 vs 1000x750)".into(),
    }
}

fn w2_same_dimensions_different_size() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W2SameDimensionsDifferentSize,
        images: vec![
            TestImage {
                filename: "w2_a.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1920),
                    height: Some(1080),
                    color: [255, 128, 0], // orange
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "w2_b.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1920),
                    height: Some(1080),
                    color: [255, 128, 0],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0, // first when tied
        description: "Same dimensions - first in list wins on tie".into(),
    }
}

fn w3_same_dimensions_same_size() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W3SameDimensionsSameSize,
        images: vec![
            TestImage {
                filename: "w3_a.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(1000),
                    color: [255, 255, 0], // yellow
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "w3_b.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(1000),
                    color: [255, 255, 0],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0,
        description: "Identical dimensions and size - first wins".into(),
    }
}

fn w4_some_missing_dimensions() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W4SomeMissingDimensions,
        images: vec![
            TestImage {
                filename: "w4_with_dims.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1600),
                    height: Some(1200),
                    color: [0, 255, 0], // green
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "w4_no_dims.jpg".into(),
                image_spec: ImageSpec {
                    width: None,
                    height: None,
                    color: [0, 255, 0],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0,
        description: "Asset with dimensions beats asset without".into(),
    }
}

fn w5_only_one_has_dimensions() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W5OnlyOneHasDimensions,
        images: vec![
            TestImage {
                filename: "w5_no_dims.jpg".into(),
                image_spec: ImageSpec {
                    width: None,
                    height: None,
                    color: [0, 128, 255], // sky blue
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "w5_with_dims.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(800),
                    height: Some(600),
                    color: [0, 128, 255],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 1,
        description: "Only second asset has dimensions - it wins".into(),
    }
}

fn w6_all_missing_dimensions() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W6AllMissingDimensions,
        images: vec![
            TestImage {
                filename: "w6_a.jpg".into(),
                image_spec: ImageSpec {
                    width: None,
                    height: None,
                    color: [0, 0, 255], // blue
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "w6_b.jpg".into(),
                image_spec: ImageSpec {
                    width: None,
                    height: None,
                    color: [0, 0, 255],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0,
        description: "No dimensions on any - first wins".into(),
    }
}

fn w7_three_plus_duplicates() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W7ThreePlusDuplicates,
        images: vec![
            TestImage {
                filename: "w7_small.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(640),
                    height: Some(480),
                    color: [128, 0, 255], // purple
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "w7_large.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(3000),
                    height: Some(2000),
                    color: [128, 0, 255],
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "w7_medium.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1920),
                    height: Some(1080),
                    color: [128, 0, 255],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 1, // largest
        description: "3 duplicates - largest dimensions wins".into(),
    }
}

fn w8_same_pixels_different_aspect() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::W8SamePixelsDifferentAspect,
        images: vec![
            TestImage {
                filename: "w8_wide.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1000), // 2M pixels
                    color: [255, 0, 255], // magenta
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "w8_tall.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(2000), // 2M pixels
                    color: [255, 0, 255],
                },
                exif_spec: ExifSpec::default(),
            },
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
            TestImage {
                filename: "c1_winner_no_gps.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(3000),
                    height: Some(2000),
                    color: [200, 100, 100],
                },
                exif_spec: ExifSpec::default(), // no GPS
            },
            TestImage {
                filename: "c1_loser_has_gps.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1500),
                    height: Some(1000),
                    color: [200, 100, 100],
                },
                exif_spec: ExifSpec {
                    gps: Some((51.5074, -0.1278)), // London
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "Winner lacks GPS, loser has it - consolidate GPS".into(),
    }
}

fn c2_winner_lacks_datetime_loser_has() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C2WinnerLacksDatetimeLoserHas,
        images: vec![
            TestImage {
                filename: "c2_winner_no_dt.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2500),
                    height: Some(1875),
                    color: [100, 200, 100],
                },
                exif_spec: ExifSpec::default(), // no datetime
            },
            TestImage {
                filename: "c2_loser_has_dt.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1250),
                    height: Some(938),
                    color: [100, 200, 100],
                },
                exif_spec: ExifSpec {
                    datetime: Some(Utc.with_ymd_and_hms(2024, 6, 15, 14, 30, 0).unwrap()),
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "Winner lacks datetime, loser has it".into(),
    }
}

fn c3_winner_lacks_description_loser_has() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C3WinnerLacksDescriptionLoserHas,
        images: vec![
            TestImage {
                filename: "c3_winner_no_desc.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2200),
                    height: Some(1650),
                    color: [100, 100, 200],
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "c3_loser_has_desc.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1100),
                    height: Some(825),
                    color: [100, 100, 200],
                },
                exif_spec: ExifSpec {
                    description: Some("Sunset at the beach".into()),
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "Winner lacks description, loser has it".into(),
    }
}

fn c4_winner_lacks_all_loser_has_all() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C4WinnerLacksAllLoserHasAll,
        images: vec![
            TestImage {
                filename: "c4_winner_bare.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(4000),
                    height: Some(3000),
                    color: [150, 150, 50],
                },
                exif_spec: ExifSpec::default(), // no metadata
            },
            TestImage {
                filename: "c4_loser_rich.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [150, 150, 50],
                },
                exif_spec: ExifSpec {
                    gps: Some((40.7128, -74.0060)), // NYC
                    datetime: Some(Utc.with_ymd_and_hms(2023, 12, 25, 10, 0, 0).unwrap()),
                    timezone: Some("-05:00".into()),
                    camera_make: Some("Canon".into()),
                    camera_model: Some("EOS R5".into()),
                    description: Some("Christmas morning".into()),
                },
            },
        ],
        expected_winner_index: 0,
        description: "Winner has no metadata, loser has everything".into(),
    }
}

fn c5_both_have_gps() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C5BothHaveGps,
        images: vec![
            TestImage {
                filename: "c5_a_gps.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2400),
                    height: Some(1600),
                    color: [50, 150, 150],
                },
                exif_spec: ExifSpec {
                    gps: Some((48.8566, 2.3522)), // Paris
                    ..Default::default()
                },
            },
            TestImage {
                filename: "c5_b_gps.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1200),
                    height: Some(800),
                    color: [50, 150, 150],
                },
                exif_spec: ExifSpec {
                    gps: Some((48.8566, 2.3522)), // Same GPS
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "Both have same GPS - no consolidation needed".into(),
    }
}

fn c6_multiple_losers_contribute() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C6MultipleLosersContribute,
        images: vec![
            TestImage {
                filename: "c6_winner.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(3200),
                    height: Some(2400),
                    color: [200, 50, 150],
                },
                exif_spec: ExifSpec::default(), // nothing
            },
            TestImage {
                filename: "c6_loser_gps.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1600),
                    height: Some(1200),
                    color: [200, 50, 150],
                },
                exif_spec: ExifSpec {
                    gps: Some((35.6762, 139.6503)), // Tokyo
                    ..Default::default()
                },
            },
            TestImage {
                filename: "c6_loser_dt.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(800),
                    height: Some(600),
                    color: [200, 50, 150],
                },
                exif_spec: ExifSpec {
                    datetime: Some(Utc.with_ymd_and_hms(2024, 3, 20, 9, 0, 0).unwrap()),
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "Multiple losers contribute different metadata".into(),
    }
}

fn c7_no_loser_has_needed() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C7NoLoserHasNeeded,
        images: vec![
            TestImage {
                filename: "c7_winner.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2800),
                    height: Some(2100),
                    color: [100, 200, 50],
                },
                exif_spec: ExifSpec::default(), // needs GPS
            },
            TestImage {
                filename: "c7_loser.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1400),
                    height: Some(1050),
                    color: [100, 200, 50],
                },
                exif_spec: ExifSpec::default(), // also no GPS
            },
        ],
        expected_winner_index: 0,
        description: "Winner lacks GPS but no loser has it either".into(),
    }
}

fn c8_winner_has_everything() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::C8WinnerHasEverything,
        images: vec![
            TestImage {
                filename: "c8_winner_full.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(3600),
                    height: Some(2400),
                    color: [50, 100, 200],
                },
                exif_spec: ExifSpec {
                    gps: Some((37.7749, -122.4194)), // SF
                    datetime: Some(Utc.with_ymd_and_hms(2024, 7, 4, 12, 0, 0).unwrap()),
                    timezone: Some("-07:00".into()),
                    camera_make: Some("Sony".into()),
                    camera_model: Some("A7R IV".into()),
                    description: Some("Golden Gate at noon".into()),
                },
            },
            TestImage {
                filename: "c8_loser_bare.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1800),
                    height: Some(1200),
                    color: [50, 100, 200],
                },
                exif_spec: ExifSpec::default(), // nothing
            },
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
            TestImage {
                filename: "f1_london.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [255, 100, 100],
                },
                exif_spec: ExifSpec {
                    gps: Some((51.5074, -0.1278)), // London
                    ..Default::default()
                },
            },
            TestImage {
                filename: "f1_paris.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(750),
                    color: [255, 100, 100],
                },
                exif_spec: ExifSpec {
                    gps: Some((48.8566, 2.3522)), // Paris - clearly different
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "GPS conflict - London vs Paris (should flag conflict)".into(),
    }
}

fn f2_gps_within_threshold() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F2GpsWithinThreshold,
        images: vec![
            TestImage {
                filename: "f2_pos_a.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [100, 255, 100],
                },
                exif_spec: ExifSpec {
                    gps: Some((51.50740, -0.12780)),
                    ..Default::default()
                },
            },
            TestImage {
                filename: "f2_pos_b.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(750),
                    color: [100, 255, 100],
                },
                exif_spec: ExifSpec {
                    gps: Some((51.50745, -0.12785)), // ~5m away, within threshold
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "GPS within threshold - should NOT conflict".into(),
    }
}

fn f3_timezone_conflict() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F3TimezoneConflict,
        images: vec![
            TestImage {
                filename: "f3_tz_a.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [100, 100, 255],
                },
                exif_spec: ExifSpec {
                    timezone: Some("+00:00".into()), // UTC
                    ..Default::default()
                },
            },
            TestImage {
                filename: "f3_tz_b.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(750),
                    color: [100, 100, 255],
                },
                exif_spec: ExifSpec {
                    timezone: Some("-08:00".into()), // PST
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "Timezone conflict - UTC vs PST".into(),
    }
}

fn f4_camera_conflict() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F4CameraConflict,
        images: vec![
            TestImage {
                filename: "f4_canon.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [200, 200, 100],
                },
                exif_spec: ExifSpec {
                    camera_make: Some("Canon".into()),
                    camera_model: Some("EOS R5".into()),
                    ..Default::default()
                },
            },
            TestImage {
                filename: "f4_nikon.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(750),
                    color: [200, 200, 100],
                },
                exif_spec: ExifSpec {
                    camera_make: Some("Nikon".into()),
                    camera_model: Some("Z6 II".into()),
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "Camera conflict - Canon vs Nikon".into(),
    }
}

fn f5_capture_time_conflict() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F5CaptureTimeConflict,
        images: vec![
            TestImage {
                filename: "f5_morning.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [200, 100, 200],
                },
                exif_spec: ExifSpec {
                    datetime: Some(Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap()),
                    ..Default::default()
                },
            },
            TestImage {
                filename: "f5_evening.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(750),
                    color: [200, 100, 200],
                },
                exif_spec: ExifSpec {
                    datetime: Some(Utc.with_ymd_and_hms(2024, 1, 15, 20, 0, 0).unwrap()), // 12h diff
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "Capture time conflict - morning vs evening".into(),
    }
}

fn f6_multiple_conflicts() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F6MultipleConflicts,
        images: vec![
            TestImage {
                filename: "f6_a.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [150, 150, 150],
                },
                exif_spec: ExifSpec {
                    gps: Some((51.5074, -0.1278)), // London
                    camera_make: Some("Canon".into()),
                    timezone: Some("+00:00".into()),
                    ..Default::default()
                },
            },
            TestImage {
                filename: "f6_b.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(750),
                    color: [150, 150, 150],
                },
                exif_spec: ExifSpec {
                    gps: Some((40.7128, -74.0060)), // NYC
                    camera_make: Some("Sony".into()),
                    timezone: Some("-05:00".into()),
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "Multiple conflicts - GPS, camera, timezone all differ".into(),
    }
}

fn f7_no_conflicts() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::F7NoConflicts,
        images: vec![
            TestImage {
                filename: "f7_a.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [180, 180, 180],
                },
                exif_spec: ExifSpec {
                    gps: Some((51.5074, -0.1278)),
                    camera_make: Some("Canon".into()),
                    ..Default::default()
                },
            },
            TestImage {
                filename: "f7_b.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(750),
                    color: [180, 180, 180],
                },
                exif_spec: ExifSpec {
                    gps: Some((51.5074, -0.1278)), // Same GPS
                    camera_make: Some("Canon".into()), // Same camera
                    ..Default::default()
                },
            },
        ],
        expected_winner_index: 0,
        description: "No conflicts - metadata matches".into(),
    }
}

// ===== Edge Case Scenarios =====

fn x1_single_asset_group() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X1SingleAssetGroup,
        images: vec![TestImage {
            filename: "x1_single.jpg".into(),
            image_spec: ImageSpec {
                width: Some(1920),
                height: Some(1080),
                color: [100, 100, 100],
            },
            exif_spec: ExifSpec::default(),
        }],
        expected_winner_index: 0,
        description: "Single asset group - trivial case".into(),
    }
}

fn x2_large_group() -> ScenarioFixture {
    let images: Vec<TestImage> = (0..12)
        .map(|i| TestImage {
            filename: format!("x2_dup_{:02}.jpg", i),
            image_spec: ImageSpec {
                width: Some(1000 + i * 100),
                height: Some(750 + i * 75),
                color: [50 + (i as u8) * 10, 50, 50],
            },
            exif_spec: ExifSpec::default(),
        })
        .collect();

    ScenarioFixture {
        scenario: TestScenario::X2LargeGroup,
        images,
        expected_winner_index: 11, // largest
        description: "12 duplicates - largest should win".into(),
    }
}

fn x3_large_file() -> ScenarioFixture {
    // Note: actual file will be small, but test validates handling
    ScenarioFixture {
        scenario: TestScenario::X3LargeFile,
        images: vec![
            TestImage {
                filename: "x3_large.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(8000),
                    height: Some(6000), // 48MP
                    color: [200, 200, 200],
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "x3_small.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [200, 200, 200],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0,
        description: "Large file handling (48MP image)".into(),
    }
}

fn x4_special_chars_filename() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X4SpecialCharsFilename,
        images: vec![
            TestImage {
                filename: "x4_photo (1).jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [255, 200, 100],
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "x4_photo-copy_2024.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(750),
                    color: [255, 200, 100],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0,
        description: "Filenames with spaces, parens, hyphens".into(),
    }
}

fn x5_video() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X5Video,
        images: vec![
            TestImage {
                filename: "x5_video_hd.mp4".into(),
                image_spec: ImageSpec {
                    width: Some(1920),
                    height: Some(1080),
                    color: [0, 0, 255], // blue for video indicator
                },
                exif_spec: ExifSpec::default(),
            },
            TestImage {
                filename: "x5_video_sd.mp4".into(),
                image_spec: ImageSpec {
                    width: Some(640),
                    height: Some(480),
                    color: [0, 0, 255],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0,
        description: "Video duplicates - HD vs SD".into(),
    }
}

fn x6_heic() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X6Heic,
        images: vec![
            TestImage {
                filename: "x6_photo.heic".into(),
                image_spec: ImageSpec {
                    width: Some(4032),
                    height: Some(3024), // iPhone resolution
                    color: [100, 200, 255],
                },
                exif_spec: ExifSpec {
                    camera_make: Some("Apple".into()),
                    camera_model: Some("iPhone 15 Pro".into()),
                    ..Default::default()
                },
            },
            TestImage {
                filename: "x6_photo_converted.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2016),
                    height: Some(1512),
                    color: [100, 200, 255],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0,
        description: "HEIC vs converted JPEG".into(),
    }
}

fn x7_png() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X7Png,
        images: vec![
            TestImage {
                filename: "x7_image.png".into(),
                image_spec: ImageSpec {
                    width: Some(2048),
                    height: Some(1536),
                    color: [200, 255, 200],
                },
                exif_spec: ExifSpec::default(), // PNG has limited EXIF
            },
            TestImage {
                filename: "x7_image.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1024),
                    height: Some(768),
                    color: [200, 255, 200],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0,
        description: "PNG vs JPEG - PNG larger".into(),
    }
}

fn x8_raw() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X8Raw,
        images: vec![
            TestImage {
                filename: "x8_photo.cr3".into(), // Canon RAW
                image_spec: ImageSpec {
                    width: Some(6720),
                    height: Some(4480),
                    color: [150, 100, 50],
                },
                exif_spec: ExifSpec {
                    camera_make: Some("Canon".into()),
                    camera_model: Some("EOS R5".into()),
                    ..Default::default()
                },
            },
            TestImage {
                filename: "x8_photo.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(6720),
                    height: Some(4480),
                    color: [150, 100, 50],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0, // First on same dimensions
        description: "RAW vs JPEG - same dimensions".into(),
    }
}

fn x9_unicode_description() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X9UnicodeDescription,
        images: vec![
            TestImage {
                filename: "x9_unicode.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [255, 255, 200],
                },
                exif_spec: ExifSpec {
                    description: Some("日本の桜 🌸 Cherry blossoms in 東京".into()),
                    ..Default::default()
                },
            },
            TestImage {
                filename: "x9_plain.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(750),
                    color: [255, 255, 200],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0,
        description: "Unicode in description - Japanese, emoji".into(),
    }
}

fn x10_very_old_date() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X10VeryOldDate,
        images: vec![
            TestImage {
                filename: "x10_old.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1600),
                    height: Some(1200),
                    color: [180, 150, 100], // sepia-ish
                },
                exif_spec: ExifSpec {
                    datetime: Some(Utc.with_ymd_and_hms(1985, 7, 20, 12, 0, 0).unwrap()),
                    ..Default::default()
                },
            },
            TestImage {
                filename: "x10_scan.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(800),
                    height: Some(600),
                    color: [180, 150, 100],
                },
                exif_spec: ExifSpec::default(),
            },
        ],
        expected_winner_index: 0,
        description: "Very old date (1985) - film scan scenario".into(),
    }
}

fn x11_future_date() -> ScenarioFixture {
    ScenarioFixture {
        scenario: TestScenario::X11FutureDate,
        images: vec![
            TestImage {
                filename: "x11_future.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(2000),
                    height: Some(1500),
                    color: [100, 255, 255],
                },
                exif_spec: ExifSpec {
                    datetime: Some(Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()),
                    ..Default::default()
                },
            },
            TestImage {
                filename: "x11_normal.jpg".into(),
                image_spec: ImageSpec {
                    width: Some(1000),
                    height: Some(750),
                    color: [100, 255, 255],
                },
                exif_spec: ExifSpec {
                    datetime: Some(Utc.with_ymd_and_hms(2024, 6, 15, 14, 0, 0).unwrap()),
                    ..Default::default()
                },
            },
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
        assert_eq!(fixtures.len(), 34, "Should have exactly 34 fixtures");
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
