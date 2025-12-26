//! Immich API library for duplicate management.
//!
//! This library provides a typed client for interacting with the Immich REST API,
//! with a focus on duplicate detection and metadata-aware duplicate selection.

pub mod error;
pub mod models;

pub use error::{ImmichError, Result};
