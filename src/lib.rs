pub mod numeric;
pub mod image_processing;
pub mod jxl;

#[cfg(test)]
mod tests {
    mod numeric_tests;
    mod jxl_tests;
    mod image_processing_tests;
}

// Re-export commonly used types and functions
pub use image_processing::{
    caption_file_exists_and_not_empty, get_image_dimensions, is_image_file, process_image,
    remove_letterbox, remove_letterbox_with_threshold, remove_transparency,
    rename_file_without_image_extension,
};
pub use jxl::{convert_jxl_to_png, is_jxl_file, process_jxl_file};
