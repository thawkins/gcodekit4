//! SVG-based canvas renderer for designer shapes
//! Renders shapes as SVG path commands for display using Slint Path elements
//! 
//! Features:
//! - Bright yellow crosshair at world origin (0,0)
//! - Shape rendering with selection indicators
//! - Viewport-based coordinate transformation

use crate::designer::Canvas;

/// Render crosshair at origin (0,0) as SVG path
pub fn render_crosshair(canvas: &Canvas, width: u32, height: u32) -> String {
    let viewport = canvas.viewport();
    
    // Convert world origin to screen coordinates
    let (origin_x, origin_y) = viewport.world_to_pixel(0.0, 0.0);
    
    let mut path = String::new();
    
    // Horizontal line (X axis) - only check if within reasonable bounds
    // Allow a small buffer outside canvas for visibility
    if origin_y >= -10.0 && origin_y <= (height as f64 + 10.0) {
        path.push_str(&format!("M 0 {} L {} {} ", origin_y, width, origin_y));
    }
    
    // Vertical line (Y axis) - only check if within reasonable bounds
    if origin_x >= -10.0 && origin_x <= (width as f64 + 10.0) {
        path.push_str(&format!("M {} 0 L {} {} ", origin_x, origin_x, height));
    }
    
    path
}

/// Render all shapes as SVG path
pub fn render_shapes(canvas: &Canvas, _width: u32, _height: u32) -> String {
    let viewport = canvas.viewport();
    let mut path = String::new();
    
    for shape_obj in canvas.shapes() {
        // Skip selected shapes - they'll be rendered separately
        if shape_obj.selected {
            continue;
        }
        
        let shape_path = render_shape_trait(&*shape_obj.shape, viewport);
        path.push_str(&shape_path);
    }
    
    path
}

/// Render selected shapes with highlight
pub fn render_selected_shapes(canvas: &Canvas, _width: u32, _height: u32) -> String {
    let viewport = canvas.viewport();
    let mut path = String::new();
    
    for shape_obj in canvas.shapes() {
        if !shape_obj.selected {
            continue;
        }
        
        let shape_path = render_shape_trait(&*shape_obj.shape, viewport);
        path.push_str(&shape_path);
    }
    
    path
}

/// Render selection handles for selected shapes
pub fn render_selection_handles(canvas: &Canvas, _width: u32, _height: u32) -> String {
    let viewport = canvas.viewport();
    let mut path = String::new();
    const HANDLE_SIZE: f64 = 8.0; // Increased from 6.0 for better visibility
    
    for shape_obj in canvas.shapes() {
        if !shape_obj.selected {
            continue;
        }
        
        let (x1, y1, x2, y2) = shape_obj.shape.bounding_box();
        
        // Convert to screen coordinates
        // x1 < x2 in world coords, sx1 < sx2 in screen coords (X not flipped)
        // y1 < y2 in world coords, but sy1 > sy2 in screen coords (Y IS flipped)
        let (sx1, sy1) = viewport.world_to_pixel(x1, y1);
        let (sx2, sy2) = viewport.world_to_pixel(x2, y2);
        
        // X coordinates maintain order (left < right)
        // Y coordinates are flipped, so we need min/max
        let screen_left = sx1;    // x1 is left in world, sx1 is left in screen
        let screen_right = sx2;   // x2 is right in world, sx2 is right in screen
        let screen_top = sy1.min(sy2);    // Top of screen is lower pixel Y
        let screen_bottom = sy1.max(sy2); // Bottom of screen is higher pixel Y
        
        // Calculate handle positions (corners and center) in screen space
        let handles = [
            (screen_left, screen_top),              // Top-left (screen)
            (screen_right, screen_top),             // Top-right (screen)
            (screen_left, screen_bottom),           // Bottom-left (screen)
            (screen_right, screen_bottom),          // Bottom-right (screen)
            ((screen_left + screen_right) / 2.0, (screen_top + screen_bottom) / 2.0), // Center
        ];
        
        // Draw handles as small rectangles
        for (hx, hy) in &handles {
            let x = hx - HANDLE_SIZE / 2.0;
            let y = hy - HANDLE_SIZE / 2.0;
            path.push_str(&format!(
                "M {} {} L {} {} L {} {} L {} {} Z ",
                x, y,
                x + HANDLE_SIZE, y,
                x + HANDLE_SIZE, y + HANDLE_SIZE,
                x, y + HANDLE_SIZE
            ));
        }
    }
    
    path
}

/// Render a single shape as SVG path (trait object version)
fn render_shape_trait(shape: &dyn crate::designer::shapes::Shape, viewport: &crate::designer::viewport::Viewport) -> String {
    use crate::designer::shapes::{ShapeType};
    
    // Get shape type and bounding box
    let shape_type = shape.shape_type();
    let (x1, y1, x2, y2) = shape.bounding_box();
    
    match shape_type {
        ShapeType::Rectangle => {
            let (sx1, sy1) = viewport.world_to_pixel(x1, y1);
            let (sx2, sy2) = viewport.world_to_pixel(x2, y2);
            
            format!(
                "M {} {} L {} {} L {} {} L {} {} Z ",
                sx1, sy1,
                sx2, sy1,
                sx2, sy2,
                sx1, sy2
            )
        }
        ShapeType::Circle => {
            let center_x = (x1 + x2) / 2.0;
            let center_y = (y1 + y2) / 2.0;
            let radius = ((x2 - x1) / 2.0).abs();
            
            let (cx, cy) = viewport.world_to_pixel(center_x, center_y);
            
            // Calculate screen radius using viewport zoom
            let screen_radius = radius * viewport.zoom();
            
            // Draw circle as 4 arc segments (more accurate than polygon approximation)
            format!(
                "M {} {} A {} {} 0 0 1 {} {} A {} {} 0 0 1 {} {} A {} {} 0 0 1 {} {} A {} {} 0 0 1 {} {} Z ",
                cx + screen_radius, cy,
                screen_radius, screen_radius, cx, cy + screen_radius,
                screen_radius, screen_radius, cx - screen_radius, cy,
                screen_radius, screen_radius, cx, cy - screen_radius,
                screen_radius, screen_radius, cx + screen_radius, cy
            )
        }
        ShapeType::Line => {
            let (sx1, sy1) = viewport.world_to_pixel(x1, y1);
            let (sx2, sy2) = viewport.world_to_pixel(x2, y2);
            
            format!("M {} {} L {} {} ", sx1, sy1, sx2, sy2)
        }
        ShapeType::Ellipse => {
            let center_x = (x1 + x2) / 2.0;
            let center_y = (y1 + y2) / 2.0;
            let rx = ((x2 - x1) / 2.0).abs();
            let ry = ((y2 - y1) / 2.0).abs();
            
            let (cx, cy) = viewport.world_to_pixel(center_x, center_y);
            
            // Calculate screen radii using viewport zoom
            let screen_rx = rx * viewport.zoom();
            let screen_ry = ry * viewport.zoom();
            
            // Draw ellipse as 4 arc segments
            format!(
                "M {} {} A {} {} 0 0 1 {} {} A {} {} 0 0 1 {} {} A {} {} 0 0 1 {} {} A {} {} 0 0 1 {} {} Z ",
                cx + screen_rx, cy,
                screen_rx, screen_ry, cx, cy + screen_ry,
                screen_rx, screen_ry, cx - screen_rx, cy,
                screen_rx, screen_ry, cx, cy - screen_ry,
                screen_rx, screen_ry, cx + screen_rx, cy
            )
        }
        ShapeType::Polygon | ShapeType::RoundRectangle => {
            // For polygon and rounded rectangle, we'll use a simple bounding box for now
            // since we don't have access to the detailed shape data through the trait object
            let (sx1, sy1) = viewport.world_to_pixel(x1, y1);
            let (sx2, sy2) = viewport.world_to_pixel(x2, y2);
            
            format!(
                "M {} {} L {} {} L {} {} L {} {} Z ",
                sx1, sy1,
                sx2, sy1,
                sx2, sy2,
                sx1, sy2
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::designer::shapes::*;

    #[test]
    fn test_render_empty_canvas() {
        let canvas = Canvas::new();
        let path = render_shapes(&canvas, 800, 600);
        assert_eq!(path, "");
    }

    #[test]
    fn test_render_crosshair() {
        let canvas = Canvas::new();
        let path = render_crosshair(&canvas, 800, 600);
        assert!(!path.is_empty());
        assert!(path.contains("M"));
        assert!(path.contains("L"));
    }

    #[test]
    fn test_render_rectangle() {
        let mut canvas = Canvas::new();
        let rect = Rectangle::new(10.0, 10.0, 50.0, 50.0);
        canvas.add_shape(Box::new(rect));
        
        let path = render_shapes(&canvas, 800, 600);
        assert!(!path.is_empty());
        assert!(path.contains("M"));
        assert!(path.contains("L"));
        assert!(path.contains("Z"));
    }

    #[test]
    fn test_render_circle() {
        let mut canvas = Canvas::new();
        let circle = Circle::new(Point::new(30.0, 30.0), 20.0);
        canvas.add_shape(Box::new(circle));
        
        let path = render_shapes(&canvas, 800, 600);
        assert!(!path.is_empty());
        assert!(path.contains("M"));
        assert!(path.contains("A")); // Arc commands for circle
    }
}
