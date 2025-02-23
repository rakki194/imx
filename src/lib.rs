//! IMX: Image Processing and Manipulation Library
//!
//! This crate provides a comprehensive set of tools for image processing, manipulation,
//! and visualization. It includes functionality for:
//!
//! - Image processing operations (resizing, format conversion, transparency handling)
//! - JPEG XL format support and conversion
//! - Numerical operations for image data
//! - XY plotting capabilities for data visualization
//!
//! # Features
//!
//! - Efficient image processing with support for common operations
//! - JPEG XL format handling with conversion to PNG
//! - Letterboxing removal and transparency handling
//! - Data visualization through XY plotting
//!
//! # Example
//!
//! ```rust,no_run
//! use std::path::PathBuf;
//! use imx::{process_image, PlotConfig, create_plot};
//! use anyhow::Result;
//!
//! async fn example() -> Result<()> {
//!     // Process an image
//!     let path = PathBuf::from("input.jpg");
//!     process_image(path, |p| async move {
//!         // Process the image here
//!         Ok(())
//!     }).await?;
//!     
//!     // Create a plot
//!     let config = PlotConfig {
//!         images: vec![PathBuf::from("image1.png")],
//!         output: PathBuf::from("output.png"),
//!         rows: 1,
//!         row_labels: vec![],
//!         column_labels: vec![],
//!     };
//!     create_plot(&config)?;
//!     
//!     Ok(())
//! }
//! ```

#![warn(clippy::all, clippy::pedantic)]

/// Image processing module providing functions for image manipulation,
/// format conversion, and various transformations.
pub mod image_processing;

/// JPEG XL format support module with functions for handling JXL files
/// and converting them to other formats.
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::Path;
/// use anyhow::Result;
/// use imx::jxl::process_jxl_file;
///
/// async fn example() -> Result<()> {
///     let input = Path::new("image.jxl");
///     
///     // Process with a simple closure
///     process_jxl_file(input, Some(|path| async move {
///         // Custom processing logic here
///         Ok(())
///     })).await?;
///     
///     Ok(())
/// }
/// ```
///
/// You can also use more complex processing logic:
///
/// ```rust,no_run
/// use std::path::Path;
/// use anyhow::Result;
/// use imx::jxl::process_jxl_file;
///
/// async fn example() -> Result<()> {
///     let input = Path::new("image.jxl");
///     
///     // Process with more complex logic
///     process_jxl_file(input, Some(|path| async move {
///         // Load the PNG file
///         let img = image::open(&path)?;
///         
///         // Apply some transformations
///         let processed = img.grayscale();
///         
///         // Save back to the same path
///         processed.save(path)?;
///         Ok(())
///     })).await?;
///     
///     Ok(())
/// }
/// ```
pub mod jxl;

/// Numerical operations module for image data processing and analysis.
pub mod numeric;

/// XY plotting module for creating visualizations and graphs.
pub mod xyplot;

// Re-export commonly used types and functions
pub use image_processing::{
    get_image_dimensions, is_image_file, process_image, remove_letterbox,
    remove_letterbox_with_threshold, remove_transparency,
};
pub use jxl::{convert_jxl_to_png, is_jxl_file, process_jxl_file};
pub use xyplot::{create_plot, PlotConfig};

#[cfg(test)]
mod tests {
    mod image_processing_tests;
    mod jxl_tests;
    mod numeric_tests;
    mod xyplot_tests;
}
