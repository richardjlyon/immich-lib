//! Immich API library for duplicate management.
//!
//! This library provides a typed client for interacting with the Immich REST API,
//! with a focus on duplicate detection and metadata-aware duplicate selection.
//!
//! # Example
//!
//! ```no_run
//! use immich_lib::ImmichClient;
//!
//! # async fn example() -> immich_lib::Result<()> {
//! let client = ImmichClient::new("https://immich.example.com", "your-api-key")?;
//! let duplicates = client.get_duplicates().await?;
//!
//! for group in duplicates {
//!     println!("Duplicate group {} has {} assets", group.duplicate_id, group.assets.len());
//! }
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod error;
pub mod models;
pub mod scoring;

pub use client::ImmichClient;
pub use error::{ImmichError, Result};
pub use scoring::{detect_conflicts, DuplicateAnalysis, MetadataConflict, MetadataScore, ScoredAsset};
