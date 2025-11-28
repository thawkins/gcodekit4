//! 2D G-Code Visualizer
//! Parses G-Code toolpaths for canvas-based visualization

use super::toolpath_cache::ToolpathCache;
use super::viewport::{Bounds, ViewportTransform};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const CANVAS_PADDING: f32 = 20.0;
const _CANVAS_PADDING_2X: f32 = 40.0;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 50.0;
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Movement command
#[derive(Debug, Clone)]
pub enum GCodeCommand {
    Move {
        from: Point2D,
        to: Point2D,
        rapid: bool,
        intensity: Option<f32>,
    },
    Arc {
        from: Point2D,
        to: Point2D,
        center: Point2D,
        clockwise: bool,
        intensity: Option<f32>,
    },
    Dwell {
        pos: Point2D,
        duration: f32,
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
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub current_pos: Point2D,
    pub current_intensity: f32,
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
    toolpath_cache: ToolpathCache,
    viewport: ViewportTransform,
}

impl Visualizer2D {
    /// Create new 2D visualizer
    pub fn new() -> Self {
        Self {
            min_x: -1000.0,
            max_x: 1000.0,
            min_y: -1000.0,
            max_y: 1000.0,
            current_pos: Point2D::new(0.0, 0.0),
            current_intensity: 0.0,
            zoom_scale: 1.0,
            x_offset: 0.0,
            y_offset: 0.0,
            show_grid: true,
            scale_factor: DEFAULT_SCALE_FACTOR,
            toolpath_cache: ToolpathCache::new(),
            viewport: ViewportTransform::new(CANVAS_PADDING),
        }
    }

    /// Calculate and set offsets to position origin (0,0) at bottom-left of canvas
    pub fn set_default_view(&mut self, _canvas_width: f32, canvas_height: f32) {
        let (x_offset, y_offset) = self.viewport.offsets_to_place_world_point(
            self.min_x,
            self.min_y,
            self.zoom_scale,
            self.scale_factor,
            canvas_height,
            0.0,
            0.0,
            5.0,
            canvas_height - 5.0,
        );
        self.x_offset = x_offset;
        self.y_offset = y_offset;
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
        // Find end of number
        let end_idx = after_g
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after_g.len());

        if end_idx == 0 {
            return None;
        }

        after_g[..end_idx].parse::<u32>().ok()
    }

    /// Parse G-Code and extract movement commands
    pub fn parse_gcode(&mut self, gcode: &str) {
        let mut hasher = DefaultHasher::new();
        gcode.hash(&mut hasher);
        let new_hash = hasher.finish();

        if !self.toolpath_cache.needs_update(new_hash) {
            return; // Already parsed this content
        }

        let mut commands = Vec::new();
        let mut current_pos = Point2D::new(0.0, 0.0);
        self.current_intensity = 0.0;
        let mut bounds = Bounds::new();
        let mut _g0_count = 0;
        let mut _g1_count = 0;
        let mut _g2_count = 0;
        let mut _g3_count = 0;

        for line in gcode.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with(';') || line.starts_with('(') {
                continue;
            }

            if let Some(gcode_num) = Self::extract_gcode_num(line) {
                match gcode_num {
                    0 => {
                        _g0_count += 1;
                        Self::parse_linear_move(
                            &mut commands,
                            line,
                            &mut current_pos,
                            &mut self.current_intensity,
                            &mut bounds,
                            true,
                        );
                    }
                    1 => {
                        _g1_count += 1;
                        Self::parse_linear_move(
                            &mut commands,
                            line,
                            &mut current_pos,
                            &mut self.current_intensity,
                            &mut bounds,
                            false,
                        );
                    }
                    2 => {
                        _g2_count += 1;
                        Self::parse_arc_move(
                            &mut commands,
                            line,
                            &mut current_pos,
                            &mut self.current_intensity,
                            &mut bounds,
                            true,
                        );
                    }
                    3 => {
                        _g3_count += 1;
                        Self::parse_arc_move(
                            &mut commands,
                            line,
                            &mut current_pos,
                            &mut self.current_intensity,
                            &mut bounds,
                            false,
                        );
                    }
                    4 => {
                        Self::parse_dwell(
                            &mut commands,
                            line,
                            &mut current_pos,
                        );
                    }
                    _ => {}
                }
            }
        }

        (self.min_x, self.max_x, self.min_y, self.max_y) =
            bounds.finalize_with_padding(BOUNDS_PADDING_FACTOR);
        self.current_pos = current_pos;

        self.toolpath_cache.update(new_hash, commands);
    }

    /// Calculate viewbox for the current view state
    pub fn get_viewbox(&self, width: f32, height: f32) -> (f32, f32, f32, f32) {
        self.viewport.viewbox(
            self.min_x,
            self.min_y,
            self.zoom_scale,
            self.scale_factor,
            self.x_offset,
            self.y_offset,
            width,
            height,
        )
    }

    fn parse_dwell(
        commands: &mut Vec<GCodeCommand>,
        line: &str,
        current_pos: &mut Point2D,
    ) {
        let mut duration = 0.0;
        for part in line.split_whitespace() {
            if part.len() < 2 { continue; }
            let first_char = part.chars().next().unwrap();
            match first_char {
                'P' | 'X' => {
                    if let Ok(val) = part[1..].parse::<f32>() {
                        duration = val;
                    }
                }
                _ => {}
            }
        }
        commands.push(GCodeCommand::Dwell {
            pos: *current_pos,
            duration,
        });
    }

    fn parse_linear_move(
        commands: &mut Vec<GCodeCommand>,
        line: &str,
        current_pos: &mut Point2D,
        current_intensity: &mut f32,
        bounds: &mut Bounds,
        is_rapid: bool,
    ) {
        // Extract X and Y directly without HashMap allocation
        let mut new_x = current_pos.x;
        let mut new_y = current_pos.y;
        let mut x_found = false;
        let mut y_found = false;

        for part in line.split_whitespace() {
            if part.len() < 2 {
                continue;
            }
            let first_char = part.chars().next().unwrap();
            match first_char {
                'X' => {
                    if let Ok(val) = part[1..].parse::<f32>() {
                        new_x = val;
                        x_found = true;
                    }
                }
                'Y' => {
                    if let Ok(val) = part[1..].parse::<f32>() {
                        new_y = val;
                        y_found = true;
                    }
                }
                'S' => {
                    if let Ok(val) = part[1..].parse::<f32>() {
                        *current_intensity = val;
                    }
                }
                _ => {}
            }
        }

        // Only create a command if at least one axis changed
        if x_found || y_found {
            let to = Point2D::new(new_x, new_y);
            commands.push(GCodeCommand::Move {
                from: *current_pos,
                to,
                rapid: is_rapid,
                intensity: Some(*current_intensity),
            });

            bounds.update(current_pos.x, current_pos.y);
            bounds.update(new_x, new_y);
            *current_pos = to;
        }
    }

    fn parse_arc_move(
        commands: &mut Vec<GCodeCommand>,
        line: &str,
        current_pos: &mut Point2D,
        current_intensity: &mut f32,
        bounds: &mut Bounds,
        clockwise: bool,
    ) {
        let mut new_x = None;
        let mut new_y = None;
        let mut offset_i = None;
        let mut offset_j = None;

        for part in line.split_whitespace() {
            if part.len() < 2 {
                continue;
            }
            let first_char = part.chars().next().unwrap();
            match first_char {
                'X' => {
                    if let Ok(val) = part[1..].parse::<f32>() {
                        new_x = Some(val);
                    }
                }
                'Y' => {
                    if let Ok(val) = part[1..].parse::<f32>() {
                        new_y = Some(val);
                    }
                }
                'I' => {
                    if let Ok(val) = part[1..].parse::<f32>() {
                        offset_i = Some(val);
                    }
                }
                'J' => {
                    if let Ok(val) = part[1..].parse::<f32>() {
                        offset_j = Some(val);
                    }
                }
                'S' => {
                    if let Ok(val) = part[1..].parse::<f32>() {
                        *current_intensity = val;
                    }
                }
                _ => {}
            }
        }

        if let (Some(x), Some(y), Some(i), Some(j)) = (new_x, new_y, offset_i, offset_j) {
            let to = Point2D::new(x, y);
            let center = Point2D::new(current_pos.x + i, current_pos.y + j);

            commands.push(GCodeCommand::Arc {
                from: *current_pos,
                to,
                center,
                clockwise,
                intensity: Some(*current_intensity),
            });

            bounds.update(current_pos.x, current_pos.y);
            bounds.update(x, y);
            *current_pos = to;
        }
    }

    /// Extract multiple parameters from G-Code line
    // Deprecated: Use direct parsing in parse_linear_move/parse_arc_move instead
    #[allow(dead_code)]
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
        self.toolpath_cache.len()
    }

    /// Get bounds information
    pub fn get_bounds(&self) -> (f32, f32, f32, f32) {
        (self.min_x, self.max_x, self.min_y, self.max_y)
    }

    pub fn toolpath_svg(&self) -> &str {
        self.toolpath_cache.toolpath_svg()
    }

    pub fn rapid_svg(&self) -> &str {
        self.toolpath_cache.rapid_svg()
    }

    pub fn g1_svg(&self) -> &str {
        self.toolpath_cache.g1_svg()
    }

    pub fn g2_svg(&self) -> &str {
        self.toolpath_cache.g2_svg()
    }

    pub fn g3_svg(&self) -> &str {
        self.toolpath_cache.g3_svg()
    }

    pub fn g4_svg(&self) -> &str {
        self.toolpath_cache.g4_svg()
    }

    pub fn commands(&self) -> &[GCodeCommand] {
        self.toolpath_cache.commands()
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

        for cmd in self.toolpath_cache.commands() {
            match cmd {
                GCodeCommand::Move { to, rapid, .. } => {
                    if !rapid {
                        bounds.update(to.x, to.y);
                    }
                }
                GCodeCommand::Arc { to, .. } => {
                    bounds.update(to.x, to.y);
                }
                GCodeCommand::Dwell { pos, .. } => {
                    bounds.update(pos.x, pos.y);
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

        self.zoom_scale = scale / self.scale_factor;
        self.x_offset = center_x - (bbox_min_x_padded - self.min_x) * scale - CANVAS_PADDING;
        self.y_offset = (bbox_min_y_padded - self.min_y) * scale + CANVAS_PADDING - center_y;
    }

    /// Get bounds of cutting moves only (excluding rapid moves)
    pub fn get_cutting_bounds(&self) -> Option<(f32, f32, f32, f32)> {
        let mut bounds = Bounds::new();
        let mut has_cutting_moves = false;

        for cmd in self.toolpath_cache.commands() {
            match cmd {
                GCodeCommand::Move { to, rapid, .. } => {
                    if !*rapid {
                        bounds.update(to.x, to.y);
                        has_cutting_moves = true;
                    }
                }
                GCodeCommand::Arc { to, .. } => {
                    bounds.update(to.x, to.y);
                    has_cutting_moves = true;
                }
                GCodeCommand::Dwell { pos, .. } => {
                    bounds.update(pos.x, pos.y);
                    has_cutting_moves = true;
                }
            }
        }

        if has_cutting_moves && bounds.is_valid() {
            Some((bounds.min_x, bounds.max_x, bounds.min_y, bounds.max_y))
        } else {
            None
        }
    }

    /// Get the start point of the toolpath (for debugging/testing)
    pub fn get_start_point(&self) -> Option<Point2D> {
        self.toolpath_cache.commands().first().map(|cmd| match cmd {
            GCodeCommand::Move { from, .. } => *from,
            GCodeCommand::Arc { from, .. } => *from,
            GCodeCommand::Dwell { pos, .. } => *pos,
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
