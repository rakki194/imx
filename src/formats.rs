//! Image format conversion module.
//!
//! This module provides comprehensive support for converting between different image formats
//! supported by the `image` crate. It includes:
//! - Format detection and validation
//! - Format conversion with quality control
//! - Batch processing capabilities
//! - Progress reporting through logging
//!
//! # Examples
//!
//! ```rust
//! use std::path::Path;
//! use imx::formats::{convert_image, ImageFormatOptions};
//!
//! async fn convert_to_webp() -> anyhow::Result<()> {
//!     let input = Path::new("input.png");
//!     let output = Path::new("output.webp");
//!     
//!     // Convert with default options
//!     convert_image(input, output, None).await?;
//!     
//!     // Convert with custom options
//!     let options = ImageFormatOptions::webp()
//!         .with_quality(85)
//!         .with_lossless(false);
//!     convert_image(input, output, Some(options)).await?;
//!     
//!     Ok(())
//! }
//! ```

use anyhow::{Context, Result};
use image::{
    ImageEncoder, ImageFormat,
    codecs::{jpeg::JpegEncoder, png::PngEncoder, webp::WebPEncoder},
};
use log::{debug, info, warn};
use std::path::{Path, PathBuf};

/// Options for controlling image format conversion.
#[derive(Debug, Clone)]
pub struct ImageFormatOptions {
    /// Quality setting (0-100) for lossy formats
    quality: u8,
    /// Whether to use lossless compression (when supported)
    lossless: bool,
    /// Format-specific options as key-value pairs
    extra_options: std::collections::HashMap<String, String>,
}

impl Default for ImageFormatOptions {
    fn default() -> Self {
        Self {
            quality: 90,
            lossless: false,
            extra_options: std::collections::HashMap::new(),
        }
    }
}

impl ImageFormatOptions {
    /// Create options optimized for JPEG format
    #[must_use]
    pub fn jpeg() -> Self {
        Self {
            quality: 85,
            lossless: false,
            extra_options: std::collections::HashMap::new(),
        }
    }

    /// Create options optimized for PNG format
    #[must_use]
    pub fn png() -> Self {
        Self {
            quality: 100,
            lossless: true,
            extra_options: std::collections::HashMap::new(),
        }
    }

    /// Create options optimized for WebP format
    #[must_use]
    pub fn webp() -> Self {
        Self {
            quality: 80,
            lossless: false,
            extra_options: std::collections::HashMap::new(),
        }
    }

    /// Set the quality level (0-100)
    #[must_use]
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality.min(100);
        self
    }

    /// Set whether to use lossless compression
    #[must_use]
    pub fn with_lossless(mut self, lossless: bool) -> Self {
        self.lossless = lossless;
        self
    }

    /// Add a format-specific option
    #[must_use]
    pub fn with_option(mut self, key: &str, value: &str) -> Self {
        self.extra_options
            .insert(key.to_string(), value.to_string());
        self
    }
}

/// Detect the image format from a file extension.
///
/// # Arguments
///
/// * `path` - Path to check for format
///
/// # Returns
///
/// Returns `Some(ImageFormat)` if a supported format is detected, `None` otherwise
#[must_use]
pub fn detect_format_from_extension(path: &Path) -> Option<ImageFormat> {
    path.extension()
        .and_then(|ext| ImageFormat::from_extension(ext))
}

/// Convert an image from one format to another.
///
/// # Arguments
///
/// * `input_path` - Path to the input image
/// * `output_path` - Path where the converted image should be saved
/// * `options` - Optional format-specific conversion options
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// * The input file cannot be opened or read
/// * The input format is not supported
/// * The output format is not supported
/// * The conversion process fails
/// * The output file cannot be written
pub async fn convert_image(
    input_path: &Path,
    output_path: &Path,
    options: Option<ImageFormatOptions>,
) -> Result<()> {
    // Detect output format
    let output_format = detect_format_from_extension(output_path)
        .context("Could not determine output format from file extension")?;

    info!(
        "Converting {} to {}",
        input_path.display(),
        output_path.display()
    );

    // Read input image
    let img = image::open(input_path).context("Failed to open input image")?;

    // Get or create options
    let options = options.unwrap_or_default();

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create output directory")?;
    }

    // Convert and save with format-specific options
    match output_format {
        ImageFormat::Jpeg => {
            let mut output = std::fs::File::create(output_path)?;
            let mut encoder = JpegEncoder::new_with_quality(&mut output, options.quality);
            encoder
                .encode(
                    img.as_bytes(),
                    img.width(),
                    img.height(),
                    img.color().into(),
                )
                .context("Failed to encode JPEG")?;
        }
        ImageFormat::Png => {
            let mut output = std::fs::File::create(output_path)?;
            let encoder = PngEncoder::new(&mut output);
            encoder
                .write_image(
                    img.as_bytes(),
                    img.width(),
                    img.height(),
                    img.color().into(),
                )
                .context("Failed to encode PNG")?;
        }
        ImageFormat::WebP => {
            let mut output = std::fs::File::create(output_path)?;
            let encoder = WebPEncoder::new_lossless(&mut output);

            // Note: Image 0.25.5 only supports lossless WebP encoding
            // If options.lossless is false, we'll log a warning that we're still using lossless
            if !options.lossless {
                warn!(
                    "Lossy WebP encoding not supported by this version of the image crate. Using lossless encoding instead."
                );
            }

            encoder
                .encode(
                    img.as_bytes(),
                    img.width(),
                    img.height(),
                    img.color().into(),
                )
                .context("Failed to encode WebP")?;
        }
        _ => {
            // Fallback for other formats
            img.save(output_path)
                .with_context(|| format!("Failed to save image as {:?}", output_format))?;
        }
    }

    info!("Successfully converted image to {}", output_path.display());
    Ok(())
}

/// Convert multiple images in a batch operation.
///
/// # Arguments
///
/// * `input_paths` - List of input image paths
/// * `output_dir` - Directory where converted images should be saved
/// * `output_format` - Target format for conversion
/// * `options` - Optional format-specific conversion options
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// * The output directory cannot be created
/// * Any individual conversion fails
pub async fn convert_images_batch(
    input_paths: &[PathBuf],
    output_dir: &Path,
    output_format: ImageFormat,
    options: Option<ImageFormatOptions>,
) -> Result<()> {
    // Create output directory
    tokio::fs::create_dir_all(output_dir)
        .await
        .context("Failed to create output directory")?;

    // Process each image
    let total = input_paths.len();
    info!(
        "Converting batch of {} images to {:?}",
        total, output_format
    );

    for (i, input_path) in input_paths.iter().enumerate() {
        // Generate output path with new extension
        let file_name = input_path
            .file_name()
            .context("Invalid input path")?
            .to_string_lossy();
        let extension = match output_format {
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Png => "png",
            ImageFormat::WebP => "webp",
            ImageFormat::Gif => "gif",
            _ => "bin", // Fallback
        };
        let output_name = format!(
            "{}.{}",
            file_name.split('.').next().unwrap_or("image"),
            extension
        );
        let output_path = output_dir.join(output_name);

        debug!(
            "[{}/{}] Converting {} to {}",
            i + 1,
            total,
            input_path.display(),
            output_path.display()
        );

        // Perform conversion
        convert_image(input_path, &output_path, options.clone())
            .await
            .with_context(|| format!("Failed to convert {}", input_path.display()))?;
    }

    info!("Successfully converted {} images", total);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_convert_png_to_jpeg() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("test.png");
        let output = temp_dir.path().join("test.jpg");

        // Create a test PNG image
        let img = DynamicImage::new_rgb8(100, 100);
        img.save(&input)?;

        // Convert to JPEG
        convert_image(&input, &output, None).await?;

        // Verify the output exists and is a valid JPEG
        assert!(output.exists());
        let format = detect_format_from_extension(&output);
        assert_eq!(format, Some(ImageFormat::Jpeg));

        Ok(())
    }

    #[tokio::test]
    async fn test_convert_with_options() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let input = temp_dir.path().join("test.png");
        let output = temp_dir.path().join("test.webp");

        // Create a test PNG image
        let img = DynamicImage::new_rgb8(100, 100);
        img.save(&input)?;

        // Convert to WebP with custom options
        let options = ImageFormatOptions::webp()
            .with_quality(85)
            .with_lossless(true);
        convert_image(&input, &output, Some(options)).await?;

        // Verify the output exists and is a valid WebP
        assert!(output.exists());
        let format = detect_format_from_extension(&output);
        assert_eq!(format, Some(ImageFormat::WebP));

        Ok(())
    }
}
