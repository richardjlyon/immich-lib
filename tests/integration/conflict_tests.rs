//! Conflict detection integration tests.
//!
//! Tests F1-F7 scenarios against a live Immich instance to verify
//! conflict detection correctly identifies metadata discrepancies.

use immich_lib::scoring::MetadataConflict;
use immich_lib::DuplicateAnalysis;

use super::fixtures::load_manifest;
use super::harness::TestHarness;
use super::winner_tests::{fetch_full_duplicates, find_group_for_manifest, ScenarioResult, ScenarioStatus};

/// Check if an analysis has a specific conflict type.
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

/// Run conflict detection tests for F scenarios.
///
/// Checks both winner selection AND conflict detection accuracy.
fn run_conflict_tests(
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

                // First check winner selection
                if analysis.winner.filename != manifest.expected_winner {
                    results.push(ScenarioResult {
                        scenario: manifest.scenario.clone(),
                        status: ScenarioStatus::Failed,
                        details: Some(format!(
                            "Wrong winner: expected '{}', got '{}'",
                            manifest.expected_winner, analysis.winner.filename
                        )),
                    });
                    continue;
                }

                // Now check conflict detection based on scenario
                let (expected_conflicts, conflict_check) = match code.to_lowercase().as_str() {
                    "f1" => ("GPS conflict", has_gps_conflict(&analysis)),
                    "f2" => ("NO GPS conflict (within threshold)", !has_gps_conflict(&analysis)),
                    "f3" => ("Timezone conflict", has_timezone_conflict(&analysis)),
                    "f4" => ("Camera conflict", has_camera_conflict(&analysis)),
                    "f5" => ("Capture time conflict", has_capture_time_conflict(&analysis)),
                    "f6" => (
                        "Multiple conflicts (GPS + camera + timezone)",
                        has_gps_conflict(&analysis)
                            && has_camera_conflict(&analysis)
                            && has_timezone_conflict(&analysis)
                    ),
                    "f7" => (
                        "NO conflicts",
                        analysis.conflicts.is_empty()
                    ),
                    _ => ("Unknown scenario", true),
                };

                if conflict_check {
                    let conflict_summary: Vec<String> = analysis.conflicts.iter().map(|c| {
                        match c {
                            MetadataConflict::Gps { values } => format!("GPS({} locations)", values.len()),
                            MetadataConflict::Timezone { values } => format!("TZ({:?})", values),
                            MetadataConflict::CameraInfo { values } => format!("Camera({:?})", values),
                            MetadataConflict::CaptureTime { values } => format!("Time({} times)", values.len()),
                        }
                    }).collect();

                    results.push(ScenarioResult {
                        scenario: manifest.scenario.clone(),
                        status: ScenarioStatus::Passed,
                        details: Some(format!(
                            "Winner '{}', conflicts: {}. Expected: {}",
                            analysis.winner.filename,
                            if conflict_summary.is_empty() { "none".to_string() } else { conflict_summary.join(", ") },
                            expected_conflicts
                        )),
                    });
                } else {
                    let actual_conflicts: Vec<String> = analysis.conflicts.iter().map(|c| {
                        match c {
                            MetadataConflict::Gps { .. } => "GPS",
                            MetadataConflict::Timezone { .. } => "Timezone",
                            MetadataConflict::CameraInfo { .. } => "Camera",
                            MetadataConflict::CaptureTime { .. } => "CaptureTime",
                        }.to_string()
                    }).collect();

                    results.push(ScenarioResult {
                        scenario: manifest.scenario.clone(),
                        status: ScenarioStatus::Failed,
                        details: Some(format!(
                            "Expected: {}, Actual conflicts: {:?}",
                            expected_conflicts,
                            if actual_conflicts.is_empty() { vec!["none".to_string()] } else { actual_conflicts }
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

/// Test conflict detection for all F scenarios (F1-F7).
///
/// Scenarios:
/// - F1: GPS conflict (London vs Paris) - should detect GPS conflict
/// - F2: GPS within threshold (~5m) - should NOT detect GPS conflict
/// - F3: Timezone conflict (UTC vs PST) - should detect timezone conflict
/// - F4: Camera conflict (Canon vs Nikon) - should detect camera conflict
/// - F5: Capture time conflict (12h difference) - should detect time conflict
/// - F6: Multiple conflicts (GPS + camera + timezone) - should detect all
/// - F7: No conflicts (metadata matches) - should detect no conflicts
///
/// Run with: `cargo test --test integration_tests test_conflict_detection -- --ignored`
#[test]
#[ignore]
fn test_conflict_detection() {
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

    // Test F scenarios
    let scenarios = ["f1", "f2", "f3", "f4", "f5", "f6", "f7"];
    let results = run_conflict_tests(&scenarios, &groups);

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
    println!("\n=== Conflict Detection Summary ===");
    println!("Passed: {}", passed);
    println!("Warned: {} (scenarios not detected as duplicates)", warned);
    println!("Failed: {}", failed);

    // Fail the test if any assertions failed
    assert_eq!(failed, 0, "Some conflict detection tests failed");
}
