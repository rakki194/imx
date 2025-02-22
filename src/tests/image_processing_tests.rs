#![warn(clippy::all, clippy::pedantic)]

use crate::image_processing;
use image::{GenericImageView, ImageBuffer, Rgba};
use std::fs;
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
    let test_cases = vec![
        ("test.jpg", true),
        ("test.jpeg", true),
        ("test.png", true),
        ("test.jxl", true),
        ("test.webp", true),
        ("test.txt", false),
        ("test", false),
        ("test.JPG", true),
        ("test.JPEG", true),
        ("test.PNG", true),
    ];

    for (path, expected) in test_cases {
        assert_eq!(
            image_processing::is_image_file(std::path::Path::new(path)),
            expected,
            "Failed for path: {}",
            path
        );
    }
}

#[tokio::test]
async fn test_caption_file_exists_and_not_empty() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Test non-existent file
    let non_existent = temp_dir.path().join("non_existent.txt");
    assert!(!image_processing::caption_file_exists_and_not_empty(&non_existent).await);

    // Test empty file
    let empty_file = temp_dir.path().join("empty.txt");
    fs::write(&empty_file, "")?;
    assert!(!image_processing::caption_file_exists_and_not_empty(&empty_file).await);

    // Test file with only whitespace
    let whitespace_file = temp_dir.path().join("whitespace.txt");
    fs::write(&whitespace_file, "   \n  \t  ")?;
    assert!(!image_processing::caption_file_exists_and_not_empty(&whitespace_file).await);

    // Test file with content
    let content_file = temp_dir.path().join("content.txt");
    fs::write(&content_file, "This is a caption")?;
    assert!(image_processing::caption_file_exists_and_not_empty(&content_file).await);

    Ok(())
}

#[tokio::test]
async fn test_rename_file_without_image_extension() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Test .jpg extension
    let jpg_file = temp_dir.path().join("test.jpg");
    fs::write(&jpg_file, "dummy data")?;
    image_processing::rename_file_without_image_extension(&jpg_file).await?;
    assert!(!jpg_file.exists());
    assert!(temp_dir.path().join("test").exists());

    // Test .jpeg extension
    let jpeg_file = temp_dir.path().join("test2.jpeg");
    fs::write(&jpeg_file, "dummy data")?;
    image_processing::rename_file_without_image_extension(&jpeg_file).await?;
    assert!(!jpeg_file.exists());
    assert!(temp_dir.path().join("test2").exists());

    // Test .png extension
    let png_file = temp_dir.path().join("test3.png");
    fs::write(&png_file, "dummy data")?;
    image_processing::rename_file_without_image_extension(&png_file).await?;
    assert!(!png_file.exists());
    assert!(temp_dir.path().join("test3").exists());

    // Test non-image extension
    let txt_file = temp_dir.path().join("test4.txt");
    fs::write(&txt_file, "dummy data")?;
    image_processing::rename_file_without_image_extension(&txt_file).await?;
    assert!(txt_file.exists()); // Should not be renamed

    Ok(())
}
