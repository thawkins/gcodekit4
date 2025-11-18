#[test]
fn test_tigershead_svg_conversion() {
    use gcodekit4_camtools::{VectorEngraver, VectorEngravingParameters};
    
    let params = VectorEngravingParameters::default();
    // Use the correct path from project root
    let asset_path = if std::path::Path::new("assets/svg/tigershead.svg").exists() {
        "assets/svg/tigershead.svg"
    } else {
        "../../assets/svg/tigershead.svg"
    };
    
    let engraver = VectorEngraver::from_file(asset_path, params)
        .expect("Failed to create engraver");
    
    let gcode = engraver.generate_gcode()
        .expect("Failed to generate G-code");
    
    let lines: Vec<_> = gcode.lines().collect();
    println!("Generated {} lines of G-code", lines.len());
    assert!(lines.len() > 100, "Should generate substantial G-code");
    
    // Check that we have actual movement commands with coordinates
    let move_lines: Vec<_> = lines.iter()
        .filter(|l| l.starts_with("G0 X") || l.starts_with("G1 X"))
        .collect();
    
    println!("Movement commands: {}", move_lines.len());
    assert!(move_lines.len() > 100, "Should have many movement commands");
    
    // Print sample coordinates
    println!("Sample coordinates:");
    for line in move_lines.iter().take(5) {
        println!("  {}", line);
    }
}
