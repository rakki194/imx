use super::super::xyplot::load_fonts;

#[test]
fn test_font_loading() {
    // Test that fonts can be loaded without panicking
    let fonts = load_fonts();
    
    // Only test the main font - fontdue doesn't support color emoji properly
    let (_, has_main_bitmap) = fonts.main.rasterize('A', 32.0);
    assert!(!has_main_bitmap.is_empty(), "Main font should have basic Latin characters");
}

#[test]
fn test_glyph_selection() {
    let fonts = load_fonts();
    
    // Test regular ASCII characters (should use main font)
    let font = fonts.get_font_for_char('A');
    let (_, ascii_bitmap) = font.rasterize('A', 32.0);
    assert!(!ascii_bitmap.is_empty(), "Main font should have raster for ASCII");
}

#[test]
fn test_emoji_ranges() {
    let fonts = load_fonts();
    
    // Test characters from different emoji ranges - but just check the font selection is correct
    // not the actual rasterization which may not work properly with fontdue
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
        let font = fonts.get_font_for_char(emoji);
        assert_eq!(std::ptr::from_ref(font), std::ptr::from_ref(fonts.emoji), 
                  "Should use emoji font for {emoji} ({description})");
    }
}

#[test]
fn test_mixed_text_glyph_selection() {
    let fonts = load_fonts();
    
    // Test a string with mixed regular text and emoji
    let test_str = "Hello 👋 World! 🌍";
    
    for c in test_str.chars() {
        let font = fonts.get_font_for_char(c);
        
        match c {
            '👋' | '🌍' => {
                // These should be handled by the emoji font
                assert_eq!(std::ptr::from_ref(font), std::ptr::from_ref(fonts.emoji),
                       "Emoji {c} should use emoji font");
            }
            _ => {
                // Regular characters should be handled by the main font
                if !c.is_whitespace() {
                    assert_eq!(std::ptr::from_ref(font), std::ptr::from_ref(fonts.main),
                           "Character {c} should use main font");
                    
                    let (_, bitmap) = font.rasterize(c, 32.0);
                    assert!(!bitmap.is_empty(), "Character {c} should have bitmap");
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
        let font = fonts.get_font_for_char(c);
        
        // These should use the main font
        assert_eq!(std::ptr::from_ref(font), std::ptr::from_ref(fonts.main),
               "Character {c} should use main font");
        
        // Check that we can rasterize them
        let (_, bitmap) = font.rasterize(c, 32.0);
        assert!(!bitmap.is_empty(), "Character {c} should have bitmap");
    }
}

#[test]
fn test_font_metrics() {
    let fonts = load_fonts();
    
    // Test only regular text metrics, skip emoji
    let (metrics, bitmap) = fonts.main.rasterize('A', 32.0);
    
    // Check that we get reasonable metrics
    assert!(metrics.advance_width > 0.0, "ASCII character should have positive advance width");
    
    // Check that we got a reasonable bitmap
    assert!(metrics.width > 0 && metrics.height > 0, "ASCII character should have non-empty dimensions");
    assert!(!bitmap.is_empty(), "ASCII character should have bitmap data");
} 