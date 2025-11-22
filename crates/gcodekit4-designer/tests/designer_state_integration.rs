//! Designer state manager integration tests

use gcodekit4_designer::{DesignerState, DrawingMode, Point};

#[test]
fn test_designer_state_complete_workflow() {
    let mut state = DesignerState::new();

    // Add some shapes
    state.canvas.add_rectangle(0.0, 0.0, 50.0, 30.0);
    state
        .canvas
        .add_circle(Point::new(100.0, 100.0), 20.0);
    state.canvas.add_line(
        Point::new(0.0, 0.0),
        Point::new(100.0, 100.0),
    );

    assert_eq!(state.canvas.shapes().len(), 3);

    // Test drawing modes
    state.set_mode(0); // Select
    state.set_mode(1); // Rectangle
    assert_eq!(state.canvas.mode(), DrawingMode::Rectangle);

    // Test zoom
    let initial_zoom = state.canvas.zoom();
    state.zoom_in();
    assert!(state.canvas.zoom() > initial_zoom);

    // Test selection
    state.canvas.select_at(&Point::new(25.0, 15.0));
    assert!(state.canvas.selected_id().is_some());

    // Test deletion
    state.delete_selected();
    assert_eq!(state.canvas.shapes().len(), 2);

    // Test G-code generation
    let gcode = state.generate_gcode();
    assert!(!gcode.is_empty());
    assert!(state.gcode_generated);
    assert!(gcode.contains("G00")); // Should have rapid moves
    assert!(gcode.contains("G01")); // Should have linear moves

    // Test tool parameters
    state.set_feed_rate(200.0);
    state.set_spindle_speed(5000);
    state.set_tool_diameter(4.0);
    state.set_cut_depth(-3.0);

    // Generate again with new parameters
    let gcode2 = state.generate_gcode();
    assert!(!gcode2.is_empty());

    // Test clear
    state.clear_canvas();
    assert_eq!(state.canvas.shapes().len(), 0);
    assert!(!state.gcode_generated);
}

#[test]
fn test_designer_state_rectangle_workflow() {
    let mut state = DesignerState::new();

    // Design a rectangle
    state.set_mode(1); // Rectangle mode
    state.canvas.add_rectangle(10.0, 10.0, 100.0, 50.0);

    assert_eq!(state.canvas.shapes().len(), 1);

    // Generate G-code
    let gcode = state.generate_gcode();
    assert!(state.gcode_generated);
    assert!(gcode.contains("G90"));
    assert!(gcode.contains("G21"));
    assert!(gcode.contains("M3")); // Spindle on
    assert!(gcode.contains("M5")); // Spindle off
}

#[test]
fn test_designer_state_multi_shape_design() {
    let mut state = DesignerState::new();

    // Create a complex design
    for i in 0..5 {
        state
            .canvas
            .add_rectangle((i as f64) * 20.0, 0.0, 15.0, 15.0);
    }

    assert_eq!(state.canvas.shapes().len(), 5);

    // Generate G-code for all shapes
    let gcode = state.generate_gcode();
    assert!(state.gcode_generated);
    assert!(!gcode.is_empty());

    // Verify G-code has multiple sections
    let g00_count = gcode.matches("G00").count();
    let g01_count = gcode.matches("G01").count();

    assert!(g00_count > 5); // At least one rapid move per shape
    assert!(g01_count > 5); // At least one linear move per shape
}
