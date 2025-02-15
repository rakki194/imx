use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use image::{GenericImageView, ImageBuffer, Rgba, ImageFormat};
use log::info;
use tokio::fs;

/// Determines if the given path is an image file.
#[must_use = "Determines if the path is an image file and the result should be checked"]
pub fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "jxl" | "webp"),
        None => false,
    }
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
pub fn get_image_dimensions(path: &Path) -> Result<(u32, u32)> {
    let img = image::open(path).context("Failed to open image")?;
    Ok(img.dimensions())
}

/// Removes letterboxing from an image by cropping black borders.
/// 
/// # Arguments
/// 
/// * `path` - Path to the image file
/// 
/// # Returns
/// 
/// Returns a `Result<()>` indicating success or failure
pub async fn remove_letterbox(path: &Path) -> Result<()> {
    let img_bytes = fs::read(path).await?;
    let img = image::load_from_memory(&img_bytes)
        .context("Failed to load image from memory")?;

    let (width, height) = img.dimensions();

    let mut top = 0;
    let mut bottom = height - 1;
    let mut left = 0;
    let mut right = width - 1;

    // Find top
    'outer: for y in 0..height {
        for x in 0..width {
            if img.get_pixel(x, y)[0] != 0 {
                top = y;
                break 'outer;
            }
        }
    }

    // Find bottom
    'outer: for y in (0..height).rev() {
        for x in 0..width {
            if img.get_pixel(x, y)[0] != 0 {
                bottom = y;
                break 'outer;
            }
        }
    }

    // Find left
    'outer: for x in 0..width {
        for y in 0..height {
            if img.get_pixel(x, y)[0] != 0 {
                left = x;
                break 'outer;
            }
        }
    }

    // Find right
    'outer: for x in (0..width).rev() {
        for y in 0..height {
            if img.get_pixel(x, y)[0] != 0 {
                right = x;
                break 'outer;
            }
        }
    }

    let cropped = img.crop_imm(left, top, right - left + 1, bottom - top + 1);
    let mut buf = Vec::new();
    cropped
        .write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png)
        .context("Failed to write cropped image to buffer")?;
    fs::write(path, buf).await?;

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

/// Checks if a caption file exists and is not empty.
#[must_use = "Checks if the caption file exists and is not empty and the result should be checked"]
pub async fn caption_file_exists_and_not_empty(path: &Path) -> bool {
    if path.exists() {
        match fs::read_to_string(path).await {
            Ok(content) => !content.trim().is_empty(),
            Err(_) => false,
        }
    } else {
        false
    }
}

/// Renames a file to remove the image extension.
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be renamed.
#[must_use = "Renames a file and requires handling of the result to ensure the file is properly renamed"]
pub async fn rename_file_without_image_extension(path: &Path) -> std::io::Result<()> {
    if let Some(old_name) = path.to_str() {
        if old_name.contains(".jpeg") || old_name.contains(".png") || old_name.contains(".jpg") {
            let new_name = old_name.replace(".jpeg", "").replace(".png", "").replace(".jpg", "");
            fs::rename(old_name, &new_name).await?;
            info!("Renamed {old_name} to {new_name}");
        }
    }
    Ok(())
} 