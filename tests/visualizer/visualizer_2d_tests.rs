//! 2D Visualizer module integration tests

use gcodekit4::visualizer::{GCodeCommand, Point2D, Visualizer2D};

#[test]
fn test_point2d_creation() {
    let point = Point2D::new(5.0, 10.0);
    assert_eq!(point.x, 5.0);
    assert_eq!(point.y, 10.0);
}

#[test]
fn test_visualizer2d_creation() {
    let viz = Visualizer2D::new();
    assert_eq!(viz.zoom_scale, 1.0);
    assert_eq!(viz.x_offset, 0.0);
    assert_eq!(viz.y_offset, 0.0);
    assert!(!viz.show_grid);
    assert_eq!(viz.scale_factor, 1.0);
}

#[test]
fn test_visualizer2d_default() {
    let viz = Visualizer2D::default();
    assert_eq!(viz.zoom_scale, 1.0);
}

#[test]
fn test_grid_toggle() {
    let mut viz = Visualizer2D::new();
    assert!(!viz.is_grid_visible());

    viz.toggle_grid();
    assert!(viz.is_grid_visible());

    viz.toggle_grid();
    assert!(!viz.is_grid_visible());
}

#[test]
fn test_grid_set_visible() {
    let mut viz = Visualizer2D::new();

    viz.set_grid_visible(true);
    assert!(viz.is_grid_visible());

    viz.set_grid_visible(false);
    assert!(!viz.is_grid_visible());
}

#[test]
fn test_scale_factor() {
    let mut viz = Visualizer2D::new();
    assert_eq!(viz.get_scale_factor(), 1.0);

    viz.set_scale_factor(2.5);
    assert_eq!(viz.get_scale_factor(), 2.5);

    viz.set_scale_factor(0.05);
    assert_eq!(viz.get_scale_factor(), 0.1);

    viz.set_scale_factor(150.0);
    assert_eq!(viz.get_scale_factor(), 100.0);
}

#[test]
fn test_parse_simple_gcode() {
    let mut viz = Visualizer2D::new();
    let gcode = "G1 X10 Y10\nG1 X20 Y20";

    viz.parse_gcode(gcode);

    assert_eq!(viz.get_command_count(), 2);
}

#[test]
fn test_parse_gcode_with_comments() {
    let mut viz = Visualizer2D::new();
    let gcode = "; This is a comment\nG1 X10 Y10\n(another comment)\nG1 X20 Y20";

    viz.parse_gcode(gcode);

    assert_eq!(viz.get_command_count(), 2);
}

#[test]
fn test_parse_gcode_rapid_move() {
    let mut viz = Visualizer2D::new();
    let gcode = "G0 X10 Y10";

    viz.parse_gcode(gcode);

    assert_eq!(viz.get_command_count(), 1);
    if let GCodeCommand::Move { rapid, .. } = &viz.commands[0] {
        assert!(rapid);
    } else {
        panic!("Expected Move command");
    }
}

#[test]
fn test_parse_gcode_feed_move() {
    let mut viz = Visualizer2D::new();
    let gcode = "G1 X10 Y10";

    viz.parse_gcode(gcode);

    assert_eq!(viz.get_command_count(), 1);
    if let GCodeCommand::Move { rapid, .. } = &viz.commands[0] {
        assert!(!rapid);
    } else {
        panic!("Expected Move command");
    }
}

#[test]
fn test_parse_gcode_arc_clockwise() {
    let mut viz = Visualizer2D::new();
    let gcode = "G2 X10 Y10 I5 J5";

    viz.parse_gcode(gcode);

    assert_eq!(viz.get_command_count(), 1);
    if let GCodeCommand::Arc { clockwise, .. } = &viz.commands[0] {
        assert!(clockwise);
    } else {
        panic!("Expected Arc command");
    }
}

#[test]
fn test_parse_gcode_arc_counterclockwise() {
    let mut viz = Visualizer2D::new();
    let gcode = "G3 X10 Y10 I5 J5";

    viz.parse_gcode(gcode);

    assert_eq!(viz.get_command_count(), 1);
    if let GCodeCommand::Arc { clockwise, .. } = &viz.commands[0] {
        assert!(!clockwise);
    } else {
        panic!("Expected Arc command");
    }
}

#[test]
fn test_get_bounds() {
    let mut viz = Visualizer2D::new();
    let gcode = "G1 X10 Y10\nG1 X20 Y20";

    viz.parse_gcode(gcode);

    let (min_x, max_x, min_y, max_y) = viz.get_bounds();
    assert!(min_x <= 10.0);
    assert!(max_x >= 20.0);
    assert!(min_y <= 10.0);
    assert!(max_y >= 20.0);
}

#[test]
fn test_zoom_in() {
    let mut viz = Visualizer2D::new();
    let initial_zoom = viz.zoom_scale;

    viz.zoom_in();

    assert!(viz.zoom_scale > initial_zoom);
}

#[test]
fn test_zoom_out() {
    let mut viz = Visualizer2D::new();
    let initial_zoom = viz.zoom_scale;

    viz.zoom_out();

    assert!(viz.zoom_scale < initial_zoom);
}

#[test]
fn test_zoom_limits() {
    let mut viz = Visualizer2D::new();

    for _ in 0..100 {
        viz.zoom_in();
    }
    assert!(viz.zoom_scale <= 5.0);

    for _ in 0..100 {
        viz.zoom_out();
    }
    assert!(viz.zoom_scale >= 0.1);
}

#[test]
fn test_reset_zoom() {
    let mut viz = Visualizer2D::new();

    viz.zoom_in();
    viz.zoom_in();
    viz.reset_zoom();

    assert_eq!(viz.zoom_scale, 1.0);
}

#[test]
fn test_get_zoom_percent() {
    let mut viz = Visualizer2D::new();
    assert_eq!(viz.get_zoom_percent(), 100);

    viz.zoom_in();
    assert!(viz.get_zoom_percent() > 100);

    viz.reset_zoom();
    viz.zoom_out();
    assert!(viz.get_zoom_percent() < 100);
}

#[test]
fn test_pan_operations() {
    let mut viz = Visualizer2D::new();
    let canvas_width = 800.0;
    let canvas_height = 600.0;

    viz.pan_right(canvas_width);
    assert!(viz.x_offset > 0.0);

    viz.reset_pan();
    assert_eq!(viz.x_offset, 0.0);

    viz.pan_left(canvas_width);
    assert!(viz.x_offset < 0.0);

    viz.reset_pan();
    viz.pan_up(canvas_height);
    assert!(viz.y_offset > 0.0);

    viz.reset_pan();
    viz.pan_down(canvas_height);
    assert!(viz.y_offset < 0.0);
}

#[test]
fn test_reset_pan() {
    let mut viz = Visualizer2D::new();

    viz.pan_right(800.0);
    viz.pan_up(600.0);
    viz.reset_pan();

    assert_eq!(viz.x_offset, 0.0);
    assert_eq!(viz.y_offset, 0.0);
}

#[test]
fn test_fit_to_view() {
    let mut viz = Visualizer2D::new();
    let gcode = "G1 X10 Y10\nG1 X50 Y50";
    viz.parse_gcode(gcode);

    viz.fit_to_view(800.0, 600.0);

    assert!(viz.zoom_scale > 0.0);
}

#[test]
fn test_fit_to_view_empty() {
    let mut viz = Visualizer2D::new();

    viz.fit_to_view(800.0, 600.0);

    assert_eq!(viz.zoom_scale, 1.0);
    assert_eq!(viz.x_offset, 0.0);
    assert_eq!(viz.y_offset, 0.0);
}

#[test]
fn test_render_empty() {
    let viz = Visualizer2D::new();
    let img_data = viz.render(800, 600);

    assert!(!img_data.is_empty());
}

#[test]
fn test_render_with_gcode() {
    let mut viz = Visualizer2D::new();
    let gcode = "G1 X10 Y10\nG1 X20 Y20";
    viz.parse_gcode(gcode);

    let img_data = viz.render(800, 600);

    assert!(!img_data.is_empty());
}

#[test]
fn test_render_with_grid() {
    let mut viz = Visualizer2D::new();
    let gcode = "G1 X10 Y10\nG1 X20 Y20";
    viz.parse_gcode(gcode);
    viz.set_grid_visible(true);

    let img_data = viz.render(800, 600);

    assert!(!img_data.is_empty());
}

#[test]
fn test_complex_gcode_parsing() {
    let mut viz = Visualizer2D::new();
    let gcode = r#"
        G0 X0 Y0
        G1 X10 Y10
        G2 X20 Y10 I10 J0
        G3 X30 Y20 I5 J5
        G0 X0 Y0
    "#;

    viz.parse_gcode(gcode);

    assert!(viz.get_command_count() > 0);
    let (min_x, max_x, min_y, max_y) = viz.get_bounds();
    assert!(min_x < max_x);
    assert!(min_y < max_y);
}
