use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use image::{GenericImageView, ImageBuffer, Rgba, ImageFormat};
use log::info;
use tokio::fs;
use jxl_oxide::{JxlImage, PixelFormat};

/// Determines if the given path is an image file.
#[must_use = "Determines if the path is an image file and the result should be checked"]
pub fn is_image_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "jxl" | "webp"),
        None => false,
    }
}

/// Check if a file is a JXL image
/// 
/// # Arguments
/// 
/// * `path` - Path to the file to check
/// 
/// # Returns
/// 
/// Returns true if the file has a .jxl extension
pub fn is_jxl_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("jxl"))
}

/// Convert a JXL image to PNG format
/// 
/// # Arguments
/// 
/// * `input_path` - Path to the input JXL file
/// * `output_path` - Path where the PNG file should be saved
/// 
/// # Returns
/// 
/// Returns a `Result<()>` indicating success or failure
pub async fn convert_jxl_to_png(input_path: &Path, output_path: &Path) -> Result<()> {
    info!("Converting JXL to PNG: {} -> {}", input_path.display(), output_path.display());
    
    // Read JXL file
    let jxl_data = tokio::fs::read(input_path)
        .await
        .with_context(|| format!("Failed to read JXL file: {}", input_path.display()))?;
    
    // Decode JXL
    let image = JxlImage::read_with_defaults(&jxl_data[..])
        .map_err(|e| anyhow::anyhow!("Failed to decode JXL file {}: {}", input_path.display(), e))?;
    
    // Convert to RGBA
    let (width, height) = (image.width(), image.height());
    let mut rgba: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    
    let render = image.render_frame(0)
        .map_err(|e| anyhow::anyhow!("Failed to render JXL frame {}: {}", input_path.display(), e))?;
    let mut stream = render.stream();
    
    // Create a buffer to hold the pixel data
    let channels = stream.channels() as usize;
    let mut pixel_data = vec![0.0f32; width as usize * height as usize * channels];
    stream.write_to_buffer(&mut pixel_data);
    
    // Convert pixel data to RGBA
    for y in 0..height {
        for x in 0..width {
            let pixel_idx = ((y * width + x) as usize) * channels;
            let pixel = match image.pixel_format() {
                PixelFormat::Rgba => Rgba([
                    (pixel_data[pixel_idx] * 255.0) as u8,
                    (pixel_data[pixel_idx + 1] * 255.0) as u8,
                    (pixel_data[pixel_idx + 2] * 255.0) as u8,
                    (pixel_data[pixel_idx + 3] * 255.0) as u8,
                ]),
                PixelFormat::Rgb => Rgba([
                    (pixel_data[pixel_idx] * 255.0) as u8,
                    (pixel_data[pixel_idx + 1] * 255.0) as u8,
                    (pixel_data[pixel_idx + 2] * 255.0) as u8,
                    255,
                ]),
                _ => anyhow::bail!("Unsupported JXL pixel format: {:?}", image.pixel_format()),
            };
            rgba.put_pixel(x, y, pixel);
        }
    }
    
    // Save as PNG
    rgba.save(output_path)
        .with_context(|| format!("Failed to save PNG file: {}", output_path.display()))?;
    
    info!("Successfully converted JXL to PNG");
    Ok(())
}

/// Process a JXL file by converting it to PNG and optionally applying a processing function
/// 
/// # Arguments
/// 
/// * `input_path` - Path to the input JXL file
/// * `processor` - Optional function to process the PNG file after conversion
/// 
/// # Returns
/// 
/// Returns a `Result<()>` indicating success or failure
pub async fn process_jxl_file<'a, F, Fut>(input_path: &Path, processor: Option<F>) -> Result<()>
where
    F: for<'r> FnOnce(&'r Path) -> Fut + Send + 'a,
    Fut: std::future::Future<Output = Result<()>> + Send + 'a,
{
    if !is_jxl_file(input_path) {
        anyhow::bail!("Not a JXL file: {}", input_path.display());
    }

    // Create temporary PNG path
    let png_path = input_path.with_extension("png");
    
    // Try to convert to PNG, but continue even if it fails
    let conversion_result = convert_jxl_to_png(input_path, &png_path).await;
    
    // Apply processor if provided
    if let Some(processor) = processor {
        processor(&png_path).await?;
    }
    
    // Delete original JXL file
    tokio::fs::remove_file(input_path)
        .await
        .with_context(|| format!("Failed to remove original JXL file: {}", input_path.display()))?;
    
    info!("Successfully processed JXL file: {}", input_path.display());
    
    // Return the conversion error if it failed
    conversion_result
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
/// Uses a threshold value of 0 (exact black) for letterbox detection.
/// 
/// # Arguments
/// 
/// * `path` - Path to the image file
/// 
/// # Returns
/// 
/// Returns a `Result<()>` indicating success or failure
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
pub async fn remove_letterbox_with_threshold(path: &Path, threshold: u8) -> Result<()> {
    let img_bytes = fs::read(path).await?;
    let img = image::load_from_memory(&img_bytes)
        .context("Failed to load image from memory")?;

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
        info!("Cropped image from {}x{} to {}x{}", 
            width, height, 
            right - left + 1, bottom - top + 1
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use std::io::Write;
    use std::future::Future;
    use std::pin::Pin;

    #[tokio::test]
    async fn test_is_jxl_file() {
        assert!(is_jxl_file(Path::new("test.jxl")));
        assert!(is_jxl_file(Path::new("test.JXL")));
        assert!(!is_jxl_file(Path::new("test.png")));
        assert!(!is_jxl_file(Path::new("test")));
    }

    #[tokio::test]
    async fn test_process_jxl_file_invalid_extension() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let invalid_file = temp_dir.path().join("test.png");
        fs::write(&invalid_file, b"not a jxl file")?;

        let result = process_jxl_file::<fn(&Path) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>, Pin<Box<dyn Future<Output = Result<()>> + Send>>>(&invalid_file, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a JXL file"));
        Ok(())
    }

    #[tokio::test]
    async fn test_process_jxl_file_with_processor() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let jxl_file = temp_dir.path().join("test.jxl");
        fs::write(&jxl_file, b"dummy jxl data")?;

        let processed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let processed_clone = processed.clone();

        let processor = move |path: &Path| {
            let path = path.to_owned();
            let processed = processed_clone.clone();
            Box::pin(async move {
                // Create a PNG file since JXL conversion will fail
                let mut file = fs::File::create(&path)?;
                file.write_all(b"dummy png data")?;
                
                assert_eq!(path.extension().unwrap(), "png");
                processed.store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            })
        };

        // The JXL processing will fail, but the processor should still be called
        let result = process_jxl_file(&jxl_file, Some(processor)).await;
        assert!(result.is_err()); // Should fail due to invalid JXL data
        assert!(processed.load(std::sync::atomic::Ordering::SeqCst)); // But processor should still be called
        Ok(())
    }

    #[tokio::test]
    async fn test_remove_letterbox() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let image_path = temp_dir.path().join("test.png");
        
        // Create a test image with letterboxing
        let width = 100;
        let height = 100;
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

        // Fill with letterboxing
        for y in 0..height {
            for x in 0..width {
                let pixel = if y < height/4 || y > height*3/4 {
                    Rgba([0, 0, 0, 255]) // Black letterbox
                } else {
                    Rgba([255, 255, 255, 255]) // White content
                };
                img.put_pixel(x, y, pixel);
            }
        }

        img.save(&image_path)?;

        // Process the image
        remove_letterbox_with_threshold(&image_path, 10).await?;

        // Verify the result
        let processed_img = image::open(&image_path)?;
        let (_, height) = processed_img.dimensions();
        assert!(height < 100); // Should be cropped
        Ok(())
    }
} 