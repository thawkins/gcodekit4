use gcodekit4_camtools::vector_engraver::{VectorEngraver, VectorEngravingParameters};
use lyon::path::Path;
use lyon::math::point;

#[test]
fn test_default_parameters() {
    let params = VectorEngravingParameters::default();
    assert_eq!(params.feed_rate, 600.0);
    assert_eq!(params.cut_power, 100.0);
    assert_eq!(params.engrave_power, 50.0);
    assert!(!params.multi_pass);
}

#[test]
fn test_svg_file_validation() {
    let result = VectorEngraver::from_file("test.txt", VectorEngravingParameters::default());
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    // Either unsupported format or file not found is acceptable
    assert!(
        error_msg.contains("Unsupported file format") || error_msg.contains("File not found")
    );
}

#[test]
fn test_svg_path_tokenization() {
    let tokens = VectorEngraver::tokenize_svg_path("M 10 20 L 30 40 Z");
    assert_eq!(tokens.len(), 7);
    assert_eq!(tokens[0], "M");
    assert_eq!(tokens[1], "10");
}

#[test]
fn test_estimate_time() {
    let mut params = VectorEngravingParameters::default();
    params.feed_rate = 100.0;

    let mut builder = Path::builder();
    builder.begin(point(0.0, 0.0));
    builder.line_to(point(100.0, 0.0));
    builder.end(false);
    let path = builder.build();

    let engraver = VectorEngraver {
        file_path: "test.svg".to_string(),
        params,
        paths: vec![path],
        scale_factor: 1.0,
    };

    let time = engraver.estimate_time();
    assert!(time > 0.0);
}

#[test]
fn test_svg_with_dtd_parsing() {
    // Test with actual SVG that may have DTD
    let tiger_path = "assets/svg/tiger_head_zhThh.svg";
    
    // Skip test if file doesn't exist
    if !std::path::Path::new(tiger_path).exists() {
        return;
    }
    
    let params = VectorEngravingParameters::default();
    let result = VectorEngraver::from_file(tiger_path, params);
    
    // Should successfully parse DTD SVG
    assert!(result.is_ok(), "Failed to parse SVG with DTD: {:?}", result);
}
