# imx

A Rust library for image processing and manipulation, providing functionality for letterbox removal, transparency handling, and JXL format support.

## Features

- 🖼️ Image Processing
  - Remove letterboxing from images with configurable threshold
  - Remove transparency (convert to black)
  - Get image dimensions
  - Process images in batches
- 📸 Format Support
  - JPEG/JPG
  - PNG
  - WebP
  - JXL (JPEG XL) with automatic PNG conversion
- 🛠️ Utilities
  - File type detection
  - Async/await support
  - Error handling with context
  - Detailed logging

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
imx = "0.1.1"
```

## Usage Examples

### Remove Letterboxing

```rust
use imx::{remove_letterbox, remove_letterbox_with_threshold};
use anyhow::Result;

async fn process_image() -> Result<()> {
    // Remove letterboxing with default threshold (0)
    remove_letterbox("path/to/image.jpg").await?;

    // Remove letterboxing with custom threshold
    remove_letterbox_with_threshold("path/to/image.png", 15).await?;
    Ok(())
}
```

### Process JXL Images

```rust
use imx::{is_jxl_file, process_jxl_file};
use anyhow::Result;

async fn process_jxl() -> Result<()> {
    let path = "path/to/image.jxl";
    if is_jxl_file(path) {
        // Convert to PNG and optionally process
        process_jxl_file(path, Some(|png_path| async move {
            // Process the PNG file
            Ok(())
        })).await?;
    }
    Ok(())
}
```

### Remove Transparency

```rust
use imx::remove_transparency;
use anyhow::Result;

async fn process_transparent_image() -> Result<()> {
    remove_transparency("path/to/image.png").await?;
    Ok(())
}
```

### Get Image Dimensions

```rust
use imx::get_image_dimensions;
use anyhow::Result;

fn check_image_size() -> Result<()> {
    let (width, height) = get_image_dimensions("path/to/image.jpg")?;
    println!("Image dimensions: {}x{}", width, height);
    Ok(())
}
```

### Check File Types

```rust
use imx::{is_image_file, is_jxl_file};

fn check_files() {
    assert!(is_image_file("image.jpg"));
    assert!(is_image_file("image.png"));
    assert!(is_image_file("image.jxl"));
    assert!(is_jxl_file("image.jxl"));
    assert!(!is_jxl_file("image.png"));
}
```

## Error Handling

All functions return `Result` types with detailed error context:

```rust
use imx::remove_letterbox_with_threshold;
use anyhow::{Context, Result};

async fn process_with_error_handling(path: &str) -> Result<()> {
    remove_letterbox_with_threshold(path, 10)
        .await
        .with_context(|| format!("Failed to process image: {}", path))?;
    Ok(())
}
```

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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.
