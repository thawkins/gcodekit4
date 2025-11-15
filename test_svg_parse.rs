use gcodekit4_parser::processing::VectorEngraver;
use gcodekit4_parser::processing::VectorEngravingParameters;

fn main() {
    let params = VectorEngravingParameters::default();
    match VectorEngraver::from_file("assets/svg/tigershead.svg", params) {
        Ok(engraver) => {
            match engraver.generate_gcode() {
                Ok(gcode) => {
                    let lines: Vec<_> = gcode.lines().collect();
                    println!("Generated {} lines of G-code", lines.len());
                    println!("First 20 lines:");
                    for line in lines.iter().take(20) {
                        println!("{}", line);
                    }
                    
                    // Check coordinates
                    let mut coords = Vec::new();
                    for line in &lines {
                        if line.starts_with("G0 X") || line.starts_with("G1 X") {
                            coords.push(line);
                        }
                    }
                    if coords.len() > 0 {
                        println!("\nSample coordinates (first 5):");
                        for line in coords.iter().take(5) {
                            println!("{}", line);
                        }
                    }
                }
                Err(e) => println!("Error generating G-code: {}", e),
            }
        }
        Err(e) => println!("Error creating engraver: {}", e),
    }
}
