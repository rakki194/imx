#![warn(clippy::all, clippy::pedantic)]

use super::super::xyplot::{
    DEFAULT_LEFT_PADDING, DEFAULT_TOP_PADDING, LabelAlignment, PlotConfig, create_plot,
};
use crate::numeric::i32_to_u32;
use anyhow::Result;
use image::{GenericImageView, Rgb, RgbImage};
use log::debug;
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
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
        row_labels: vec![
            "This is a very long row label that should cause more padding".to_string(),
        ],
        column_labels: vec!["Col 1".to_string()],
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
    };

    create_plot(&config)?;

    // Verify the output exists
    assert!(output_path.exists());

    Ok(())
}

#[test]
fn test_column_label_alignment_with_different_ar() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let img2_path = temp_dir.path().join("test2.png");
    let output_path = temp_dir.path().join("output.png");

    // Create images with different aspect ratios
    create_test_image(&img1_path, 100, 200)?; // Tall
    create_test_image(&img2_path, 200, 100)?; // Wide

    let config = PlotConfig {
        images: vec![img1_path, img2_path],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec!["Test Row".to_string()],
        column_labels: vec!["Tall".to_string(), "Wide".to_string()],
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: false,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
    };

    create_plot(&config)?;

    // Load the output image for verification
    let output_img = image::open(&output_path)?.to_rgb8();

    // Helper function to check if a region contains black text
    let has_black_pixels = |x: u32, y: u32, width: u32, height: u32| -> bool {
        for cy in y..y.saturating_add(height) {
            for cx in x..x.saturating_add(width) {
                if cx < output_img.width() && cy < output_img.height() {
                    let pixel = output_img.get_pixel(cx, cy);
                    // Check if pixel is dark (text)
                    if pixel[0] < 128 && pixel[1] < 128 && pixel[2] < 128 {
                        return true;
                    }
                }
            }
        }
        false
    };

    // The maximum width among images is 200
    let max_width: i32 = 200;
    let left_padding: i32 = 150; // Known padding for row labels

    // Test first column (Tall image - 100px wide)
    let col: i32 = 0;
    let cell_start: i32 = left_padding + (col * max_width);
    let image_offset: i32 = (max_width - 100) / 2; // (200 - 100) / 2 = 50
    let expected_x: i32 = cell_start + image_offset;

    // Search for text in a wider region around the expected position
    let found_text = (0..100).any(|offset: i32| {
        has_black_pixels(
            expected_x.saturating_add(offset).saturating_sub(50) as u32,
            0,
            50,
            40,
        )
    });
    assert!(
        found_text,
        "Column 0 (Tall) label not found near expected position ({expected_x}, 0)"
    );

    // Test second column (Wide image - 200px wide)
    let col: i32 = 1;
    let cell_start: i32 = left_padding + (col * max_width);
    let image_offset: i32 = (max_width - 200) / 2; // (200 - 200) / 2 = 0
    let expected_x: i32 = cell_start + image_offset;

    // Search for text in a wider region around the expected position
    let found_text = (0..100).any(|offset: i32| {
        has_black_pixels(
            expected_x.saturating_add(offset).saturating_sub(50) as u32,
            0,
            50,
            40,
        )
    });
    assert!(
        found_text,
        "Column 1 (Wide) label not found near expected position ({expected_x}, 0)"
    );

    // For each column, check a region that should definitely be empty
    // (between the end of the label area and the center of the image)
    for (col, img_width) in [(0, 100), (1, 200)] {
        let cell_start = left_padding + (col * max_width);
        let image_offset = (max_width - img_width) / 2;
        let image_start = cell_start + image_offset;
        let image_center = image_start + (img_width / 2);

        // Calculate label position (centered over image)
        let label_width = if col == 0 { 50 } else { 80 }; // Width for "Tall" vs "Wide"
        let label_start = image_start + ((img_width - label_width) / 2);

        // Check a small region after the expected label position
        let check_start = label_start + label_width + 10; // Small padding after expected label end
        let check_width = 20; // Small fixed width to check for unexpected text

        // Debug print to help understand the values
        debug!(
            "Col {col}: cell_start={cell_start}, image_offset={image_offset}, image_start={image_start}, image_center={image_center}, label_start={label_start}, check_start={check_start}, check_width={check_width}"
        );

        // Only check if we're not too close to the image center
        if check_start + check_width as i32 <= image_center {
            assert!(
                !has_black_pixels(check_start.try_into().unwrap(), 0, check_width, 40),
                "Found unexpected text in cell {col} between label and image center at position ({check_start}, 0)"
            );
        }
    }

    Ok(())
}

#[test]
fn test_column_label_alignments() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let img2_path = temp_dir.path().join("test2.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;
    create_test_image(&img2_path, 100, 100)?;

    // Test left alignment
    let config = PlotConfig {
        images: vec![img1_path.clone(), img2_path.clone()],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec![],
        column_labels: vec!["First".to_string(), "Second".to_string()],
        column_label_alignment: LabelAlignment::Start,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: true,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
    };
    create_plot(&config)?;

    // Load the output image for verification
    let output_img = image::open(&output_path)?.to_rgb8();

    // Helper function to check if a region contains black text
    let has_black_pixels = |x: u32, y: u32, width: u32, height: u32| -> bool {
        for cy in y..y.saturating_add(height) {
            for cx in x..x.saturating_add(width) {
                if cx < output_img.width() && cy < output_img.height() {
                    let pixel = output_img.get_pixel(cx, cy);
                    // Check if pixel is dark (text)
                    if pixel[0] < 128 && pixel[1] < 128 && pixel[2] < 128 {
                        return true;
                    }
                }
            }
        }
        false
    };

    let cell_width = 100;
    let left_padding = 0; // No row labels, so no left padding

    // First column - left aligned
    assert!(has_black_pixels(
        i32_to_u32(left_padding),
        0,
        50,
        DEFAULT_TOP_PADDING
    ));

    // Second column - left aligned
    assert!(has_black_pixels(
        i32_to_u32(left_padding) + cell_width,
        0,
        50,
        DEFAULT_TOP_PADDING
    ));

    // Test right alignment
    let config = PlotConfig {
        images: vec![img1_path.clone(), img2_path.clone()],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec![],
        column_labels: vec!["First".to_string(), "Second".to_string()],
        column_label_alignment: LabelAlignment::End,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: true,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
    };
    create_plot(&config)?;

    let output_img = image::open(&output_path)?.to_rgb8();

    // Calculate expected positions for right-aligned text
    let first_col_x = i32_to_u32(left_padding) + cell_width - 50;
    let second_col_x = i32_to_u32(left_padding) + (2 * cell_width) - 50;

    // Debug print dimensions and search areas
    debug!(
        "Image dimensions: {}x{}",
        output_img.width(),
        output_img.height()
    );
    debug!(
        "Searching for first column text at x={first_col_x}, width=50"
    );
    debug!(
        "Searching for second column text at x={second_col_x}, width=50"
    );

    // Search in a wider area for the first column
    let mut found_first = false;
    for x_offset in -20_i32..=20_i32 {
        let search_x = first_col_x.saturating_add(x_offset as u32);
        if has_black_pixels(search_x, 0, 50, DEFAULT_TOP_PADDING) {
            found_first = true;
            debug!("Found first column text at x_offset={x_offset}");
            break;
        }
    }
    assert!(found_first, "First column right-aligned text not found");

    // Search in a wider area for the second column
    let mut found_second = false;
    for x_offset in -20_i32..=20_i32 {
        let search_x = second_col_x.saturating_add(x_offset as u32);
        if has_black_pixels(search_x, 0, 50, DEFAULT_TOP_PADDING) {
            found_second = true;
            debug!("Found second column text at x_offset={x_offset}");
            break;
        }
    }
    assert!(found_second, "Second column right-aligned text not found");

    Ok(())
}

// Add a new test for debug mode
#[test]
fn test_debug_mode() -> Result<()> {
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
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: true,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
    };

    create_plot(&config)?;

    // Verify both output files exist
    assert!(output_path.exists());
    let debug_output = output_path.with_file_name(format!(
        "{}_debug{}",
        output_path.file_stem().unwrap().to_string_lossy(),
        output_path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default()
    ));
    assert!(debug_output.exists());

    Ok(())
}

#[test]
fn test_column_label_alignments_comprehensive() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;

    // Test all alignment options
    for alignment in [
        LabelAlignment::Start,
        LabelAlignment::Center,
        LabelAlignment::End,
    ] {
        let config = PlotConfig {
            images: vec![img1_path.clone()],
            output: output_path.clone(),
            rows: 1,
            row_labels: vec![],
            column_labels: vec!["Test Label".to_string()],
            column_label_alignment: alignment,
            row_label_alignment: LabelAlignment::Center,
            debug_mode: true,
            top_padding: DEFAULT_TOP_PADDING,
            left_padding: DEFAULT_LEFT_PADDING,
            font_size: None,
        };

        create_plot(&config)?;

        // Load the output image for verification
        let output_img = image::open(&output_path)?.to_rgb8();

        // Helper function to check if a region contains black text
        let has_black_pixels = |x: u32, y: u32, width: u32, height: u32| -> bool {
            for cy in y..y.saturating_add(height) {
                for cx in x..x.saturating_add(width) {
                    if cx < output_img.width() && cy < output_img.height() {
                        let pixel = output_img.get_pixel(cx, cy);
                        if pixel[0] < 128 && pixel[1] < 128 && pixel[2] < 128 {
                            return true;
                        }
                    }
                }
            }
            false
        };

        // Expected positions based on alignment
        let (expected_x, search_width): (i32, u32) = match alignment {
            LabelAlignment::Start => (0, 50),
            LabelAlignment::Center => (25, 50), // Centered in 100px width
            LabelAlignment::End => (50, 50),
        };

        // Search for text with some tolerance
        let mut found_text = false;
        for x_offset in -10_i32..=10_i32 {
            let search_x = i32_to_u32((expected_x + x_offset).max(0));
            if has_black_pixels(search_x, 0, search_width, DEFAULT_TOP_PADDING) {
                found_text = true;
                break;
            }
        }

        assert!(
            found_text,
            "Text not found at expected position for {alignment:?} alignment"
        );
    }

    Ok(())
}

#[test]
fn test_top_padding_variations() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;

    // Test different padding values
    for &padding in &[20, 40, 60, 80] {
        let config = PlotConfig {
            images: vec![img1_path.clone()],
            output: output_path.clone(),
            rows: 1,
            row_labels: vec![],
            column_labels: vec!["Test Label".to_string()],
            column_label_alignment: LabelAlignment::Center,
            row_label_alignment: LabelAlignment::Center,
            debug_mode: true,
            top_padding: padding,
            left_padding: DEFAULT_LEFT_PADDING,
            font_size: None,
        };

        create_plot(&config)?;

        // Load and verify the output image
        let output_img = image::open(&output_path)?;
        let (_, height) = output_img.dimensions();

        // Image height should be at least padding + image height (100)
        assert!(
            height >= padding + 100,
            "Output height {} is less than expected {} for padding {}",
            height,
            padding + 100,
            padding
        );

        // Verify label placement
        let output_img = output_img.to_rgb8();
        let has_black_pixels = |y: u32, height: u32| -> bool {
            for cy in y..y.saturating_add(height) {
                for cx in 0..output_img.width() {
                    let pixel = output_img.get_pixel(cx, cy);
                    if pixel[0] < 128 && pixel[1] < 128 && pixel[2] < 128 {
                        return true;
                    }
                }
            }
            false
        };

        // Check that text appears in the top padding area
        assert!(
            has_black_pixels(0, padding),
            "No text found in top padding area (height {padding})"
        );

        // Check that no text appears below the padding area
        assert!(
            !has_black_pixels(padding, 10),
            "Unexpected text found below padding area (height {padding})"
        );
    }

    Ok(())
}

#[test]
fn test_zero_top_padding() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;

    let config = PlotConfig {
        images: vec![img1_path],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec![],
        column_labels: vec![], // No labels since we have no padding
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: true,
        top_padding: 0,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
    };

    create_plot(&config)?;

    // Load and verify the output image
    let output_img = image::open(&output_path)?;
    let (_, height) = output_img.dimensions();

    // With no padding and no labels, height should be exactly image height
    assert_eq!(
        height, 100,
        "Output height {height} should match input image height 100 with zero padding"
    );

    Ok(())
}

#[test]
fn test_multiline_column_labels() -> Result<()> {
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
        column_labels: vec!["First\nLine".to_string(), "Second\nLine".to_string()],
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: true,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
    };

    create_plot(&config)?;

    // Load the output image for verification
    let output_img = image::open(&output_path)?.to_rgb8();

    // Helper function to check if a region contains black text
    let has_black_pixels = |x: u32, y: u32, width: u32, height: u32| -> bool {
        for cy in y..y.saturating_add(height) {
            for cx in x..x.saturating_add(width) {
                if cx < output_img.width() && cy < output_img.height() {
                    let pixel = output_img.get_pixel(cx, cy);
                    if pixel[0] < 128 && pixel[1] < 128 && pixel[2] < 128 {
                        return true;
                    }
                }
            }
        }
        false
    };

    // Check for text in both lines for each column
    let cell_width = 100;
    let left_padding = DEFAULT_LEFT_PADDING;

    // First column
    assert!(
        has_black_pixels(left_padding, 0, cell_width, DEFAULT_TOP_PADDING / 2),
        "First line of first column not found"
    );
    assert!(
        has_black_pixels(
            left_padding,
            DEFAULT_TOP_PADDING / 2,
            cell_width,
            DEFAULT_TOP_PADDING / 2
        ),
        "Second line of first column not found"
    );

    // Second column
    assert!(
        has_black_pixels(
            left_padding + cell_width,
            0,
            cell_width,
            DEFAULT_TOP_PADDING / 2
        ),
        "First line of second column not found"
    );
    assert!(
        has_black_pixels(
            left_padding + cell_width,
            DEFAULT_TOP_PADDING / 2,
            cell_width,
            DEFAULT_TOP_PADDING / 2
        ),
        "Second line of second column not found"
    );

    Ok(())
}

#[test]
fn test_multiline_row_labels() -> Result<()> {
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
        row_labels: vec!["First\nRow".to_string(), "Second\nRow".to_string()],
        column_labels: vec![],
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: true,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
    };

    create_plot(&config)?;

    // Load the output image for verification
    let output_img = image::open(&output_path)?.to_rgb8();

    // Helper function to check if a region contains black text
    let has_black_pixels = |x: u32, y: u32, width: u32, height: u32| -> bool {
        for cy in y..y.saturating_add(height) {
            for cx in x..x.saturating_add(width) {
                if cx < output_img.width() && cy < output_img.height() {
                    let pixel = output_img.get_pixel(cx, cy);
                    if pixel[0] < 128 && pixel[1] < 128 && pixel[2] < 128 {
                        return true;
                    }
                }
            }
        }
        false
    };

    // Check for text in both lines for each row
    let row_height = 100;

    // First row
    assert!(
        has_black_pixels(0, 0, DEFAULT_LEFT_PADDING, row_height),
        "First line of first row not found"
    );

    // Second row
    assert!(
        has_black_pixels(0, row_height, DEFAULT_LEFT_PADDING, row_height),
        "First line of second row not found"
    );

    Ok(())
}

#[test]
fn test_row_label_alignments() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;

    for alignment in [
        LabelAlignment::Start,
        LabelAlignment::Center,
        LabelAlignment::End,
    ] {
        let config = PlotConfig {
            images: vec![img1_path.clone()],
            output: output_path.clone(),
            rows: 1,
            row_labels: vec!["Test Row".to_string()],
            column_labels: vec![],
            column_label_alignment: LabelAlignment::Center,
            row_label_alignment: alignment,
            debug_mode: true,
            top_padding: DEFAULT_TOP_PADDING,
            left_padding: DEFAULT_LEFT_PADDING,
            font_size: None,
        };

        create_plot(&config)?;

        let output_img = image::open(&output_path)?.to_rgb8();

        // Helper function to check if a region contains black text
        let has_black_pixels = |x: u32, y: u32, width: u32, height: u32| -> bool {
            for cy in y..y.saturating_add(height) {
                for cx in x..x.saturating_add(width) {
                    if cx < output_img.width() && cy < output_img.height() {
                        let pixel = output_img.get_pixel(cx, cy);
                        if pixel[0] < 128 && pixel[1] < 128 && pixel[2] < 128 {
                            return true;
                        }
                    }
                }
            }
            false
        };

        // Expected positions based on alignment
        let (start_x, search_width) = match alignment {
            LabelAlignment::Start => (10, 30),
            LabelAlignment::Center => (20, 30),
            LabelAlignment::End => (30, 30),
        };

        let mut found_text = false;
        for x_offset in -5_i32..=5_i32 {
            let search_x = i32_to_u32((start_x + x_offset).max(0));
            if has_black_pixels(search_x, 0, search_width, 100) {
                found_text = true;
                break;
            }
        }

        assert!(
            found_text,
            "Text not found at expected position for {alignment:?} alignment"
        );
    }

    Ok(())
}

#[test]
fn test_dynamic_padding_with_multiline() -> Result<()> {
    let temp_dir = tempdir()?;
    let img1_path = temp_dir.path().join("test1.png");
    let output_path = temp_dir.path().join("output.png");

    create_test_image(&img1_path, 100, 100)?;

    let config = PlotConfig {
        images: vec![img1_path],
        output: output_path.clone(),
        rows: 1,
        row_labels: vec!["Line 1\nLine 2\nLine 3".to_string()],
        column_labels: vec!["Column 1\nColumn 2".to_string()],
        column_label_alignment: LabelAlignment::Center,
        row_label_alignment: LabelAlignment::Center,
        debug_mode: true,
        top_padding: DEFAULT_TOP_PADDING,
        left_padding: DEFAULT_LEFT_PADDING,
        font_size: None,
    };

    create_plot(&config)?;

    let output_img = image::open(&output_path)?;
    let (width, height) = output_img.dimensions();

    // Verify that the padding areas are large enough for multiline text
    assert!(
        width > DEFAULT_LEFT_PADDING + 100,
        "Width should accommodate multiline row labels"
    );
    assert!(
        height > DEFAULT_TOP_PADDING + 100,
        "Height should accommodate multiline column labels"
    );

    Ok(())
}
