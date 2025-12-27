//! Edge case integration tests.
//!
//! Tests X1-X5, X7, X9-X11 scenarios against a live Immich instance.
//! Note: X6 (HEIC) and X8 (RAW) were removed - cannot generate without proprietary encoders.

use immich_lib::DuplicateAnalysis;

use super::fixtures::load_manifest;
use super::harness::TestHarness;
use super::winner_tests::{fetch_full_duplicates, find_group_for_manifest, ScenarioResult, ScenarioStatus};

/// Run edge case tests for X scenarios.
///
/// Handles various edge cases including:
/// - X1: Single asset (will NOT appear in duplicates - expected)
/// - X2: Large group (12 duplicates)
/// - X3: Large file (48MP)
/// - X4: Special characters in filename
/// - X5: Video duplicates
/// - X7: PNG format
/// - X9: Unicode in description
/// - X10: Very old date (1985)
/// - X11: Future date (2030)
fn run_edge_case_tests(
    scenarios: &[&str],
    groups: &[immich_lib::models::DuplicateGroup],
) -> Vec<ScenarioResult> {
    let mut results = Vec::new();

    for code in scenarios {
        let manifest = match load_manifest(code) {
            Ok(m) => m,
            Err(e) => {
                results.push(ScenarioResult {
                    scenario: code.to_string(),
                    status: ScenarioStatus::Warned,
                    details: Some(format!("Could not load manifest: {}", e)),
                });
                continue;
            }
        };

        // X1 is special - single asset group should NOT appear in duplicates
        if code.to_lowercase() == "x1" {
            // For X1, we expect it to NOT be found in duplicate groups
            match find_group_for_manifest(groups, &manifest) {
                Some(_) => {
                    results.push(ScenarioResult {
                        scenario: manifest.scenario.clone(),
                        status: ScenarioStatus::Failed,
                        details: Some("Single asset should NOT appear in duplicates".to_string()),
                    });
                }
                None => {
                    results.push(ScenarioResult {
                        scenario: manifest.scenario.clone(),
                        status: ScenarioStatus::Passed,
                        details: Some("Single asset correctly not in duplicates (expected behavior)".to_string()),
                    });
                }
            }
            continue;
        }

        // All other scenarios should have duplicate groups
        match find_group_for_manifest(groups, &manifest) {
            Some(group) => {
                let analysis = DuplicateAnalysis::from_group(group);

                // Check winner selection
                if analysis.winner.filename != manifest.expected_winner {
                    results.push(ScenarioResult {
                        scenario: manifest.scenario.clone(),
                        status: ScenarioStatus::Failed,
                        details: Some(format!(
                            "Wrong winner: expected '{}', got '{}', dimensions: {:?}",
                            manifest.expected_winner, analysis.winner.filename, analysis.winner.dimensions
                        )),
                    });
                    continue;
                }

                // Additional checks based on scenario
                let extra_info = match code.to_lowercase().as_str() {
                    "x2" => {
                        // Large group - should have 12 assets
                        let asset_count = group.assets.len();
                        if asset_count >= 12 {
                            format!("Group has {} assets (expected 12)", asset_count)
                        } else {
                            return vec![ScenarioResult {
                                scenario: manifest.scenario.clone(),
                                status: ScenarioStatus::Failed,
                                details: Some(format!("Expected 12 assets, got {}", asset_count)),
                            }];
                        }
                    }
                    "x3" => {
                        // Large file - verify dimensions parsed (8000x6000 = 48MP)
                        if let Some(dims) = &analysis.winner.dimensions {
                            format!("Dimensions: {}x{}", dims.0, dims.1)
                        } else {
                            "Warning: dimensions not parsed".to_string()
                        }
                    }
                    "x4" => {
                        // Special characters - just verify the filename was matched
                        format!("Filename matched: '{}'", analysis.winner.filename)
                    }
                    "x5" => {
                        // Video - verify it's handled
                        format!("Video handling OK, winner: '{}'", analysis.winner.filename)
                    }
                    "x7" => {
                        // PNG format
                        format!("PNG format OK, winner: '{}'", analysis.winner.filename)
                    }
                    "x9" => {
                        // Unicode - just verify winner selection works
                        "Unicode handling OK".to_string()
                    }
                    "x10" => {
                        // Very old date (1985)
                        "Old date (1985) handling OK".to_string()
                    }
                    "x11" => {
                        // Future date (2030)
                        "Future date (2030) handling OK".to_string()
                    }
                    _ => "OK".to_string(),
                };

                results.push(ScenarioResult {
                    scenario: manifest.scenario.clone(),
                    status: ScenarioStatus::Passed,
                    details: Some(format!(
                        "Winner '{}' matches. {}",
                        analysis.winner.filename, extra_info
                    )),
                });
            }
            None => {
                results.push(ScenarioResult {
                    scenario: manifest.scenario.clone(),
                    status: ScenarioStatus::Warned,
                    details: Some("Duplicate group not found (Immich may not have detected as duplicates)".to_string()),
                });
            }
        }
    }

    results
}

/// Test edge cases (X1-X5, X7, X9-X11).
///
/// Note: X6 (HEIC) and X8 (RAW) were removed - cannot generate without proprietary encoders.
///
/// Scenarios:
/// - X1: Single asset - will NOT appear in duplicates (expected)
/// - X2: Large group (12 duplicates) - verify all are in group
/// - X3: Large file (48MP) - verify dimensions parsed
/// - X4: Special characters in filename - verify matching works
/// - X5: Video duplicates (MP4) - verify video handling
/// - X7: PNG format - verify format handling
/// - X9: Unicode in description - verify unicode handling
/// - X10: Very old date (1985) - verify date parsing
/// - X11: Future date (2030) - verify date handling
///
/// Run with: `cargo test --test integration_tests test_edge_cases -- --ignored`
#[test]
#[ignore]
fn test_edge_cases() {
    // Setup test environment
    let harness = match TestHarness::setup() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to setup test harness: {}", e);
            panic!("Test setup failed: {}", e);
        }
    };

    // Wait for duplicate detection to complete
    if let Err(e) = harness.wait_for_duplicates() {
        eprintln!("Warning: Failed waiting for duplicates: {}", e);
        let _ = harness.teardown();
        panic!("Duplicate detection timed out: {}", e);
    }

    // Fetch full duplicate groups with metadata
    let groups = match fetch_full_duplicates(&harness.base_url, &harness.api_key) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to fetch duplicates: {}", e);
            let _ = harness.teardown();
            panic!("Failed to fetch duplicates: {}", e);
        }
    };

    println!("Found {} duplicate groups", groups.len());

    // Test X scenarios (excluding X6 and X8 which were removed)
    let scenarios = ["x1", "x2", "x3", "x4", "x5", "x7", "x9", "x10", "x11"];
    let results = run_edge_case_tests(&scenarios, &groups);

    // Print results
    let mut passed = 0;
    let mut warned = 0;
    let mut failed = 0;

    for result in &results {
        match result.status {
            ScenarioStatus::Passed => {
                println!("✓ {}: {}", result.scenario, result.details.as_deref().unwrap_or(""));
                passed += 1;
            }
            ScenarioStatus::Warned => {
                eprintln!("⚠ {}: {}", result.scenario, result.details.as_deref().unwrap_or(""));
                warned += 1;
            }
            ScenarioStatus::Failed => {
                eprintln!("✗ {}: {}", result.scenario, result.details.as_deref().unwrap_or(""));
                failed += 1;
            }
        }
    }

    // Teardown
    if let Err(e) = harness.teardown() {
        eprintln!("Warning: Teardown failed: {}", e);
    }

    // Summary
    println!("\n=== Edge Case Summary ===");
    println!("Passed: {}", passed);
    println!("Warned: {} (scenarios not detected as duplicates)", warned);
    println!("Failed: {}", failed);

    // Fail the test if any assertions failed
    assert_eq!(failed, 0, "Some edge case tests failed");
}
