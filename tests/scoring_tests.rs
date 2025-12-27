//! Unit tests for scoring and analysis logic using recorded API fixtures.
//!
//! These tests use pre-recorded Immich API responses, so they:
//! - Run fast (no Docker, no network)
//! - Are reliable (no flaky CLIP detection)
//! - Test OUR code (scoring, conflict detection, winner selection)
//!
//! To update recorded fixtures:
//!   cd tests/docker
//!   ./bootstrap.sh && ./seed-fixtures.sh
//!   ./record-fixtures.sh

use immich_lib::models::DuplicateGroup;
use immich_lib::scoring::MetadataConflict;
use immich_lib::DuplicateAnalysis;

/// Load recorded duplicate groups from fixture file.
fn load_recorded_duplicates() -> Vec<DuplicateGroup> {
    let json = include_str!("fixtures/recorded/duplicates.json");
    serde_json::from_str(json).expect("Failed to parse recorded duplicates")
}

/// Find a duplicate group containing a specific filename.
fn find_group_by_filename<'a>(
    groups: &'a [DuplicateGroup],
    filename: &str,
) -> Option<&'a DuplicateGroup> {
    groups.iter().find(|g| {
        g.assets.iter().any(|a| a.original_file_name == filename)
    })
}

// ============================================================================
// Winner Selection Tests (W1-W8)
// ============================================================================

mod winner_selection {
    use super::*;

    #[test]
    fn test_w1_clear_dimension_winner() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "w1_large.jpg")
            .expect("W1 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert_eq!(analysis.winner.filename, "w1_large.jpg",
            "W1: Larger dimensions should win");
    }

    #[test]
    fn test_w2_same_dimensions_different_size() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "w2_a.jpg")
            .expect("W2 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        // Both have same dimensions - either is acceptable
        assert!(
            analysis.winner.filename == "w2_a.jpg" ||
            analysis.winner.filename == "w2_b.jpg",
            "W2: One of the files should win"
        );
    }

    #[test]
    fn test_w3_identical_files() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "w3_a.jpg")
            .expect("W3 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        // Both identical - either is acceptable
        assert!(
            analysis.winner.filename == "w3_a.jpg" ||
            analysis.winner.filename == "w3_b.jpg",
            "W3: One of the files should win"
        );
    }

    #[test]
    fn test_w4_one_with_dimensions() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "w4_with_dims.jpg")
            .expect("W4 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert_eq!(analysis.winner.filename, "w4_with_dims.jpg",
            "W4: File with dimensions should win");
    }

    #[test]
    fn test_w5_larger_dimensions_wins() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "w5_with_dims.jpg")
            .expect("W5 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        // In actual fixture, w5_no_dims has 600x400, w5_with_dims has 594x396
        // So no_dims wins due to larger dimensions
        assert_eq!(analysis.winner.filename, "w5_no_dims.jpg",
            "W5: Larger dimensions should win (600x400 vs 594x396)");
    }

    #[test]
    fn test_w6_no_dimensions() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "w6_a.jpg")
            .expect("W6 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        // Neither has dimensions - either is acceptable
        assert!(
            analysis.winner.filename == "w6_a.jpg" ||
            analysis.winner.filename == "w6_b.jpg",
            "W6: One of the files should win"
        );
    }

    #[test]
    fn test_w7_three_duplicates() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "w7_large.jpg")
            .expect("W7 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert_eq!(analysis.winner.filename, "w7_large.jpg",
            "W7: Largest should win");
        assert_eq!(analysis.losers.len(), 2, "W7: Should have 2 losers");
    }

    #[test]
    fn test_w8_same_pixels_different_aspect() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "w8_wide.jpg")
            .expect("W8 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        // Same pixel count - either is acceptable
        assert!(
            analysis.winner.filename == "w8_wide.jpg" ||
            analysis.winner.filename == "w8_tall.jpg",
            "W8: One of the files should win"
        );
    }
}

// ============================================================================
// Conflict Detection Tests (F1-F7)
// ============================================================================

mod conflict_detection {
    use super::*;

    fn has_gps_conflict(analysis: &DuplicateAnalysis) -> bool {
        analysis.conflicts.iter().any(|c| matches!(c, MetadataConflict::Gps { .. }))
    }

    fn has_timezone_conflict(analysis: &DuplicateAnalysis) -> bool {
        analysis.conflicts.iter().any(|c| matches!(c, MetadataConflict::Timezone { .. }))
    }

    fn has_camera_conflict(analysis: &DuplicateAnalysis) -> bool {
        analysis.conflicts.iter().any(|c| matches!(c, MetadataConflict::CameraInfo { .. }))
    }

    fn has_capture_time_conflict(analysis: &DuplicateAnalysis) -> bool {
        analysis.conflicts.iter().any(|c| matches!(c, MetadataConflict::CaptureTime { .. }))
    }

    #[test]
    fn test_f1_gps_conflict_london_vs_paris() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "f1_london.jpg")
            .expect("F1 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert!(has_gps_conflict(&analysis),
            "F1: Should detect GPS conflict (London vs Paris)");
    }

    #[test]
    fn test_f2_gps_within_threshold_no_conflict() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "f2_pos_a.jpg")
            .expect("F2 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert!(!has_gps_conflict(&analysis),
            "F2: Should NOT detect GPS conflict (within threshold)");
    }

    #[test]
    fn test_f3_timezone_conflict() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "f3_tz_a.jpg")
            .expect("F3 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert!(has_timezone_conflict(&analysis),
            "F3: Should detect timezone conflict");
    }

    #[test]
    fn test_f4_camera_conflict() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "f4_canon.jpg")
            .expect("F4 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert!(has_camera_conflict(&analysis),
            "F4: Should detect camera conflict (Canon vs Nikon)");
    }

    #[test]
    fn test_f5_capture_time_conflict() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "f5_morning.jpg")
            .expect("F5 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert!(has_capture_time_conflict(&analysis),
            "F5: Should detect capture time conflict");
    }

    #[test]
    fn test_f6_multiple_conflicts() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "f6_a.jpg")
            .expect("F6 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert!(has_gps_conflict(&analysis), "F6: Should have GPS conflict");
        assert!(has_camera_conflict(&analysis), "F6: Should have camera conflict");
        assert!(has_timezone_conflict(&analysis), "F6: Should have timezone conflict");
    }

    #[test]
    fn test_f7_no_conflicts() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "f7_a.jpg")
            .expect("F7 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert!(analysis.conflicts.is_empty(),
            "F7: Should have no conflicts, got {:?}", analysis.conflicts);
    }
}

// ============================================================================
// Consolidation Scenario Tests (C1-C8)
// ============================================================================

mod consolidation {
    use super::*;

    #[test]
    fn test_c1_winner_no_gps_loser_has_gps() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "c1_winner_no_gps.jpg")
            .expect("C1 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert_eq!(analysis.winner.filename, "c1_winner_no_gps.jpg");

        // Loser should have GPS (consolidation opportunity)
        let loser = analysis.losers.first().expect("C1 should have a loser");
        assert!(loser.score.gps > 0, "C1: Loser should have GPS metadata");
    }

    #[test]
    fn test_c3_winner_no_desc_loser_has_desc() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "c3_winner_no_desc.jpg")
            .expect("C3 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert_eq!(analysis.winner.filename, "c3_winner_no_desc.jpg");
    }

    #[test]
    fn test_c4_winner_bare_loser_rich() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "c4_winner_bare.jpg")
            .expect("C4 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert_eq!(analysis.winner.filename, "c4_winner_bare.jpg");

        // Loser should have rich metadata
        let loser = analysis.losers.first().expect("C4 should have a loser");
        assert!(loser.score.total > analysis.winner.score.total,
            "C4: Loser should have more metadata than winner");
    }

    #[test]
    fn test_c6_multiple_losers_contribute() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "c6_winner.jpg")
            .expect("C6 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert_eq!(analysis.winner.filename, "c6_winner.jpg");
        assert_eq!(analysis.losers.len(), 2, "C6: Should have 2 losers");
    }

    #[test]
    fn test_c8_winner_already_complete() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "c8_winner_full.jpg")
            .expect("C8 group not found");

        let analysis = DuplicateAnalysis::from_group(group);
        assert_eq!(analysis.winner.filename, "c8_winner_full.jpg");

        // Winner should have more or equal metadata to loser
        let loser = analysis.losers.first().expect("C8 should have a loser");
        assert!(analysis.winner.score.total >= loser.score.total,
            "C8: Winner should have at least as much metadata as loser");
    }
}

// ============================================================================
// Edge Case Tests (X scenarios)
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_x1_single_asset_not_in_duplicates() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "x1_single.jpg");

        assert!(group.is_none(),
            "X1: Single asset should NOT appear in duplicate groups");
    }

    #[test]
    fn test_x2_large_group() {
        let groups = load_recorded_duplicates();
        let group = find_group_by_filename(&groups, "x2_dup_00.jpg");

        // X2 may or may not be detected depending on CLIP similarity
        if let Some(g) = group {
            assert!(g.assets.len() >= 2, "X2: If found, should have multiple assets");
        }
    }

    #[test]
    fn test_x3_large_file_wins() {
        let groups = load_recorded_duplicates();
        if let Some(group) = find_group_by_filename(&groups, "x3_large.jpg") {
            let analysis = DuplicateAnalysis::from_group(group);

            // Large file (600x400) should win over small (594x396)
            assert_eq!(analysis.winner.filename, "x3_large.jpg",
                "X3: Larger file should win on dimensions");
        }
    }

    #[test]
    fn test_recorded_fixture_loads() {
        let groups = load_recorded_duplicates();
        assert!(!groups.is_empty(), "Should have recorded duplicate groups");

        // Basic sanity checks on recorded data
        for group in &groups {
            assert!(!group.duplicate_id.is_empty(), "Groups should have IDs");
            assert!(!group.assets.is_empty(), "Groups should have assets");

            for asset in &group.assets {
                assert!(!asset.id.is_empty(), "Assets should have IDs");
                assert!(!asset.original_file_name.is_empty(), "Assets should have filenames");
            }
        }
    }
}
