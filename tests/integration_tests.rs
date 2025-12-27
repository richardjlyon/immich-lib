//! Integration tests for immich-lib.
//!
//! These tests run against a Docker-based Immich instance.
//! Use `tests/docker/bootstrap.sh` to start the environment before running.

mod integration;

use integration::fixtures::{list_scenarios, load_manifest};

/// Test that all scenario manifests can be loaded.
#[test]
fn test_all_manifests_load() {
    let scenarios = list_scenarios().expect("Should list scenarios");

    assert!(!scenarios.is_empty(), "Should have at least one scenario");

    for scenario in &scenarios {
        let manifest = load_manifest(scenario)
            .unwrap_or_else(|e| panic!("Failed to load manifest for {}: {}", scenario, e));

        assert!(!manifest.images.is_empty(), "Scenario {} should have images", scenario);
        assert!(!manifest.expected_winner.is_empty(), "Scenario {} should have expected_winner", scenario);

        // Verify expected_winner is in the images list
        assert!(
            manifest.images.contains(&manifest.expected_winner),
            "Scenario {}: expected_winner '{}' not in images list",
            scenario,
            manifest.expected_winner
        );
    }

    println!("Successfully loaded {} scenario manifests", scenarios.len());
}

/// Test that scenario codes follow expected pattern.
#[test]
fn test_scenario_categories() {
    let scenarios = list_scenarios().expect("Should list scenarios");

    // Count scenarios by category
    let winner_count = scenarios.iter().filter(|s| s.starts_with('w')).count();
    let consolidation_count = scenarios.iter().filter(|s| s.starts_with('c')).count();
    let conflict_count = scenarios.iter().filter(|s| s.starts_with('f')).count();
    let edge_count = scenarios.iter().filter(|s| s.starts_with('x')).count();

    println!("Scenario categories:");
    println!("  Winner selection (W): {}", winner_count);
    println!("  Consolidation (C): {}", consolidation_count);
    println!("  Conflicts (F): {}", conflict_count);
    println!("  Edge cases (X): {}", edge_count);

    // Should have scenarios in each category
    assert!(winner_count > 0, "Should have winner selection scenarios");
    assert!(consolidation_count > 0, "Should have consolidation scenarios");
    assert!(conflict_count > 0, "Should have conflict scenarios");
    assert!(edge_count > 0, "Should have edge case scenarios");
}
