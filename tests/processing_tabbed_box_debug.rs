//! Debug test to find when tabs become larger than thickness

use gcodekit4_parser::processing::tabbed_box::{BoxParameters, TabbedBoxMaker, TabType, BoxType, LayoutStyle};

#[test]
fn test_various_configurations() {
    let configs = vec![
        ("No kerf, outside dims", 3.0, 0.0, false),
        ("No kerf, inside dims", 3.0, 0.0, true),
        ("With kerf, outside dims", 3.0, 0.5, false),
        ("With kerf, inside dims", 3.0, 0.5, true),
        ("Large kerf, outside", 3.0, 1.0, false),
        ("Large kerf, inside", 3.0, 1.0, true),
    ];
    
    for (name, thickness, kerf, inside_dims) in configs {
        println!("\n=== {} ===", name);
        println!("Thickness: {}mm, Kerf: {}mm, Inside dims: {}", thickness, kerf, inside_dims);
        
        let params = BoxParameters {
            length: 100.0,
            width: 100.0,
            height: 100.0,
            thickness,
            tab_width: 25.0,
            kerf,
            spacing: 5.0,
            box_type: BoxType::FullBox,
            tab_type: TabType::Laser,
            layout_style: LayoutStyle::Diagrammatic,
            inside_dimensions: inside_dims,
            dividers_length: 0,
            dividers_width: 0,
            laser_passes: 1,
            laser_power: 1000,
            feed_rate: 500.0,
        };
        
        let mut maker = TabbedBoxMaker::new(params).expect("Failed to create TabbedBoxMaker");
        maker.generate().expect("Failed to generate box");
        let gcode = maker.to_gcode(500.0, 100.0, 3.0);
        
        // Extract all Y and X coordinates
        let mut y_coords: Vec<f32> = Vec::new();
        let mut x_coords: Vec<f32> = Vec::new();
        
        for line in gcode.lines() {
            if line.starts_with("G1") || line.starts_with("G0") {
                if let Some(y_start) = line.find("Y") {
                    let y_str: String = line[y_start+1..]
                        .chars()
                        .take_while(|c| c.is_numeric() || *c == '.' || *c == '-')
                        .collect();
                    if let Ok(y) = y_str.parse::<f32>() {
                        y_coords.push(y);
                    }
                }
                if let Some(x_start) = line.find("X") {
                    let x_str: String = line[x_start+1..]
                        .chars()
                        .take_while(|c| c.is_numeric() || *c == '.' || *c == '-')
                        .collect();
                    if let Ok(x) = x_str.parse::<f32>() {
                        x_coords.push(x);
                    }
                }
            }
        }
        
        let min_y = y_coords.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_y = y_coords.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min_x = x_coords.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_x = x_coords.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        
        println!("X range: {:.2} to {:.2}", min_x, max_x);
        println!("Y range: {:.2} to {:.2}", min_y, max_y);
        
        if min_y < -0.1 {
            let tab_depth_y = min_y.abs();
            println!("Tab protrusion in Y direction: {:.2}mm", tab_depth_y);
            if (tab_depth_y - thickness).abs() > 0.1 {
                println!("⚠️  WARNING: Tab depth {:.2}mm != thickness {:.2}mm (diff: {:.2}mm)", 
                         tab_depth_y, thickness, tab_depth_y - thickness);
            }
        }
        
        if min_x < -0.1 {
            let tab_depth_x = min_x.abs();
            println!("Tab protrusion in X direction: {:.2}mm", tab_depth_x);
            if (tab_depth_x - thickness).abs() > 0.1 {
                println!("⚠️  WARNING: Tab depth {:.2}mm != thickness {:.2}mm (diff: {:.2}mm)", 
                         tab_depth_x, thickness, tab_depth_x - thickness);
            }
        }
    }
}
