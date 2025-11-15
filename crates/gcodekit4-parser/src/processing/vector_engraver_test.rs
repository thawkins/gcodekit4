#[cfg(test)]
mod tests {
    use crate::processing::VectorEngraver;
    use std::fs;

    #[test]
    fn test_svg_with_dtd_parsing() {
        // Test with the actual tiger SVG that has DTD
        let tiger_path = "assets/svg/tiger_head_zhThh.svg";
        
        // Skip test if file doesn't exist
        if !std::path::Path::new(tiger_path).exists() {
            println!("SVG test file not found, skipping");
            return;
        }
        
        let params = super::super::VectorEngravingParameters::default();
        let result = VectorEngraver::from_file(tiger_path, params);
        
        match result {
            Ok(engraver) => {
                println!("✓ Successfully parsed SVG with DTD");
                println!("  Paths found: {:?}", engraver.paths.len());
                assert!(!engraver.paths.is_empty(), "Should have found at least one path");
            }
            Err(e) => {
                panic!("Failed to parse SVG: {}", e);
            }
        }
    }
}
