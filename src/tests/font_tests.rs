use super::super::xyplot::load_fonts;
use ab_glyph::{Font, ScaleFont};

#[test]
fn test_font_loading() {
    // Test that fonts can be loaded without panicking
    let fonts = load_fonts();
    assert!(fonts.main.glyph_id('A').0 > 0, "Main font should have basic Latin characters");
    assert!(fonts.emoji.glyph_id('😀').0 > 0, "Emoji font should have emoji characters");
}

#[test]
fn test_glyph_selection() {
    let fonts = load_fonts();
    
    // Test regular ASCII characters (should use main font)
    let (ascii_id, selected_font) = fonts.glyph_id('A');
    assert!(selected_font.outline(ascii_id).is_some(), "Main font should have outline for ASCII");
    
    // Test emoji character (should use emoji font)
    let (emoji_id, selected_font) = fonts.glyph_id('😀');
    assert!(selected_font.glyph_raster_image2(emoji_id, u16::MAX).is_some(), 
           "Emoji font should have color glyph for emoji");
    
    // Test emoji character that definitely exists in Noto Color Emoji
    let (emoji_id, selected_font) = fonts.glyph_id('🎨');
    assert!(selected_font.glyph_raster_image2(emoji_id, u16::MAX).is_some(),
           "Emoji font should have color glyph for palette emoji");
}

#[test]
fn test_emoji_ranges() {
    let fonts = load_fonts();
    
    // Test characters from different emoji ranges
    let test_cases = [
        // Emoticons (1F600-1F64F)
        ('😀', "Basic emoticon"),
        ('😎', "Face with sunglasses"),
        // Transport and Map Symbols (1F680-1F6FF)
        ('🚀', "Rocket"),
        ('🛸', "Flying saucer"),
        // Miscellaneous Symbols and Pictographs (1F300-1F5FF)
        ('🌈', "Rainbow"),
        ('🎨', "Artist palette"),
        // Additional emoticons and symbols (1F900-1F9FF)
        ('🤖', "Robot face"),
        ('🦄', "Unicorn"),
        // Supplemental Symbols and Pictographs (1FA70-1FAFF)
        ('🩷', "Pink heart"),
        ('🫂', "People hugging"),
    ];
    
    for (emoji, description) in test_cases {
        let (glyph_id, font) = fonts.glyph_id(emoji);
        assert!(font.glyph_raster_image2(glyph_id, u16::MAX).is_some(),
               "Should find color glyph for {} ({})", emoji, description);
    }
}

#[test]
fn test_mixed_text_glyph_selection() {
    let fonts = load_fonts();
    
    // Test a string with mixed regular text and emoji
    let test_str = "Hello 👋 World! 🌍";
    
    for c in test_str.chars() {
        let (glyph_id, font) = fonts.glyph_id(c);
        match c {
            '👋' | '🌍' => {
                // These should be handled by the emoji font
                assert!(font.glyph_raster_image2(glyph_id, u16::MAX).is_some(),
                       "Emoji {} should have color glyph", c);
            }
            _ => {
                // Regular characters should have outlines in the main font
                if !c.is_whitespace() {
                    assert!(font.outline(glyph_id).is_some(),
                           "Character {} should have outline", c);
                }
            }
        }
    }
}

#[test]
fn test_fallback_behavior() {
    let fonts = load_fonts();
    
    // Test characters that should fall back to the main font
    // even though they might be in emoji ranges
    let fallback_chars = [
        '#',  // Basic ASCII that might be in emoji font
        '©',  // Copyright symbol
        '®',  // Registered trademark
        '™',  // Trademark
    ];
    
    for &c in &fallback_chars {
        let (glyph_id, font) = fonts.glyph_id(c);
        assert!(font.outline(glyph_id).is_some(),
               "Character {} should have outline in fallback font", c);
    }
}

#[test]
fn test_font_metrics() {
    let fonts = load_fonts();
    
    // Test that emoji and regular text have reasonable metrics
    let test_cases = [
        ('A', "ASCII character"),
        ('😀', "Basic emoji"),
        ('🌍', "Complex emoji"),
    ];
    
    for (c, description) in test_cases {
        let (glyph_id, font) = fonts.glyph_id(c);
        let scaled_font = font.as_scaled(ab_glyph::PxScale::from(32.0));
        
        // Check that we get reasonable advance metrics
        let advance = scaled_font.h_advance(glyph_id);
        assert!(advance > 0.0, 
                "{} ({}) should have positive advance width", c, description);
        
        // For emoji, verify we get color glyphs at reasonable sizes
        if c.is_ascii_alphabetic() {
            assert!(font.outline(glyph_id).is_some(),
                   "{} should have outline", description);
        } else {
            assert!(font.glyph_raster_image2(glyph_id, 32).is_some(),
                   "{} should have color glyph at size 32", description);
        }
    }
} 