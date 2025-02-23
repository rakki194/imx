//! JPEG XL image format handling module.
//!
//! This module provides functionality for working with JPEG XL (JXL) image files, including:
//! - JXL file detection
//! - Conversion from JXL to PNG format
//! - Processing JXL files with custom transformations
//!
//! The module uses the `jxl-oxide` crate for JXL decoding and supports both RGB and RGBA color formats.
//!
//! # Examples
//!
//! ```rust
//! use std::path::Path;
//! use imx::jxl::{convert_jxl_to_png, process_jxl_file};
//!
//! async fn process_jxl() -> anyhow::Result<()> {
//!     let input = Path::new("image.jxl");
//!     let output = Path::new("image.png");
//!     
//!     // Simple conversion
//!     convert_jxl_to_png(input, output).await?;
//!     
//!     // Process with custom function
//!     process_jxl_file(input, Some(|path| async move {
//!         // Custom processing logic here
//!         Ok(())
//!     })).await?;
//!     
//!     Ok(())
//! }
//! ```

#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Context, Result};
use image::{ImageBuffer, Rgba};
use jxl_oxide::{JxlImage, PixelFormat};
use log::info;
use std::path::{Path, PathBuf};

/// Checks if a file is a JPEG XL image by examining its file extension.
///
/// This function performs a case-insensitive check for the ".jxl" extension.
/// Note that this is a simple extension check and does not verify the file contents.
/// For more thorough validation, use `convert_jxl_to_png` which will attempt to decode the file.
///
/// # Arguments
///
/// * `path` - Path to the file to check
///
/// # Returns
///
/// Returns `true` if the file has a `.jxl` extension (case-insensitive), `false` otherwise
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use imx::jxl::is_jxl_file;
///
/// let path = Path::new("image.jxl");
/// assert!(is_jxl_file(path));
///
/// let path = Path::new("image.png");
/// assert!(!is_jxl_file(path));
/// ```
#[must_use]
pub fn is_jxl_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("jxl"))
}

/// Converts a JPEG XL image to PNG format.
///
/// This function performs the following steps:
/// 1. Reads the JXL file from disk
/// 2. Decodes the JXL data using `jxl-oxide`
/// 3. Converts the pixel data to RGBA format
/// 4. Saves the result as a PNG file
///
/// The function supports both RGB and RGBA JXL images. For RGB images,
/// the alpha channel will be set to fully opaque (255).
///
/// # Arguments
///
/// * `input_path` - Path to the input JXL file
/// * `output_path` - Path where the PNG file should be saved
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// * The JXL file cannot be read from disk
/// * The JXL data is invalid or corrupted
/// * The JXL frame cannot be rendered
/// * The pixel format is not RGB or RGBA
/// * The PNG file cannot be saved to disk
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use imx::jxl::convert_jxl_to_png;
///
/// async fn convert() -> anyhow::Result<()> {
///     let input = Path::new("input.jxl");
///     let output = Path::new("output.png");
///     convert_jxl_to_png(input, output).await?;
///     Ok(())
/// }
/// ```
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
                PixelFormat::Rgba => {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let rgba = [
                        (pixel_data[pixel_idx] * 255.0) as u8,
                        (pixel_data[pixel_idx + 1] * 255.0) as u8,
                        (pixel_data[pixel_idx + 2] * 255.0) as u8,
                        (pixel_data[pixel_idx + 3] * 255.0) as u8,
                    ];
                    Rgba(rgba)
                }
                PixelFormat::Rgb => {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let rgb = [
                        (pixel_data[pixel_idx] * 255.0) as u8,
                        (pixel_data[pixel_idx + 1] * 255.0) as u8,
                        (pixel_data[pixel_idx + 2] * 255.0) as u8,
                        255,
                    ];
                    Rgba(rgb)
                }
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

/// Processes a JXL file by converting it to PNG and optionally applying a custom transformation.
///
/// This function:
/// 1. Verifies the input is a JXL file
/// 2. Converts it to PNG format
/// 3. Applies an optional processing function
/// 4. Removes the original JXL file
///
/// The processor function is called with the path to the converted PNG file.
/// This allows for additional transformations to be applied after conversion.
///
/// # Type Parameters
///
/// * `F` - Type of the processor function
/// * `Fut` - Future type returned by the processor function
///
/// # Arguments
///
/// * `input_path` - Path to the input JXL file
/// * `processor` - Optional async function to process the PNG file after conversion
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// * The input file is not a JXL file
/// * The JXL to PNG conversion fails
/// * The processor function returns an error
/// * The original JXL file cannot be removed
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::{Path, PathBuf};
/// use anyhow::Result;
/// use imx::jxl::process_jxl_file;
///
/// async fn example() -> Result<()> {
///     let input = Path::new("image.jxl");
///     
///     // Process with a custom function
///     process_jxl_file(input, Some(|path: PathBuf| async move {
///         // Custom processing logic here
///         Ok(())
///     })).await?;
///     
///     Ok(())
/// }
/// ```
pub async fn process_jxl_file<F, Fut>(input_path: &Path, processor: Option<F>) -> Result<()>
where
    F: FnOnce(PathBuf) -> Fut + Send,
    Fut: std::future::Future<Output = Result<()>> + Send,
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
        processor(png_path.clone()).await?;
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
