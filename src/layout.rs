use image::{Rgb, RgbImage};
use std::path::Path;

/// Represents a rectangular region in the layout
#[derive(Debug, Clone, Copy)]
pub struct LayoutRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Represents different types of layout elements
#[derive(Debug)]
pub enum LayoutElement {
    Image {
        rect: LayoutRect,
        path: String,
    },
    RowLabel {
        rect: LayoutRect,
        text: String,
    },
    ColumnLabel {
        rect: LayoutRect,
        text: String,
    },
    Padding {
        rect: LayoutRect,
        description: String,
    },
}

/// Represents the complete layout of the plot
#[derive(Debug)]
pub struct Layout {
    pub elements: Vec<LayoutElement>,
    pub total_width: u32,
    pub total_height: u32,
}

impl Layout {
    /// Creates a new empty layout
    #[must_use] pub fn new(width: u32, height: u32) -> Self {
        Self {
            elements: Vec::new(),
            total_width: width,
            total_height: height,
        }
    }

    /// Adds an element to the layout
    pub fn add_element(&mut self, element: LayoutElement) {
        self.elements.push(element);
    }

    /// Renders the layout as a debug visualization
    #[must_use] pub fn render_debug(&self) -> RgbImage {
        let mut canvas = RgbImage::new(self.total_width, self.total_height);
        
        // Fill with white background
        for pixel in canvas.pixels_mut() {
            *pixel = Rgb([255, 255, 255]);
        }

        // Define colors for different element types
        let image_color = Rgb([200, 200, 255]); // Light blue
        let row_label_color = Rgb([255, 200, 200]); // Light red
        let col_label_color = Rgb([200, 255, 200]); // Light green
        let padding_color = Rgb([240, 240, 240]); // Light gray

        // Draw each element
        for element in &self.elements {
            let color = match element {
                LayoutElement::Image { path, .. } => {
                    (image_color, format!("Image: {}", Path::new(path).file_name().unwrap_or_default().to_string_lossy()))
                }
                LayoutElement::RowLabel { text, .. } => {
                    (row_label_color, format!("Row: {text}"))
                }
                LayoutElement::ColumnLabel { text, .. } => {
                    (col_label_color, format!("Col: {text}"))
                }
                LayoutElement::Padding { description, .. } => {
                    (padding_color, format!("Pad: {description}"))
                }
            }.0;

            let rect = match element {
                LayoutElement::Image { rect, .. } |
                LayoutElement::RowLabel { rect, .. } |
                LayoutElement::ColumnLabel { rect, .. } |
                LayoutElement::Padding { rect, .. } => rect,
            };

            // Draw filled rectangle
            for y in rect.y.max(0)..rect.y.saturating_add(rect.height as i32) {
                for x in rect.x.max(0)..rect.x.saturating_add(rect.width as i32) {
                    if x >= 0 && y >= 0 && x < canvas.width() as i32 && y < canvas.height() as i32 {
                        canvas.put_pixel(x as u32, y as u32, color);
                    }
                }
            }

            // Draw border
            let border_color = Rgb([100, 100, 100]);
            for x in rect.x.max(0)..rect.x.saturating_add(rect.width as i32) {
                if x >= 0 && x < canvas.width() as i32 {
                    if rect.y >= 0 && rect.y < canvas.height() as i32 {
                        canvas.put_pixel(x as u32, rect.y as u32, border_color);
                    }
                    let bottom_y = rect.y.saturating_add(rect.height as i32 - 1);
                    if bottom_y >= 0 && bottom_y < canvas.height() as i32 {
                        canvas.put_pixel(x as u32, bottom_y as u32, border_color);
                    }
                }
            }
            for y in rect.y.max(0)..rect.y.saturating_add(rect.height as i32) {
                if y >= 0 && y < canvas.height() as i32 {
                    if rect.x >= 0 && rect.x < canvas.width() as i32 {
                        canvas.put_pixel(rect.x as u32, y as u32, border_color);
                    }
                    let right_x = rect.x.saturating_add(rect.width as i32 - 1);
                    if right_x >= 0 && right_x < canvas.width() as i32 {
                        canvas.put_pixel(right_x as u32, y as u32, border_color);
                    }
                }
            }
        }

        canvas
    }
}
