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
//! use imx::xyplot::{PlotConfig, create_plot, LabelAlignment};
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
//!         column_label_alignment: LabelAlignment::Center,
//!         row_label_alignment: LabelAlignment::Center,
//!         debug_mode: false,
//!         top_padding: 40,
//!         left_padding: 40,
//!     };
//!
//!     create_plot(&config)?;
//!     Ok(())
//! }
//! ```

#![warn(clippy::all, clippy::pedantic)]

use crate::numeric::{f32_to_i32, f32_to_u32, f32_to_u8, i32_to_f32_for_pos, i32_to_u32, u32_to_i32};
use ab_glyph::{Font, FontRef, GlyphId, Point, PxScale, ScaleFont};
use anyhow::{Context, Result};
use image::{Rgb, RgbImage};
use rgb::FromSlice;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use crate::layout::{Layout, LayoutElement, LayoutRect};

/// Default space reserved at the top of the plot for labels and padding
pub const DEFAULT_TOP_PADDING: u32 = 40;

/// Default space reserved at the left of the plot for row labels
pub const DEFAULT_LEFT_PADDING: u32 = 40;

/// A pair of fonts for rendering text and emoji characters.
///
/// This struct manages two fonts:
/// - A main font (`DejaVu` Sans) for regular text
/// - An emoji font (Noto Color Emoji) for emoji characters
///
/// The struct automatically selects the appropriate font for each character
/// based on glyph availability.
#[derive(Clone, Copy)]
pub(crate) struct FontPair<'a> {
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

/// Alignment options for labels
#[derive(Debug, Clone, Copy)]
pub enum LabelAlignment {
    /// Place labels at the left/top edge
    Start,
    /// Center labels (default)
    Center,
    /// Place labels at the right/bottom edge
    End,
}

impl Default for LabelAlignment {
    fn default() -> Self {
        Self::Center
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
/// use imx::xyplot::{PlotConfig, LabelAlignment};
///
/// let config = PlotConfig {
///     images: vec![PathBuf::from("image1.png")],
///     output: PathBuf::from("output.png"),
///     rows: 1,
///     row_labels: vec!["Row 1".to_string()],
///     column_labels: vec!["Col 1".to_string()],
///     column_label_alignment: LabelAlignment::Center,
///     row_label_alignment: LabelAlignment::Center,
///     debug_mode: false,
///     top_padding: 40,
///     left_padding: 40,
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
    /// Supports multiline text using '\n' as separator
    pub row_labels: Vec<String>,
    /// Optional labels for each column (empty Vec for no labels)
    /// Supports multiline text using '\n' as separator
    pub column_labels: Vec<String>,
    /// Alignment of column labels relative to their images
    pub column_label_alignment: LabelAlignment,
    /// Alignment of row labels relative to their images
    pub row_label_alignment: LabelAlignment,
    /// Whether to output a debug visualization of the layout
    pub debug_mode: bool,
    /// Space reserved at the top of the plot for labels and padding
    pub top_padding: u32,
    /// Space reserved at the left of the plot for labels and padding
    pub left_padding: u32,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            images: Vec::new(),
            output: PathBuf::from("output.jpg"),
            rows: 1,
            row_labels: Vec::new(),
            column_labels: Vec::new(),
            column_label_alignment: LabelAlignment::default(),
            row_label_alignment: LabelAlignment::default(),
            debug_mode: false,
            top_padding: DEFAULT_TOP_PADDING,
            left_padding: DEFAULT_LEFT_PADDING,
        }
    }
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

/// Loads and initializes the fonts for text rendering.
///
/// This function loads both the main font (`DejaVu` Sans) and emoji font (Noto Color Emoji)
/// from embedded binary data. The fonts are stored as static data to ensure they live
/// for the entire program duration.
fn load_fonts() -> FontPair<'static> {
    // Define static font data
    static MAIN_FONT_DATA: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");
    static EMOJI_FONT_DATA: &[u8] = include_bytes!("../assets/NotoColorEmoji.ttf");

    // Create static fonts using lazy_static or once_cell pattern
    static MAIN_FONT: OnceLock<FontRef<'static>> = OnceLock::new();
    static EMOJI_FONT: OnceLock<FontRef<'static>> = OnceLock::new();

    // Initialize fonts if not already initialized
    let main_font = MAIN_FONT
        .get_or_init(|| FontRef::try_from_slice(MAIN_FONT_DATA).expect("Failed to load main font"));
    let emoji_font = EMOJI_FONT.get_or_init(|| {
        FontRef::try_from_slice(EMOJI_FONT_DATA).expect("Failed to load emoji font")
    });

    FontPair {
        main: main_font,
        emoji: emoji_font,
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

/// Calculate the width of a label string using the given fonts and scale.
/// This is used internally for both row and column label width calculations.
pub(crate) fn calculate_label_width(label: &str, fonts: FontPair, scale: f32) -> f32 {
    let mut width = 0.0;
    for c in label.chars() {
        let (id, font) = fonts.glyph_id(c);
        let scaled_font = font.as_scaled(PxScale::from(scale));
        width += scaled_font.h_advance(id);
    }
    width
}

/// Calculate the width and height needed for a multiline label
fn calculate_label_dimensions(label: &str, fonts: FontPair, scale: f32) -> (f32, u32) {
    let lines: Vec<&str> = label.split('\n').collect();
    let line_height = f32_to_u32(scale);
    let total_height = line_height * u32::try_from(lines.len()).unwrap_or(0);
    
    let max_width = lines.iter()
        .map(|line| calculate_label_width(line, fonts, scale))
        .fold(0.0, f32::max);
    
    (max_width, total_height)
}

fn draw_multiline_text(
    canvas: &mut RgbImage,
    text: &str,
    x: i32,
    y: i32,
    scale: f32,
    fonts: FontPair,
    color: Rgb<u8>,
) {
    let line_height = f32_to_i32(scale);
    for (i, line) in text.split('\n').enumerate() {
        let line_y = y + i32::try_from(i).unwrap_or(0) * line_height;
        draw_text(canvas, line, x, line_y, scale, fonts, color);
    }
}

#[allow(clippy::too_many_lines)]
fn calculate_layout(
    config: &PlotConfig,
    max_width: u32,
    max_height: u32,
    cols: u32,
) -> Layout {
    let has_labels = !config.row_labels.is_empty() || !config.column_labels.is_empty();
    let fonts = load_fonts();

    // Calculate maximum dimensions for labels
    let (max_row_label_width, _max_row_label_height) = if config.row_labels.is_empty() {
        (0.0f32, 0)
    } else {
        config.row_labels.iter()
            .map(|label| calculate_label_dimensions(label, fonts, 24.0))
            .fold((0.0f32, 0), |(w, h), (lw, lh)| (w.max(lw), h.max(lh)))
    };

    let (_max_col_label_width, max_col_label_height) = if config.column_labels.is_empty() {
        (0.0f32, 0)
    } else {
        config.column_labels.iter()
            .map(|label| calculate_label_dimensions(label, fonts, 24.0))
            .fold((0.0f32, 0), |(w, h), (lw, lh)| (w.max(lw), h.max(lh)))
    };

    // Calculate layout dimensions
    let left_padding = if config.row_labels.is_empty() {
        0
    } else {
        config.left_padding.max(i32_to_u32(f32_to_i32(max_row_label_width)) + 20)
    };

    let top_padding = if config.column_labels.is_empty() {
        0
    } else {
        config.top_padding.max(max_col_label_height + 20)
    };

    let row_height = max_height + if has_labels { top_padding } else { 0 };
    let canvas_height = row_height * config.rows + if has_labels { top_padding } else { 0 };
    let canvas_width = max_width * cols + left_padding;

    let mut layout = Layout::new(canvas_width, canvas_height);

    // Add padding elements
    if left_padding > 0 {
        layout.add_element(LayoutElement::Padding {
            rect: LayoutRect {
                x: 0,
                y: 0,
                width: left_padding,
                height: canvas_height,
            },
            description: "Left padding for row labels".to_string(),
        });
    }

    if has_labels {
        layout.add_element(LayoutElement::Padding {
            rect: LayoutRect {
                x: u32_to_i32(left_padding),
                y: 0,
                width: canvas_width - left_padding,
                height: top_padding,
            },
            description: "Top padding for column labels".to_string(),
        });
    }

    // Add column labels
    if !config.column_labels.is_empty() {
        for (col, (label, img_path)) in config.column_labels.iter().zip(config.images.iter()).enumerate() {
            let img = image::open(img_path).unwrap().to_rgb8();
            let img_width = img.width();
            let cell_start = u32::try_from(col).unwrap_or(0) * max_width + left_padding;
            let x_offset = (max_width - img_width) / 2;
            
            let (label_width, label_height) = calculate_label_dimensions(label, fonts, 24.0);
            let img_width_i32 = u32_to_i32(img_width);
            let label_width_i32 = f32_to_i32(label_width);
            
            let label_x = match config.column_label_alignment {
                LabelAlignment::Start => u32_to_i32(cell_start) + u32_to_i32(x_offset),
                LabelAlignment::Center => u32_to_i32(cell_start) + u32_to_i32(x_offset) + 
                    (img_width_i32 - label_width_i32) / 2,
                LabelAlignment::End => u32_to_i32(cell_start) + u32_to_i32(x_offset) + 
                    img_width_i32 - label_width_i32,
            };
            
            layout.add_element(LayoutElement::ColumnLabel {
                rect: LayoutRect {
                    x: label_x,
                    y: u32_to_i32(top_padding / 2 - label_height / 2),
                    width: i32_to_u32(label_width_i32),
                    height: label_height,
                },
                text: label.clone(),
            });
        }
    }

    // Add row labels and images
    for (i, img_path) in config.images.iter().enumerate() {
        let i = u32::try_from(i).unwrap_or(0);
        let row = i / cols;
        let col = i % cols;

        let x_start = col * max_width + left_padding;
        let y_start = row * row_height + top_padding;

        if let Some(row_label) = config.row_labels.get(row as usize) {
            let (label_width, label_height) = calculate_label_dimensions(row_label, fonts, 24.0);
            let available_width = left_padding - 20;
            let available_width_i32 = u32_to_i32(available_width);
            let label_width_i32 = f32_to_i32(label_width);

            let label_x = match config.row_label_alignment {
                LabelAlignment::Start => 10,
                LabelAlignment::Center => 10 + (available_width_i32 - label_width_i32) / 2,
                LabelAlignment::End => 10 + available_width_i32 - label_width_i32,
            };

            layout.add_element(LayoutElement::RowLabel {
                rect: LayoutRect {
                    x: label_x,
                    y: u32_to_i32(y_start + max_height / 2 - label_height / 2),
                    width: i32_to_u32(label_width_i32),
                    height: label_height,
                },
                text: row_label.clone(),
            });
        }

        let img = image::open(img_path).unwrap().to_rgb8();
        let (img_width, img_height) = img.dimensions();
        let x_offset = (max_width - img_width) / 2;
        let y_offset = (max_height - img_height) / 2;

        layout.add_element(LayoutElement::Image {
            rect: LayoutRect {
                x: u32_to_i32(x_start + x_offset),
                y: u32_to_i32(y_start + y_offset),
                width: img_width,
                height: img_height,
            },
            path: img_path.to_string_lossy().into_owned(),
        });
    }

    layout
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
/// - Optional debug mode for visualizing layout
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
/// This function will return an error if:
/// - The input images cannot be opened or read
/// - The output file cannot be created or written
/// - The number of row or column labels doesn't match the grid dimensions
///
/// # Panics
///
/// This function will panic if:
/// - The output path has no file stem
/// - The output path has an invalid extension
pub fn create_plot(config: &PlotConfig) -> Result<()> {
    let cols = validate_plot_config(config)?;
    let (max_width, max_height) = find_max_dimensions(&config.images)?;

    let layout = calculate_layout(config, max_width, max_height, cols);

    if config.debug_mode {
        let debug_output = config.output.with_file_name(format!(
            "{}_debug{}",
            config.output.file_stem().unwrap().to_string_lossy(),
            config.output.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default()
        ));
        layout.render_debug().save(&debug_output)
            .with_context(|| format!("Failed to save debug layout: {debug_output:?}"))?;
    }

    let mut canvas = RgbImage::new(layout.total_width, layout.total_height);
    for pixel in canvas.pixels_mut() {
        *pixel = Rgb([255, 255, 255]);
    }

    let fonts = load_fonts();

    // Draw the actual plot using the layout information
    for element in layout.elements {
        match element {
            LayoutElement::Image { rect, path } => {
                let img = image::open(Path::new(&path))?.to_rgb8();
                for (x, y, pixel) in img.enumerate_pixels() {
                    let canvas_x = i32_to_u32(rect.x + u32_to_i32(x));
                    let canvas_y = i32_to_u32(rect.y + u32_to_i32(y));
                    if canvas_x < canvas.width() && canvas_y < canvas.height() {
                        canvas.put_pixel(canvas_x, canvas_y, *pixel);
                    }
                }
            }
            LayoutElement::RowLabel { rect, text } | LayoutElement::ColumnLabel { rect, text } => {
                draw_multiline_text(
                    &mut canvas,
                    &text,
                    rect.x,
                    rect.y,
                    24.0,
                    fonts,
                    Rgb([0, 0, 0]),
                );
            }
            LayoutElement::Padding { .. } => {}
        }
    }

    canvas.save(&config.output)
        .with_context(|| format!("Failed to save output image: {:?}", config.output))?;

    Ok(())
}
