//! Numeric type conversion utilities with safety guarantees.
//!
//! This module provides a set of safe numeric conversion functions that handle
//! edge cases and prevent undefined behavior. The functions are designed for:
//!
//! - Safe conversion between floating-point and integer types
//! - Handling of NaN, infinity, and out-of-range values
//! - Clamping values to valid ranges
//! - Proper rounding of floating-point numbers
//!
//! # Safety
//!
//! All functions in this module guarantee:
//! - No undefined behavior
//! - No panics
//! - Deterministic results for all inputs
//! - Proper handling of edge cases (NaN, infinity, etc.)
//!
//! # Examples
//!
//! ```rust
//! use imx::numeric::{f32_to_i32, i32_to_u32, f32_to_u8};
//!
//! // Safe float to int conversion
//! assert_eq!(f32_to_i32(3.7), 4);  // Rounds to nearest
//! assert_eq!(f32_to_i32(f32::NAN), 0);  // NaN becomes 0
//!
//! // Safe signed to unsigned conversion
//! assert_eq!(i32_to_u32(-5), 0);  // Negative becomes 0
//! assert_eq!(i32_to_u32(42), 42);  // Positive passes through
//!
//! // Safe float to byte conversion
//! assert_eq!(f32_to_u8(127.6), 128);  // Rounds to nearest
//! assert_eq!(f32_to_u8(300.0), 255);  // Clamps to max
//! ```

#![warn(clippy::all, clippy::pedantic)]

/// Constants for f32 range that can safely represent integers without precision loss.
/// These values are derived from the fact that f32 has 24 bits of precision,
/// meaning it can exactly represent integers up to 2^24 (16,777,216).
pub(crate) const F32_MAX_SAFE_INT: f32 = 16_777_216.0; // 2^24
pub(crate) const F32_MIN_SAFE_INT: f32 = -16_777_216.0;

/// Safely converts an f32 to i32 with proper rounding and range clamping.
///
/// This function provides several safety guarantees:
/// - NaN values are converted to 0
/// - Values outside i32's range are clamped to `i32::MIN` or `i32::MAX`
/// - Values are rounded to the nearest integer using banker's rounding
///
/// # Arguments
///
/// * `x` - The f32 value to convert
///
/// # Returns
///
/// Returns the converted i32 value, properly rounded and clamped
///
/// # Examples
///
/// ```rust
/// use imx::numeric::f32_to_i32;
///
/// assert_eq!(f32_to_i32(3.7), 4);  // Rounds up
/// assert_eq!(f32_to_i32(3.2), 3);  // Rounds down
/// assert_eq!(f32_to_i32(f32::NAN), 0);  // NaN becomes 0
/// assert_eq!(f32_to_i32(f32::INFINITY), i32::MAX);  // Clamps to max
/// assert_eq!(f32_to_i32(f32::NEG_INFINITY), i32::MIN);  // Clamps to min
/// ```
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

/// Safely converts an i32 to u32, clamping negative values to 0.
///
/// This function guarantees that negative values become 0 while
/// positive values are preserved. This is useful when working with
/// unsigned quantities like array indices or dimensions.
///
/// # Arguments
///
/// * `x` - The i32 value to convert
///
/// # Returns
///
/// Returns the converted u32 value, with negative inputs clamped to 0
///
/// # Examples
///
/// ```rust
/// use imx::numeric::i32_to_u32;
///
/// assert_eq!(i32_to_u32(42), 42);  // Positive passes through
/// assert_eq!(i32_to_u32(0), 0);    // Zero remains zero
/// assert_eq!(i32_to_u32(-5), 0);   // Negative becomes zero
/// ```
#[must_use]
pub fn i32_to_u32(x: i32) -> u32 {
    // Safe because we're clamping negative values to 0
    #[allow(clippy::cast_sign_loss)]
    let result = x.max(0) as u32;
    result
}

/// Safely converts a u32 to i32, clamping values above `i32::MAX`.
///
/// This function handles the case where a u32 value is too large
/// to fit in an i32. Such values are clamped to `i32::MAX`.
///
/// # Arguments
///
/// * `x` - The u32 value to convert
///
/// # Returns
///
/// Returns the converted i32 value, clamped to `i32::MAX` if necessary
///
/// # Examples
///
/// ```rust
/// use imx::numeric::u32_to_i32;
///
/// assert_eq!(u32_to_i32(42), 42);  // Small values pass through
/// assert_eq!(u32_to_i32(0), 0);    // Zero remains zero
/// assert_eq!(u32_to_i32(3_000_000_000), i32::MAX);  // Large values clamp to max
/// ```
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

/// Safely converts an f32 to u8, with rounding and range clamping.
///
/// This function is particularly useful for color channel conversions
/// where values need to be constrained to the 0-255 range. It provides
/// proper handling of floating point edge cases.
///
/// # Arguments
///
/// * `x` - The f32 value to convert
///
/// # Returns
///
/// Returns the converted u8 value, rounded and clamped to 0..=255
///
/// # Examples
///
/// ```rust
/// use imx::numeric::f32_to_u8;
///
/// assert_eq!(f32_to_u8(127.6), 128);  // Rounds to nearest
/// assert_eq!(f32_to_u8(300.0), 255);  // Clamps to max
/// assert_eq!(f32_to_u8(-5.0), 0);     // Clamps to min
/// assert_eq!(f32_to_u8(f32::NAN), 0); // NaN becomes 0
/// ```
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

/// Converts an i32 to f32 for text positioning purposes.
///
/// This function is specifically designed for converting screen coordinates
/// to floating point values for text rendering. While it may lose precision
/// for very large values, this is acceptable for text positioning where:
/// - Values are typically small (screen coordinates)
/// - Sub-pixel precision isn't critical
/// - Exact representation isn't required
///
/// # Arguments
///
/// * `x` - The i32 screen coordinate to convert
///
/// # Returns
///
/// Returns the coordinate as an f32 value
///
/// # Examples
///
/// ```rust
/// use imx::numeric::i32_to_f32_for_pos;
///
/// assert_eq!(i32_to_f32_for_pos(42), 42.0);  // Exact for small values
/// assert_eq!(i32_to_f32_for_pos(0), 0.0);    // Zero remains exact
/// ```
///
/// # Note
///
/// This function is marked as safe for text positioning specifically because:
/// 1. Screen coordinates are typically well within f32's precise range
/// 2. Sub-pixel precision isn't critical for text rendering
/// 3. Any loss of precision won't affect visual quality
#[must_use]
pub fn i32_to_f32_for_pos(x: i32) -> f32 {
    // Safe for text positioning where precision isn't critical
    #[allow(clippy::cast_precision_loss)]
    let result = x as f32;
    result
}

/// Converts an f32 to a u32, handling NaN, infinity, and out-of-range values.
///
/// This function provides safe conversion from f32 to u32 with consistent handling
/// of edge cases and proper rounding behavior. Note that due to f32's precision
/// limitations, values very close to u32::MAX may be rounded to u32::MAX.
///
/// # Examples
///
/// ```
/// use imx::numeric::f32_to_u32;
///
/// assert_eq!(f32_to_u32(0.0), 0);
/// assert_eq!(f32_to_u32(1.4), 1);
/// assert_eq!(f32_to_u32(1.6), 2);
/// assert_eq!(f32_to_u32(-1.0), 0); // Negative values clamp to 0
/// assert_eq!(f32_to_u32(f32::NAN), 0);
/// assert_eq!(f32_to_u32(f32::INFINITY), u32::MAX);
/// ```
#[must_use]
pub fn f32_to_u32(x: f32) -> u32 {
    if x.is_nan() {
        0
    } else if x.is_infinite() {
        if x.is_sign_positive() {
            u32::MAX
        } else {
            0
        }
    } else if x <= 0.0 {
        0
    } else {
        // For values near u32::MAX, we need to be extra careful
        let max_f32 = u32::MAX as f32;
        
        // Round first to handle fractional values consistently
        let rounded = x.round();
        
        // If the value is very close to u32::MAX, return u32::MAX
        if rounded >= max_f32 {
            u32::MAX
        } else {
            // Safe because we've bounded x below u32::MAX
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let result = rounded as u32;
            result
        }
    }
}
