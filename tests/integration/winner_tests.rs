//! Winner selection integration tests.
//!
//! Tests W1-W8 scenarios against a live Immich instance to verify
//! the scoring algorithm correctly selects winners based on dimensions.
//!
//! These tests are grouped into a single test function to ensure Docker
//! setup/teardown happens only once and tests run sequentially.

use immich_lib::models::DuplicateGroup;
use immich_lib::DuplicateAnalysis;
use reqwest::blocking::Client;
use std::time::Duration;

use super::fixtures::{load_manifest, Manifest};
use super::harness::TestHarness;

/// Fetch full duplicate groups with asset metadata from the API.
///
/// The harness returns minimal data; this fetches complete DuplicateGroup
/// objects needed for scoring analysis.
pub fn fetch_full_duplicates(
    base_url: &str,
    api_key: &str,
) -> Result<Vec<DuplicateGroup>, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let url = format!("{}/api/duplicates", base_url);
    let response = client
        .get(&url)
        .header("x-api-key", api_key)
        .send()?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()).into());
    }

    let groups: Vec<DuplicateGroup> = response.json()?;
    Ok(groups)
}

/// Find a duplicate group matching a manifest's files.
pub fn find_group_for_manifest<'a>(
    groups: &'a [DuplicateGroup],
    manifest: &Manifest,
) -> Option<&'a DuplicateGroup> {
    for group in groups {
        let group_filenames: Vec<&str> = group
            .assets
            .iter()
            .map(|a| a.original_file_name.as_str())
            .collect();

        // Check if all manifest images are present in this group
        let all_present = manifest.images.iter().all(|img| {
            group_filenames.iter().any(|gf| gf == img)
        });

        if all_present && group.assets.len() >= manifest.images.len() {
            return Some(group);
        }
    }
    None
}

/// Test result for a single scenario.
pub struct ScenarioResult {
    pub scenario: String,
    pub status: ScenarioStatus,
    pub details: Option<String>,
}

pub enum ScenarioStatus {
    Passed,
    Warned,
    Failed,
}

/// Run winner selection tests for a set of scenario codes.
///
/// Returns results for each scenario tested.
pub fn run_winner_tests(
    scenarios: &[&str],
    groups: &[DuplicateGroup],
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
                    results.push(ScenarioResult {
                        scenario: manifest.scenario.clone(),
                        status: ScenarioStatus::Passed,
                        details: Some(format!("Winner '{}' matches expected", analysis.winner.filename)),
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

/// Test winner selection for all W scenarios (W1-W8).
///
/// This test requires a running Docker Immich instance with seeded fixtures.
/// Run with: `cargo test --test integration_tests test_winner_selection -- --ignored`
#[test]
#[ignore]
fn test_winner_selection() {
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

    // Test W scenarios
    let scenarios = ["w1", "w2", "w3", "w4", "w5", "w6", "w7", "w8"];
    let results = run_winner_tests(&scenarios, &groups);

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
    println!("\n=== Winner Selection Summary ===");
    println!("Passed: {}", passed);
    println!("Warned: {} (scenarios not detected as duplicates)", warned);
    println!("Failed: {}", failed);

    // Fail the test if any assertions failed
    assert_eq!(failed, 0, "Some winner selection tests failed");
}
