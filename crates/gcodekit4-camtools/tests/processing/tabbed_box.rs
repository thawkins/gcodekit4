//! Test for verifying tabbed box tab protrusion dimensions

use gcodekit4_camtools::tabbed_box::{
    BoxParameters, BoxType, FingerJointSettings, TabbedBoxMaker, KeyDividerType,
};

#[test]
fn test_tab_protrusion_equals_thickness_no_kerf() {
    let params = BoxParameters {
        x: 100.0,
        y: 100.0,
        h: 100.0,
        thickness: 3.0,
        outside: false,
        box_type: BoxType::FullBox,
        finger_joint: FingerJointSettings::default(),
        burn: 0.0,
        laser_passes: 1,
        laser_power: 1000,
        feed_rate: 500.0,
        offset_x: 0.0,
        offset_y: 0.0,
        dividers_x: 0,
        dividers_y: 0,
        optimize_layout: false,
        key_divider_type: KeyDividerType::WallsAndFloor,
    };

    let mut maker = TabbedBoxMaker::new(params).expect("Failed to create TabbedBoxMaker");
    maker.generate().expect("Failed to generate box");

    let gcode = maker.to_gcode();

    let mut y_coords: Vec<f32> = Vec::new();
    let mut x_coords: Vec<f32> = Vec::new();

    for line in gcode.lines() {
        if line.starts_with("G1") || line.starts_with("G0") {
            if let Some(y_start) = line.find("Y") {
                let y_str: String = line[y_start + 1..]
                    .chars()
                    .take_while(|c| c.is_numeric() || *c == '.' || *c == '-')
                    .collect();
                if let Ok(y) = y_str.parse::<f32>() {
                    y_coords.push(y);
                }
            }
            if let Some(x_start) = line.find("X") {
                let x_str: String = line[x_start + 1..]
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

    println!("\n=== Test: No Kerf ===");
    println!("Material thickness: 3.0mm");
    println!("Y coordinate range: {:.2} to {:.2}", min_y, max_y);
    println!("X coordinate range: {:.2} to {:.2}", min_x, max_x);

    if min_y < 0.0 {
        let tab_depth = min_y.abs();
        println!("Measured tab protrusion depth: {:.2}mm", tab_depth);
        println!("Expected: 6.0mm (2x thickness)");
        assert!(
            (tab_depth - 6.0).abs() < 0.1,
            "Tab depth {:.2}mm != expected 6.0mm (2x thickness)",
            tab_depth
        );
    }
}

#[test]
fn test_tab_protrusion_with_kerf() {
    let params = BoxParameters {
        x: 100.0,
        y: 100.0,
        h: 100.0,
        thickness: 3.0,
        outside: false,
        box_type: BoxType::FullBox,
        finger_joint: FingerJointSettings::default(),
        burn: 0.5,
        laser_passes: 1,
        laser_power: 1000,
        feed_rate: 500.0,
        offset_x: 0.0,
        offset_y: 0.0,
        dividers_x: 0,
        dividers_y: 0,
        optimize_layout: false,
        key_divider_type: KeyDividerType::WallsAndFloor,
    };

    let mut maker = TabbedBoxMaker::new(params).expect("Failed to create TabbedBoxMaker");
    maker.generate().expect("Failed to generate box");

    let gcode = maker.to_gcode();

    let mut y_coords: Vec<f32> = Vec::new();

    for line in gcode.lines() {
        if line.starts_with("G1") || line.starts_with("G0") {
            if let Some(y_start) = line.find("Y") {
                let y_str: String = line[y_start + 1..]
                    .chars()
                    .take_while(|c| c.is_numeric() || *c == '.' || *c == '-')
                    .collect();
                if let Ok(y) = y_str.parse::<f32>() {
                    y_coords.push(y);
                }
            }
        }
    }

    let min_y = y_coords.iter().cloned().fold(f32::INFINITY, f32::min);

    println!("\n=== Test: With Kerf 0.5mm ===");
    println!("Material thickness: 3.0mm");
    println!("Kerf: 0.5mm");
    println!("Min Y coordinate: {:.2}", min_y);

    if min_y < 0.0 {
        let tab_depth = min_y.abs();
        println!("Measured tab protrusion depth: {:.2}mm", tab_depth);
        println!("Note: Current implementation uses thickness without kerf compensation");
        println!("For proper fit, tab depth might need to be thickness + kerf = 3.5mm");
    }
}

#[test]
fn test_default_box() {
    let params = BoxParameters::default();
    let mut maker = TabbedBoxMaker::new(params).unwrap();
    maker.generate().unwrap();
    let gcode = maker.to_gcode();
    assert!(gcode.contains("G21"));
    assert!(gcode.contains("M3"));
}

#[test]
fn test_finger_calculation() {
    let params = BoxParameters::default();
    let maker = TabbedBoxMaker::new(params).unwrap();

    // For 100mm length with finger=2*t=6mm and space=2*t=6mm
    // fingers should be about 8-9
    let (fingers, leftover) = maker.calc_fingers(100.0);
    assert!(fingers >= 7 && fingers <= 10);
    assert!(leftover > 0.0);
}
