//! Canvas-based G-Code Visualizer using SVG Path Commands
//! Renders G-Code toolpaths as SVG path data for Slint Path elements

use super::visualizer_2d::{GCodeCommand, Point2D, Visualizer2D};

const CANVAS_PADDING: f32 = 20.0;
const CANVAS_PADDING_2X: f32 = 40.0;
const GRID_MAJOR_STEP_MM: f32 = 10.0;
const GRID_MAJOR_VISIBILITY_SCALE: f32 = 0.3;

/// Transform helper for coordinate conversion
struct CoordTransform {
    min_x: f32,
    min_y: f32,
    scale: f32,
    height: f32,
    x_offset: f32,
    y_offset: f32,
}

impl CoordTransform {
    fn new(
        min_x: f32,
        min_y: f32,
        scale: f32,
        height: f32,
        x_offset: f32,
        y_offset: f32,
    ) -> Self {
        Self {
            min_x,
            min_y,
            scale,
            height,
            x_offset,
            y_offset,
        }
    }

    fn world_to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        let screen_x = (x - self.min_x) * self.scale + CANVAS_PADDING + self.x_offset;
        let screen_y = self.height - (y - self.min_y) * self.scale - CANVAS_PADDING + self.y_offset;
        (screen_x, screen_y)
    }

    fn point_to_screen(&self, point: Point2D) -> (f32, f32) {
        self.world_to_screen(point.x, point.y)
    }
}

/// Render toolpath as SVG path commands
pub fn render_toolpath_to_path(visualizer: &Visualizer2D, width: u32, height: u32) -> String {
    if visualizer.commands.is_empty() {
        return String::new();
    }

    let scale = calculate_scale(visualizer, width, height);
    let transform = CoordTransform::new(
        visualizer.min_x,
        visualizer.min_y,
        scale,
        height as f32,
        visualizer.x_offset,
        visualizer.y_offset,
    );

    let mut path = String::new();

    for cmd in &visualizer.commands {
        match cmd {
            GCodeCommand::Move { from, to, rapid } => {
                if *rapid {
                    continue; // Skip rapid moves for now
                }
                
                let (x1, y1) = transform.point_to_screen(*from);
                let (x2, y2) = transform.point_to_screen(*to);

                if path.is_empty() {
                    path.push_str(&format!("M {} {} ", x1, y1));
                }
                path.push_str(&format!("L {} {} ", x2, y2));
            }
            GCodeCommand::Arc {
                from,
                to,
                center,
                clockwise,
            } => {
                let (x1, y1) = transform.point_to_screen(*from);
                let (x2, y2) = transform.point_to_screen(*to);

                // Calculate radius in screen space
                let radius = ((from.x - center.x).powi(2) + (from.y - center.y).powi(2)).sqrt();
                let screen_radius = radius * scale;

                if !screen_radius.is_finite() || screen_radius < 0.001 {
                    continue;
                }

                if path.is_empty() {
                    path.push_str(&format!("M {} {} ", x1, y1));
                }

                // SVG arc: A rx ry x-axis-rotation large-arc-flag sweep-flag x y
                let large_arc = 0; // For now, assume small arcs
                let sweep = if *clockwise { 1 } else { 0 };
                path.push_str(&format!(
                    "A {} {} 0 {} {} {} {} ",
                    screen_radius, screen_radius, large_arc, sweep, x2, y2
                ));
            }
        }
    }

    path
}

/// Render grid as SVG path commands
pub fn render_grid_to_path(visualizer: &Visualizer2D, width: u32, height: u32) -> String {
    if visualizer.zoom_scale < GRID_MAJOR_VISIBILITY_SCALE {
        return String::new();
    }

    let scale = calculate_scale(visualizer, width, height);
    let transform = CoordTransform::new(
        visualizer.min_x,
        visualizer.min_y,
        scale,
        height as f32,
        visualizer.x_offset,
        visualizer.y_offset,
    );

    let mut path = String::new();

    let screen_width = width as f32 - CANVAS_PADDING_2X;
    let screen_height = height as f32 - CANVAS_PADDING_2X;

    let view_min_x = transform.min_x - CANVAS_PADDING / scale;
    let view_max_x = view_min_x + screen_width / scale;
    let view_min_y = transform.min_y - CANVAS_PADDING / scale;
    let view_max_y = view_min_y + screen_height / scale;

    // Draw vertical grid lines (10mm spacing)
    let start_x = (view_min_x / GRID_MAJOR_STEP_MM).floor() * GRID_MAJOR_STEP_MM;
    let mut x = start_x;
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 500;

    while x <= view_max_x && iterations < MAX_ITERATIONS {
        let (screen_x, _) = transform.world_to_screen(x, 0.0);
        if screen_x >= 0.0 && screen_x < width as f32 {
            path.push_str(&format!("M {} 0 L {} {} ", screen_x, screen_x, height));
        }
        x += GRID_MAJOR_STEP_MM;
        iterations += 1;
    }

    // Draw horizontal grid lines (10mm spacing)
    let start_y = (view_min_y / GRID_MAJOR_STEP_MM).floor() * GRID_MAJOR_STEP_MM;
    let mut y = start_y;
    iterations = 0;

    while y <= view_max_y && iterations < MAX_ITERATIONS {
        let (_, screen_y) = transform.world_to_screen(0.0, y);
        if screen_y >= 0.0 && screen_y < height as f32 {
            path.push_str(&format!("M 0 {} L {} {} ", screen_y, width, screen_y));
        }
        y += GRID_MAJOR_STEP_MM;
        iterations += 1;
    }

    path
}

/// Calculate scale factor based on visualizer bounds and canvas size
fn calculate_scale(visualizer: &Visualizer2D, width: u32, height: u32) -> f32 {
    let x_range = visualizer.max_x - visualizer.min_x;
    let y_range = visualizer.max_y - visualizer.min_y;

    let scale_x = if x_range > 0.0 && x_range.is_finite() {
        (width as f32 - CANVAS_PADDING_2X) / x_range
    } else {
        1.0
    };

    let scale_y = if y_range > 0.0 && y_range.is_finite() {
        (height as f32 - CANVAS_PADDING_2X) / y_range
    } else {
        1.0
    };

    let base_scale = scale_x.min(scale_y).clamp(0.1, 100.0);
    base_scale * visualizer.zoom_scale * visualizer.scale_factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_visualizer() {
        let visualizer = Visualizer2D::new();
        let path = render_toolpath_to_path(&visualizer, 800, 600);
        assert_eq!(path, "");
    }

    #[test]
    fn test_render_simple_line() {
        let mut visualizer = Visualizer2D::new();
        visualizer.parse_gcode("G0 X0 Y0\nG1 X10 Y10\n");
        let path = render_toolpath_to_path(&visualizer, 800, 600);
        assert!(!path.is_empty());
        assert!(path.contains("M"));
        assert!(path.contains("L"));
    }

    #[test]
    fn test_grid_visibility() {
        let mut visualizer = Visualizer2D::new();
        visualizer.zoom_scale = 0.2; // Below threshold
        let path = render_grid_to_path(&visualizer, 800, 600);
        assert_eq!(path, "");

        visualizer.zoom_scale = 0.5; // Above threshold
        let path = render_grid_to_path(&visualizer, 800, 600);
        assert!(!path.is_empty());
    }
}
