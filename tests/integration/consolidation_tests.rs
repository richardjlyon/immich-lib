//! Consolidation scenario integration tests.
//!
//! Tests C1-C8 scenarios against a live Immich instance to verify
//! winner selection is correct for consolidation cases.
//!
//! NOTE: Full consolidation (copying metadata from losers to winner) is
//! tested via the execute command. These tests verify correct winner
//! selection as a prerequisite for consolidation.

use immich_lib::DuplicateAnalysis;

use super::fixtures::load_manifest;
use super::harness::TestHarness;
use super::winner_tests::{fetch_full_duplicates, find_group_for_manifest, ScenarioResult, ScenarioStatus};

/// Run consolidation scenario tests for a set of scenario codes.
///
/// Similar to winner tests but also logs consolidation opportunities.
fn run_consolidation_tests(
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

        match find_group_for_manifest(groups, &manifest) {
            Some(group) => {
                let analysis = DuplicateAnalysis::from_group(group);

                if analysis.winner.filename == manifest.expected_winner {
                    // Check for consolidation opportunities
                    let losers_with_metadata = analysis
                        .losers
                        .iter()
                        .filter(|l| l.score.total > 0)
                        .count();

                    let loser_info = if !analysis.losers.is_empty() {
                        let loser_names: Vec<&str> = analysis
                            .losers
                            .iter()
                            .map(|l| l.filename.as_str())
                            .collect();
                        format!(
                            "Losers: {:?}, {} with metadata",
                            loser_names, losers_with_metadata
                        )
                    } else {
                        "No losers".to_string()
                    };

                    results.push(ScenarioResult {
                        scenario: manifest.scenario.clone(),
                        status: ScenarioStatus::Passed,
                        details: Some(format!(
                            "Winner '{}' matches. {}",
                            analysis.winner.filename, loser_info
                        )),
                    });
                } else {
                    results.push(ScenarioResult {
                        scenario: manifest.scenario.clone(),
                        status: ScenarioStatus::Failed,
                        details: Some(format!(
                            "Expected: {}, Actual: {}, Dimensions: {:?}",
                            manifest.expected_winner,
                            analysis.winner.filename,
                            analysis.winner.dimensions
                        )),
                    });
                }
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

/// Test consolidation scenarios (C1-C8).
///
/// These tests verify that winner selection is correct for scenarios
/// where metadata consolidation would apply. The consolidation algorithm
/// itself is tested separately via the execute command.
///
/// Scenarios:
/// - C1: Winner lacks GPS, loser has it (consolidation opportunity)
/// - C2: Winner lacks datetime, loser has it
/// - C3: Winner lacks description, loser has it
/// - C4: Winner lacks all metadata, loser has everything
/// - C5: Both have same GPS (no consolidation needed)
/// - C6: Multiple losers contribute different metadata
/// - C7: No loser has the needed metadata
/// - C8: Winner already has all metadata
///
/// Run with: `cargo test --test integration_tests test_consolidation_scenarios -- --ignored`
#[test]
#[ignore]
fn test_consolidation_scenarios() {
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

    // Test C scenarios
    let scenarios = ["c1", "c2", "c3", "c4", "c5", "c6", "c7", "c8"];
    let results = run_consolidation_tests(&scenarios, &groups);

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
    println!("\n=== Consolidation Scenario Summary ===");
    println!("Passed: {}", passed);
    println!("Warned: {} (scenarios not detected as duplicates)", warned);
    println!("Failed: {}", failed);

    // Fail the test if any assertions failed
    assert_eq!(failed, 0, "Some consolidation scenario tests failed");
}
