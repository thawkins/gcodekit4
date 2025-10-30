//! Integration tests for mouse event coordinate mapping in designer

use gcodekit4::designer::{Canvas, Point};

#[test]
fn test_pixel_click_to_world_selection() {
    let mut canvas = Canvas::new();
    
    // Add a rectangle at world coordinates (100, 100) to (200, 200)
    canvas.add_rectangle(100.0, 100.0, 100.0, 100.0);
    
    // At 1:1 zoom with no pan, pixel (150, 150) should map to world (150, 150)
    let click_pixel = Point::new(150.0, 150.0);
    let world_point = canvas.pixel_to_world(click_pixel.x, click_pixel.y);
    
    // Click at world (150, 150) - should select the rectangle
    canvas.select_at(&world_point);
    assert_eq!(canvas.selected_id(), Some(1));
}

#[test]
fn test_pixel_click_with_zoom() {
    let mut canvas = Canvas::new();
    
    // Add a rectangle
    canvas.add_rectangle(100.0, 100.0, 100.0, 100.0);
    
    // Zoom in 2x
    canvas.set_zoom(2.0);
    
    // At 2x zoom, pixel (200, 200) maps to world (100, 100)
    // This is the top-left corner of the rectangle
    let world_point = canvas.pixel_to_world(200.0, 200.0);
    assert!((world_point.x - 100.0).abs() < 0.01);
    assert!((world_point.y - 100.0).abs() < 0.01);
    
    // Now the click should select the shape
    canvas.select_at(&world_point);
    assert_eq!(canvas.selected_id(), Some(1));
}

#[test]
fn test_pixel_click_with_pan() {
    let mut canvas = Canvas::new();
    
    // Add a rectangle
    canvas.add_rectangle(100.0, 100.0, 100.0, 100.0);
    
    // Pan by (50, 75)
    canvas.set_pan(50.0, 75.0);
    
    // At 1:1 zoom with pan (50, 75), pixel (150, 175) maps to world (100, 100)
    let world_point = canvas.pixel_to_world(150.0, 175.0);
    assert!((world_point.x - 100.0).abs() < 0.01);
    assert!((world_point.y - 100.0).abs() < 0.01);
    
    // Now the click should select the shape
    canvas.select_at(&world_point);
    assert_eq!(canvas.selected_id(), Some(1));
}

#[test]
fn test_pixel_click_with_zoom_and_pan() {
    let mut canvas = Canvas::new();
    
    // Add a rectangle
    canvas.add_rectangle(100.0, 100.0, 100.0, 100.0);
    
    // Zoom 2x and pan (100, 100)
    canvas.set_zoom(2.0);
    canvas.set_pan(100.0, 100.0);
    
    // Pixel (200, 200) should map to world (50, 50)
    let world_point = canvas.pixel_to_world(200.0, 200.0);
    assert!((world_point.x - 50.0).abs() < 0.01);
    assert!((world_point.y - 50.0).abs() < 0.01);
    
    // This should NOT select the rectangle (which is at 100-200)
    canvas.select_at(&world_point);
    assert_eq!(canvas.selected_id(), None);
}

#[test]
fn test_drag_delta_pixel_to_world() {
    let mut canvas = Canvas::new();
    
    // Add a rectangle
    canvas.add_rectangle(100.0, 100.0, 100.0, 100.0);
    canvas.select_at(&Point::new(150.0, 150.0));
    
    // At 1:1 zoom, pixel delta (10, 10) = world delta (10, 10)
    let viewport = canvas.viewport();
    let pixel_dx = 10.0;
    let pixel_dy = 10.0;
    let world_dx = pixel_dx / viewport.zoom();
    let world_dy = pixel_dy / viewport.zoom();
    
    assert!((world_dx - 10.0).abs() < 0.01);
    assert!((world_dy - 10.0).abs() < 0.01);
}

#[test]
fn test_drag_delta_with_zoom() {
    let mut canvas = Canvas::new();
    
    // Add and select a rectangle
    canvas.add_rectangle(100.0, 100.0, 100.0, 100.0);
    canvas.select_at(&Point::new(150.0, 150.0));
    
    // Zoom 2x
    canvas.set_zoom(2.0);
    
    // At 2x zoom, pixel delta (10, 10) = world delta (5, 5)
    let viewport = canvas.viewport();
    let pixel_dx = 10.0;
    let pixel_dy = 10.0;
    let world_dx = pixel_dx / viewport.zoom();
    let world_dy = pixel_dy / viewport.zoom();
    
    assert!((world_dx - 5.0).abs() < 0.01);
    assert!((world_dy - 5.0).abs() < 0.01);
    
    // Move by this delta
    canvas.move_selected(world_dx, world_dy);
    
    // Rectangle should now be at (105, 105) to (205, 205)
    if let Some(selected_id) = canvas.selected_id() {
        for obj in canvas.shapes() {
            if obj.id == selected_id {
                let (x1, y1, _x2, _y2) = obj.shape.bounding_box();
                assert!((x1 - 105.0).abs() < 0.01);
                assert!((y1 - 105.0).abs() < 0.01);
            }
        }
    }
}

#[test]
fn test_handle_detection_pixel_to_world() {
    let mut canvas = Canvas::new();
    
    // Add and select a rectangle
    canvas.add_rectangle(100.0, 100.0, 100.0, 100.0);
    canvas.select_at(&Point::new(150.0, 150.0));
    
    // At 1:1 zoom, pixel (100, 100) should map to world (100, 100) - top-left handle
    let world_point = canvas.pixel_to_world(100.0, 100.0);
    assert!((world_point.x - 100.0).abs() < 0.01);
    assert!((world_point.y - 100.0).abs() < 0.01);
}

#[test]
fn test_handle_detection_with_zoom() {
    let mut canvas = Canvas::new();
    
    // Add and select a rectangle at (100, 100) with size (100, 100)
    canvas.add_rectangle(100.0, 100.0, 100.0, 100.0);
    canvas.select_at(&Point::new(150.0, 150.0));
    
    // Zoom 2x
    canvas.set_zoom(2.0);
    
    // At 2x zoom, the world (100, 100) is now at pixel (200, 200)
    let (pixel_x, pixel_y) = canvas.world_to_pixel(100.0, 100.0);
    assert!((pixel_x - 200.0).abs() < 0.01);
    assert!((pixel_y - 200.0).abs() < 0.01);
    
    // Clicking on pixel (200, 200) should map back to world (100, 100)
    let world_point = canvas.pixel_to_world(pixel_x, pixel_y);
    assert!((world_point.x - 100.0).abs() < 0.01);
    assert!((world_point.y - 100.0).abs() < 0.01);
}

#[test]
fn test_selection_after_zoom_pan_sequence() {
    let mut canvas = Canvas::new();
    
    // Add shapes
    canvas.add_rectangle(50.0, 50.0, 100.0, 100.0);
    canvas.add_circle(Point::new(300.0, 300.0), 50.0);
    
    // Zoom and pan
    canvas.zoom_in();
    canvas.pan_by(50.0, 50.0);
    
    // Try to select the first rectangle at pixel coordinates
    // First, figure out what pixel coordinates map to the rectangle center (100, 100)
    let (pixel_x, pixel_y) = canvas.world_to_pixel(100.0, 100.0);
    
    // Click at that pixel position
    let world_point = canvas.pixel_to_world(pixel_x, pixel_y);
    canvas.select_at(&world_point);
    
    // Should have selected the rectangle
    assert_eq!(canvas.selected_id(), Some(1));
}

#[test]
fn test_multiple_shapes_selection_with_zoom() {
    let mut canvas = Canvas::new();
    
    // Add multiple shapes
    canvas.add_rectangle(50.0, 50.0, 100.0, 100.0);
    canvas.add_rectangle(200.0, 200.0, 100.0, 100.0);
    
    // Zoom in
    canvas.set_zoom(2.0);
    
    // Click on second rectangle at world (250, 250)
    let (pixel_x, pixel_y) = canvas.world_to_pixel(250.0, 250.0);
    let world_point = canvas.pixel_to_world(pixel_x, pixel_y);
    canvas.select_at(&world_point);
    
    // Should have selected the second rectangle
    assert_eq!(canvas.selected_id(), Some(2));
    
    // Click on first rectangle at world (100, 100)
    let (pixel_x, pixel_y) = canvas.world_to_pixel(100.0, 100.0);
    let world_point = canvas.pixel_to_world(pixel_x, pixel_y);
    canvas.select_at(&world_point);
    
    // Should have selected the first rectangle
    assert_eq!(canvas.selected_id(), Some(1));
}

#[test]
fn test_handle_size_scales_with_zoom() {
    let mut canvas = Canvas::new();
    
    // Add and select a rectangle
    canvas.add_rectangle(100.0, 100.0, 100.0, 100.0);
    canvas.select_at(&Point::new(150.0, 150.0));
    
    // Get viewport
    let viewport = canvas.viewport();
    
    // At 1:1 zoom, handle size is 8.0 / 1.0 = 8.0 world units
    let mut handle_size = 8.0 / viewport.zoom();
    assert!((handle_size - 8.0).abs() < 0.01);
    
    // Zoom 2x
    canvas.set_zoom(2.0);
    let viewport = canvas.viewport();
    
    // At 2x zoom, handle size is 8.0 / 2.0 = 4.0 world units
    handle_size = 8.0 / viewport.zoom();
    assert!((handle_size - 4.0).abs() < 0.01);
}

#[test]
fn test_click_miss_with_zoom() {
    let mut canvas = Canvas::new();
    
    // Add a rectangle at (100, 100) to (200, 200)
    canvas.add_rectangle(100.0, 100.0, 100.0, 100.0);
    
    // Zoom 2x
    canvas.set_zoom(2.0);
    
    // Click outside the rectangle at world (50, 50)
    let world_point = canvas.pixel_to_world(100.0, 100.0);
    canvas.select_at(&world_point);
    
    // Should not have selected anything
    assert_eq!(canvas.selected_id(), None);
}
