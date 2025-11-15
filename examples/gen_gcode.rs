fn main() {
    use gcodekit4_parser::processing::{VectorEngraver, VectorEngravingParameters};
    
    let params = VectorEngravingParameters::default();
    match VectorEngraver::from_file("assets/svg/tigershead.svg", params) {
        Ok(engraver) => {
            println!("✓ Engraver created");
            match engraver.generate_gcode() {
                Ok(gcode) => {
                    let lines: Vec<_> = gcode.lines().collect();
                    println!("✓ Generated {} lines", lines.len());
                    let g0 = lines.iter().filter(|l| l.starts_with("G0 X")).count();
                    let g1 = lines.iter().filter(|l| l.starts_with("G1 X")).count();
                    println!("✓ Commands: {} G0, {} G1", g0, g1);
                    std::fs::write("assets/gcode/tigershead.gcode", &gcode).ok();
                    println!("✓ Saved");
                }
                Err(e) => eprintln!("✗ Error: {}", e),
            }
        }
        Err(e) => eprintln!("✗ Error: {}", e),
    }
}
