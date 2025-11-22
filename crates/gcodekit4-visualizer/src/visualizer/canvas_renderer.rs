//! Canvas-based G-Code Visualizer using SVG Path Commands
//! Renders G-Code toolpaths as SVG path data for Slint Path elements

use super::visualizer_2d::Visualizer2D;

const GRID_MAJOR_STEP_MM: f32 = 10.0;

/// Render toolpath as SVG path commands (cutting moves only)
pub fn render_toolpath_to_path(visualizer: &Visualizer2D, _width: u32, _height: u32) -> String {
    visualizer.cached_path.clone()
}

/// Render rapid moves (G0) as SVG path commands
pub fn render_rapid_moves_to_path(visualizer: &Visualizer2D, _width: u32, _height: u32) -> String {
    visualizer.cached_rapid_path.clone()
}

/// Render grid as SVG path commands
pub fn render_grid_to_path(visualizer: &Visualizer2D, width: u32, height: u32) -> (String, f32) {
    // Get the visible world bounds from the viewbox calculation
    let (vb_x, vb_y, vb_w, vb_h) = visualizer.get_viewbox(width as f32, height as f32);
    
    // SVG coordinates:
    // Left = vb_x
    // Right = vb_x + vb_w
    // Top = vb_y
    // Bottom = vb_y + vb_h
    
    // Convert back to World coordinates for grid calculation
    // World X = SVG X
    // World Y = -SVG Y
    
    let world_left = vb_x;
    let world_right = vb_x + vb_w;
    let world_top = -vb_y;
    let world_bottom = -(vb_y + vb_h);
    
    let min_y = world_bottom.min(world_top);
    let max_y = world_bottom.max(world_top);
    let world_width = world_right - world_left;
    let world_height = max_y - min_y;

    // Adaptive grid spacing
    let mut step = GRID_MAJOR_STEP_MM;
    while (world_width / step) > 100.0 || (world_height / step) > 100.0 {
        step *= 10.0;
    }

    // Round to nearest grid line
    let start_x = (world_left / step).floor() * step;
    let end_x = (world_right / step).ceil() * step;

    let start_y = (min_y / step).floor() * step;
    let end_y = (max_y / step).ceil() * step;

    // Estimate capacity: ~50 chars per line * (num_x + num_y lines)
    let num_x = ((end_x - start_x) / step).abs() as usize + 2;
    let num_y = ((end_y - start_y) / step).abs() as usize + 2;
    let mut path = String::with_capacity((num_x + num_y) * 50);
    
    use std::fmt::Write;

    const MAX_ITERATIONS: usize = 100000;

    // Draw vertical grid lines
    let mut x = start_x;
    let mut iterations = 0;
    while x <= end_x && iterations < MAX_ITERATIONS {
        // Line from bottom to top in world coords -> top to bottom in SVG coords
        // SVG X = x
        // SVG Y range: -min_y to -max_y
        // But we can just draw from -max_y to -min_y (or vice versa)
        // Actually, we want to cover the viewbox height.
        // Viewbox Y range is vb_y to vb_y + vb_h.
        // So we can just draw from vb_y to vb_y + vb_h.
        let _ = write!(path, "M {:.2} {:.2} L {:.2} {:.2} ", x, vb_y, x, vb_y + vb_h);
        x += step;
        iterations += 1;
    }

    // Draw horizontal grid lines
    let mut y = start_y;
    iterations = 0;
    while y <= end_y && iterations < MAX_ITERATIONS {
        // Horizontal line at World Y = y
        // SVG Y = -y
        let svg_y = -y;
        let _ = write!(path, "M {:.2} {:.2} L {:.2} {:.2} ", vb_x, svg_y, vb_x + vb_w, svg_y);
        y += step;
        iterations += 1;
    }

    (path, step)
}

/// Render origin marker at (0,0) as yellow cross
pub fn render_origin_to_path(visualizer: &Visualizer2D, width: u32, height: u32) -> String {
    let (vb_x, vb_y, vb_w, vb_h) = visualizer.get_viewbox(width as f32, height as f32);
    
    let mut path = String::with_capacity(100);
    use std::fmt::Write;

    // Vertical line (full height of viewbox) at X=0
    let _ = write!(path, "M 0 {:.2} L 0 {:.2} ", vb_y, vb_y + vb_h);

    // Horizontal line (full width of viewbox) at Y=0 (SVG Y=0)
    let _ = write!(path, "M {:.2} 0 L {:.2} 0 ", vb_x, vb_x + vb_w);

    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_empty_visualizer() {
        let visualizer = Visualizer2D::new();
        let (path, _) = render_grid_to_path(&visualizer, 800, 600);
        assert!(!path.is_empty());
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
        visualizer.zoom_scale = 0.2; // Low zoom
        let (path, _) = render_grid_to_path(&visualizer, 800, 600);
        // Grid should still be visible with adaptive spacing
        assert!(!path.is_empty());

        visualizer.zoom_scale = 0.5; // Higher zoom
        let (path, _) = render_grid_to_path(&visualizer, 800, 600);
        assert!(!path.is_empty());
    }
}
