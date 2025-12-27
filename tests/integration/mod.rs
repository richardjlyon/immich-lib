//! Integration test infrastructure for immich-lib.
//!
//! Provides test harness, fixture loading, and assertion utilities
//! for testing against a Docker-based Immich instance.

pub mod assertions;
pub mod consolidation_tests;
pub mod fixtures;
pub mod harness;
pub mod winner_tests;

pub use assertions::{assert_winner_matches, find_scenario_group};
pub use fixtures::{list_scenarios, load_manifest, Manifest};
pub use harness::TestHarness;
