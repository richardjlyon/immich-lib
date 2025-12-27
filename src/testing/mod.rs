//! Test scenario detection for duplicate groups.
//!
//! This module provides functionality to analyze duplicate groups
//! and categorize them by test scenario for validation purposes.

pub mod detector;
pub mod report;
pub mod scenarios;

pub use detector::detect_scenarios;
pub use report::{format_report, ScenarioReport};
pub use scenarios::{ScenarioMatch, TestScenario};
