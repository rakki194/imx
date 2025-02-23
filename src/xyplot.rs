use crate::numeric::{f32_to_i32, f32_to_u8, i32_to_f32_for_pos, i32_to_u32, u32_to_i32};
use ab_glyph::{Font, FontRef, GlyphId, Point, PxScale, ScaleFont};
use anyhow::{Context, Result};
use image::{Rgb, RgbImage};
use rgb::FromSlice;
use std::path::PathBuf;

// Constants for layout
const TOP_PADDING: u32 = 40; // Space for labels and padding at the top

#[derive(Clone, Copy)]
struct FontPair<'a> {
    main: &'a FontRef<'a>,
    emoji: &'a FontRef<'a>,
}

impl<'a> FontPair<'a> {
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

/// Configuration for creating an image plot
#[derive(Debug)]
pub struct PlotConfig {
    /// List of image file paths to plot
    pub images: Vec<PathBuf>,
    /// Output file name for the generated plot
    pub output: PathBuf,
    /// Number of rows to display the images
    pub rows: u32,
    /// List of optional labels for each row
    pub row_labels: Vec<String>,
    /// List of optional labels for each column
    pub column_labels: Vec<String>,
}

/// Creates a plot of images arranged in a grid with optional labels
///
/// # Arguments
///
/// * `config` - Configuration for the plot including images, layout, and labels
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure
///
/// # Errors
///
/// Returns an error if:
/// * The number of row labels doesn't match the number of rows
/// * The number of column labels doesn't match the number of columns
/// * Any image file cannot be opened
/// * The output file cannot be written
pub fn create_plot(config: &PlotConfig) -> Result<()> {
    let PlotConfig {
        images,
        output,
        rows,
        row_labels,
        column_labels,
    } = config;

    // Validate inputs
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

    // Load fonts
    let font_data = include_bytes!("../assets/DejaVuSans.ttf");
    let main_font = FontRef::try_from_slice(font_data).context("Failed to load main font")?;

    let emoji_font_data = include_bytes!("../assets/NotoColorEmoji.ttf");
    let emoji_font =
        FontRef::try_from_slice(emoji_font_data).context("Failed to load emoji font")?;

    let fonts = FontPair {
        main: &main_font,
        emoji: &emoji_font,
    };

    // Calculate dynamic left padding based on longest row label
    let left_padding = if row_labels.iter().any(|l| !l.is_empty()) {
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
        
        f32_to_i32(max_label_width + 40.0) // Add some padding after the text
    } else {
        0
    };

    // Find maximum image dimensions in the grid
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

    // Calculate canvas dimensions with space for labels
    let has_labels = !row_labels.is_empty() || !column_labels.is_empty();
    let row_height = max_height + if has_labels { TOP_PADDING } else { 0 };
    let canvas_height = row_height * rows + if has_labels { TOP_PADDING } else { 0 };
    let canvas_width = max_width * cols + i32_to_u32(left_padding);

    // Create canvas
    let mut canvas = RgbImage::new(canvas_width, canvas_height);
    // Fill with white
    for pixel in canvas.pixels_mut() {
        *pixel = Rgb([255, 255, 255]);
    }

    // Add column labels
    if !column_labels.is_empty() {
        for (col, label) in column_labels.iter().enumerate() {
            let x = u32_to_i32(
                u32::try_from(col).unwrap_or(0) * max_width + i32_to_u32(left_padding),
            );
            let y = u32_to_i32(TOP_PADDING / 2);

            draw_text(
                &mut canvas,
                label,
                x,
                y,
                24.0,
                fonts,
                Rgb([0, 0, 0]),
            );
        }
    }

    // Place images and labels
    for (i, img_path) in images.iter().enumerate() {
        let i = u32::try_from(i)?;
        let row = i / cols;
        let col = i % cols;

        // Calculate positions
        let x_start = col * max_width + i32_to_u32(left_padding);
        let y_start = row * row_height + TOP_PADDING;

        // Add row label if provided
        if let Some(row_label) = row_labels.get(row as usize) {
            let x = 20;
            let y = u32_to_i32(y_start + max_height / 2);
            draw_text(&mut canvas, row_label, x, y, 24.0, fonts, Rgb([0, 0, 0]));
        }

        // Load and place image
        let img = image::open(img_path)
            .with_context(|| format!("Failed to open image: {img_path:?}"))?
            .to_rgb8();
        let (img_width, img_height) = img.dimensions();

        // Center the image in its cell
        let x_offset = (max_width - img_width) / 2;
        let y_offset = (max_height - img_height) / 2;

        // Copy image to canvas
        for (x, y, pixel) in img.enumerate_pixels() {
            let canvas_x = x_start + x_offset + x;
            let canvas_y = y_start + y_offset + y;
            if canvas_x < canvas_width && canvas_y < canvas_height {
                canvas.put_pixel(canvas_x, canvas_y, *pixel);
            }
        }
    }

    // Save the generated plot
    canvas
        .save(output)
        .with_context(|| format!("Failed to save output image: {output:?}"))?;

    Ok(())
}
