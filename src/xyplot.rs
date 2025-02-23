//! XY plotting module for creating image grids with labels.
//!
//! This module provides functionality for creating plots of images arranged in a grid layout
//! with optional row and column labels. Features include:
//!
//! - Flexible grid layout with customizable rows and columns
//! - Support for row and column labels
//! - Unicode text rendering with emoji support
//! - Automatic image scaling and alignment
//! - White background with configurable text colors
//!
//! The module uses the `ab_glyph` library for text rendering and supports both regular text
//! (using DejaVu Sans) and emoji (using Noto Color Emoji).
//!
//! # Examples
//!
//! ```rust,no_run
//! use std::path::PathBuf;
//! use anyhow::Result;
//! use imx::xyplot::{PlotConfig, create_plot};
//!
//! async fn example() -> Result<()> {
//!     let config = PlotConfig {
//!         images: vec![
//!             PathBuf::from("image1.png"),
//!             PathBuf::from("image2.png"),
//!         ],
//!         output: PathBuf::from("output.png"),
//!         rows: 1,
//!         row_labels: vec!["Row 1".to_string()],
//!         column_labels: vec!["Col 1".to_string(), "Col 2".to_string()],
//!     };
//!
//!     create_plot(&config)?;
//!     Ok(())
//! }
//! ```

#![warn(clippy::all, clippy::pedantic)]

use crate::numeric::{f32_to_i32, f32_to_u8, i32_to_f32_for_pos, i32_to_u32, u32_to_i32};
use ab_glyph::{Font, FontRef, GlyphId, Point, PxScale, ScaleFont};
use anyhow::{Context, Result};
use image::{Rgb, RgbImage};
use rgb::FromSlice;
use std::path::PathBuf;

// Constants for layout
/// Space reserved at the top of the plot for labels and padding
const TOP_PADDING: u32 = 40;

/// A pair of fonts for rendering text and emoji characters.
///
/// This struct manages two fonts:
/// - A main font (`DejaVu` Sans) for regular text
/// - An emoji font (Noto Color Emoji) for emoji characters
///
/// The struct automatically selects the appropriate font for each character
/// based on glyph availability.
#[derive(Clone, Copy)]
struct FontPair<'a> {
    /// Main font for regular text (`DejaVu` Sans)
    main: &'a FontRef<'a>,
    /// Emoji font for emoji characters (Noto Color Emoji)
    emoji: &'a FontRef<'a>,
}

impl<'a> FontPair<'a> {
    /// Gets the appropriate glyph ID and font for a character.
    ///
    /// This method attempts to use the main font first, falling back to
    /// the emoji font if the character is not available in the main font.
    ///
    /// # Arguments
    ///
    /// * `c` - The character to get the glyph for
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - The glyph ID for the character
    /// - A reference to the font that contains the glyph
    fn glyph_id(&self, c: char) -> (GlyphId, &'a FontRef<'a>) {
        let main_id = self.main.glyph_id(c);
        // Check if the main font has a real glyph for this char (not a .notdef glyph)
        if self.main.outline(main_id).is_some() {
            (main_id, self.main)
        } else {
            let emoji_id = self.emoji.glyph_id(c);
            (emoji_id, self.emoji)
        }
    }
}

/// Draws text on an image with support for regular characters and emoji.
///
/// This function handles text rendering with the following features:
/// - Mixed regular text and emoji support
/// - Anti-aliasing with alpha blending
/// - Color emoji rendering
/// - Proper text positioning and scaling
///
/// # Arguments
///
/// * `canvas` - The image to draw on
/// * `text` - The text to draw
/// * `x` - The x-coordinate for text placement
/// * `y` - The y-coordinate for text placement
/// * `scale` - The font size scale factor
/// * `fonts` - The font pair to use for rendering
/// * `color` - The color to use for regular text (emoji use their own colors)
fn draw_text(
    canvas: &mut RgbImage,
    text: &str,
    x: i32,
    y: i32,
    scale: f32,
    fonts: FontPair,
    color: Rgb<u8>,
) {
    let px_scale = PxScale::from(scale);

    // Layout the glyphs in a line with 20 pixels padding
    let mut glyphs = Vec::new();
    let mut cursor = Point {
        x: i32_to_f32_for_pos(x),
        y: i32_to_f32_for_pos(y),
    };

    // First pass: calculate positions and collect glyphs
    for c in text.chars() {
        let (id, font) = fonts.glyph_id(c);
        let scaled_font = font.as_scaled(px_scale);
        // Create a glyph with scale and position
        let glyph = id.with_scale_and_position(px_scale, cursor);
        cursor.x += scaled_font.h_advance(id);
        glyphs.push((glyph, font));
    }

    // Second pass: render glyphs
    for (glyph, font) in glyphs {
        let scaled_font = font.as_scaled(px_scale);
        let glyph_position = glyph.position;
        let glyph_id = glyph.id;

        if let Some(outlined) = scaled_font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|x, y, coverage| {
                let alpha = f32_to_u8(coverage * 255.0);
                if alpha == 0 {
                    return;
                }

                #[allow(clippy::cast_precision_loss)]
                let px = i32_to_u32(f32_to_i32((x as f32) + bounds.min.x));
                #[allow(clippy::cast_precision_loss)]
                let py = i32_to_u32(f32_to_i32((y as f32) + bounds.min.y));

                if px < canvas.width() && py < canvas.height() {
                    let pixel = canvas.get_pixel_mut(px, py);
                    let blend = |a: u8, b: u8, alpha: u8| -> u8 {
                        let a = f32::from(a);
                        let b = f32::from(b);
                        let alpha = f32::from(alpha) / 255.0;
                        f32_to_u8(a * (1.0 - alpha) + b * alpha)
                    };

                    pixel[0] = blend(pixel[0], color[0], alpha);
                    pixel[1] = blend(pixel[1], color[1], alpha);
                    pixel[2] = blend(pixel[2], color[2], alpha);
                }
            });
        }

        // Check for color emoji image
        if let Some(img) = font.glyph_raster_image2(glyph_id, u16::MAX) {
            let img_width = u32::from(img.width);
            let scale_factor = scale / f32::from(img.pixels_per_em);

            let pixels: &[rgb::RGB8] = img.data.as_rgb();
            for (img_y, row) in pixels.chunks(img_width as usize).enumerate() {
                for (img_x, pixel) in row.iter().enumerate() {
                    #[allow(clippy::cast_precision_loss)]
                    let src_x = img_x as f32 * scale_factor;
                    #[allow(clippy::cast_precision_loss)]
                    let src_y = img_y as f32 * scale_factor;

                    let canvas_x = i32_to_u32(f32_to_i32(
                        glyph_position.x + src_x + img.origin.x * scale_factor,
                    ));
                    let canvas_y = i32_to_u32(f32_to_i32(
                        glyph_position.y + src_y + img.origin.y * scale_factor,
                    ));

                    if canvas_x < canvas.width() && canvas_y < canvas.height() {
                        let canvas_pixel = canvas.get_pixel_mut(canvas_x, canvas_y);
                        canvas_pixel[0] = pixel.r;
                        canvas_pixel[1] = pixel.g;
                        canvas_pixel[2] = pixel.b;
                    }
                }
            }
        }
    }
}

/// Configuration for creating an image plot with labels.
///
/// This struct defines the layout and content of an image grid plot,
/// including the source images, output location, and optional labels.
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use imx::xyplot::PlotConfig;
///
/// let config = PlotConfig {
///     images: vec![PathBuf::from("image1.png")],
///     output: PathBuf::from("output.png"),
///     rows: 1,
///     row_labels: vec!["Row 1".to_string()],
///     column_labels: vec!["Col 1".to_string()],
/// };
/// ```
#[derive(Debug)]
pub struct PlotConfig {
    /// List of image file paths to include in the plot
    pub images: Vec<PathBuf>,
    /// Output file path where the plot will be saved
    pub output: PathBuf,
    /// Number of rows in the image grid
    pub rows: u32,
    /// Optional labels for each row (empty Vec for no labels)
    pub row_labels: Vec<String>,
    /// Optional labels for each column (empty Vec for no labels)
    pub column_labels: Vec<String>,
}

fn validate_plot_config(config: &PlotConfig) -> Result<u32> {
    let PlotConfig {
        images,
        rows,
        row_labels,
        column_labels,
        ..
    } = config;

    if !row_labels.is_empty() && row_labels.len() != *rows as usize {
        anyhow::bail!(
            "Number of row labels ({}) should match the number of rows ({})",
            row_labels.len(),
            rows
        );
    }

    let cols = u32::try_from(images.len())
        .map_err(|_| anyhow::anyhow!("Too many images"))?
        .div_ceil(*rows);

    if !column_labels.is_empty() && column_labels.len() != cols as usize {
        anyhow::bail!(
            "Number of column labels ({}) should match the number of columns ({})",
            column_labels.len(),
            cols
        );
    }

    Ok(cols)
}

fn load_fonts() -> Result<FontPair<'static>> {
    let font_data = include_bytes!("../assets/DejaVuSans.ttf");
    let main_font = FontRef::try_from_slice(font_data).context("Failed to load main font")?;

    let emoji_font_data = include_bytes!("../assets/NotoColorEmoji.ttf");
    let emoji_font = FontRef::try_from_slice(emoji_font_data).context("Failed to load emoji font")?;

    Ok(FontPair {
        main: &main_font,
        emoji: &emoji_font,
    })
}

fn calculate_left_padding(row_labels: &[String], fonts: FontPair) -> i32 {
    if row_labels.iter().any(|l| !l.is_empty()) {
        let max_label_width = row_labels
            .iter()
            .map(|label| {
                let mut width = 0.0;
                for c in label.chars() {
                    let (id, font) = fonts.glyph_id(c);
                    let scaled_font = font.as_scaled(PxScale::from(24.0));
                    width += scaled_font.h_advance(id);
                }
                width
            })
            .fold(0.0, f32::max);

        f32_to_i32(max_label_width + 40.0)
    } else {
        0
    }
}

fn find_max_dimensions(images: &[PathBuf]) -> Result<(u32, u32)> {
    let mut max_width = 0;
    let mut max_height = 0;
    for img_path in images {
        let img = image::open(img_path)
            .with_context(|| format!("Failed to open image: {img_path:?}"))?
            .to_rgb8();
        let (width, height) = img.dimensions();
        max_width = max_width.max(width);
        max_height = max_height.max(height);
    }
    Ok((max_width, max_height))
}

fn draw_column_labels(
    canvas: &mut RgbImage,
    column_labels: &[String],
    max_width: u32,
    left_padding: i32,
    fonts: FontPair,
) {
    for (col, label) in column_labels.iter().enumerate() {
        let x = u32_to_i32(u32::try_from(col).unwrap_or(0) * max_width + i32_to_u32(left_padding));
        let y = u32_to_i32(TOP_PADDING / 2);
        draw_text(canvas, label, x, y, 24.0, fonts, Rgb([0, 0, 0]));
    }
}

fn place_image(
    canvas: &mut RgbImage,
    img_path: &Path,
    x_start: u32,
    y_start: u32,
    max_width: u32,
    max_height: u32,
) -> Result<()> {
    let img = image::open(img_path)
        .with_context(|| format!("Failed to open image: {img_path:?}"))?
        .to_rgb8();
    let (img_width, img_height) = img.dimensions();

    let x_offset = (max_width - img_width) / 2;
    let y_offset = (max_height - img_height) / 2;

    for (x, y, pixel) in img.enumerate_pixels() {
        let canvas_x = x_start + x_offset + x;
        let canvas_y = y_start + y_offset + y;
        if canvas_x < canvas.width() && canvas_y < canvas.height() {
            canvas.put_pixel(canvas_x, canvas_y, *pixel);
        }
    }
    Ok(())
}

/// Creates a plot of images arranged in a grid with optional labels.
///
/// This function creates a new image containing a grid of the input images
/// with optional row and column labels. The layout is determined by the
/// specified number of rows, with columns calculated automatically based
/// on the number of input images.
///
/// Features:
/// - Automatic grid layout calculation
/// - Optional row and column labels
/// - White background
/// - Unicode text support with emoji
/// - Automatic image spacing and alignment
///
/// # Arguments
///
/// * `config` - Configuration struct specifying the plot layout and content
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// * The number of row labels doesn't match the number of rows
/// * The number of column labels doesn't match the calculated number of columns
/// * Any input image file cannot be opened or is invalid
/// * The fonts cannot be loaded
/// * The output file cannot be written
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use anyhow::Result;
/// use imx::xyplot::{PlotConfig, create_plot};
///
/// fn example() -> Result<()> {
///     let config = PlotConfig {
///         images: vec![PathBuf::from("image1.png")],
///         output: PathBuf::from("output.png"),
///         rows: 1,
///         row_labels: vec!["Row 1".to_string()],
///         column_labels: vec!["Col 1".to_string()],
///     };
///
///     create_plot(&config)?;
///     Ok(())
/// }
/// ```
pub fn create_plot(config: &PlotConfig) -> Result<()> {
    let cols = validate_plot_config(config)?;
    let fonts = load_fonts()?;
    let left_padding = calculate_left_padding(&config.row_labels, fonts);
    let (max_width, max_height) = find_max_dimensions(&config.images)?;

    let has_labels = !config.row_labels.is_empty() || !config.column_labels.is_empty();
    let row_height = max_height + if has_labels { TOP_PADDING } else { 0 };
    let canvas_height = row_height * config.rows + if has_labels { TOP_PADDING } else { 0 };
    let canvas_width = max_width * cols + i32_to_u32(left_padding);

    let mut canvas = RgbImage::new(canvas_width, canvas_height);
    for pixel in canvas.pixels_mut() {
        *pixel = Rgb([255, 255, 255]);
    }

    if !config.column_labels.is_empty() {
        draw_column_labels(&mut canvas, &config.column_labels, max_width, left_padding, fonts);
    }

    for (i, img_path) in config.images.iter().enumerate() {
        let i = u32::try_from(i)?;
        let row = i / cols;
        let col = i % cols;

        let x_start = col * max_width + i32_to_u32(left_padding);
        let y_start = row * row_height + TOP_PADDING;

        if let Some(row_label) = config.row_labels.get(row as usize) {
            let x = 20;
            let y = u32_to_i32(y_start + max_height / 2);
            draw_text(&mut canvas, row_label, x, y, 24.0, fonts, Rgb([0, 0, 0]));
        }

        place_image(&mut canvas, img_path, x_start, y_start, max_width, max_height)?;
    }

    canvas
        .save(&config.output)
        .with_context(|| format!("Failed to save output image: {:?}", config.output))?;

    Ok(())
}
