//! Test image generator for integration tests.
//!
//! Creates test images by transforming real base photos, ensuring
//! CLIP-based duplicate detection works correctly in Immich.

use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::{DateTime, Utc};

use crate::error::{ImmichError, Result};

/// Transform specification for creating image variants.
///
/// Specifies how to transform a base image to create a test fixture.
/// All fixtures in a duplicate group should use the same base image
/// with different transforms (size, quality) to ensure CLIP sees them
/// as duplicates.
#[derive(Debug, Clone)]
pub struct TransformSpec {
    /// Base image filename (in tests/fixtures/base/)
    pub base_image: String,
    /// Target width in pixels (None = use base image width)
    pub width: Option<u32>,
    /// Target height in pixels (None = scale proportionally from width)
    pub height: Option<u32>,
    /// JPEG quality 1-100 (default 85)
    pub quality: u8,
    /// Strip dimension EXIF tags (for testing missing dimensions)
    pub strip_dimensions: bool,
}

impl TransformSpec {
    /// Create a new transform spec with default quality.
    pub fn new(base_image: impl Into<String>) -> Self {
        Self {
            base_image: base_image.into(),
            width: None,
            height: None,
            quality: 85,
            strip_dimensions: false,
        }
    }

    /// Set target dimensions.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Scale to a percentage of original size.
    pub fn with_scale(mut self, scale_percent: u32) -> Self {
        // Will be applied during generation by reading base image dimensions
        // Store as negative width to signal percentage scaling
        self.width = Some(scale_percent);
        self.height = None; // Signal proportional scaling
        self
    }

    /// Set JPEG quality.
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality;
        self
    }

    /// Strip dimension EXIF tags from output.
    pub fn without_dimensions(mut self) -> Self {
        self.strip_dimensions = true;
        self
    }
}

impl Default for TransformSpec {
    fn default() -> Self {
        Self::new("base_landscape.jpg")
    }
}

/// EXIF metadata specification.
#[derive(Debug, Clone, Default)]
pub struct ExifSpec {
    /// GPS coordinates (latitude, longitude)
    pub gps: Option<(f64, f64)>,
    /// Capture datetime
    pub datetime: Option<DateTime<Utc>>,
    /// Timezone string (e.g., "+05:00")
    pub timezone: Option<String>,
    /// Camera manufacturer
    pub camera_make: Option<String>,
    /// Camera model
    pub camera_model: Option<String>,
    /// Image description
    pub description: Option<String>,
}

/// Complete test image specification.
#[derive(Debug, Clone)]
pub struct TestImage {
    /// Output filename
    pub filename: String,
    /// Transform to apply to base image
    pub transform: TransformSpec,
    /// EXIF metadata to embed
    pub exif: ExifSpec,
}

impl TestImage {
    /// Create a new test image specification.
    pub fn new(filename: impl Into<String>, transform: TransformSpec) -> Self {
        Self {
            filename: filename.into(),
            transform,
            exif: ExifSpec::default(),
        }
    }

    /// Add EXIF metadata.
    pub fn with_exif(mut self, exif: ExifSpec) -> Self {
        self.exif = exif;
        self
    }
}

/// Generate a test image by transforming a base image.
///
/// Loads a base image from `base_dir`, applies transforms (resize, recompress),
/// saves to `output_dir`, and applies EXIF metadata.
///
/// # Arguments
/// * `spec` - The test image specification
/// * `base_dir` - Directory containing base images
/// * `output_dir` - Directory to write the output image
///
/// # Returns
/// Path to the generated image file
pub fn generate_image(spec: &TestImage, base_dir: &Path, output_dir: &Path) -> Result<PathBuf> {
    use image::imageops::FilterType;
    use image::ImageFormat;

    let ext = Path::new(&spec.filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let output_path = output_dir.join(&spec.filename);

    // Handle special formats
    match ext.as_str() {
        "mp4" | "mov" | "avi" => {
            return generate_video(&spec.filename, output_dir, spec.transform.width, spec.transform.height);
        }
        "heic" | "heif" => {
            return Err(ImmichError::Io(std::io::Error::other(
                "HEIC encoding not available - requires platform-specific encoder",
            )));
        }
        "cr3" | "cr2" | "nef" | "arw" | "dng" | "raf" | "orf" => {
            return Err(ImmichError::Io(std::io::Error::other(
                format!("RAW format .{} encoding not available - requires proprietary encoder", ext),
            )));
        }
        _ => {}
    }

    // Load base image
    let base_path = base_dir.join(&spec.transform.base_image);
    let img = image::open(&base_path).map_err(|e| {
        ImmichError::Io(std::io::Error::other(format!(
            "Failed to load base image {}: {}",
            base_path.display(),
            e
        )))
    })?;

    // Calculate target dimensions
    let (target_width, target_height) = match (spec.transform.width, spec.transform.height) {
        (Some(w), Some(h)) => (w, h),
        (Some(scale), None) if scale <= 100 => {
            // Interpret as percentage scale
            let w = (img.width() * scale) / 100;
            let h = (img.height() * scale) / 100;
            (w.max(1), h.max(1))
        }
        (Some(w), None) => {
            // Scale height proportionally
            let h = (img.height() * w) / img.width();
            (w, h.max(1))
        }
        (None, Some(h)) => {
            // Scale width proportionally
            let w = (img.width() * h) / img.height();
            (w.max(1), h)
        }
        (None, None) => (img.width(), img.height()),
    };

    // Resize if needed
    let resized = if target_width != img.width() || target_height != img.height() {
        img.resize_exact(target_width, target_height, FilterType::Lanczos3)
    } else {
        img
    };

    // Save with specified quality
    match ext.as_str() {
        "png" => {
            resized
                .save_with_format(&output_path, ImageFormat::Png)
                .map_err(|e| {
                    ImmichError::Io(std::io::Error::other(format!("Failed to save PNG: {}", e)))
                })?;
        }
        _ => {
            // JPEG with quality control
            let mut output_file = std::fs::File::create(&output_path).map_err(|e| {
                ImmichError::Io(std::io::Error::other(format!(
                    "Failed to create output file: {}",
                    e
                )))
            })?;

            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut output_file,
                spec.transform.quality,
            );
            resized.write_with_encoder(encoder).map_err(|e| {
                ImmichError::Io(std::io::Error::other(format!("Failed to encode JPEG: {}", e)))
            })?;
        }
    }

    // Apply EXIF metadata
    apply_exif(&output_path, &spec.exif, spec.transform.strip_dimensions)?;

    Ok(output_path)
}

/// Generate a test video with specified dimensions.
fn generate_video(
    filename: &str,
    output_dir: &Path,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<PathBuf> {
    let output_path = output_dir.join(filename);

    let w = width.unwrap_or(1920);
    let h = height.unwrap_or(1080);
    let size = format!("{}x{}", w, h);

    let output = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            &format!("color=c=blue:s={}:d=1", size),
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            output_path.to_string_lossy().as_ref(),
        ])
        .output()
        .map_err(|e| {
            ImmichError::Io(std::io::Error::other(format!(
                "Failed to run ffmpeg: {}. Is ffmpeg installed?",
                e
            )))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ImmichError::Io(std::io::Error::other(format!(
            "ffmpeg failed: {}",
            stderr
        ))));
    }

    Ok(output_path)
}

/// Apply EXIF metadata to an image using exiftool CLI.
fn apply_exif(path: &Path, exif: &ExifSpec, strip_dimensions: bool) -> Result<()> {
    let mut args: Vec<String> = vec!["-overwrite_original".to_string()];

    // GPS coordinates
    if let Some((lat, lon)) = exif.gps {
        let lat_ref = if lat >= 0.0 { "N" } else { "S" };
        let lon_ref = if lon >= 0.0 { "E" } else { "W" };
        args.push(format!("-GPSLatitude={}", lat.abs()));
        args.push(format!("-GPSLatitudeRef={}", lat_ref));
        args.push(format!("-GPSLongitude={}", lon.abs()));
        args.push(format!("-GPSLongitudeRef={}", lon_ref));
    }

    // Datetime
    if let Some(dt) = &exif.datetime {
        let formatted = dt.format("%Y:%m:%d %H:%M:%S").to_string();
        args.push(format!("-DateTimeOriginal={}", formatted));
    }

    // Timezone
    if let Some(tz) = &exif.timezone {
        args.push(format!("-OffsetTimeOriginal={}", tz));
    }

    // Camera info
    if let Some(make) = &exif.camera_make {
        args.push(format!("-Make={}", make));
    }
    if let Some(model) = &exif.camera_model {
        args.push(format!("-Model={}", model));
    }

    // Description
    if let Some(desc) = &exif.description {
        args.push(format!("-ImageDescription={}", desc));
    }

    // Strip dimension EXIF if requested
    if strip_dimensions {
        args.push("-ImageWidth=".to_string());
        args.push("-ExifImageWidth=".to_string());
        args.push("-ImageHeight=".to_string());
        args.push("-ExifImageHeight=".to_string());
    }

    // Only run exiftool if we have args to apply
    if args.len() > 1 {
        args.push(path.to_string_lossy().to_string());

        let output = Command::new("exiftool")
            .args(&args)
            .output()
            .map_err(|e| {
                ImmichError::Io(std::io::Error::other(format!(
                    "Failed to run exiftool: {}. Is exiftool installed?",
                    e
                )))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ImmichError::Io(std::io::Error::other(format!(
                "exiftool failed: {}",
                stderr
            ))));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_spec_builder() {
        let spec = TransformSpec::new("base_landscape.jpg")
            .with_size(1000, 750)
            .with_quality(90);

        assert_eq!(spec.base_image, "base_landscape.jpg");
        assert_eq!(spec.width, Some(1000));
        assert_eq!(spec.height, Some(750));
        assert_eq!(spec.quality, 90);
    }

    #[test]
    fn test_transform_spec_scale() {
        let spec = TransformSpec::new("base_portrait.jpg").with_scale(50);

        assert_eq!(spec.width, Some(50));
        assert_eq!(spec.height, None);
    }
}
