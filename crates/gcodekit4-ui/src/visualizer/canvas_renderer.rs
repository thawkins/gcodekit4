//! Canvas-based G-Code Visualizer using SVG Path Commands
//! Renders G-Code toolpaths as SVG path data for Slint Path elements

use super::visualizer_2d::{GCodeCommand, Point2D, Visualizer2D};

const CANVAS_PADDING: f32 = 20.0;
const _CANVAS_PADDING_2X: f32 = 40.0;
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
    fn new(min_x: f32, min_y: f32, scale: f32, height: f32, x_offset: f32, y_offset: f32) -> Self {
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
        // Flip Y axis: higher Y values should move up the screen (smaller screen_y)
        let screen_y =
            self.height - ((y - self.min_y) * self.scale + CANVAS_PADDING - self.y_offset);
        (screen_x, screen_y)
    }

    fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        let world_x = ((screen_x - CANVAS_PADDING - self.x_offset) / self.scale) + self.min_x;
        // Reverse the Y flip transformation
        let world_y =
            ((self.height - screen_y - CANVAS_PADDING + self.y_offset) / self.scale) + self.min_y;
        (world_x, world_y)
    }

    fn point_to_screen(&self, point: Point2D) -> (f32, f32) {
        self.world_to_screen(point.x, point.y)
    }
}

/// Render toolpath as SVG path commands (cutting moves only)
pub fn render_toolpath_to_path(visualizer: &Visualizer2D, width: u32, height: u32) -> String {
    if visualizer.commands.is_empty() {
        tracing::debug!("No commands to render");
        return String::new();
    }

    tracing::debug!(
        "Rendering toolpath: {} total commands",
        visualizer.commands.len()
    );

    let scale = calculate_scale(visualizer, width, height);
    let transform = CoordTransform::new(
        visualizer.min_x,
        visualizer.min_y,
        scale,
        height as f32,
        visualizer.x_offset,
        visualizer.y_offset,
    );

    tracing::debug!(
        "Scale: {}, bounds: ({},{}) to ({},{}), offsets: ({},{})",
        scale,
        visualizer.min_x,
        visualizer.min_y,
        visualizer.max_x,
        visualizer.max_y,
        visualizer.x_offset,
        visualizer.y_offset
    );

    let mut path = String::new();
    let mut last_draw_pos: Option<(f32, f32)> = None;
    let mut cutting_count = 0;
    let mut rapid_count = 0;

    for (idx, cmd) in visualizer.commands.iter().enumerate() {
        match cmd {
            GCodeCommand::Move { from, to, rapid } => {
                let (x1, y1) = transform.point_to_screen(*from);
                let (x2, y2) = transform.point_to_screen(*to);

                if *rapid {
                    rapid_count += 1;
                    // Rapid moves break path continuity (rendered separately)
                    last_draw_pos = None;
                    continue;
                }

                cutting_count += 1;

                if idx < 5 {
                    tracing::debug!(
                        "Cutting move #{}: ({},{}) -> ({},{}) screen: ({},{}) -> ({},{})",
                        cutting_count,
                        from.x,
                        from.y,
                        to.x,
                        to.y,
                        x1,
                        y1,
                        x2,
                        y2
                    );
                }

                // For cutting moves, ensure we start with a move command if needed
                if last_draw_pos.is_none() {
                    path.push_str(&format!("M {} {} ", x1, y1));
                }

                // Add the line to the destination
                path.push_str(&format!("L {} {} ", x2, y2));
                last_draw_pos = Some((x2, y2));
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

                // Ensure we have a starting position
                if last_draw_pos.is_none() {
                    path.push_str(&format!("M {} {} ", x1, y1));
                }

                // Check if this is a full circle (from == to)
                let is_full_circle = (from.x - to.x).abs() < 0.001 && (from.y - to.y).abs() < 0.001;

                if is_full_circle {
                    // SVG can't render a full circle arc in one command
                    // Split into two semicircles

                    // Calculate midpoint on opposite side of circle
                    let mid_x_world = center.x + (center.x - from.x);
                    let mid_y_world = center.y + (center.y - from.y);
                    let (mid_x, mid_y) = transform.world_to_screen(mid_x_world, mid_y_world);

                    let sweep = if *clockwise { 1 } else { 0 };

                    // First semicircle: from start to midpoint
                    path.push_str(&format!(
                        "A {} {} 0 0 {} {} {} ",
                        screen_radius, screen_radius, sweep, mid_x, mid_y
                    ));

                    // Second semicircle: from midpoint back to start
                    path.push_str(&format!(
                        "A {} {} 0 0 {} {} {} ",
                        screen_radius, screen_radius, sweep, x2, y2
                    ));
                } else {
                    // Regular arc - determine if it's a large arc (>180 degrees)
                    let start_angle = ((from.y - center.y).atan2(from.x - center.x)).to_degrees();
                    let end_angle = ((to.y - center.y).atan2(to.x - center.x)).to_degrees();

                    let mut arc_angle = if *clockwise {
                        start_angle - end_angle
                    } else {
                        end_angle - start_angle
                    };

                    if arc_angle < 0.0 {
                        arc_angle += 360.0;
                    }

                    let large_arc = if arc_angle > 180.0 { 1 } else { 0 };
                    let sweep = if *clockwise { 1 } else { 0 };

                    path.push_str(&format!(
                        "A {} {} 0 {} {} {} {} ",
                        screen_radius, screen_radius, large_arc, sweep, x2, y2
                    ));
                }

                last_draw_pos = Some((x2, y2));
                cutting_count += 1;
            }
        }
    }

    tracing::debug!(
        "Rendered toolpath: {} cutting moves, {} rapid moves, path length: {}",
        cutting_count,
        rapid_count,
        path.len()
    );

    path
}

/// Render rapid moves (G0) as SVG path commands
pub fn render_rapid_moves_to_path(visualizer: &Visualizer2D, width: u32, height: u32) -> String {
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
                    let (x1, y1) = transform.point_to_screen(*from);
                    let (x2, y2) = transform.point_to_screen(*to);
                    path.push_str(&format!("M {} {} L {} {} ", x1, y1, x2, y2));
                }
            }
            GCodeCommand::Arc { .. } => {
                // Arcs are always cutting moves, skip
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
    const MAX_ITERATIONS: usize = 2000;

    // Calculate the world coordinate range needed to fill entire viewport
    // Add extra margin to ensure full coverage
    let margin_mm = 100.0;
    let (world_left, world_top) = transform.screen_to_world(-margin_mm, -margin_mm);
    let (world_right, world_bottom) =
        transform.screen_to_world(width as f32 + margin_mm, height as f32 + margin_mm);

    // Round to nearest grid line, ensuring we cover the full range
    let start_x = (world_left / GRID_MAJOR_STEP_MM).floor() * GRID_MAJOR_STEP_MM;
    let end_x = (world_right / GRID_MAJOR_STEP_MM).ceil() * GRID_MAJOR_STEP_MM;

    // Y is flipped, so world_top > world_bottom. Use min/max to get correct range
    let min_y = world_bottom.min(world_top);
    let max_y = world_bottom.max(world_top);
    let start_y = (min_y / GRID_MAJOR_STEP_MM).floor() * GRID_MAJOR_STEP_MM;
    let end_y = (max_y / GRID_MAJOR_STEP_MM).ceil() * GRID_MAJOR_STEP_MM;

    // Draw vertical grid lines at 10mm spacing
    let mut x = start_x;
    let mut iterations = 0;
    while x <= end_x && iterations < MAX_ITERATIONS {
        let (screen_x, _) = transform.world_to_screen(x, 0.0);
        // Draw line across full height, no need to clip
        path.push_str(&format!("M {} 0 L {} {} ", screen_x, screen_x, height));
        x += GRID_MAJOR_STEP_MM;
        iterations += 1;
    }

    // Draw horizontal grid lines at 10mm spacing
    let mut y = start_y;
    iterations = 0;
    while y <= end_y && iterations < MAX_ITERATIONS {
        let (_, screen_y) = transform.world_to_screen(0.0, y);
        // Draw line across full width, no need to clip
        path.push_str(&format!("M 0 {} L {} {} ", screen_y, width, screen_y));
        y += GRID_MAJOR_STEP_MM;
        iterations += 1;
    }

    path
}

/// Render origin marker at (0,0) as yellow cross
pub fn render_origin_to_path(visualizer: &Visualizer2D, width: u32, height: u32) -> String {
    let scale = calculate_scale(visualizer, width, height);

    let transform = CoordTransform::new(
        visualizer.min_x,
        visualizer.min_y,
        scale,
        height as f32,
        visualizer.x_offset,
        visualizer.y_offset,
    );

    let (origin_x, origin_y) = transform.world_to_screen(0.0, 0.0);

    let mut path = String::new();
    let cross_size = 15.0;

    // Thicker lines for visibility - draw cross with 2px width by drawing twice
    for offset in [0.0, 1.0, 2.0] {
        // Vertical line of cross
        path.push_str(&format!(
            "M {} {} L {} {} ",
            origin_x + offset,
            origin_y - cross_size,
            origin_x + offset,
            origin_y + cross_size
        ));
        // Horizontal line of cross
        path.push_str(&format!(
            "M {} {} L {} {} ",
            origin_x - cross_size,
            origin_y + offset,
            origin_x + cross_size,
            origin_y + offset
        ));
    }

    path
}

/// Calculate scale factor based on visualizer bounds and canvas size
fn calculate_scale(visualizer: &Visualizer2D, _width: u32, _height: u32) -> f32 {
    // Use 1:1 scale (1 pixel = 1mm) as base, then apply zoom and scale_factor
    let base_scale = 1.0;
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
