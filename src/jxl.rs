#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Context, Result};
use image::{ImageBuffer, Rgba};
use jxl_oxide::{JxlImage, PixelFormat};
use log::info;
use std::path::Path;

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
    info!(
        "Converting JXL to PNG: {} -> {}",
        input_path.display(),
        output_path.display()
    );

    // Read JXL file
    let jxl_data = tokio::fs::read(input_path)
        .await
        .with_context(|| format!("Failed to read JXL file: {}", input_path.display()))?;

    // Decode JXL
    let image = JxlImage::read_with_defaults(&jxl_data[..]).map_err(|e| {
        anyhow::anyhow!("Failed to decode JXL file {}: {}", input_path.display(), e)
    })?;

    // Convert to RGBA
    let (width, height) = (image.width(), image.height());
    let mut rgba: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    let render = image.render_frame(0).map_err(|e| {
        anyhow::anyhow!("Failed to render JXL frame {}: {}", input_path.display(), e)
    })?;
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
    tokio::fs::remove_file(input_path).await.with_context(|| {
        format!(
            "Failed to remove original JXL file: {}",
            input_path.display()
        )
    })?;

    info!("Successfully processed JXL file: {}", input_path.display());

    // Return the conversion error if it failed
    conversion_result
}
