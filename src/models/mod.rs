//! API response model types.
//!
//! These types map to the Immich API response DTOs.

mod asset;
mod duplicate;
mod exif;
mod execution;

pub use asset::{AssetResponse, AssetType};
pub use duplicate::DuplicateGroup;
pub use exif::ExifInfo;
pub use execution::{
    ConsolidationResult, ExecutionConfig, ExecutionReport, GroupResult, OperationResult,
};
