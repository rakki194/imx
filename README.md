# imx

A Rust library for image processing and manipulation, providing functionality for letterbox removal, transparency handling, JXL format support, and image grid plotting.

## Features

- 🖼️ Image Processing
  - Remove letterboxing from images with configurable threshold
  - Remove transparency (convert to black)
  - Get image dimensions
  - Process images in batches
  - Create image grid plots with labels
  - Safe numeric conversions for image data
- 📸 Format Support
  - JPEG/JPG
  - PNG
  - WebP
  - JXL (JPEG XL) with automatic PNG conversion
- 🛠️ Utilities
  - File type detection with magic number validation
  - Async/await support with proper lifetime handling
  - Error handling with detailed context
  - Structured logging with info/warn levels
  - Safe numeric type conversions (f32 ↔ i32 ↔ u32 ↔ u8)
  - Unicode text rendering with emoji support
  - Automatic image scaling and alignment

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
imx = "0.1.7"
```

## Usage Examples

### Image Processing

```rust
use imx::{remove_letterbox, remove_letterbox_with_threshold, remove_transparency};
use anyhow::Result;
use std::path::Path;

async fn process_image() -> Result<()> {
    // Remove letterboxing with default threshold (0)
    remove_letterbox(Path::new("path/to/image.jpg")).await?;

    // Remove letterboxing with custom threshold (15 for near-black pixels)
    remove_letterbox_with_threshold(Path::new("path/to/image.png"), 15).await?;

    // Remove transparency (convert transparent pixels to black)
    remove_transparency(Path::new("path/to/image.png")).await?;

    Ok(())
}
```

### JXL Processing

The JXL processing functions require careful handling of lifetimes. Here are the recommended patterns:

```rust
use imx::jxl::{is_jxl_file, process_jxl_file, convert_jxl_to_png};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::future::Future;
use std::pin::Pin;

// Define a type alias for the future to make the code cleaner
type BoxFut<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

// Method 1: Using an inline closure with proper lifetime handling
async fn process_jxl_inline() -> Result<()> {
    let path = Path::new("path/to/image.jxl");
    if is_jxl_file(path) {
        process_jxl_file(path, Some(|p: &Path| -> BoxFut<'_> {
            Box::pin(async move {
                // Process the PNG file here
                Ok(())
            })
        })).await?;
    }
    Ok(())
}

// Method 2: Using a separate function with explicit lifetime parameter
fn process_png<'a>(path: &'a Path) -> BoxFut<'a> {
    Box::pin(async move {
        // Process the PNG file here
        Ok(())
    })
}

async fn process_jxl_separate_fn() -> Result<()> {
    let path = Path::new("path/to/image.jxl");
    if is_jxl_file(path) {
        process_jxl_file(path, Some(process_png)).await?;
    }
    Ok(())
}

// Direct JXL to PNG conversion
async fn convert_jxl() -> Result<()> {
    let input = Path::new("input.jxl");
    let output = Path::new("output.png");
    convert_jxl_to_png(input, output).await?;
    Ok(())
}
```

### Create Image Grid Plots

```rust
use imx::{PlotConfig, create_plot};
use std::path::PathBuf;
use anyhow::Result;

fn create_image_grid() -> Result<()> {
    let config = PlotConfig {
        images: vec![
            PathBuf::from("image1.jpg"),
            PathBuf::from("image2.jpg"),
            PathBuf::from("image3.jpg"),
            PathBuf::from("image4.jpg"),
        ],
        output: PathBuf::from("output.jpg"),
        rows: 2,
        row_labels: vec!["Row 1".to_string(), "Row 2".to_string()],
        column_labels: vec!["Col 1".to_string(), "Col 2".to_string()],
    };

    create_plot(&config)?;
    Ok(())
}
```

### Safe Numeric Conversions

```rust
use imx::numeric::{f32_to_i32, i32_to_u32, f32_to_u8, i32_to_f32_for_pos};

fn convert_numbers() {
    // Safe f32 to i32 conversion (handles NaN, infinity, and out-of-range values)
    let int_val = f32_to_i32(123.45); // Rounds to 123
    assert_eq!(f32_to_i32(f32::NAN), 0); // NaN becomes 0
    
    // Safe i32 to u32 conversion (clamps negative values to 0)
    let uint_val = i32_to_u32(-10); // Returns 0
    assert_eq!(i32_to_u32(42), 42); // Positive passes through
    
    // Safe f32 to u8 conversion (clamps to 0..=255)
    let byte_val = f32_to_u8(300.0); // Returns 255
    assert_eq!(f32_to_u8(-5.0), 0); // Negative becomes 0

    // Safe i32 to f32 conversion for text positioning
    let pos = i32_to_f32_for_pos(42); // Converts to 42.0
}
```

### Check File Types

```rust
use imx::{is_image_file, is_jxl_file};
use std::path::Path;

fn check_files() {
    // Checks both extension and magic numbers for validation
    assert!(is_image_file(Path::new("image.jpg")));
    assert!(is_image_file(Path::new("image.png")));
    assert!(is_image_file(Path::new("image.webp")));
    assert!(is_image_file(Path::new("image.jxl")));
    
    // JXL-specific check
    assert!(is_jxl_file(Path::new("image.jxl")));
    assert!(!is_jxl_file(Path::new("image.png")));
}
```

## Error Handling

All functions return `Result` types with detailed error context. The library uses `anyhow` for error handling:

```rust
use imx::remove_letterbox_with_threshold;
use anyhow::{Context, Result};
use std::path::Path;

async fn process_with_error_handling(path: &str) -> Result<()> {
    remove_letterbox_with_threshold(Path::new(path), 10)
        .await
        .with_context(|| format!("Failed to process image: {}", path))?;
    Ok(())
}
```

## Best Practices

1. **Path Handling**: Always use `Path` or `PathBuf` types instead of raw strings for file paths.
2. **JXL Processing**: When using `process_jxl_file`, properly handle lifetimes using either:
   - A type alias for boxed futures (`BoxFut<'a>`)
   - Explicit lifetime parameters in separate functions
3. **Error Handling**: Use `.with_context()` to add meaningful error messages
4. **Async Functions**: Most image processing functions are async - use them with `.await`
5. **Type Safety**: Use the provided numeric conversion functions instead of raw casts
6. **Grid Plotting**: Ensure consistent image dimensions for best results
7. **Labels**: Unicode and emoji are supported in grid plot labels

## Testing

Run the test suite:

```bash
cargo test
```

The test suite includes:
- Unit tests for all major functions
- Integration tests with sample images
- Error handling tests
- Format-specific tests (JXL, PNG, etc.)
- Numeric conversion tests
- Grid plotting tests
- Lifetime handling tests

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.
