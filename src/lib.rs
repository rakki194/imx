#![warn(clippy::all, clippy::pedantic)]

pub mod image_processing;
pub mod jxl;
pub mod numeric;

// Re-export commonly used types and functions
pub use image_processing::{
    get_image_dimensions, is_image_file, process_image, remove_letterbox,
    remove_letterbox_with_threshold, remove_transparency,
};
pub use jxl::{convert_jxl_to_png, is_jxl_file, process_jxl_file};

#[cfg(test)]
mod tests {
    mod image_processing_tests;
    mod jxl_tests;
    mod numeric_tests;
}
