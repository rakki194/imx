use super::super::xyplot::{PlotConfig, create_plot};
use anyhow::Result;
use image::{Rgb, RgbImage};
use tempfile::tempdir;

fn create_test_image(path: &std::path::Path, width: u32, height: u32) -> Result<()> {
    let mut img = RgbImage::new(width, height);
    // Fill with a test pattern
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = Rgb([((x * 255) / width) as u8, ((y * 255) / height) as u8, 128u8]);
    }
    img.save(path)?;
    Ok(())
}

#[test]
fn test_basic_plot() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let img2_path = temp_dir.path().join("test2.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;
    create_test_image(&img2_path, 100, 100)?;

    let config = PlotConfig {
        images: vec![img1_path, img2_path],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec![],
        column_labels: vec![],
    };

    create_plot(&config)?;
    assert!(output_path.exists());
    Ok(())
}

#[test]
fn test_with_labels() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let img2_path = temp_dir.path().join("test2.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;
    create_test_image(&img2_path, 100, 100)?;

    let config = PlotConfig {
        images: vec![img1_path, img2_path],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec!["Row 1".to_string()],
        column_labels: vec!["Col 1".to_string(), "Col 2".to_string()],
    };

    create_plot(&config)?;
    assert!(output_path.exists());
    Ok(())
}

#[test]
fn test_with_row_and_column_labels() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let img2_path = temp_dir.path().join("test2.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;
    create_test_image(&img2_path, 100, 100)?;

    let config = PlotConfig {
        images: vec![img1_path, img2_path],
        output: output_path.clone(),
        rows: 2,
        row_labels: vec!["Row 1".to_string(), "Row 2".to_string()],
        column_labels: vec!["Col 1".to_string()],
    };

    create_plot(&config)?;
    assert!(output_path.exists());
    Ok(())
}

#[test]
fn test_different_image_sizes() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let img2_path = temp_dir.path().join("test2.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 200, 150)?;
    create_test_image(&img2_path, 100, 100)?;

    let config = PlotConfig {
        images: vec![img1_path, img2_path],
        output: output_path.clone(),
        rows: 2,
        row_labels: vec![],
        column_labels: vec![],
    };

    create_plot(&config)?;
    assert!(output_path.exists());
    Ok(())
}

#[test]
fn test_single_image() -> Result<()> {
    let temp_dir = tempdir()?;
    let img_path = temp_dir.path().join("test1.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img_path, 100, 100)?;

    let config = PlotConfig {
        images: vec![img_path],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec![],
        column_labels: vec![],
    };

    create_plot(&config)?;
    assert!(output_path.exists());
    Ok(())
}

#[test]
fn test_many_images() -> Result<()> {
    let temp_dir = tempdir()?;
    let output_path = temp_dir.path().join("output.png");
    let mut image_paths = Vec::new();

    // Create 9 test images
    for i in 0..9 {
        let img_path = temp_dir.path().join(format!("test{i}.png"));
        create_test_image(&img_path, 100, 100)?;
        image_paths.push(img_path);
    }

    let config = PlotConfig {
        images: image_paths,
        output: output_path.clone(),
        rows: 3,
        row_labels: vec![
            "Top".to_string(),
            "Middle".to_string(),
            "Bottom".to_string(),
        ],
        column_labels: vec![
            "Left".to_string(),
            "Center".to_string(),
            "Right".to_string(),
        ],
    };

    create_plot(&config)?;
    assert!(output_path.exists());
    Ok(())
}

#[test]
#[should_panic(expected = "Number of row labels (2) should match the number of rows (1)")]
fn test_mismatched_row_labels() {
    let temp_dir = tempdir().unwrap();
    let img1_path = temp_dir.path().join("test1.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100).unwrap();

    let config = PlotConfig {
        images: vec![img1_path],
        output: output_path,
        rows: 1,
        row_labels: vec!["Row 1".to_string(), "Row 2".to_string()],
        column_labels: vec![],
    };

    create_plot(&config).unwrap();
}

#[test]
#[should_panic(expected = "Number of column labels (2) should match the number of columns (1)")]
fn test_mismatched_column_labels() {
    let temp_dir = tempdir().unwrap();
    let img1_path = temp_dir.path().join("test1.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100).unwrap();

    let config = PlotConfig {
        images: vec![img1_path],
        output: output_path,
        rows: 1,
        row_labels: vec![],
        column_labels: vec!["Col 1".to_string(), "Col 2".to_string()],
    };

    create_plot(&config).unwrap();
}

#[test]
fn test_empty_labels() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let img2_path = temp_dir.path().join("test2.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;
    create_test_image(&img2_path, 100, 100)?;

    let config = PlotConfig {
        images: vec![img1_path, img2_path],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec![],
        column_labels: vec![],
    };

    create_plot(&config)?;
    assert!(output_path.exists());
    Ok(())
}

#[test]
fn test_dynamic_padding() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;

    // Test with a very long row label
    let config = PlotConfig {
        images: vec![img1_path],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec!["This is a very long row label that should cause more padding".to_string()],
        column_labels: vec!["Col 1".to_string()],
    };

    create_plot(&config)?;
    
    // Verify the output exists and has correct dimensions
    let output_img = image::open(&output_path)?;
    let (width, _) = output_img.dimensions();
    assert!(width > 500); // Should have significant padding for the long label
    
    Ok(())
}

#[test]
fn test_different_size_images() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let img2_path = temp_dir.path().join("test2.png");
    let img3_path = temp_dir.path().join("test3.png");
    let output_path = temp_dir.path().join("output.png");

    // Create images with different dimensions
    create_test_image(&img1_path, 100, 100)?;
    create_test_image(&img2_path, 200, 150)?;
    create_test_image(&img3_path, 150, 200)?;

    let config = PlotConfig {
        images: vec![img1_path, img2_path, img3_path],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec!["Row 1".to_string()],
        column_labels: vec!["Small".to_string(), "Wide".to_string(), "Tall".to_string()],
    };

    create_plot(&config)?;
    
    // Verify the output exists and has correct dimensions
    let output_img = image::open(&output_path)?;
    let (width, height) = output_img.dimensions();
    
    // The height should accommodate the tallest image (200) plus padding
    assert!(height > 200);
    // The width should accommodate 3 images of width 200 (the widest) plus padding
    assert!(width > 600);
    
    Ok(())
}

#[test]
fn test_label_alignment() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let img2_path = temp_dir.path().join("test2.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;
    create_test_image(&img2_path, 100, 100)?;

    let config = PlotConfig {
        images: vec![img1_path, img2_path],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec!["Test Row".to_string()],
        column_labels: vec!["First".to_string(), "Second".to_string()],
    };

    create_plot(&config)?;
    
    // Verify the output exists
    assert!(output_path.exists());
    
    Ok(())
}
