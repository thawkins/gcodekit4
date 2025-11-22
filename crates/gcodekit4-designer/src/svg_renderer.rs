//! SVG-based canvas renderer for designer shapes
//! Renders shapes as SVG path commands for display using Slint Path elements
//!
//! Features:
//! - Bright yellow crosshair at world origin (0,0)
//! - Shape rendering with selection indicators
//! - Viewport-based coordinate transformation

use crate::{Canvas, font_manager};
use rusttype::{Scale, point as rt_point, OutlineBuilder};

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

/// Render grid as SVG path commands
pub fn render_grid(canvas: &Canvas, width: u32, height: u32) -> (String, f64) {
    let viewport = canvas.viewport();
    let mut path = String::new();
    const MAX_ITERATIONS: usize = 100000;
    const GRID_MAJOR_STEP_MM: f64 = 10.0;

    // Calculate the world coordinate range needed to fill entire viewport
    // Add extra margin to ensure full coverage
    let margin_pixels = 500.0;
    let top_left = viewport.pixel_to_world(-margin_pixels, -margin_pixels);
    let bottom_right = viewport.pixel_to_world(width as f64 + margin_pixels, height as f64 + margin_pixels);

    let world_left = top_left.x.min(bottom_right.x);
    let world_right = top_left.x.max(bottom_right.x);
    let world_bottom = top_left.y.min(bottom_right.y);
    let world_top = top_left.y.max(bottom_right.y);

    let world_width = world_right - world_left;
    let world_height = world_top - world_bottom;

    // Adaptive grid spacing
    // Start with 10mm, increase by 10x if too dense
    let mut step = GRID_MAJOR_STEP_MM;
    while (world_width / step) > 100.0 || (world_height / step) > 100.0 {
        step *= 10.0;
    }

    // Round to nearest grid line, ensuring we cover the full range
    let start_x = (world_left / step).floor() * step;
    let end_x = (world_right / step).ceil() * step;

    let start_y = (world_bottom / step).floor() * step;
    let end_y = (world_top / step).ceil() * step;

    // Draw vertical grid lines
    let mut x = start_x;
    let mut iterations = 0;
    while x <= end_x && iterations < MAX_ITERATIONS {
        let (screen_x, _) = viewport.world_to_pixel(x, 0.0);
        // Draw line across full height, no need to clip
        path.push_str(&format!("M {} 0 L {} {} ", screen_x, screen_x, height));
        x += step;
        iterations += 1;
    }

    // Draw horizontal grid lines
    let mut y = start_y;
    iterations = 0;
    while y <= end_y && iterations < MAX_ITERATIONS {
        let (_, screen_y) = viewport.world_to_pixel(0.0, y);
        // Draw line across full width, no need to clip
        path.push_str(&format!("M 0 {} L {} {} ", screen_y, width, screen_y));
        y += step;
        iterations += 1;
    }

    (path, step)
}

/// Render origin marker at (0,0) as yellow cross
pub fn render_origin(canvas: &Canvas, width: u32, height: u32) -> String {
    let viewport = canvas.viewport();
    let (origin_x, origin_y) = viewport.world_to_pixel(0.0, 0.0);

    let mut path = String::new();

    // Vertical line (full height)
    path.push_str(&format!(
        "M {} 0 L {} {} ",
        origin_x, origin_x, height
    ));

    // Horizontal line (full width)
    path.push_str(&format!(
        "M 0 {} L {} {} ",
        origin_y, width, origin_y
    ));

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
        let screen_left = sx1; // x1 is left in world, sx1 is left in screen
        let screen_right = sx2; // x2 is right in world, sx2 is right in screen
        let screen_top = sy1.min(sy2); // Top of screen is lower pixel Y
        let screen_bottom = sy1.max(sy2); // Bottom of screen is higher pixel Y

        // Calculate handle positions (corners and center) in screen space
        let handles = [
            (screen_left, screen_top),     // Top-left (screen)
            (screen_right, screen_top),    // Top-right (screen)
            (screen_left, screen_bottom),  // Bottom-left (screen)
            (screen_right, screen_bottom), // Bottom-right (screen)
            (
                (screen_left + screen_right) / 2.0,
                (screen_top + screen_bottom) / 2.0,
            ), // Center
        ];

        // Draw handles as small rectangles
        for (hx, hy) in &handles {
            let x = hx - HANDLE_SIZE / 2.0;
            let y = hy - HANDLE_SIZE / 2.0;
            path.push_str(&format!(
                "M {} {} L {} {} L {} {} L {} {} Z ",
                x,
                y,
                x + HANDLE_SIZE,
                y,
                x + HANDLE_SIZE,
                y + HANDLE_SIZE,
                x,
                y + HANDLE_SIZE
            ));
        }
    }

    path
}

/// Render a single shape as SVG path (trait object version)
fn render_shape_trait(
    shape: &dyn crate::shapes::Shape,
    viewport: &crate::viewport::Viewport,
) -> String {
    use crate::shapes::ShapeType;

    // Get shape type and bounding box
    let shape_type = shape.shape_type();
    let (x1, y1, x2, y2) = shape.bounding_box();

    match shape_type {
        ShapeType::Rectangle => {
            let (sx1, sy1) = viewport.world_to_pixel(x1, y1);
            let (sx2, sy2) = viewport.world_to_pixel(x2, y2);

            format!(
                "M {} {} L {} {} L {} {} L {} {} Z ",
                sx1, sy1, sx2, sy1, sx2, sy2, sx1, sy2
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
        ShapeType::Text => {
            if let Some(text_shape) = shape.as_any().downcast_ref::<crate::shapes::TextShape>() {
                let font = font_manager::get_font();
                let scale = Scale::uniform(text_shape.font_size as f32);
                let v_metrics = font.v_metrics(scale);
                
                let start = rt_point(text_shape.x as f32, text_shape.y as f32 + v_metrics.ascent);
                
                let mut builder = SvgPathBuilder {
                    path: String::new(),
                    viewport: viewport.clone(),
                };
                
                for glyph in font.layout(&text_shape.text, scale, start) {
                    glyph.build_outline(&mut builder);
                }
                
                builder.path
            } else {
                // Fallback
                let (sx1, sy1) = viewport.world_to_pixel(x1, y1);
                let (sx2, sy2) = viewport.world_to_pixel(x2, y2);

                format!(
                    "M {} {} L {} {} L {} {} L {} {} Z ",
                    sx1, sy1, sx2, sy1, sx2, sy2, sx1, sy2
                )
            }
        }
        ShapeType::Path => {
            if let Some(path_shape) = shape.as_any().downcast_ref::<crate::shapes::PathShape>() {
                let mut path_str = String::new();
                for event in path_shape.path.iter() {
                    match event {
                        lyon::path::Event::Begin { at } => {
                            let (sx, sy) = viewport.world_to_pixel(at.x as f64, at.y as f64);
                            path_str.push_str(&format!("M {} {} ", sx, sy));
                        }
                        lyon::path::Event::Line { from: _, to } => {
                            let (sx, sy) = viewport.world_to_pixel(to.x as f64, to.y as f64);
                            path_str.push_str(&format!("L {} {} ", sx, sy));
                        }
                        lyon::path::Event::Quadratic { from: _, ctrl, to } => {
                            let (cx, cy) = viewport.world_to_pixel(ctrl.x as f64, ctrl.y as f64);
                            let (sx, sy) = viewport.world_to_pixel(to.x as f64, to.y as f64);
                            path_str.push_str(&format!("Q {} {} {} {} ", cx, cy, sx, sy));
                        }
                        lyon::path::Event::Cubic { from: _, ctrl1, ctrl2, to } => {
                            let (c1x, c1y) = viewport.world_to_pixel(ctrl1.x as f64, ctrl1.y as f64);
                            let (c2x, c2y) = viewport.world_to_pixel(ctrl2.x as f64, ctrl2.y as f64);
                            let (sx, sy) = viewport.world_to_pixel(to.x as f64, to.y as f64);
                            path_str.push_str(&format!("C {} {} {} {} {} {} ", c1x, c1y, c2x, c2y, sx, sy));
                        }
                        lyon::path::Event::End { last: _, first: _, close } => {
                            if close {
                                path_str.push_str("Z ");
                            }
                        }
                    }
                }
                path_str
            } else {
                // Fallback
                let (sx1, sy1) = viewport.world_to_pixel(x1, y1);
                let (sx2, sy2) = viewport.world_to_pixel(x2, y2);
                format!("M {} {} L {} {} L {} {} L {} {} Z ", sx1, sy1, sx2, sy1, sx2, sy2, sx1, sy2)
            }
        }
    }
}

struct SvgPathBuilder {
    path: String,
    viewport: crate::viewport::Viewport,
}

impl OutlineBuilder for SvgPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let (sx, sy) = self.viewport.world_to_pixel(x as f64, y as f64);
        self.path.push_str(&format!("M {} {} ", sx, sy));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let (sx, sy) = self.viewport.world_to_pixel(x as f64, y as f64);
        self.path.push_str(&format!("L {} {} ", sx, sy));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (sx1, sy1) = self.viewport.world_to_pixel(x1 as f64, y1 as f64);
        let (sx, sy) = self.viewport.world_to_pixel(x as f64, y as f64);
        self.path.push_str(&format!("Q {} {} {} {} ", sx1, sy1, sx, sy));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (sx1, sy1) = self.viewport.world_to_pixel(x1 as f64, y1 as f64);
        let (sx2, sy2) = self.viewport.world_to_pixel(x2 as f64, y2 as f64);
        let (sx, sy) = self.viewport.world_to_pixel(x as f64, y as f64);
        self.path.push_str(&format!("C {} {} {} {} {} {} ", sx1, sy1, sx2, sy2, sx, sy));
    }

    fn close(&mut self) {
        self.path.push_str("Z ");
    }
}


