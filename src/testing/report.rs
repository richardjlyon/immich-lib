//! Report formatting for test scenario coverage.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::scenarios::{ScenarioMatch, TestScenario};

/// Test scenario coverage report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioReport {
    /// Total groups analyzed
    pub total_groups: usize,

    /// Matches grouped by scenario
    pub coverage: HashMap<String, Vec<ScenarioMatch>>,

    /// Scenarios with no matches
    pub uncovered: Vec<String>,

    /// Unexpected patterns discovered
    pub unexpected: Vec<String>,
}

impl ScenarioReport {
    /// Create a new report from scenario matches.
    pub fn from_matches(matches: Vec<ScenarioMatch>, total_groups: usize) -> Self {
        let mut coverage: HashMap<String, Vec<ScenarioMatch>> = HashMap::new();

        // Group matches by scenario
        for m in matches {
            let key = m.scenario.to_string();
            coverage.entry(key).or_default().push(m);
        }

        // Find uncovered scenarios
        let all_scenarios = TestScenario::all();
        let uncovered: Vec<String> = all_scenarios
            .iter()
            .filter(|s| !coverage.contains_key(&s.to_string()))
            .map(|s| s.to_string())
            .collect();

        Self {
            total_groups,
            coverage,
            uncovered,
            unexpected: Vec::new(),
        }
    }

    /// Add an unexpected pattern.
    pub fn add_unexpected(&mut self, pattern: String) {
        self.unexpected.push(pattern);
    }
}

/// Format the report for text output.
pub fn format_report(report: &ScenarioReport) -> String {
    let mut output = String::new();

    output.push_str("=== Test Scenario Coverage Report ===\n\n");

    // Coverage statistics
    let total_scenarios = TestScenario::all().len();
    let covered_count = report.coverage.len();
    let coverage_pct = (covered_count as f64 / total_scenarios as f64) * 100.0;

    // Covered scenarios by category
    output.push_str(&format!(
        "COVERED ({}/{} scenarios, {:.0}%):\n",
        covered_count, total_scenarios, coverage_pct
    ));

    // Group by category
    let categories = ["Winner Selection", "Consolidation", "Conflicts", "Edge Cases"];
    for category in categories {
        let category_scenarios: Vec<(&String, &Vec<ScenarioMatch>)> = report
            .coverage
            .iter()
            .filter(|(k, _)| {
                let prefix = k.chars().next().unwrap_or('?');
                match category {
                    "Winner Selection" => prefix == 'W',
                    "Consolidation" => prefix == 'C',
                    "Conflicts" => prefix == 'F',
                    "Edge Cases" => prefix == 'X',
                    _ => false,
                }
            })
            .collect();

        if !category_scenarios.is_empty() {
            output.push_str(&format!("\n  {}:\n", category));
            for (scenario, matches) in category_scenarios {
                output.push_str(&format!("    {}: {} groups\n", scenario, matches.len()));
                // Show first example
                if let Some(first) = matches.first() {
                    output.push_str(&format!(
                        "      Example: {} ({})\n",
                        first.duplicate_id, first.details
                    ));
                }
            }
        }
    }

    // Uncovered scenarios
    if !report.uncovered.is_empty() {
        output.push_str(&format!(
            "\nNOT COVERED ({} scenarios):\n",
            report.uncovered.len()
        ));
        for scenario in &report.uncovered {
            output.push_str(&format!("  {}: 0 groups\n", scenario));
        }
    }

    // Unexpected patterns
    if !report.unexpected.is_empty() {
        output.push_str("\nUNEXPECTED PATTERNS:\n");
        for pattern in &report.unexpected {
            output.push_str(&format!("  - {}\n", pattern));
        }
    }

    // Summary
    output.push_str("\n=== Summary ===\n");
    output.push_str(&format!("Total groups analyzed: {}\n", report.total_groups));
    output.push_str(&format!(
        "Scenarios covered: {}/{} ({:.0}%)\n",
        covered_count, total_scenarios, coverage_pct
    ));
    output.push_str(&format!(
        "Synthetic images needed for: {} scenarios\n",
        report.uncovered.len()
    ));

    output
}
