//! Assertion utilities for integration tests.
//!
//! Provides helpers for matching duplicate groups and verifying winner selection.

use super::fixtures::Manifest;
use super::harness::DuplicateGroupResponse;

use immich_lib::DuplicateAnalysis;

/// Find the duplicate group matching a manifest's files.
///
/// Matches by checking if the group's asset filenames match the manifest's images.
///
/// # Arguments
///
/// * `duplicates` - Slice of duplicate groups from the API
/// * `manifest` - The manifest to match against
///
/// # Returns
///
/// The matching group if found, None otherwise.
#[allow(dead_code)]
pub fn find_scenario_group<'a>(
    duplicates: &'a [DuplicateGroupResponse],
    manifest: &Manifest,
) -> Option<&'a DuplicateGroupResponse> {
    for group in duplicates {
        // Get filenames from the group
        let group_filenames: Vec<&str> = group
            .assets
            .iter()
            .map(|a| a.original_file_name.as_str())
            .collect();

        // Check if all manifest images are present in this group
        let all_present = manifest.images.iter().all(|img| {
            group_filenames.iter().any(|gf| gf == img)
        });

        // Check if group has approximately the right number of files
        // (allow for some flexibility in case of partial matches)
        if all_present && group.assets.len() >= manifest.images.len() {
            return Some(group);
        }
    }

    None
}

/// Assert that the winner matches the expected winner from the manifest.
///
/// # Arguments
///
/// * `analysis` - The duplicate analysis result
/// * `expected_winner` - The expected winner filename
///
/// # Panics
///
/// Panics if the winner doesn't match, with a descriptive message.
#[allow(dead_code)]
pub fn assert_winner_matches(analysis: &DuplicateAnalysis, expected_winner: &str) {
    let actual = &analysis.winner.filename;

    assert_eq!(
        actual, expected_winner,
        "Winner mismatch!\n\
         Expected: {}\n\
         Actual: {}\n\
         Group ID: {}\n\
         Losers: {:?}",
        expected_winner,
        actual,
        analysis.duplicate_id,
        analysis.losers.iter().map(|l| &l.filename).collect::<Vec<_>>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::harness::{DuplicateAsset, DuplicateGroupResponse};

    #[test]
    fn test_find_scenario_group_found() {
        let groups = vec![
            DuplicateGroupResponse {
                duplicate_id: "group1".to_string(),
                assets: vec![
                    DuplicateAsset {
                        id: "a1".to_string(),
                        original_file_name: "photo1.jpg".to_string(),
                    },
                    DuplicateAsset {
                        id: "a2".to_string(),
                        original_file_name: "photo2.jpg".to_string(),
                    },
                ],
            },
            DuplicateGroupResponse {
                duplicate_id: "group2".to_string(),
                assets: vec![
                    DuplicateAsset {
                        id: "a3".to_string(),
                        original_file_name: "w1_large.jpg".to_string(),
                    },
                    DuplicateAsset {
                        id: "a4".to_string(),
                        original_file_name: "w1_small.jpg".to_string(),
                    },
                ],
            },
        ];

        let manifest = Manifest {
            scenario: "W1".to_string(),
            description: "Test".to_string(),
            images: vec!["w1_large.jpg".to_string(), "w1_small.jpg".to_string()],
            expected_winner: "w1_large.jpg".to_string(),
        };

        let found = find_scenario_group(&groups, &manifest);
        assert!(found.is_some());
        assert_eq!(found.unwrap().duplicate_id, "group2");
    }

    #[test]
    fn test_find_scenario_group_not_found() {
        let groups = vec![
            DuplicateGroupResponse {
                duplicate_id: "group1".to_string(),
                assets: vec![
                    DuplicateAsset {
                        id: "a1".to_string(),
                        original_file_name: "photo1.jpg".to_string(),
                    },
                ],
            },
        ];

        let manifest = Manifest {
            scenario: "W1".to_string(),
            description: "Test".to_string(),
            images: vec!["w1_large.jpg".to_string(), "w1_small.jpg".to_string()],
            expected_winner: "w1_large.jpg".to_string(),
        };

        let found = find_scenario_group(&groups, &manifest);
        assert!(found.is_none());
    }
}
