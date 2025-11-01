//! 2D G-Code Visualizer
//! Renders G-Code toolpaths as 2D images using image crate

use image::{ImageBuffer, Rgba, RgbaImage};
use std::collections::HashMap;

const CANVAS_PADDING: f32 = 20.0;
const CANVAS_PADDING_2X: f32 = 40.0;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 5.0;
const ZOOM_STEP: f32 = 1.1;
const PAN_PERCENTAGE: f32 = 0.1;
const BOUNDS_PADDING_FACTOR: f32 = 0.1;
const FIT_MARGIN_FACTOR: f32 = 0.05;
const ORIGIN_CROSS_SIZE: i32 = 5;
const MARKER_RADIUS: i32 = 4;
const MAX_GRID_ITERATIONS: usize = 500;
const MAX_SCALE: f32 = 100.0;
const MIN_SCALE: f32 = 0.1;
const DEFAULT_SCALE_FACTOR: f32 = 1.0;
const GRID_MAJOR_STEP_MM: f32 = 10.0;
const GRID_MINOR_STEP_MM: f32 = 1.0;
const GRID_MAJOR_VISIBILITY_SCALE: f32 = 0.3;
const GRID_MINOR_VISIBILITY_SCALE: f32 = 1.5;

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

        (
            self.min_x - padding_x,
            self.max_x + padding_x,
            self.min_y - padding_y,
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
struct CoordTransform {
    min_x: f32,
    min_y: f32,
    scale: f32,
    width: f32,
    height: f32,
    x_offset: f32,
    y_offset: f32,
}

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
        let screen_y = self.height - (y - self.min_y) * self.scale - CANVAS_PADDING + self.y_offset;
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
            min_x: 0.0,
            max_x: 100.0,
            min_y: 0.0,
            max_y: 100.0,
            current_pos: Point2D::new(0.0, 0.0),
            zoom_scale: 1.0,
            x_offset: 0.0,
            y_offset: 0.0,
            show_grid: true,
            scale_factor: DEFAULT_SCALE_FACTOR,
        }
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

    /// Parse G-Code and extract movement commands
    pub fn parse_gcode(&mut self, gcode: &str) {
        self.commands.clear();
        let mut current_pos = Point2D::new(0.0, 0.0);
        let mut bounds = Bounds::new();

        for line in gcode.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with(';') || line.starts_with('(') {
                continue;
            }

            if line.starts_with("G0") || line.starts_with("G1") {
                self.parse_linear_move(line, &mut current_pos, &mut bounds);
            } else if line.starts_with("G2") || line.starts_with("G3") {
                self.parse_arc_move(line, &mut current_pos, &mut bounds);
            }
        }

        (self.min_x, self.max_x, self.min_y, self.max_y) =
            bounds.finalize_with_padding(BOUNDS_PADDING_FACTOR);
        self.current_pos = current_pos;
    }

    fn parse_linear_move(&mut self, line: &str, current_pos: &mut Point2D, bounds: &mut Bounds) {
        let is_rapid = line.starts_with("G0");
        let params = Self::extract_params(line, &['X', 'Y']);

        if let (Some(&new_x), Some(&new_y)) = (params.get(&'X'), params.get(&'Y')) {
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

    fn parse_arc_move(&mut self, line: &str, current_pos: &mut Point2D, bounds: &mut Bounds) {
        let clockwise = line.starts_with("G2");
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
            self.x_offset = 0.0;
            self.y_offset = 0.0;
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
        self.x_offset = center_x - (bbox_min_x_padded * scale) - CANVAS_PADDING;
        self.y_offset = center_y + (bbox_min_y_padded * scale) + CANVAS_PADDING;
    }

    /// Render the 2D visualization to an image
    pub fn render(&self, width: u32, height: u32) -> Vec<u8> {
        let mut img: RgbaImage = ImageBuffer::new(width, height);

        for pixel in img.pixels_mut() {
            *pixel = Rgba([255, 255, 255, 255]);
        }

        if self.commands.is_empty() {
            return encode_image_to_bytes(&img);
        }

        let scale = self.calculate_scale(width, height);
        let transform = CoordTransform::new(
            self.min_x,
            self.min_y,
            scale,
            width as f32,
            height as f32,
            self.x_offset,
            self.y_offset,
        );

        if self.show_grid {
            self.draw_grid(&mut img, width, height, &transform, scale);
        }

        self.draw_origin(&mut img, &transform);
        self.draw_commands(&mut img, &transform);
        self.draw_markers(&mut img, &transform);

        encode_image_to_bytes(&img)
    }

    fn calculate_scale(&self, width: u32, height: u32) -> f32 {
        let x_range = self.max_x - self.min_x;
        let y_range = self.max_y - self.min_y;

        let scale_x = if x_range > 0.0 && x_range.is_finite() {
            (width as f32 - CANVAS_PADDING_2X) / x_range
        } else {
            self.scale_factor
        };

        let scale_y = if y_range > 0.0 && y_range.is_finite() {
            (height as f32 - CANVAS_PADDING_2X) / y_range
        } else {
            self.scale_factor
        };

        scale_x.min(scale_y).clamp(MIN_SCALE, MAX_SCALE) * self.zoom_scale * self.scale_factor
    }

    fn draw_grid(
        &self,
        img: &mut RgbaImage,
        width: u32,
        height: u32,
        transform: &CoordTransform,
        effective_scale: f32,
    ) {
        // Only show major grid (1cm) if zoom scale is greater than 0.3 (30%)
        if self.zoom_scale < GRID_MAJOR_VISIBILITY_SCALE {
            return;
        }

        let major_color = Rgba([200, 200, 200, 255]); // Light gray for 1cm grid
        let minor_color = Rgba([173, 216, 230, 255]); // Light blue for 1mm grid

        let screen_width = width as f32 - CANVAS_PADDING_2X;
        let screen_height = height as f32 - CANVAS_PADDING_2X;

        let view_min_x = transform.min_x - CANVAS_PADDING / effective_scale;
        let view_max_x = view_min_x + screen_width / effective_scale;
        let view_min_y = transform.min_y - CANVAS_PADDING / effective_scale;
        let view_max_y = view_min_y + screen_height / effective_scale;

        // Draw minor grid lines (1mm spacing) first if zoom > 3.0 (300%)
        if self.zoom_scale >= GRID_MINOR_VISIBILITY_SCALE {
            // Draw vertical 1mm lines
            let start_x = (view_min_x / GRID_MINOR_STEP_MM).floor() * GRID_MINOR_STEP_MM;
            let mut x = start_x;
            let mut iterations = 0;

            while x <= view_max_x && iterations < MAX_GRID_ITERATIONS {
                // Skip lines that coincide with major grid (every 10mm)
                if (x / GRID_MAJOR_STEP_MM).fract().abs() > 0.01 {
                    let (screen_x, _) = transform.world_to_screen(x, 0.0);
                    if screen_x >= 0 && screen_x < width as i32 {
                        draw_line(img, screen_x, 0, screen_x, height as i32, minor_color);
                    }
                }
                x += GRID_MINOR_STEP_MM;
                iterations += 1;
            }

            // Draw horizontal 1mm lines
            let start_y = (view_min_y / GRID_MINOR_STEP_MM).floor() * GRID_MINOR_STEP_MM;
            let mut y = start_y;
            iterations = 0;

            while y <= view_max_y && iterations < MAX_GRID_ITERATIONS {
                // Skip lines that coincide with major grid (every 10mm)
                if (y / GRID_MAJOR_STEP_MM).fract().abs() > 0.01 {
                    let (_, screen_y) = transform.world_to_screen(0.0, y);
                    if screen_y >= 0 && screen_y < height as i32 {
                        draw_line(img, 0, screen_y, width as i32, screen_y, minor_color);
                    }
                }
                y += GRID_MINOR_STEP_MM;
                iterations += 1;
            }
        }

        // Draw major grid lines (1cm spacing) on top with 2px width
        let start_x = (view_min_x / GRID_MAJOR_STEP_MM).floor() * GRID_MAJOR_STEP_MM;
        let mut x = start_x;
        let mut iterations = 0;

        while x <= view_max_x && iterations < MAX_GRID_ITERATIONS {
            let (screen_x, _) = transform.world_to_screen(x, 0.0);
            if screen_x >= 0 && screen_x < width as i32 {
                draw_thick_line(img, screen_x, 0, screen_x, height as i32, major_color);
            }
            x += GRID_MAJOR_STEP_MM;
            iterations += 1;
        }

        let start_y = (view_min_y / GRID_MAJOR_STEP_MM).floor() * GRID_MAJOR_STEP_MM;
        let mut y = start_y;
        iterations = 0;

        while y <= view_max_y && iterations < MAX_GRID_ITERATIONS {
            let (_, screen_y) = transform.world_to_screen(0.0, y);
            if screen_y >= 0 && screen_y < height as i32 {
                draw_thick_line(img, 0, screen_y, width as i32, screen_y, major_color);
            }
            y += GRID_MAJOR_STEP_MM;
            iterations += 1;
        }
    }

    fn draw_origin(&self, img: &mut RgbaImage, transform: &CoordTransform) {
        let (origin_x, origin_y) = transform.world_to_screen(0.0, 0.0);
        draw_cross(
            img,
            origin_x,
            origin_y,
            ORIGIN_CROSS_SIZE,
            Rgba([100, 100, 100, 200]),
        );
    }

    fn draw_commands(&self, img: &mut RgbaImage, transform: &CoordTransform) {
        for cmd in &self.commands {
            match cmd {
                GCodeCommand::Move { from, to, rapid } => {
                    let (x1, y1) = transform.point_to_screen(*from);
                    let (x2, y2) = transform.point_to_screen(*to);

                    let color = if *rapid {
                        Rgba([150, 150, 150, 200])
                    } else {
                        Rgba([0, 100, 200, 255])
                    };

                    draw_line(img, x1, y1, x2, y2, color);
                }
                GCodeCommand::Arc {
                    from,
                    to,
                    center,
                    clockwise,
                } => {
                    draw_arc(
                        img,
                        *from,
                        *to,
                        *center,
                        *clockwise,
                        transform,
                        Rgba([200, 50, 50, 255]),
                    );
                }
            }
        }
    }

    fn draw_markers(&self, img: &mut RgbaImage, transform: &CoordTransform) {
        if let Some(start_point) = self.get_start_point() {
            let (start_x, start_y) = transform.point_to_screen(start_point);
            draw_circle(img, start_x, start_y, MARKER_RADIUS, Rgba([0, 200, 0, 255]));
        }

        let (end_x, end_y) = transform.point_to_screen(self.current_pos);
        draw_circle(img, end_x, end_y, MARKER_RADIUS, Rgba([200, 0, 0, 255]));
    }

    fn get_start_point(&self) -> Option<Point2D> {
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
fn safe_to_i32(value: f32) -> i32 {
    if !value.is_finite() {
        return 0;
    }
    value.clamp(i32::MIN as f32 + 1.0, i32::MAX as f32 - 1.0) as i32
}

/// Draw a line using Bresenham's line algorithm
fn draw_line(img: &mut RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgba<u8>) {
    let (width, height) = img.dimensions();

    let x0 = x0.clamp(i32::MIN + 1, i32::MAX - 1);
    let y0 = y0.clamp(i32::MIN + 1, i32::MAX - 1);
    let x1 = x1.clamp(i32::MIN + 1, i32::MAX - 1);
    let y1 = y1.clamp(i32::MIN + 1, i32::MAX - 1);

    let mut x = x0;
    let mut y = y0;
    let dx = ((x1 - x0).abs() as i64).min(width as i64);
    let dy = ((y1 - y0).abs() as i64).min(height as i64);
    let sx = if x0 < x1 { 1i32 } else { -1i32 };
    let sy = if y0 < y1 { 1i32 } else { -1i32 };
    let mut err = if dx > dy { dx - dy } else { dy - dx };

    // Safety: limit iterations to prevent infinite loops
    let max_iterations = (dx + dy).min(100000) as usize;
    let mut iterations = 0;

    loop {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            img.put_pixel(x as u32, y as u32, color);
        }

        if x == x1 && y == y1 {
            break;
        }

        iterations += 1;
        if iterations > max_iterations {
            break; // Safety exit
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x = x.saturating_add(sx);
        }
        if e2 < dx {
            err += dx;
            y = y.saturating_add(sy);
        }
    }
}

/// Draw a thick line (2 pixels wide)
fn draw_thick_line(img: &mut RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgba<u8>) {
    // Draw the main line
    draw_line(img, x0, y0, x1, y1, color);

    // Draw parallel line to make it 2 pixels thick
    if x0 == x1 {
        // Vertical line - draw one pixel to the right
        draw_line(img, x0 + 1, y0, x1 + 1, y1, color);
    } else if y0 == y1 {
        // Horizontal line - draw one pixel down
        draw_line(img, x0, y0 + 1, x1, y1 + 1, color);
    }
}

/// Draw a circle
fn draw_circle(img: &mut RgbaImage, cx: i32, cy: i32, radius: i32, color: Rgba<u8>) {
    let (width, height) = img.dimensions();
    let r2 = radius * radius;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= r2 {
                let x = cx + dx;
                let y = cy + dy;
                if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}

/// Draw a cross marker
fn draw_cross(img: &mut RgbaImage, cx: i32, cy: i32, size: i32, color: Rgba<u8>) {
    for i in -size..=size {
        draw_line(img, cx - size, cy + i, cx + size, cy + i, color);
        draw_line(img, cx + i, cy - size, cx + i, cy + size, color);
    }
}

/// Draw an arc segment
fn draw_arc(
    img: &mut RgbaImage,
    from: Point2D,
    to: Point2D,
    center: Point2D,
    _clockwise: bool,
    transform: &CoordTransform,
    color: Rgba<u8>,
) {
    let radius = ((from.x - center.x).powi(2) + (from.y - center.y).powi(2)).sqrt();

    if !radius.is_finite() || radius < 0.001 {
        return;
    }

    let start_angle = (from.y - center.y).atan2(from.x - center.x);
    let end_angle = (to.y - center.y).atan2(to.x - center.x);
    let angle_diff = (end_angle - start_angle).abs();

    let steps = ((radius * angle_diff) as i32).clamp(10, 1000) as usize;

    let (mut prev_x, mut prev_y) = transform.point_to_screen(from);

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let angle = start_angle + (end_angle - start_angle) * t;

        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();

        let (screen_x, screen_y) = transform.world_to_screen(x, y);

        draw_line(img, prev_x, prev_y, screen_x, screen_y, color);

        prev_x = screen_x;
        prev_y = screen_y;
    }
}

/// Encode RGBA image to PNG bytes
fn encode_image_to_bytes(img: &RgbaImage) -> Vec<u8> {
    use image::ImageEncoder;
    let mut bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
    match encoder.write_image(
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgba8,
    ) {
        Ok(_) => bytes,
        Err(e) => {
            tracing::error!("PNG encoding error: {}", e);
            Vec::new()
        }
    }
}
