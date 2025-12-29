//! API response model types.
//!
//! These types map to the Immich API response DTOs.

mod album;
mod asset;
mod duplicate;
mod exif;
mod execution;

pub use album::{AddAssetsRequest, AlbumResponse, RemoveAssetsRequest};
pub use asset::{AssetResponse, AssetType};
pub use duplicate::DuplicateGroup;
pub use exif::ExifInfo;
pub use execution::{
    AlbumTransferResult, ConsolidationResult, ExecutionConfig, ExecutionReport, GroupResult,
    OperationResult,
};
