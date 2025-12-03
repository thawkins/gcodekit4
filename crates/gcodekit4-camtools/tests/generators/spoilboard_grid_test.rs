use gcodekit4_camtools::spoilboard_grid::{SpoilboardGridGenerator, SpoilboardGridParameters};

#[test]
fn test_spoilboard_grid_generation_metric() {
    let params = SpoilboardGridParameters {
        width: 100.0,
        height: 100.0,
        grid_spacing: 10.0,
        feed_rate: 1000.0,
        laser_power: 500.0,
        laser_mode: "M3".to_string(),
    };

    let generator = SpoilboardGridGenerator::new(params);
    let gcode = generator.generate().unwrap();

    assert!(gcode.contains("G21 ; Set units to millimeters"));
    assert!(gcode.contains("X100.0"));
    assert!(gcode.contains("Y100.0"));
    
    // Check for specific line coordinates (Zigzag pattern)
    // X=0: Up (Start Y=0)
    assert!(gcode.contains("G0 X0.000 Y0.000"));
    // X=10: Down (Start Y=100)
    assert!(gcode.contains("G0 X10.000 Y100.000"));
    // X=20: Up (Start Y=0)
    assert!(gcode.contains("G0 X20.000 Y0.000"));
}

#[test]
fn test_spoilboard_grid_generation_imperial_converted() {
    // Simulate 4x4 inch grid with 1 inch spacing
    // 4 inches = 101.6 mm
    // 1 inch = 25.4 mm
    let params = SpoilboardGridParameters {
        width: 101.6,
        height: 101.6,
        grid_spacing: 25.4,
        feed_rate: 1000.0,
        laser_power: 500.0,
        laser_mode: "M4".to_string(),
    };

    let generator = SpoilboardGridGenerator::new(params);
    let gcode = generator.generate().unwrap();

    assert!(gcode.contains("G21 ; Set units to millimeters")); // Always generates metric G-code
    
    // Check dimensions in comment
    assert!(gcode.contains("; Dimensions: 101.6 x 101.6 mm"));
    
    // Check for 1 inch spacing (25.4 mm) with Zigzag pattern
    // X=0: Up (Start Y=0)
    assert!(gcode.contains("G0 X0.000 Y0.000"));
    // X=25.4: Down (Start Y=101.6)
    assert!(gcode.contains("G0 X25.400 Y101.600"));
    // X=50.8: Up (Start Y=0)
    assert!(gcode.contains("G0 X50.800 Y0.000"));
    // X=76.2: Down (Start Y=101.6)
    assert!(gcode.contains("G0 X76.200 Y101.600"));
    // X=101.6: Up (Start Y=0)
    assert!(gcode.contains("G0 X101.600 Y0.000"));
}

#[test]
fn test_spoilboard_grid_generation_fractional_inch_converted() {
    // Simulate 0.5 inch spacing (12.7 mm)
    let params = SpoilboardGridParameters {
        width: 25.4, // 1 inch
        height: 25.4, // 1 inch
        grid_spacing: 12.7, // 0.5 inch
        feed_rate: 1000.0,
        laser_power: 500.0,
        laser_mode: "M3".to_string(),
    };

    let generator = SpoilboardGridGenerator::new(params);
    let gcode = generator.generate().unwrap();

    // Should have lines at 0, 12.7, 25.4
    // X=0: Up (Start Y=0)
    assert!(gcode.contains("G0 X0.000 Y0.000"));
    // X=12.7: Down (Start Y=25.4)
    assert!(gcode.contains("G0 X12.700 Y25.400"));
    // X=25.4: Up (Start Y=0)
    assert!(gcode.contains("G0 X25.400 Y0.000"));
}
