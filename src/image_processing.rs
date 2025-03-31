//! Image processing module for handling various image formats and transformations.
//!
//! This module provides functionality for:
//! - Image format detection and validation
//! - Transparency handling and removal
//! - Letterbox detection and removal
//! - Image dimension querying
//! - Batch image processing
//!
//! # Examples
//!
//! ```rust
//! use std::path::Path;
//! use imx::image_processing::{remove_transparency, remove_letterbox};
//!
//! async fn process_images() -> anyhow::Result<()> {
//!     let image_path = Path::new("image.png");
//!     
//!     // Remove transparency from an image
//!     remove_transparency(image_path).await?;
//!     
//!     // Remove letterboxing
//!     remove_letterbox(image_path).await?;
//!     
//!     Ok(())
//! }
//! ```

#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Context, Result};
use image::{GenericImageView, ImageBuffer, ImageFormat, Rgba};
use log::{info, warn};
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Represents a detected image format based on file magic numbers
///
/// This enum provides a type-safe way to handle different image formats
/// and includes methods for working with file extensions and format conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedImageFormat {
    /// JPEG image format (magic numbers: FF D8 FF)
    Jpeg,
    /// PNG image format (magic numbers: 89 50 4E 47 0D 0A 1A 0A)
    Png,
    /// WebP image format (magic numbers: 52 49 46 46 ... 57 45 42 50)
    WebP,
    /// JPEG XL image format (magic numbers: FF 0A)
    Jxl,
}

impl DetectedImageFormat {
    /// Get the standard file extension for this format
    #[must_use]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpeg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::Jxl => "jxl",
        }
    }

    /// Get all valid file extensions for this format
    #[must_use]
    pub fn all_extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Jpeg => &["jpg", "jpeg"],
            Self::Png => &["png"],
            Self::WebP => &["webp"],
            Self::Jxl => &["jxl"],
        }
    }

    /// Convert to the corresponding `image::ImageFormat`
    #[must_use]
    pub fn to_image_format(&self) -> Option<ImageFormat> {
        match self {
            Self::Jpeg => Some(ImageFormat::Jpeg),
            Self::Png => Some(ImageFormat::Png),
            Self::WebP => Some(ImageFormat::WebP),
            Self::Jxl => None, // image crate might not support JXL
        }
    }
}

/// Determines the actual image format from file magic numbers.
///
/// # Arguments
///
/// * `buffer` - A buffer containing at least the first 12 bytes of the file
///
/// # Returns
///
/// Returns `Some(DetectedImageFormat)` if a known image format is detected, `None` otherwise
#[must_use]
pub fn detect_image_format(buffer: &[u8; 12]) -> Option<DetectedImageFormat> {
    match buffer {
        [0xFF, 0xD8, 0xFF, ..] => Some(DetectedImageFormat::Jpeg),
        [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some(DetectedImageFormat::Png),
        [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x57, 0x45, 0x42, 0x50] => {
            Some(DetectedImageFormat::WebP)
        }
        [0xFF, 0x0A, ..] => Some(DetectedImageFormat::Jxl),
        _ => None,
    }
}

/// Determines if the given path is an image file by checking both extension and file contents.
#[must_use = "Determines if the path is an image file and the result should be checked"]
pub fn is_image_file(path: &Path) -> bool {
    // Get the file extension
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);

    // Check if it's a supported extension
    let has_valid_extension = extension
        .as_deref()
        .is_some_and(|ext| matches!(ext, "jpg" | "jpeg" | "png" | "jxl" | "webp"));

    if !has_valid_extension {
        return false;
    }

    // Then verify file contents
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut buffer = [0u8; 12];
        if file.read_exact(&mut buffer).is_ok() {
            // Detect actual format from magic numbers
            if let Some(actual_format) = detect_image_format(&buffer) {
                // Check for extension mismatch
                if let Some(ext) = extension {
                    let claimed_format = if ext == "jpg" { "jpeg" } else { &ext };
                    if claimed_format != actual_format.extension() {
                        warn!(
                            "File extension mismatch for {}: claims to be {} but appears to be {}",
                            path.display(),
                            claimed_format.to_uppercase(),
                            actual_format.extension().to_uppercase()
                        );
                    }
                }
                return true;
            }

            // Try to open with image crate as fallback
            return image::open(path).is_ok();
        }
    }

    false
}

/// Removes transparency from an image, making transparent pixels black and fully opaque.
///
/// # Arguments
///
/// * `path` - Path to the image file
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// * The image file cannot be opened
/// * The modified image cannot be saved
pub async fn remove_transparency(path: &Path) -> Result<()> {
    if !is_image_file(path) {
        return Ok(());
    }

    info!("Processing image: {}", path.display());

    let img = image::open(path).context("Failed to open image")?;
    let (width, height) = img.dimensions();

    let mut new_image = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.pixels() {
        let new_pixel = if pixel[3] == 0 {
            Rgba([0, 0, 0, 255]) // Black, fully opaque
        } else {
            pixel
        };
        new_image.put_pixel(x, y, new_pixel);
    }

    new_image.save(path).context("Failed to save image")?;
    info!("Processed and saved: {}", path.display());

    Ok(())
}

/// Gets the dimensions of an image.
///
/// # Arguments
///
/// * `path` - Path to the image file
///
/// # Returns
///
/// Returns a `Result` containing a tuple of (width, height)
///
/// # Errors
///
/// Returns an error if the image file cannot be opened
pub fn get_image_dimensions(path: &Path) -> Result<(u32, u32)> {
    let img = image::open(path).context("Failed to open image")?;
    Ok(img.dimensions())
}

/// Removes letterboxing from an image by cropping black borders.
/// Uses a threshold value of 0 (exact black) for letterbox detection.
///
/// # Arguments
///
/// * `path` - Path to the image file
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// * The image file cannot be opened
/// * The modified image cannot be saved
pub async fn remove_letterbox(path: &Path) -> Result<()> {
    remove_letterbox_with_threshold(path, 0).await
}

/// Removes letterboxing from an image by cropping borders based on a threshold value.
///
/// # Arguments
///
/// * `path` - Path to the image file
/// * `threshold` - Threshold value (0-255) for detecting letterbox borders.
///   Pixels with RGB values below this threshold are considered part of the letterbox.
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// * The image file cannot be read
/// * The image cannot be loaded from memory
/// * The cropped image cannot be written to a buffer
/// * The modified image cannot be saved
pub async fn remove_letterbox_with_threshold(path: &Path, threshold: u8) -> Result<()> {
    let img_bytes = fs::read(path).await?;
    let img = image::load_from_memory(&img_bytes).context("Failed to load image from memory")?;

    let (width, height) = img.dimensions();

    let mut top = 0;
    let mut bottom = height - 1;
    let mut left = 0;
    let mut right = width - 1;

    // Helper function to check if a pixel is part of the letterbox
    let is_letterbox = |pixel: Rgba<u8>| -> bool {
        pixel[0] <= threshold && pixel[1] <= threshold && pixel[2] <= threshold
    };

    // Find top
    'outer: for y in 0..height {
        for x in 0..width {
            if !is_letterbox(img.get_pixel(x, y)) {
                top = y;
                break 'outer;
            }
        }
    }

    // Find bottom
    'outer: for y in (0..height).rev() {
        for x in 0..width {
            if !is_letterbox(img.get_pixel(x, y)) {
                bottom = y;
                break 'outer;
            }
        }
    }

    // Find left
    'outer: for x in 0..width {
        for y in 0..height {
            if !is_letterbox(img.get_pixel(x, y)) {
                left = x;
                break 'outer;
            }
        }
    }

    // Find right
    'outer: for x in (0..width).rev() {
        for y in 0..height {
            if !is_letterbox(img.get_pixel(x, y)) {
                right = x;
                break 'outer;
            }
        }
    }

    // Only crop if we found actual letterboxing
    if left < right && top < bottom {
        let cropped = img.crop_imm(left, top, right - left + 1, bottom - top + 1);
        let mut buf = Vec::new();
        cropped
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .context("Failed to write cropped image to buffer")?;
        fs::write(path, buf).await?;
        info!(
            "Cropped image from {}x{} to {}x{}",
            width,
            height,
            right - left + 1,
            bottom - top + 1
        );
    } else {
        info!("No letterbox detected in image");
    }

    Ok(())
}

/// Processes an image file using the provided async processor function.
///
/// This is a generic function that can be used to apply any async image processing
/// operation to a file. It handles file existence checks and error propagation.
///
/// # Type Parameters
///
/// * `F` - A function type that takes a `PathBuf` and returns a `Future`
/// * `Fut` - The specific future type returned by `F`
///
/// # Arguments
///
/// * `path` - Path to the image file to process
/// * `processor` - Async function that performs the actual image processing
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// * The file does not exist
/// * The processor function returns an error
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use anyhow::Result;
/// use imx::process_image;
///
/// async fn example() -> Result<()> {
///     let path = PathBuf::from("image.png");
///     process_image(path, |p| async move {
///         // Process the image here
///         Ok(())
///     }).await?;
///     Ok(())
/// }
/// ```
pub async fn process_image<F, Fut>(path: PathBuf, processor: F) -> Result<()>
where
    F: FnOnce(PathBuf) -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    if !is_image_file(&path) {
        return Ok(());
    }

    info!("Processing image: {}", path.display());
    processor(path).await
}
