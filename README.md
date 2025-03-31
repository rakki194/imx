# imx - Rust Image Processing Library

A comprehensive Rust library for image processing, manipulation, and visualization.

## Features

- 🖼️ **Image Processing**: Remove letterboxing, handle transparency, process in batch
- 🔄 **Format Support**: JPEG, PNG, WebP, JXL (JPEG XL)
- 🔢 **Numeric Utilities**: Safe type conversions for image data
- 📊 **XY Plotting**: Create image grid plots with labels
- ⚡ **High Performance**: Async/await support for parallel processing
- 🧰 **File Utilities**: File type detection, error handling

## Installation

Add the following to your `Cargo.toml`:

```bash
cargo add imx
```

## Logging Configuration

This library uses the `log` crate for logging and outputs detailed information about processing steps.
Configure a logger (e.g., `env_logger`, `simplelog`) in your application:

```rust
// Example with env_logger
use env_logger::{Builder, Env};

fn main() {
    // Initialize logger with INFO level by default
    Builder::from_env(Env::default().default_filter_or("info"))
        .init();
    
    // Your code that uses imx...
}
```

## Core Function Reference

### Image Processing Functions

#### `remove_letterbox`

Removes black borders (letterboxing) from an image by automatically detecting and cropping them.

```rust
async fn remove_letterbox(path: &Path) -> Result<()>
```

- **Arguments**: `path` - Path to the image file
- **Returns**: `Result<()>` indicating success or failure
- **Behavior**: Uses a threshold value of 0 (exact black) for letterbox detection
- **Error Cases**: Image cannot be opened or saved
- **Performance**: Processes the entire image; higher resolution images will take longer

#### `remove_letterbox_with_threshold`

Similar to `remove_letterbox` but allows specifying a threshold value to detect borders.

```rust
async fn remove_letterbox_with_threshold(path: &Path, threshold: u8) -> Result<()>
```

- **Arguments**:
  - `path` - Path to the image file
  - `threshold` - Threshold value (0-255) for detecting letterbox borders
- **Details**: Pixels with RGB values below this threshold are considered part of the letterbox
- **Use Cases**: Useful for images with slightly off-black letterboxing

#### `remove_transparency`

Replaces transparent pixels with black, opaque pixels.

```rust
async fn remove_transparency(path: &Path) -> Result<()>
```

- **Arguments**: `path` - Path to the image file
- **Behavior**: Scans the image and replaces any pixel with 0 alpha with black (RGB 0,0,0) and full opacity
- **When to Use**: Helpful when converting to formats that don't support transparency or when removing transparent regions

#### `get_image_dimensions`

Retrieves the width and height of an image.

```rust
fn get_image_dimensions(path: &Path) -> Result<(u32, u32)>
```

- **Arguments**: `path` - Path to the image file
- **Returns**: A tuple of `(width, height)` as `u32` values
- **Error Cases**: Image cannot be opened or is corrupt

#### `process_image`

Generic function to apply any async image processing operation to a file.

```rust
async fn process_image<F, Fut>(path: PathBuf, processor: F) -> Result<()>
where
    F: FnOnce(PathBuf) -> Fut,
    Fut: std::future::Future<Output = Result<()>>
```

- **Arguments**:
  - `path` - Path to the image file to process
  - `processor` - Async function that performs the actual image processing
- **Features**: Handles file existence checks and error propagation
- **Advanced Usage**: Allows implementing custom image processors

#### `is_image_file`

Determines if a file is an image by checking both extension and file contents.

```rust
fn is_image_file(path: &Path) -> bool
```

- **Arguments**: `path` - Path to check
- **Behavior**: Checks for valid image extension, then verifies file contents via magic numbers
- **Security**: Prevents processing of files with incorrect extensions (security measure)

#### `detect_image_format`

Detects image format from file contents using magic numbers.

```rust
fn detect_image_format(buffer: &[u8]) -> Option<DetectedImageFormat>
```

- **Arguments**: `buffer` - Byte buffer containing the file header
- **Returns**: The detected format or None if unknown
- **Supported Formats**: JPEG, PNG, WebP, JXL
- **Buffer Size**: Requires at least 12 bytes of the file header

### Format Conversion Functions

#### `convert_image`

Converts an image from one format to another with format-specific options.

```rust
async fn convert_image(
    input_path: &Path,
    output_path: &Path,
    options: Option<ImageFormatOptions>
) -> Result<()>
```

- **Arguments**:
  - `input_path` - Path to the input image
  - `output_path` - Path where the converted image should be saved
  - `options` - Optional format-specific conversion options
- **Supported Formats**: JPEG, PNG, WebP, and others supported by the `image` crate
- **Quality Control**: Options allow setting compression quality, lossless mode
- **Directory Creation**: Automatically creates destination directory if it doesn't exist

#### `convert_images_batch`

Converts multiple images in a batch operation.

```rust
async fn convert_images_batch(
    input_paths: &[PathBuf],
    output_dir: &Path,
    output_format: ImageFormat,
    options: Option<ImageFormatOptions>
) -> Result<()>
```

- **Arguments**:
  - `input_paths` - List of input image paths
  - `output_dir` - Directory where converted images should be saved
  - `output_format` - Target format for conversion
  - `options` - Optional format-specific conversion options
- **Behavior**: Processes each image, maintaining original filenames with new extensions
- **Progress Reporting**: Logs progress information during batch processing
- **Performance**: Serial processing - doesn't execute conversions in parallel

#### `detect_format_from_extension`

Detects image format from file extension.

```rust
fn detect_format_from_extension(path: &Path) -> Option<ImageFormat>
```

- **Arguments**: `path` - Path to check for format
- **Returns**: `Some(ImageFormat)` if a supported format is detected, `None` otherwise
- **Case Sensitivity**: Extension matching is case-insensitive

#### `ImageFormatOptions`

Struct for configuring image format conversion options.

```rust
struct ImageFormatOptions {
    quality: u8,           // Quality value (1-100)
    lossless: bool,        // Whether to use lossless compression
    extra_options: HashMap<String, String>  // Format-specific options
}
```

- **Default Factory Methods**:
  - `ImageFormatOptions::default()` - 85% quality, lossy compression
  - `ImageFormatOptions::jpeg()` - 85% quality, lossy compression
  - `ImageFormatOptions::png()` - 100% quality, lossless compression
  - `ImageFormatOptions::webp()` - 85% quality, lossy compression
- **Customization Methods**:
  - `.with_quality(quality: u8)` - Set specific quality level
  - `.with_lossless(lossless: bool)` - Toggle lossless compression
  - `.with_option(key: &str, value: &str)` - Add format-specific option

### JPEG XL Functions

#### `is_jxl_file`

Checks if a file is a JPEG XL image by examining its file extension.

```rust
fn is_jxl_file(path: &Path) -> bool
```

- **Arguments**: `path` - Path to the file to check
- **Behavior**: Performs a case-insensitive check for the ".jxl" extension
- **Limitations**: Only checks extension, not file contents
- **When to Use**: Quick filtering of files by extension

#### `convert_jxl_to_png`

Converts a JPEG XL image to PNG format.

```rust
async fn convert_jxl_to_png(input_path: &Path, output_path: &Path) -> Result<()>
```

- **Arguments**:
  - `input_path` - Path to the input JXL file
  - `output_path` - Path where the PNG file should be saved
- **Process**:
  1. Reads the JXL file from disk
  2. Decodes the JXL data using `jxl-oxide`
  3. Converts the pixel data to RGBA format
  4. Saves the result as a PNG file
- **Supported**: Both RGB and RGBA JXL images
- **Alpha Handling**: For RGB images, alpha channel is set to fully opaque

#### `process_jxl_file`

Processes a JXL file with an optional custom processor function.

```rust
async fn process_jxl_file<F, Fut>(
    input_path: &Path, 
    processor: Option<F>
) -> Result<()>
where
    F: FnOnce(PathBuf) -> Fut + Send,
    Fut: std::future::Future<Output = Result<()>> + Send
```

- **Arguments**:
  - `input_path` - Path to the JXL file
  - `processor` - Optional function to process the decoded PNG
- **Behavior**: Converts JXL to temporary PNG file, applies processor, then cleans up
- **Advanced Usage**: Allows custom transformations of JXL files
- **Cleanup**: Automatically removes temporary files

### Numeric Functions

#### `f32_to_i32`

Safely converts an f32 to i32, with proper rounding and range clamping.

```rust
fn f32_to_i32(x: f32) -> i32
```

- **Arguments**: `x` - The f32 value to convert
- **Returns**: The converted i32 value, properly rounded and clamped
- **Special Cases**:
  - NaN values are converted to 0
  - Values outside i32's range are clamped to `i32::MIN` or `i32::MAX`
  - Values are rounded to the nearest integer using banker's rounding
- **Safety Guarantees**: No undefined behavior or panics

#### `i32_to_u32`

Safely converts an i32 to u32, clamping negative values to 0.

```rust
fn i32_to_u32(x: i32) -> u32
```

- **Arguments**: `x` - The i32 value to convert
- **Returns**: The converted u32 value, with negative inputs clamped to 0
- **Use Cases**: Working with unsigned quantities like array indices or dimensions

#### `u32_to_i32`

Safely converts a u32 to i32, clamping values exceeding i32::MAX.

```rust
fn u32_to_i32(x: u32) -> i32
```

- **Arguments**: `x` - The u32 value to convert
- **Returns**: The converted i32 value, with large values clamped to i32::MAX
- **Safety**: Prevents truncation errors and undefined behavior

#### `f32_to_u8`

Safely converts an f32 to u8, with proper rounding and range clamping.

```rust
fn f32_to_u8(x: f32) -> u8
```

- **Arguments**: `x` - The f32 value to convert
- **Returns**: The converted u8 value, properly rounded and clamped to 0-255
- **Use Cases**: Converting floating-point color values to byte representation
- **Special Cases**: NaN becomes 0, values outside range are clamped

#### `i32_to_f32_for_pos`

Converts an i32 to f32, optimized for image positioning calculations.

```rust
fn i32_to_f32_for_pos(x: i32) -> f32
```

- **Arguments**: `x` - The i32 value to convert
- **Returns**: The converted f32 value
- **Use Cases**: Calculating positions in drawing operations
- **Precision**: Preserves exact integer values within f32's safe integer range

#### `f32_to_u32`

Safely converts an f32 to u32, with proper rounding and range clamping.

```rust
fn f32_to_u32(x: f32) -> u32
```

- **Arguments**: `x` - The f32 value to convert
- **Returns**: The converted u32 value, properly rounded and clamped
- **Safety**: Handles NaN, negative values, and overflow cases

### XY Plotting Functions

#### `create_plot`

Creates an image grid plot with optional row and column labels.

```rust
fn create_plot(config: &PlotConfig) -> Result<()>
```

- **Arguments**: `config` - Configuration for the plot
- **Returns**: Result indicating success or failure
- **Features**:
  - Arranges images in a grid layout
  - Adds row and column labels
  - Automatically scales images to uniform size
  - Handles text rendering with emoji support
  - Saves the plot as a PNG image
- **Layout**: Automatically calculates optimal layout based on image dimensions

#### `PlotConfig`

Configuration struct for creating image grid plots.

```rust
struct PlotConfig {
    images: Vec<PathBuf>,          // List of image file paths to include
    output: PathBuf,               // Output file path
    rows: u32,                     // Number of rows in the grid
    row_labels: Vec<String>,       // Optional row labels
    column_labels: Vec<String>,    // Optional column labels
    column_label_alignment: LabelAlignment,  // How to align column labels
    row_label_alignment: LabelAlignment,     // How to align row labels
    debug_mode: bool,              // Whether to output debug visualization
    top_padding: u32,              // Space at the top for labels
    left_padding: u32,             // Space at the left for row labels
    font_size: Option<f32>,        // Optional custom font size for labels
}
```

- **Defaults**:
  - `top_padding`: 40 pixels
  - `left_padding`: 40 pixels
  - `column_label_alignment`: Center
  - `row_label_alignment`: Center
  - `debug_mode`: false
  - `font_size`: None (use default font size)
- **Labels**: Support multiline text using '\n' as separator
- **Customization**: All fields can be configured to customize the plot

#### `LabelAlignment`

Enum for specifying label alignment in plots.

```rust
enum LabelAlignment {
    Start,    // Place labels at the left/top edge
    Center,   // Center labels (default)
    End,      // Place labels at the right/bottom edge
}
```

- **Use Cases**: Controls positioning of text labels relative to images
- **Default**: `Center` alignment for both row and column labels

### Layout Engine Functions

#### `Layout`

Represents the complete layout of a plot for rendering.

```rust
struct Layout {
    elements: Vec<LayoutElement>,
    total_width: u32,
    total_height: u32,
}
```

- **Components**:
  - Collection of layout elements (images, labels, padding)
  - Total dimensions of the layout
- **Methods**:
  - `new(width: u32, height: u32)` - Creates a new empty layout
  - `add_element(element: LayoutElement)` - Adds an element to the layout
  - `render_debug()` - Renders a debug visualization of the layout

#### `LayoutElement`

Represents different types of elements in a layout.

```rust
enum LayoutElement {
    Image { rect: LayoutRect, path: String },
    RowLabel { rect: LayoutRect, text: String },
    ColumnLabel { rect: LayoutRect, text: String },
    Padding { rect: LayoutRect, description: String },
}
```

- **Types**:
  - `Image` - An image to be displayed in the grid
  - `RowLabel` - A label for a row of images
  - `ColumnLabel` - A label for a column of images
  - `Padding` - Empty space in the layout
- **Debug Visualization**: Each type is color-coded in debug mode

#### `LayoutRect`

Represents a rectangular region in the layout.

```rust
struct LayoutRect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}
```

- **Coordinates**: (x,y) represent the top-left corner of the rectangle
- **Dimensions**: Width and height define the size of the rectangle
- **Signed Coordinates**: Allows for elements partially outside the visible area

## Examples

### Basic Image Processing

```rust
use std::path::Path;
use anyhow::Result;
use imx::{remove_letterbox, remove_transparency};

async fn process_images() -> Result<()> {
    // Remove letterboxing from an image
    let image_path = Path::new("input/movie_frame.jpg");
    remove_letterbox(image_path).await?;
    
    // Remove transparency from a PNG
    let png_path = Path::new("input/logo.png");
    remove_transparency(png_path).await?;
    
    Ok(())
}
```

### Image Format Conversion

```rust
use std::path::{Path, PathBuf};
use anyhow::Result;
use imx::{convert_image, convert_images_batch, ImageFormatOptions};
use image::ImageFormat;

async fn convert_images_example() -> Result<()> {
    // Convert a single image to WebP with custom quality
    let input = Path::new("input/photo.jpg");
    let output = Path::new("output/photo.webp");
    let options = ImageFormatOptions::webp()
        .with_quality(90)
        .with_lossless(false);
    
    convert_image(input, output, Some(options)).await?;
    
    // Batch convert all JPEGs in a directory to PNG
    let input_dir = Path::new("input");
    let output_dir = Path::new("output/png");
    
    // Collect input paths
    let mut input_paths: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(input_dir)? {
        let path = entry?.path();
        if path.extension().map_or(false, |ext| ext == "jpg") {
            input_paths.push(path);
        }
    }
    
    // Convert all images to PNG with lossless compression
    let png_options = ImageFormatOptions::png();
    convert_images_batch(&input_paths, output_dir, ImageFormat::Png, Some(png_options)).await?;
    
    Ok(())
}
```

### JXL Processing

```rust
use std::path::Path;
use anyhow::Result;
use imx::jxl::{convert_jxl_to_png, process_jxl_file};

async fn process_jxl_example() -> Result<()> {
    // Simple conversion from JXL to PNG
    let jxl_path = Path::new("image.jxl");
    let png_path = Path::new("image.png");
    
    convert_jxl_to_png(jxl_path, png_path).await?;
    
    // Process JXL file with custom handling
    process_jxl_file(jxl_path, Some(|temp_png_path| async move {
        // Remove letterboxing from the temporary PNG
        imx::remove_letterbox(&temp_png_path).await?;
        
        // The modified temp PNG will be used
        Ok(())
    })).await?;
    
    Ok(())
}
```

### Creating Image Grid Plots

```rust
use std::path::PathBuf;
use anyhow::Result;
use imx::xyplot::{PlotConfig, create_plot, LabelAlignment};

fn create_image_grid() -> Result<()> {
    // Create a 2x3 grid of images
    let images = vec![
        PathBuf::from("img1.png"),
        PathBuf::from("img2.png"),
        PathBuf::from("img3.png"),
        PathBuf::from("img4.png"),
        PathBuf::from("img5.png"),
        PathBuf::from("img6.png"),
    ];
    
    let config = PlotConfig {
        images,
        output: PathBuf::from("grid_output.png"),
        rows: 2,
        row_labels: vec!["Set A".to_string(), "Set B".to_string()],
        column_labels: vec!["Low".to_string(), "Medium".to_string(), "High".to_string()],
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Start,
        debug_mode: false,
        top_padding: 60,  // Extra space for column labels
        left_padding: 80, // Extra space for row labels
        font_size: None,  // Use default font size
    };
    
    create_plot(&config)?;
    Ok(())
}
```

### Safe Numeric Conversions

```rust
use imx::numeric::{f32_to_i32, f32_to_u8, i32_to_u32};

fn numeric_example() {
    // Convert float to integer with rounding
    let float_val = 123.7;
    let int_val = f32_to_i32(float_val);
    assert_eq!(int_val, 124); // Rounds to nearest
    
    // Handle NaN values safely
    let nan_val = f32_to_i32(f32::NAN);
    assert_eq!(nan_val, 0);   // NaN becomes 0
    
    // Convert to byte values (for image processing)
    let color_val = 240.8;
    let byte_val = f32_to_u8(color_val);
    assert_eq!(byte_val, 241); // Rounds to nearest, clamps to 0-255
    
    // Safe signed to unsigned conversion
    let signed_val = -5;
    let unsigned_val = i32_to_u32(signed_val);
    assert_eq!(unsigned_val, 0); // Negative becomes 0
}
```

## Best Practices

### Path Handling

Always use platform-agnostic path handling:

```rust
use std::path::Path;

// Good
let img_path = Path::new("images").join("photo.jpg");

// Avoid platform-specific separators
// let img_path = "images/photo.jpg"; // Works on Unix but not Windows
```

### Using Async Functions

Ensure your runtime is configured for async functions:

```rust
use tokio::runtime::Runtime;

// Setup a basic tokio runtime
let rt = Runtime::new().unwrap();
rt.block_on(async {
    // Call async imx functions here
    imx::remove_letterbox(Path::new("image.jpg")).await.unwrap();
});
```

## Layout Algorithm

The library uses a sophisticated layout algorithm for grid plots:

1. Images are arranged in a grid with specified number of rows
2. Columns are calculated automatically based on image count and rows
3. All images are scaled to maintain uniform size in the grid
4. Row labels are placed on the left side, aligned to the row
5. Column labels are placed above each column
6. Padding is added around all elements for visual spacing

For debugging layouts, set `debug_mode: true` in your `PlotConfig` to see a color-coded visualization of the calculated layout.
