#![warn(clippy::all, clippy::pedantic)]

use crate::image_processing;
use image::{GenericImageView, ImageBuffer, Rgba};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[tokio::test]
async fn test_remove_letterbox() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let image_path = temp_dir.path().join("test.png");

    // Create a test image with letterboxing
    let width = 100;
    let height = 100;
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // Fill with letterboxing
    for y in 0..height {
        for x in 0..width {
            let pixel = if y < height / 4 || y > height * 3 / 4 {
                Rgba([0, 0, 0, 255]) // Black letterbox
            } else {
                Rgba([255, 255, 255, 255]) // White content
            };
            img.put_pixel(x, y, pixel);
        }
    }

    img.save(&image_path)?;

    // Process the image
    image_processing::remove_letterbox_with_threshold(&image_path, 10).await?;

    // Verify the result
    let processed_img = image::open(&image_path)?;
    let (_, height) = processed_img.dimensions();
    assert!(height < 100); // Should be cropped
    Ok(())
}

#[test]
fn test_is_image_file() {
    let temp_dir = TempDir::new().unwrap();

    // Helper function to create a file with specific content
    let create_file = |name: &str, content: &[u8]| {
        let path = temp_dir.path().join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    };

    // Create valid image files
    let jpeg_path = create_file(
        "test.jpg",
        &[
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
        ],
    );
    let png_path = create_file(
        "test.png",
        &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
        ],
    );
    let webp_path = create_file(
        "test.webp",
        &[
            0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50,
        ],
    );
    let jxl_path = create_file(
        "test.jxl",
        &[
            0xFF, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
    );

    // Create mismatched extension files
    let png_as_jpg = create_file(
        "actually_png.jpg",
        &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
        ],
    );
    let jpeg_as_png = create_file(
        "actually_jpeg.png",
        &[
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
        ],
    );
    let webp_as_jxl = create_file(
        "actually_webp.jxl",
        &[
            0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50,
        ],
    );

    // Create invalid files
    let fake_jpg = create_file("fake.jpg", b"not a real jpeg");
    let empty_png = create_file("empty.png", b"");
    let small_webp = create_file("small.webp", &[0x52, 0x49, 0x46]);
    let text_file = create_file("test.txt", b"This is a text file");
    let no_ext = create_file("no_extension", b"no extension file");
    let uppercase_ext = create_file(
        "TEST.PNG",
        &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
        ],
    );

    // Test valid images with correct extensions
    assert!(
        image_processing::is_image_file(&jpeg_path),
        "Valid JPEG not recognized"
    );
    assert!(
        image_processing::is_image_file(&png_path),
        "Valid PNG not recognized"
    );
    assert!(
        image_processing::is_image_file(&webp_path),
        "Valid WebP not recognized"
    );
    assert!(
        image_processing::is_image_file(&jxl_path),
        "Valid JXL not recognized"
    );
    assert!(
        image_processing::is_image_file(&uppercase_ext),
        "Valid PNG with uppercase extension not recognized"
    );

    // Test valid images with mismatched extensions
    assert!(
        image_processing::is_image_file(&png_as_jpg),
        "PNG with JPG extension not recognized as valid image"
    );
    assert!(
        image_processing::is_image_file(&jpeg_as_png),
        "JPEG with PNG extension not recognized as valid image"
    );
    assert!(
        image_processing::is_image_file(&webp_as_jxl),
        "WebP with JXL extension not recognized as valid image"
    );

    // Test invalid files
    assert!(
        !image_processing::is_image_file(&fake_jpg),
        "Invalid JPEG content accepted"
    );
    assert!(
        !image_processing::is_image_file(&empty_png),
        "Empty PNG file accepted"
    );
    assert!(
        !image_processing::is_image_file(&small_webp),
        "Incomplete WebP file accepted"
    );
    assert!(
        !image_processing::is_image_file(&text_file),
        "Text file accepted as image"
    );
    assert!(
        !image_processing::is_image_file(&no_ext),
        "File without extension accepted"
    );

    // Test non-existent file
    assert!(
        !image_processing::is_image_file(&temp_dir.path().join("nonexistent.jpg")),
        "Non-existent file accepted"
    );
}
