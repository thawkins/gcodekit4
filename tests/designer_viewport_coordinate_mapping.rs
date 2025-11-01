//! Integration tests for Designer Viewport and Coordinate Transformations

use gcodekit4::designer::{Canvas, Point, Viewport};

#[test]
fn test_viewport_pixel_to_world_conversion() {
    let vp = Viewport::new(800.0, 600.0);

    // No transformation should give 1:1 mapping
    let world = vp.pixel_to_world(100.0, 200.0);
    assert_eq!(world.x, 100.0);
    assert_eq!(world.y, 200.0);
}

#[test]
fn test_viewport_world_to_pixel_conversion() {
    let vp = Viewport::new(800.0, 600.0);

    // No transformation should give 1:1 mapping
    let (pixel_x, pixel_y) = vp.world_to_pixel(100.0, 200.0);
    assert_eq!(pixel_x, 100.0);
    assert_eq!(pixel_y, 200.0);
}

#[test]
fn test_viewport_roundtrip_no_transform() {
    let vp = Viewport::new(800.0, 600.0);
    let original = Point::new(50.0, 75.0);

    let (pixel_x, pixel_y) = vp.world_to_pixel(original.x, original.y);
    let roundtrip = vp.pixel_to_world(pixel_x, pixel_y);

    assert!((roundtrip.x - original.x).abs() < 0.001);
    assert!((roundtrip.y - original.y).abs() < 0.001);
}

#[test]
fn test_viewport_zoom_scaling() {
    let mut vp = Viewport::new(800.0, 600.0);
    vp.set_zoom(2.0);

    // With 2x zoom, 100 pixels should equal 50 world units
    let world = vp.pixel_to_world(100.0, 0.0);
    assert!((world.x - 50.0).abs() < 0.001);
}

#[test]
fn test_viewport_pan_offset() {
    let mut vp = Viewport::new(800.0, 600.0);
    vp.set_pan(50.0, 75.0);

    // Pan should offset the mapping
    let world = vp.pixel_to_world(100.0, 100.0);
    assert!((world.x - 50.0).abs() < 0.001);
    assert!((world.y - 25.0).abs() < 0.001);
}

#[test]
fn test_viewport_zoom_and_pan_combined() {
    let mut vp = Viewport::new(800.0, 600.0);
    vp.set_zoom(2.0);
    vp.set_pan(100.0, 150.0);

    let world = vp.pixel_to_world(200.0, 250.0);
    assert!((world.x - 50.0).abs() < 0.001);
    assert!((world.y - 50.0).abs() < 0.001);
}

#[test]
fn test_viewport_zoom_in_out() {
    let mut vp = Viewport::new(800.0, 600.0);
    let initial_zoom = vp.zoom();

    vp.zoom_in();
    let zoomed_in = vp.zoom();
    assert!(zoomed_in > initial_zoom);

    vp.zoom_out();
    let zoomed_out = vp.zoom();
    assert!((zoomed_out - initial_zoom).abs() < 0.001);
}

#[test]
fn test_viewport_center_on_point() {
    let mut vp = Viewport::new(800.0, 600.0);
    vp.center_on(100.0, 200.0);

    // Center of screen should map to the centered point
    let world = vp.pixel_to_world(400.0, 300.0);
    assert!((world.x - 100.0).abs() < 0.1);
    assert!((world.y - 200.0).abs() < 0.1);
}

#[test]
fn test_viewport_fit_to_bounds() {
    let mut vp = Viewport::new(800.0, 600.0);
    vp.fit_to_bounds(0.0, 0.0, 100.0, 100.0, 0.1);

    // After fitting, should have zoomed in
    assert!(vp.zoom() > 1.0);
}

#[test]
fn test_canvas_with_viewport_zoom() {
    let mut canvas = Canvas::new();

    canvas.set_zoom(2.0);
    assert_eq!(canvas.zoom(), 2.0);

    let world = canvas.pixel_to_world(100.0, 100.0);
    assert!((world.x - 50.0).abs() < 0.001);
}

#[test]
fn test_canvas_with_viewport_pan() {
    let mut canvas = Canvas::new();

    canvas.set_pan(50.0, 75.0);
    assert_eq!(canvas.pan_x(), 50.0);
    assert_eq!(canvas.pan_y(), 75.0);

    let world = canvas.pixel_to_world(100.0, 100.0);
    assert!((world.x - 50.0).abs() < 0.001);
    assert!((world.y - 25.0).abs() < 0.001);
}

#[test]
fn test_canvas_pan_by() {
    let mut canvas = Canvas::new();

    canvas.pan_by(50.0, 75.0);
    assert_eq!(canvas.pan_x(), 50.0);
    assert_eq!(canvas.pan_y(), 75.0);

    canvas.pan_by(25.0, 25.0);
    assert_eq!(canvas.pan_x(), 75.0);
    assert_eq!(canvas.pan_y(), 100.0);
}

#[test]
fn test_canvas_zoom_controls() {
    let mut canvas = Canvas::new();
    let initial = canvas.zoom();

    canvas.zoom_in();
    assert!(canvas.zoom() > initial);

    canvas.zoom_out();
    assert!((canvas.zoom() - initial).abs() < 0.001);
}

#[test]
fn test_canvas_pan_offset_compatibility() {
    let mut canvas = Canvas::new();

    canvas.set_pan(100.0, 200.0);
    let (pan_x, pan_y) = canvas.pan_offset();

    assert_eq!(pan_x, 100.0);
    assert_eq!(pan_y, 200.0);
}

#[test]
fn test_canvas_pan_compatibility() {
    let mut canvas = Canvas::new();

    canvas.pan(50.0, 75.0);
    assert_eq!(canvas.pan_x(), 50.0);
    assert_eq!(canvas.pan_y(), 75.0);
}

#[test]
fn test_canvas_reset_view() {
    let mut canvas = Canvas::new();

    canvas.set_zoom(2.5);
    canvas.set_pan(100.0, 200.0);

    canvas.reset_view();

    assert_eq!(canvas.zoom(), 1.0);
    assert_eq!(canvas.pan_x(), 0.0);
    assert_eq!(canvas.pan_y(), 0.0);
}

#[test]
fn test_canvas_fit_all_shapes() {
    let mut canvas = Canvas::new();

    // Add some shapes
    canvas.add_rectangle(10.0, 20.0, 100.0, 80.0);
    canvas.add_circle(Point::new(150.0, 150.0), 50.0);

    // Fit all shapes
    canvas.fit_all_shapes();

    // Should have zoomed in
    assert!(canvas.zoom() > 1.0);
}

#[test]
fn test_coordinate_mapping_with_shapes() {
    let mut canvas = Canvas::new();

    // Add a rectangle at world coordinates (0, 0) to (100, 100)
    canvas.add_rectangle(0.0, 0.0, 100.0, 100.0);

    // Zoom in by 2x
    canvas.set_zoom(2.0);

    // The shape's world coordinates should remain the same
    // but pixel coordinates will be scaled
    let (pixel_x, pixel_y) = canvas.world_to_pixel(100.0, 100.0);

    // At 2x zoom, (100, 100) in world should be (200, 200) in pixels (without pan)
    assert_eq!(pixel_x, 200.0);
    assert_eq!(pixel_y, 200.0);
}

#[test]
fn test_pan_with_zoom_interaction() {
    let mut canvas = Canvas::new();

    // Zoom in
    canvas.set_zoom(2.0);

    // Pan
    canvas.set_pan(100.0, 100.0);

    // World to pixel should respect both zoom and pan
    let (pixel_x, pixel_y) = canvas.world_to_pixel(50.0, 50.0);
    assert_eq!(pixel_x, 200.0); // 50 * 2 + 100
    assert_eq!(pixel_y, 200.0); // 50 * 2 + 100

    // Pixel to world should reverse both transformations
    let world = canvas.pixel_to_world(200.0, 200.0);
    assert!((world.x - 50.0).abs() < 0.001);
    assert!((world.y - 50.0).abs() < 0.001);
}

#[test]
fn test_zoom_constraint_limits() {
    let mut canvas = Canvas::new();

    // Try to zoom too far in
    canvas.set_zoom(100.0);
    assert!(canvas.zoom() < 100.0);
    assert!(canvas.zoom() <= 10.0);

    // Try to zoom too far out
    canvas.set_zoom(0.01);
    assert!(canvas.zoom() > 0.01);
    assert!(canvas.zoom() >= 0.1);
}

#[test]
fn test_viewport_center_on_with_zoom() {
    let mut vp = Viewport::new(800.0, 600.0);
    vp.set_zoom(2.0);
    vp.center_on(100.0, 100.0);

    let world = vp.pixel_to_world(400.0, 300.0);
    assert!((world.x - 100.0).abs() < 0.1);
    assert!((world.y - 100.0).abs() < 0.1);
}

#[test]
fn test_canvas_size_aware_viewport() {
    let mut canvas = Canvas::new();

    // Canvas starts with default size
    let vp = canvas.viewport();
    assert!(vp.zoom() > 0.0);

    // Should be able to fit content
    canvas.add_rectangle(0.0, 0.0, 50.0, 50.0);
    canvas.fit_all_shapes();

    let vp = canvas.viewport();
    assert!(vp.zoom() > 1.0);
}
