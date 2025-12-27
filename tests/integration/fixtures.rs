//! Fixture loading utilities for integration tests.
//!
//! Provides manifest parsing and scenario enumeration.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

/// Manifest describing a test scenario.
#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    /// Scenario code (e.g., "W1", "C3", "X5")
    pub scenario: String,

    /// Human-readable description
    pub description: String,

    /// List of image filenames in this group
    pub images: Vec<String>,

    /// Expected winner filename
    pub expected_winner: String,
}

/// Load a manifest for a scenario.
///
/// # Arguments
///
/// * `scenario_code` - The scenario code (e.g., "w1", "c3")
///
/// # Returns
///
/// The manifest for the scenario.
pub fn load_manifest(scenario_code: &str) -> Result<Manifest, Box<dyn std::error::Error>> {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");

    let manifest_path = fixtures_dir.join(scenario_code).join("manifest.json");

    if !manifest_path.exists() {
        return Err(format!("Manifest not found: {}", manifest_path.display()).into());
    }

    let content = fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&content)?;

    Ok(manifest)
}

/// List all available scenario codes.
///
/// Returns scenario codes in lowercase (e.g., "w1", "c3", "x5").
pub fn list_scenarios() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");

    let mut scenarios = Vec::new();

    for entry in fs::read_dir(&fixtures_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip non-directories and special files
        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name().unwrap().to_string_lossy();

        // Skip docker directory and other non-scenario dirs
        if dir_name == "docker" || dir_name.starts_with('.') {
            continue;
        }

        // Check if manifest.json exists
        if path.join("manifest.json").exists() {
            scenarios.push(dir_name.to_string());
        }
    }

    // Sort for consistent ordering
    scenarios.sort();

    Ok(scenarios)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_manifest_w1() {
        let manifest = load_manifest("w1").expect("Should load W1 manifest");
        assert_eq!(manifest.scenario, "W1");
        assert!(!manifest.images.is_empty());
        assert!(!manifest.expected_winner.is_empty());
    }

    #[test]
    fn test_list_scenarios() {
        let scenarios = list_scenarios().expect("Should list scenarios");
        assert!(!scenarios.is_empty());
        // Should include at least W1
        assert!(scenarios.contains(&"w1".to_string()));
    }
}
