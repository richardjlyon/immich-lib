//! API response model types.
//!
//! These types map to the Immich API response DTOs.

mod asset;
mod duplicate;
mod exif;

pub use asset::{AssetResponse, AssetType};
pub use duplicate::DuplicateGroup;
pub use exif::ExifInfo;
