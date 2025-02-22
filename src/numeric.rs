#![warn(clippy::all, clippy::pedantic)]

/// Constants for f32 range that can safely represent integers
pub(crate) const F32_MAX_SAFE_INT: f32 = 16_777_216.0; // 2^24, maximum integer that f32 can represent exactly
pub(crate) const F32_MIN_SAFE_INT: f32 = -16_777_216.0;

/// Safely convert f32 to i32, clamping to i32's range
///
/// This function handles several edge cases:
/// - NaN values are converted to 0
/// - Values outside `i32`'s range are clamped
/// - Values are rounded to nearest integer
#[must_use]
pub fn f32_to_i32(x: f32) -> i32 {
    if x.is_nan() {
        0
    } else if x >= F32_MAX_SAFE_INT {
        i32::MAX
    } else if x <= F32_MIN_SAFE_INT {
        i32::MIN
    } else {
        // Safe because we've bounded x within safe integer range
        #[allow(clippy::cast_possible_truncation)]
        let result = x.round() as i32;
        result
    }
}

/// Safely convert i32 to u32, clamping negative values to 0
///
/// This conversion is safe because:
/// - Negative values are clamped to 0
/// - Positive values are within `u32`'s range
#[must_use]
pub fn i32_to_u32(x: i32) -> u32 {
    // Safe because we're clamping negative values to 0
    #[allow(clippy::cast_sign_loss)]
    let result = x.max(0) as u32;
    result
}

/// Safely convert u32 to i32, clamping to i32's range
///
/// This conversion is safe because:
/// - Values above `i32::MAX` are clamped
#[must_use]
pub fn u32_to_i32(x: u32) -> i32 {
    if x > i32::MAX as u32 {
        i32::MAX
    } else {
        // Safe because we've checked the upper bound
        #[allow(clippy::cast_possible_wrap)]
        let result = x as i32;
        result
    }
}

/// Safely convert f32 to u8, clamping to u8's range
///
/// This conversion is safe because:
/// - NaN values are converted to 0
/// - Values are clamped to 0..=255
/// - Values are rounded to nearest integer
#[must_use]
pub fn f32_to_u8(x: f32) -> u8 {
    if x.is_nan() {
        0
    } else if x >= 255.0 {
        255
    } else if x <= 0.0 {
        0
    } else {
        // Safe because we've bounded x within u8's range
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let result = x.round() as u8;
        result
    }
}

/// Safely convert i32 to f32 for text positioning
///
/// While this conversion can lose precision for large values,
/// it's acceptable for text positioning where:
/// - Values are typically small (screen coordinates)
/// - Sub-pixel precision isn't critical
#[must_use]
pub fn i32_to_f32_for_pos(x: i32) -> f32 {
    // Safe for text positioning where precision isn't critical
    #[allow(clippy::cast_precision_loss)]
    let result = x as f32;
    result
}
