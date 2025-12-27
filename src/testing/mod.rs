//! Test scenario detection for duplicate groups.
//!
//! This module provides functionality to analyze duplicate groups
//! and categorize them by test scenario for validation purposes.

pub mod detector;
pub mod fixtures;
pub mod generator;
pub mod report;
pub mod scenarios;

pub use detector::detect_scenarios;
pub use fixtures::{all_fixtures, ScenarioFixture};
pub use generator::{generate_image, ExifSpec, TestImage, TransformSpec};
pub use report::{format_report, ScenarioReport};
pub use scenarios::{ScenarioMatch, TestScenario};
