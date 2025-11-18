//! 2D G-Code Visualizer
//! Parses G-Code toolpaths for canvas-based visualization

use std::collections::HashMap;

const CANVAS_PADDING: f32 = 20.0;
const _CANVAS_PADDING_2X: f32 = 40.0;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 10.0;
const ZOOM_STEP: f32 = 1.1;
const PAN_PERCENTAGE: f32 = 0.1;
const BOUNDS_PADDING_FACTOR: f32 = 0.1;
const FIT_MARGIN_FACTOR: f32 = 0.05;
const _ORIGIN_CROSS_SIZE: i32 = 5;
const _MARKER_RADIUS: i32 = 4;
const _MAX_GRID_ITERATIONS: usize = 500;
const _MAX_SCALE: f32 = 100.0;
const _MIN_SCALE: f32 = 0.1;
const DEFAULT_SCALE_FACTOR: f32 = 1.0;
const _GRID_MAJOR_STEP_MM: f32 = 10.0;
const _GRID_MINOR_STEP_MM: f32 = 1.0;
const _GRID_MAJOR_VISIBILITY_SCALE: f32 = 0.3;
const _GRID_MINOR_VISIBILITY_SCALE: f32 = 1.5;

/// 2D Point for visualization
#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Bounding box with validation
#[derive(Debug, Clone, Copy)]
struct Bounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl Bounds {
    fn new() -> Self {
        Self {
            min_x: f32::MAX,
            max_x: f32::MIN,
            min_y: f32::MAX,
            max_y: f32::MIN,
        }
    }

    fn update(&mut self, x: f32, y: f32) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
    }

    fn is_valid(&self) -> bool {
        self.min_x != f32::MAX
            && self.max_x != f32::MIN
            && self.min_y != f32::MAX
            && self.max_y != f32::MIN
            && self.min_x.is_finite()
            && self.max_x.is_finite()
            && self.min_y.is_finite()
            && self.max_y.is_finite()
    }

    fn finalize_with_padding(self, padding_factor: f32) -> (f32, f32, f32, f32) {
        if !self.is_valid() {
            return (0.0, 100.0, 0.0, 100.0);
        }

        let padding_x = (self.max_x - self.min_x) * padding_factor;
        let padding_y = (self.max_y - self.min_y) * padding_factor;

        // Always ensure (0,0) is at bottom-left by including it in bounds
        let final_min_x = (self.min_x - padding_x).min(0.0);
        let final_min_y = (self.min_y - padding_y).min(0.0);

        (
            final_min_x,
            self.max_x + padding_x,
            final_min_y,
            self.max_y + padding_y,
        )
    }
}

/// Movement command
#[derive(Debug, Clone)]
pub enum GCodeCommand {
    Move {
        from: Point2D,
        to: Point2D,
        rapid: bool,
    },
    Arc {
        from: Point2D,
        to: Point2D,
        center: Point2D,
        clockwise: bool,
    },
}

/// Coordinate transformation helper
#[allow(dead_code)]
struct CoordTransform {
    min_x: f32,
    min_y: f32,
    scale: f32,
    width: f32,
    height: f32,
    x_offset: f32,
    y_offset: f32,
}

#[allow(dead_code)]
impl CoordTransform {
    fn new(
        min_x: f32,
        min_y: f32,
        scale: f32,
        width: f32,
        height: f32,
        x_offset: f32,
        y_offset: f32,
    ) -> Self {
        Self {
            min_x,
            min_y,
            scale,
            width,
            height,
            x_offset,
            y_offset,
        }
    }

    fn world_to_screen(&self, x: f32, y: f32) -> (i32, i32) {
        let screen_x = (x - self.min_x) * self.scale + CANVAS_PADDING + self.x_offset;
        // Flip Y axis: higher Y values should move up the screen (smaller screen_y)
        let screen_y =
            self.height - ((y - self.min_y) * self.scale + CANVAS_PADDING - self.y_offset);
        (safe_to_i32(screen_x), safe_to_i32(screen_y))
    }

    fn point_to_screen(&self, point: Point2D) -> (i32, i32) {
        self.world_to_screen(point.x, point.y)
    }
}

/// 2D Visualizer state
#[derive(Debug, Clone)]
pub struct Visualizer2D {
    pub commands: Vec<GCodeCommand>,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub current_pos: Point2D,
    /// Zoom/scale factor for rendering (1.0 = 100%)
    pub zoom_scale: f32,
    /// X-offset for panning the view (in pixels)
    pub x_offset: f32,
    /// Y-offset for panning the view (in pixels)
    pub y_offset: f32,
    /// Grid visibility flag
    pub show_grid: bool,
    /// Scale factor: pixels per mm (default 1.0 = 1px:1mm)
    pub scale_factor: f32,
}

impl Visualizer2D {
    /// Create new 2D visualizer
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            min_x: -1000.0,
            max_x: 1000.0,
            min_y: -1000.0,
            max_y: 1000.0,
            current_pos: Point2D::new(0.0, 0.0),
            zoom_scale: 1.0,
            x_offset: 0.0,
            y_offset: 0.0,
            show_grid: true,
            scale_factor: DEFAULT_SCALE_FACTOR,
        }
    }

    /// Calculate and set offsets to position origin (0,0) at bottom-left of canvas
    pub fn set_default_view(&mut self, _canvas_width: f32, canvas_height: f32) {
        // Target position: 5px from left, 5px from bottom
        let target_x = 5.0;
        let target_y = canvas_height - 5.0;
        let padding = 20.0;
        let effective_scale = self.zoom_scale * self.scale_factor;

        // screen_x = (0 - min_x) * scale + padding + x_offset
        // x_offset = target_x - ((0 - min_x) * scale + padding)
        self.x_offset = target_x - ((0.0 - self.min_x) * effective_scale + padding);

        // screen_y = height - ((0 - min_y) * scale + padding - y_offset)
        // y_offset = (0 - min_y) * scale + padding - (height - target_y)
        self.y_offset = (0.0 - self.min_y) * effective_scale + padding - (canvas_height - target_y);
    }

    /// Toggle grid visibility
    pub fn toggle_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    /// Set grid visibility
    pub fn set_grid_visible(&mut self, visible: bool) {
        self.show_grid = visible;
    }

    /// Get grid visibility
    pub fn is_grid_visible(&self) -> bool {
        self.show_grid
    }

    /// Set scale factor (pixels per mm)
    pub fn set_scale_factor(&mut self, factor: f32) {
        self.scale_factor = factor.clamp(0.1, 100.0);
    }

    /// Get scale factor
    pub fn get_scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Extract G-code command number from line (e.g., "G01 X10" -> Some(1))
    fn extract_gcode_num(line: &str) -> Option<u32> {
        if !line.starts_with('G') {
            return None;
        }
        let after_g = &line[1..];
        let num_str: String = after_g.chars().take_while(|c| c.is_ascii_digit()).collect();
        num_str.parse::<u32>().ok()
    }

    /// Parse G-Code and extract movement commands
    pub fn parse_gcode(&mut self, gcode: &str) {
        self.commands.clear();
        let mut current_pos = Point2D::new(0.0, 0.0);
        let mut bounds = Bounds::new();
        let mut g0_count = 0;
        let mut g1_count = 0;
        let mut g2_count = 0;
        let mut g3_count = 0;

        for line in gcode.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with(';') || line.starts_with('(') {
                continue;
            }

            if let Some(gcode_num) = Self::extract_gcode_num(line) {
                match gcode_num {
                    0 => {
                        g0_count += 1;
                        self.parse_linear_move(line, &mut current_pos, &mut bounds, true);
                    }
                    1 => {
                        g1_count += 1;
                        self.parse_linear_move(line, &mut current_pos, &mut bounds, false);
                    }
                    2 => {
                        g2_count += 1;
                        self.parse_arc_move(line, &mut current_pos, &mut bounds, true);
                    }
                    3 => {
                        g3_count += 1;
                        self.parse_arc_move(line, &mut current_pos, &mut bounds, false);
                    }
                    _ => {}
                }
            }
        }

        (self.min_x, self.max_x, self.min_y, self.max_y) =
            bounds.finalize_with_padding(BOUNDS_PADDING_FACTOR);
        self.current_pos = current_pos;
    }

    fn parse_linear_move(
        &mut self,
        line: &str,
        current_pos: &mut Point2D,
        bounds: &mut Bounds,
        is_rapid: bool,
    ) {
        let params = Self::extract_params(line, &['X', 'Y']);

        // Get new position, using current position if X or Y not specified
        let new_x = params.get(&'X').copied().unwrap_or(current_pos.x);
        let new_y = params.get(&'Y').copied().unwrap_or(current_pos.y);

        // Only create a command if at least one axis changed
        if new_x != current_pos.x || new_y != current_pos.y {
            let to = Point2D::new(new_x, new_y);
            self.commands.push(GCodeCommand::Move {
                from: *current_pos,
                to,
                rapid: is_rapid,
            });

            bounds.update(current_pos.x, current_pos.y);
            bounds.update(new_x, new_y);
            *current_pos = to;
        }
    }

    fn parse_arc_move(
        &mut self,
        line: &str,
        current_pos: &mut Point2D,
        bounds: &mut Bounds,
        clockwise: bool,
    ) {
        let params = Self::extract_params(line, &['X', 'Y', 'I', 'J']);

        if let (Some(&new_x), Some(&new_y), Some(&offset_x), Some(&offset_y)) = (
            params.get(&'X'),
            params.get(&'Y'),
            params.get(&'I'),
            params.get(&'J'),
        ) {
            let to = Point2D::new(new_x, new_y);
            let center = Point2D::new(current_pos.x + offset_x, current_pos.y + offset_y);

            self.commands.push(GCodeCommand::Arc {
                from: *current_pos,
                to,
                center,
                clockwise,
            });

            bounds.update(current_pos.x, current_pos.y);
            bounds.update(new_x, new_y);
            *current_pos = to;
        }
    }

    /// Extract multiple parameters from G-Code line
    fn extract_params(line: &str, param_names: &[char]) -> HashMap<char, f32> {
        let mut params = HashMap::new();

        for part in line.split_whitespace() {
            if part.len() < 2 {
                continue;
            }
            let first_char = part.chars().next().unwrap();
            if param_names.contains(&first_char) {
                if let Ok(value) = part[1..].parse::<f32>() {
                    params.insert(first_char, value);
                }
            }
        }

        params
    }

    /// Get number of commands parsed
    pub fn get_command_count(&self) -> usize {
        self.commands.len()
    }

    /// Get bounds information
    pub fn get_bounds(&self) -> (f32, f32, f32, f32) {
        (self.min_x, self.max_x, self.min_y, self.max_y)
    }

    /// Increase zoom by 10%
    pub fn zoom_in(&mut self) {
        self.zoom_scale = (self.zoom_scale * ZOOM_STEP).min(MAX_ZOOM);
    }

    /// Decrease zoom by 10%
    pub fn zoom_out(&mut self) {
        self.zoom_scale = (self.zoom_scale / ZOOM_STEP).max(MIN_ZOOM);
    }

    /// Reset zoom to default (100%)
    pub fn reset_zoom(&mut self) {
        self.zoom_scale = 1.0;
    }

    /// Get current zoom scale as percentage
    pub fn get_zoom_percent(&self) -> u32 {
        (self.zoom_scale * 100.0).round() as u32
    }

    /// Pan view to the right by 10% of canvas width
    pub fn pan_right(&mut self, canvas_width: f32) {
        self.x_offset += canvas_width * PAN_PERCENTAGE;
    }

    /// Pan view to the left by 10% of canvas width
    pub fn pan_left(&mut self, canvas_width: f32) {
        self.x_offset -= canvas_width * PAN_PERCENTAGE;
    }

    /// Pan view down by 10% of canvas height
    pub fn pan_down(&mut self, canvas_height: f32) {
        self.y_offset -= canvas_height * PAN_PERCENTAGE;
    }

    /// Pan view up by 10% of canvas height
    pub fn pan_up(&mut self, canvas_height: f32) {
        self.y_offset += canvas_height * PAN_PERCENTAGE;
    }

    /// Reset pan to center (offset = 0)
    pub fn reset_pan(&mut self) {
        self.x_offset = 0.0;
        self.y_offset = 0.0;
    }

    /// Calculate zoom and offset to fit all cutting commands in view with 5% margin
    pub fn fit_to_view(&mut self, canvas_width: f32, canvas_height: f32) {
        let mut bounds = Bounds::new();

        for cmd in &self.commands {
            match cmd {
                GCodeCommand::Move { to, rapid, .. } => {
                    if !rapid {
                        bounds.update(to.x, to.y);
                    }
                }
                GCodeCommand::Arc { to, .. } => {
                    bounds.update(to.x, to.y);
                }
            }
        }

        if !bounds.is_valid() {
            self.zoom_scale = 1.0;
            self.set_default_view(canvas_width, canvas_height);
            return;
        }

        let bbox_width = bounds.max_x - bounds.min_x;
        let bbox_height = bounds.max_y - bounds.min_y;

        let padded_width = bbox_width * (1.0 + 2.0 * FIT_MARGIN_FACTOR);
        let padded_height = bbox_height * (1.0 + 2.0 * FIT_MARGIN_FACTOR);

        let available_width = canvas_width - 2.0 * CANVAS_PADDING;
        let available_height = canvas_height - 2.0 * CANVAS_PADDING;

        let scale = (available_width / padded_width).min(available_height / padded_height);

        let bbox_min_x_padded = bounds.min_x - bbox_width * FIT_MARGIN_FACTOR;
        let bbox_min_y_padded = bounds.min_y - bbox_height * FIT_MARGIN_FACTOR;

        let center_x = (canvas_width - padded_width * scale) / 2.0;
        let center_y = (canvas_height - padded_height * scale) / 2.0;

        self.zoom_scale = scale;
        self.x_offset = center_x - (bbox_min_x_padded - self.min_x) * scale - CANVAS_PADDING;
        self.y_offset = center_y - (bbox_min_y_padded - self.min_y) * scale + CANVAS_PADDING;
    }

    /// Get the start point of the toolpath (for debugging/testing)
    pub fn get_start_point(&self) -> Option<Point2D> {
        self.commands.first().map(|cmd| match cmd {
            GCodeCommand::Move { from, .. } => *from,
            GCodeCommand::Arc { from, .. } => *from,
        })
    }
}

impl Default for Visualizer2D {
    fn default() -> Self {
        Self::new()
    }
}

/// Safely convert a float to i32, clamping to valid range
#[allow(dead_code)]
fn safe_to_i32(value: f32) -> i32 {
    if !value.is_finite() {
        return 0;
    }
    value.clamp(i32::MIN as f32 + 1.0, i32::MAX as f32 - 1.0) as i32
}
