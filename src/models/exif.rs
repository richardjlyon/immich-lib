//! EXIF metadata response types.

use serde::Deserialize;

/// EXIF metadata for an asset.
///
/// Most fields are optional as EXIF data may be incomplete or missing.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExifInfo {
    /// GPS latitude
    pub latitude: Option<f64>,

    /// GPS longitude
    pub longitude: Option<f64>,

    /// City name from GPS reverse geocoding
    pub city: Option<String>,

    /// State/province from GPS reverse geocoding
    pub state: Option<String>,

    /// Country from GPS reverse geocoding
    pub country: Option<String>,

    /// Timezone of the location
    pub time_zone: Option<String>,

    /// Original capture date/time from EXIF
    pub date_time_original: Option<String>,

    /// Camera manufacturer
    pub make: Option<String>,

    /// Camera model
    pub model: Option<String>,

    /// Lens model
    pub lens_model: Option<String>,

    /// Exposure time (e.g., "1/125")
    pub exposure_time: Option<String>,

    /// Aperture f-number
    pub f_number: Option<f64>,

    /// Focal length in mm
    pub focal_length: Option<f64>,

    /// ISO sensitivity
    pub iso: Option<u32>,

    /// Image width in pixels
    pub exif_image_width: Option<u32>,

    /// Image height in pixels
    pub exif_image_height: Option<u32>,

    /// File size in bytes
    pub file_size_in_byte: Option<u64>,

    /// Image description/caption
    pub description: Option<String>,

    /// User rating (1-5)
    pub rating: Option<u8>,
}
