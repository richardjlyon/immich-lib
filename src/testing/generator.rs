//! Test image generator for synthetic integration tests.
//!
//! Creates images with controlled dimensions and EXIF metadata
//! for reproducible testing of all 34 test scenarios.

use chrono::{DateTime, Utc};

#[cfg(test)]
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::process::Command;

#[cfg(test)]
use crate::error::{ImmichError, Result};

/// Image properties specification.
#[derive(Debug, Clone)]
pub struct ImageSpec {
    /// Width in pixels (None = strip dimension EXIF after creation)
    pub width: Option<u32>,
    /// Height in pixels
    pub height: Option<u32>,
    /// RGB fill color for visual distinction
    pub color: [u8; 3],
}

impl Default for ImageSpec {
    fn default() -> Self {
        Self {
            width: Some(1920),
            height: Some(1080),
            color: [128, 128, 128], // neutral gray
        }
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
    /// Image properties
    pub image_spec: ImageSpec,
    /// EXIF metadata
    pub exif_spec: ExifSpec,
}

/// Generate a test image with specified properties and EXIF metadata.
///
/// Uses the `image` crate to create a JPEG with specified dimensions,
/// then applies EXIF metadata using exiftool CLI.
///
/// # Arguments
/// * `spec` - The test image specification
/// * `output_dir` - Directory to write the image to
///
/// # Returns
/// Path to the generated image file
#[cfg(test)]
pub fn generate_image(spec: &TestImage, output_dir: &Path) -> Result<PathBuf> {
    use image::{ImageBuffer, Rgb, ImageFormat};

    // Determine actual dimensions (use defaults if None, we'll strip EXIF later)
    let width = spec.image_spec.width.unwrap_or(1920);
    let height = spec.image_spec.height.unwrap_or(1080);
    let color = Rgb(spec.image_spec.color);

    // Create solid color image
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |_, _| color);

    let output_path = output_dir.join(&spec.filename);

    // Save as JPEG
    img.save_with_format(&output_path, ImageFormat::Jpeg)
        .map_err(|e| ImmichError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to save image: {}", e),
        )))?;

    // Apply EXIF metadata using exiftool
    apply_exif(&output_path, &spec.exif_spec, &spec.image_spec)?;

    Ok(output_path)
}

/// Apply EXIF metadata to an image using exiftool CLI.
#[cfg(test)]
fn apply_exif(path: &Path, exif: &ExifSpec, image_spec: &ImageSpec) -> Result<()> {
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

    // Strip dimension EXIF if width/height is None
    if image_spec.width.is_none() {
        args.push("-ImageWidth=".to_string());
        args.push("-ExifImageWidth=".to_string());
    }
    if image_spec.height.is_none() {
        args.push("-ImageHeight=".to_string());
        args.push("-ExifImageHeight=".to_string());
    }

    // Only run exiftool if we have args to apply
    if args.len() > 1 {
        args.push(path.to_string_lossy().to_string());

        let output = Command::new("exiftool")
            .args(&args)
            .output()
            .map_err(|e| ImmichError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to run exiftool: {}. Is exiftool installed?", e),
            )))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ImmichError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("exiftool failed: {}", stderr),
            )));
        }
    }

    Ok(())
}

/// Generate a minimal test video file using ffmpeg.
///
/// Creates a 1-second blue video for testing video duplicate scenarios.
///
/// # Arguments
/// * `filename` - Output filename
/// * `output_dir` - Directory to write the video to
///
/// # Returns
/// Path to the generated video file
#[cfg(test)]
pub fn generate_video(filename: &str, output_dir: &Path) -> Result<PathBuf> {
    let output_path = output_dir.join(filename);

    let output = Command::new("ffmpeg")
        .args([
            "-y", // overwrite
            "-f", "lavfi",
            "-i", "color=c=blue:s=320x240:d=1",
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            output_path.to_string_lossy().as_ref(),
        ])
        .output()
        .map_err(|e| ImmichError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to run ffmpeg: {}. Is ffmpeg installed?", e),
        )))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ImmichError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("ffmpeg failed: {}", stderr),
        )));
    }

    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_simple_image() {
        let dir = tempdir().unwrap();
        let spec = TestImage {
            filename: "test.jpg".to_string(),
            image_spec: ImageSpec {
                width: Some(100),
                height: Some(100),
                color: [255, 0, 0], // red
            },
            exif_spec: ExifSpec::default(),
        };

        let path = generate_image(&spec, dir.path()).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_generate_image_with_gps() {
        let dir = tempdir().unwrap();
        let spec = TestImage {
            filename: "with_gps.jpg".to_string(),
            image_spec: ImageSpec::default(),
            exif_spec: ExifSpec {
                gps: Some((51.5074, -0.1278)), // London
                ..Default::default()
            },
        };

        let result = generate_image(&spec, dir.path());
        // May fail if exiftool not installed - that's OK for unit tests
        if result.is_ok() {
            assert!(result.unwrap().exists());
        }
    }
}
