fn main() {
    // Minimal test - just import and check it compiles
    use gcodekit4_parser::processing::{VectorEngraver, VectorEngravingParameters};
    use std::path::Path;
    
    let svg_path = "assets/svg/tigershead.svg";
    if !Path::new(svg_path).exists() {
        eprintln!("SVG file not found: {}", svg_path);
        return;
    }
    
    let params = VectorEngravingParameters::default();
    
    match VectorEngraver::from_file(svg_path, params) {
        Ok(engraver) => {
            println!("✓ Engraver created successfully");
            
            match engraver.generate_gcode() {
                Ok(gcode) => {
                    let lines: Vec<_> = gcode.lines().collect();
                    println!("✓ Generated {} lines of G-code", lines.len());
                    
                    // Count command types
                    let g0_count = lines.iter().filter(|l| l.starts_with("G0")).count();
                    let g1_count = lines.iter().filter(|l| l.starts_with("G1")).count();
                    println!("  - G0 commands: {}", g0_count);
                    println!("  - G1 commands: {}", g1_count);
                    
                    // Show first few move commands
                    println!("\n✓ Sample G-code:");
                    for line in lines.iter().take(30).filter(|l| !l.starts_with(";")) {
                        println!("  {}", line);
                    }
                    
                    // Save to file
                    if let Ok(_) = std::fs::write("assets/gcode/tigershead_generated.gcode", &gcode) {
                        println!("\n✓ Saved to assets/gcode/tigershead_generated.gcode");
                    }
                }
                Err(e) => eprintln!("✗ Error generating G-code: {}", e),
            }
        }
        Err(e) => eprintln!("✗ Error creating engraver: {}", e),
    }
}
