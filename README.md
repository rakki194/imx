# imx

A Rust utility library for efficient image processing and manipulation. Designed for batch image operations, transparency handling, and letterbox removal with robust async support.

## Features

- 🖼️ Asynchronous image processing using Tokio
- 🎨 Transparency removal and manipulation
- ✂️ Smart letterbox detection and removal
- 📏 Image dimension analysis
- 📝 Caption file handling
- 🔄 Batch processing capabilities
- 🛡️ Robust error handling with anyhow

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
imx = "0.1.0"
```

## Usage Examples

### Basic Image Processing

Process a single image to remove transparency:

```rust
use imx::{remove_transparency, Path};
use anyhow::Result;

async fn make_image_opaque(path: &str) -> Result<()> {
    remove_transparency(Path::new(path)).await
}
```

### Batch Image Processing

Process multiple images in a directory:

```rust
use imx::{process_image, remove_letterbox, Path, PathBuf};
use anyhow::Result;

async fn process_images(paths: Vec<PathBuf>) -> Result<()> {
    for path in paths {
        process_image(path, |p| async move {
            remove_letterbox(&p).await
        }).await?;
    }
    Ok(())
}
```

### Image Analysis

Get image dimensions and check file types:

```rust
use imx::{get_image_dimensions, is_image_file, Path};
use anyhow::Result;

fn analyze_image(path: &str) -> Result<()> {
    let path = Path::new(path);
    if is_image_file(path) {
        let (width, height) = get_image_dimensions(path)?;
        println!("Image dimensions: {}x{}", width, height);
    }
    Ok(())
}
```

### Caption File Operations

Work with image captions:

```rust
use imx::{caption_file_exists_and_not_empty, Path};

async fn check_caption(path: &str) -> bool {
    caption_file_exists_and_not_empty(Path::new(path)).await
}
```

### File Renaming

Remove image extensions from filenames:

```rust
use imx::{rename_file_without_image_extension, Path};

async fn clean_filename(path: &str) -> std::io::Result<()> {
    rename_file_without_image_extension(Path::new(path)).await
}
```

## Supported Image Formats

The library supports the following image formats:
- JPEG (.jpg, .jpeg)
- PNG (.png)
- JPEG XL (.jxl)
- WebP (.webp)

## Error Handling

All operations return `Result` types with detailed error information:
- `io::Result` for basic file operations
- `anyhow::Result` for complex operations with rich error context

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License. 