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
