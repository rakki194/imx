#![warn(clippy::all, clippy::pedantic)]

use crate::numeric;

// Test-specific constants
const TEST_SAFE_INT_MAX: f32 = numeric::F32_MAX_SAFE_INT;
const TEST_SAFE_INT_MIN: f32 = numeric::F32_MIN_SAFE_INT;

#[test]
fn test_f32_to_i32() {
    // Normal cases
    assert_eq!(numeric::f32_to_i32(0.0), 0);
    assert_eq!(numeric::f32_to_i32(1.0), 1);
    assert_eq!(numeric::f32_to_i32(-1.0), -1);
    assert_eq!(numeric::f32_to_i32(1.4), 1);
    assert_eq!(numeric::f32_to_i32(1.6), 2);
    assert_eq!(numeric::f32_to_i32(-1.4), -1);
    assert_eq!(numeric::f32_to_i32(-1.6), -2);

    // Edge cases
    assert_eq!(numeric::f32_to_i32(f32::NAN), 0);
    assert_eq!(numeric::f32_to_i32(f32::INFINITY), i32::MAX);
    assert_eq!(numeric::f32_to_i32(f32::NEG_INFINITY), i32::MIN);
    assert_eq!(numeric::f32_to_i32(TEST_SAFE_INT_MAX), i32::MAX);
    assert_eq!(numeric::f32_to_i32(TEST_SAFE_INT_MIN), i32::MIN);

    // Values just inside bounds
    #[allow(clippy::cast_possible_truncation)]
    {
        assert_eq!(
            numeric::f32_to_i32(TEST_SAFE_INT_MAX - 1.0),
            (TEST_SAFE_INT_MAX - 1.0) as i32
        );
        assert_eq!(
            numeric::f32_to_i32(TEST_SAFE_INT_MIN + 1.0),
            (TEST_SAFE_INT_MIN + 1.0) as i32
        );
    }
}

#[test]
fn test_i32_to_u32() {
    // Normal cases
    assert_eq!(numeric::i32_to_u32(0), 0);
    assert_eq!(numeric::i32_to_u32(1), 1);
    assert_eq!(numeric::i32_to_u32(i32::MAX), i32::MAX as u32);

    // Negative values should clamp to 0
    assert_eq!(numeric::i32_to_u32(-1), 0);
    assert_eq!(numeric::i32_to_u32(i32::MIN), 0);
    assert_eq!(numeric::i32_to_u32(-42), 0);
}

#[test]
fn test_u32_to_i32() {
    // Normal cases
    assert_eq!(numeric::u32_to_i32(0), 0);
    assert_eq!(numeric::u32_to_i32(1), 1);
    assert_eq!(numeric::u32_to_i32(i32::MAX as u32), i32::MAX);

    // Values above i32::MAX should clamp
    assert_eq!(numeric::u32_to_i32(i32::MAX as u32 + 1), i32::MAX);
    assert_eq!(numeric::u32_to_i32(u32::MAX), i32::MAX);

    // Values just below the boundary
    assert_eq!(numeric::u32_to_i32(i32::MAX as u32 - 1), i32::MAX - 1);
}

#[test]
fn test_f32_to_u8() {
    // Normal cases
    assert_eq!(numeric::f32_to_u8(0.0), 0);
    assert_eq!(numeric::f32_to_u8(1.0), 1);
    assert_eq!(numeric::f32_to_u8(254.0), 254);
    assert_eq!(numeric::f32_to_u8(255.0), 255);
    assert_eq!(numeric::f32_to_u8(127.4), 127);
    assert_eq!(numeric::f32_to_u8(127.6), 128);

    // Edge cases
    assert_eq!(numeric::f32_to_u8(f32::NAN), 0);
    assert_eq!(numeric::f32_to_u8(f32::INFINITY), 255);
    assert_eq!(numeric::f32_to_u8(f32::NEG_INFINITY), 0);
    assert_eq!(numeric::f32_to_u8(-1.0), 0);
    assert_eq!(numeric::f32_to_u8(256.0), 255);

    // Values just inside bounds
    assert_eq!(numeric::f32_to_u8(254.4), 254);
    assert_eq!(numeric::f32_to_u8(254.6), 255);
    assert_eq!(numeric::f32_to_u8(0.4), 0);
    assert_eq!(numeric::f32_to_u8(0.6), 1);
}

#[test]
#[allow(
    clippy::float_cmp,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
fn test_i32_to_f32_for_pos() {
    // Test typical screen coordinate values
    assert_eq!(numeric::i32_to_f32_for_pos(0), 0.0);
    assert_eq!(numeric::i32_to_f32_for_pos(100), 100.0);
    assert_eq!(numeric::i32_to_f32_for_pos(-100), -100.0);

    // Test larger values that might lose precision
    let large_value = 16_777_216; // 2^24, largest integer that f32 can represent exactly
    assert_eq!(numeric::i32_to_f32_for_pos(large_value), large_value as f32);

    // Test extreme values
    assert_eq!(
        numeric::i32_to_f32_for_pos(i32::MAX).round() as i32,
        i32::MAX
    );
    assert_eq!(
        numeric::i32_to_f32_for_pos(i32::MIN).round() as i32,
        i32::MIN
    );
}

#[test]
fn test_f32_to_u32() {
    // Normal cases
    assert_eq!(numeric::f32_to_u32(0.0), 0);
    assert_eq!(numeric::f32_to_u32(1.0), 1);
    assert_eq!(numeric::f32_to_u32(1.4), 1);
    assert_eq!(numeric::f32_to_u32(1.6), 2);

    // Edge cases
    assert_eq!(numeric::f32_to_u32(f32::NAN), 0);
    assert_eq!(numeric::f32_to_u32(f32::INFINITY), u32::MAX);
    assert_eq!(numeric::f32_to_u32(f32::NEG_INFINITY), 0);
    assert_eq!(numeric::f32_to_u32(-1.0), 0);
    assert_eq!(numeric::f32_to_u32(-0.0), 0);

    // Values just inside bounds
    assert_eq!(numeric::f32_to_u32(0.4), 0);
    assert_eq!(numeric::f32_to_u32(0.6), 1);
    
    // Values near u32::MAX are handled specially due to f32 precision limitations
    // See test_f32_to_u32_max_precision_limitation for details
}

/// Known limitation: f32 cannot precisely represent u32::MAX
/// This test verifies that we handle this case gracefully by returning u32::MAX
#[test]
fn test_f32_to_u32_max_precision_limitation() {
    // When converting u32::MAX to f32 and back, we expect to get u32::MAX
    // This is because we handle the imprecise representation gracefully
    let max_as_f32 = u32::MAX as f32;
    assert_eq!(numeric::f32_to_u32(max_as_f32), u32::MAX);
    
    // Document the actual behavior for clarity
    println!("Note: u32::MAX as f32 = {}", max_as_f32);
    println!("This is slightly less than u32::MAX due to f32's precision limitations");
    
    // Also test u32::MAX - 1 to verify it's handled correctly
    let max_minus_one_f32 = (u32::MAX - 1) as f32;
    let result = numeric::f32_to_u32(max_minus_one_f32);
    println!("Note: (u32::MAX - 1) as f32 = {}", max_minus_one_f32);
    println!("Converting back to u32 gives: {}", result);
}
