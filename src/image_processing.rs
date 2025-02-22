#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Context, Result};
use image::{GenericImageView, ImageBuffer, ImageFormat, Rgba};
use log::{info, warn};
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Determines the actual image format from file magic numbers.
fn detect_image_format(buffer: &[u8; 12]) -> Option<&'static str> {
    match buffer {
        [0xFF, 0xD8, 0xFF, ..] => Some("jpeg"),
        [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some("png"),
        [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x57, 0x45, 0x42, 0x50] => Some("webp"),
        [0xFF, 0x0A, ..] => Some("jxl"),
        _ => None,
    }
}

/// Determines if the given path is an image file by checking both extension and file contents.
#[must_use = "Determines if the path is an image file and the result should be checked"]
pub fn is_image_file(path: &Path) -> bool {
    // Get the file extension
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase);

    // Check if it's a supported extension
    let has_valid_extension = extension.as_deref().is_some_and(|ext| {
        matches!(ext, "jpg" | "jpeg" | "png" | "jxl" | "webp")
    });

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
                    if claimed_format != actual_format {
                        warn!(
                            "File extension mismatch for {}: claims to be {} but appears to be {}",
                            path.display(),
                            claimed_format.to_uppercase(),
                            actual_format.to_uppercase()
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

    info!("Processing image: {:?}", path);

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
    info!("Processed and saved: {:?}", path);

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
            .write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png)
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

/// Processes an image file with the given function.
///
/// # Arguments
///
/// * `path` - Path to the image file
/// * `processor` - Async function that processes the image
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure
///
/// # Errors
///
/// Returns an error if the processor function returns an error
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
