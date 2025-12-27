//! Integration test infrastructure for immich-lib.
//!
//! Provides test harness, fixture loading, and assertion utilities
//! for testing against a Docker-based Immich instance.

pub mod assertions;
pub mod conflict_tests;
pub mod consolidation_tests;
pub mod edge_case_tests;
pub mod fixtures;
pub mod harness;
pub mod winner_tests;
