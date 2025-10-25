//! 2D G-Code Visualizer
//! Renders G-Code toolpaths as 2D images using image crate

use image::{ImageBuffer, Rgba, RgbaImage};

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

/// Movement command
#[derive(Debug, Clone)]
pub enum GCodeCommand {
    Move { from: Point2D, to: Point2D, rapid: bool },
    Arc { from: Point2D, to: Point2D, center: Point2D, clockwise: bool },
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
        }
    }

    /// Parse G-Code and extract movement commands
    pub fn parse_gcode(&mut self, gcode: &str) {
        self.commands.clear();
        let mut current_pos = Point2D::new(0.0, 0.0);
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for line in gcode.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with(';') || line.starts_with('(') {
                continue;
            }

            // Parse G0/G1 (linear moves)
            if line.starts_with("G0") || line.starts_with("G1") {
                let is_rapid = line.starts_with("G0");
                let (x, y) = Self::extract_xy(line);
                
                if let (Some(new_x), Some(new_y)) = (x, y) {
                    let to = Point2D::new(new_x, new_y);
                    self.commands.push(GCodeCommand::Move {
                        from: current_pos,
                        to,
                        rapid: is_rapid,
                    });

                    min_x = min_x.min(new_x).min(current_pos.x);
                    max_x = max_x.max(new_x).max(current_pos.x);
                    min_y = min_y.min(new_y).min(current_pos.y);
                    max_y = max_y.max(new_y).max(current_pos.y);

                    current_pos = to;
                }
            }
            // Parse G2/G3 (arc moves)
            else if line.starts_with("G2") || line.starts_with("G3") {
                let clockwise = line.starts_with("G2");
                let (x, y) = Self::extract_xy(line);
                let (i, j) = Self::extract_ij(line);

                if let (Some(new_x), Some(new_y), Some(offset_x), Some(offset_y)) = (x, y, i, j) {
                    let to = Point2D::new(new_x, new_y);
                    let center = Point2D::new(current_pos.x + offset_x, current_pos.y + offset_y);
                    
                    self.commands.push(GCodeCommand::Arc {
                        from: current_pos,
                        to,
                        center,
                        clockwise,
                    });

                    min_x = min_x.min(new_x).min(current_pos.x);
                    max_x = max_x.max(new_x).max(current_pos.x);
                    min_y = min_y.min(new_y).min(current_pos.y);
                    max_y = max_y.max(new_y).max(current_pos.y);

                    current_pos = to;
                }
            }
        }

        // Add padding to bounds
        let padding_x = (max_x - min_x) * 0.1;
        let padding_y = (max_y - min_y) * 0.1;

        self.min_x = if min_x == f32::MAX { 0.0 } else { min_x - padding_x };
        self.max_x = if max_x == f32::MIN { 100.0 } else { max_x + padding_x };
        self.min_y = if min_y == f32::MAX { 0.0 } else { min_y - padding_y };
        self.max_y = if max_y == f32::MIN { 100.0 } else { max_y + padding_y };

        self.current_pos = current_pos;
    }

    /// Extract X and Y coordinates from G-Code line
    fn extract_xy(line: &str) -> (Option<f32>, Option<f32>) {
        let mut x = None;
        let mut y = None;

        for part in line.split_whitespace() {
            if part.starts_with('X') {
                x = part[1..].parse().ok();
            } else if part.starts_with('Y') {
                y = part[1..].parse().ok();
            }
        }

        (x, y)
    }

    /// Extract I and J (arc offsets) from G-Code line
    fn extract_ij(line: &str) -> (Option<f32>, Option<f32>) {
        let mut i = None;
        let mut j = None;

        for part in line.split_whitespace() {
            if part.starts_with('I') {
                i = part[1..].parse().ok();
            } else if part.starts_with('J') {
                j = part[1..].parse().ok();
            }
        }

        (i, j)
    }

    /// Get number of commands parsed
    pub fn get_command_count(&self) -> usize {
        self.commands.len()
    }

    /// Get bounds information
    pub fn get_bounds(&self) -> (f32, f32, f32, f32) {
        (self.min_x, self.max_x, self.min_y, self.max_y)
    }
}

impl Visualizer2D {
    /// Render the 2D visualization to an image
    pub fn render(&self, width: u32, height: u32) -> Vec<u8> {
        let mut img: RgbaImage = ImageBuffer::new(width, height);
        
        // Fill background with white
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255, 255, 255, 255]);
        }

        if self.commands.is_empty() {
            return encode_image_to_bytes(&img);
        }

        // Calculate scale to fit bounds into image
        let x_range = self.max_x - self.min_x;
        let y_range = self.max_y - self.min_y;
        
        let scale_x = if x_range > 0.0 && x_range.is_finite() {
            (width as f32 - 40.0) / x_range
        } else {
            1.0
        };
        
        let scale_y = if y_range > 0.0 && y_range.is_finite() {
            (height as f32 - 40.0) / y_range
        } else {
            1.0
        };

        // Use uniform scaling to maintain aspect ratio, clamp to reasonable values
        let scale = scale_x.min(scale_y).min(100.0).max(0.1);

        // Draw grid - disabled for now due to performance issues
        // draw_grid(&mut img, width, height, self.min_x, self.min_y, scale);

        // Draw origin
        let origin_x = safe_to_i32((0.0 - self.min_x) * scale + 20.0);
        let origin_y = safe_to_i32(height as f32 - (0.0 - self.min_y) * scale - 20.0);
        draw_cross(&mut img, origin_x, origin_y, 5, Rgba([100, 100, 100, 200]));

        // Draw commands
        for cmd in &self.commands {
            match cmd {
                GCodeCommand::Move { from, to, rapid } => {
                    let x1 = safe_to_i32((from.x - self.min_x) * scale + 20.0);
                    let y1 = safe_to_i32(height as f32 - (from.y - self.min_y) * scale - 20.0);
                    let x2 = safe_to_i32((to.x - self.min_x) * scale + 20.0);
                    let y2 = safe_to_i32(height as f32 - (to.y - self.min_y) * scale - 20.0);

                    let color = if *rapid {
                        Rgba([150, 150, 150, 200]) // Gray for rapid moves
                    } else {
                        Rgba([0, 100, 200, 255]) // Blue for cutting moves
                    };

                    draw_line(&mut img, x1, y1, x2, y2, color);
                }
                GCodeCommand::Arc {
                    from,
                    to,
                    center,
                    clockwise,
                } => {
                    let color = Rgba([200, 50, 50, 255]); // Red for arcs
                    draw_arc(
                        &mut img,
                        *from,
                        *to,
                        *center,
                        *clockwise,
                        self.min_x,
                        self.min_y,
                        scale,
                        height as f32,
                        color,
                    );
                }
            }
        }

        // Draw start point
        let start_x = safe_to_i32((self.commands.get(0).and_then(|cmd| match cmd {
            GCodeCommand::Move { from, .. } => Some(from.x),
            GCodeCommand::Arc { from, .. } => Some(from.x),
        }).unwrap_or(self.min_x) - self.min_x)
            * scale
            + 20.0);
        let start_y = safe_to_i32(height as f32
            - (self.commands.get(0).and_then(|cmd| match cmd {
                GCodeCommand::Move { from, .. } => Some(from.y),
                GCodeCommand::Arc { from, .. } => Some(from.y),
            }).unwrap_or(self.min_y) - self.min_y)
                * scale
            - 20.0);
        draw_circle(&mut img, start_x, start_y, 4, Rgba([0, 200, 0, 255]));

        // Draw end point
        let end_x = safe_to_i32((self.current_pos.x - self.min_x) * scale + 20.0);
        let end_y = safe_to_i32(height as f32 - (self.current_pos.y - self.min_y) * scale - 20.0);
        draw_circle(&mut img, end_x, end_y, 4, Rgba([200, 0, 0, 255]));

        encode_image_to_bytes(&img)
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
    
    // Clamp coordinates to image bounds
    let x0 = x0.max(i32::MIN + 1).min(i32::MAX - 1);
    let y0 = y0.max(i32::MIN + 1).min(i32::MAX - 1);
    let x1 = x1.max(i32::MIN + 1).min(i32::MAX - 1);
    let y1 = y1.max(i32::MIN + 1).min(i32::MAX - 1);
    
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

/// Draw a simple grid
#[allow(dead_code)]
fn draw_grid(
    img: &mut RgbaImage,
    width: u32,
    height: u32,
    min_x: f32,
    min_y: f32,
    scale: f32,
) {
    if scale <= 0.0 || !scale.is_finite() {
        return;
    }
    
    let grid_color = Rgba([220, 220, 220, 128]);
    let step = 10.0; // Grid step in model units
    
    // Limit to reasonable grid density
    let max_iterations = 200; // Max 200 lines in each direction
    
    // Vertical lines
    let mut x = (min_x.ceil() / step) * step;
    let mut iterations = 0;
    let screen_width = (width as f32 - 40.0).max(1.0);
    let max_x = min_x + (screen_width / scale).abs();
    
    while x <= max_x && iterations < max_iterations {
        let screen_x = safe_to_i32((x - min_x) * scale + 20.0);
        if screen_x >= 0 && screen_x < width as i32 {
            draw_line(img, screen_x, 0, screen_x, height as i32, grid_color);
        } else if screen_x > width as i32 {
            break; // No more lines will be visible
        }
        x += step;
        iterations += 1;
    }

    // Horizontal lines
    let mut y = (min_y.ceil() / step) * step;
    let mut iterations = 0;
    let screen_height = (height as f32 - 40.0).max(1.0);
    let max_y = min_y + (screen_height / scale).abs();
    
    while y <= max_y && iterations < max_iterations {
        let screen_y = safe_to_i32(height as f32 - (y - min_y) * scale - 20.0);
        if screen_y >= 0 && screen_y < height as i32 {
            draw_line(img, 0, screen_y, width as i32, screen_y, grid_color);
        } else if screen_y < 0 {
            break; // No more lines will be visible
        }
        y += step;
        iterations += 1;
    }
}

/// Draw an arc segment
fn draw_arc(
    img: &mut RgbaImage,
    from: Point2D,
    to: Point2D,
    center: Point2D,
    _clockwise: bool,
    min_x: f32,
    min_y: f32,
    scale: f32,
    height: f32,
    color: Rgba<u8>,
) {
    // Draw arc as multiple line segments for simplicity
    let radius = ((from.x - center.x).powi(2) + (from.y - center.y).powi(2)).sqrt();

    // Safety checks
    if !radius.is_finite() || radius < 0.001 {
        return; // Skip invalid arcs
    }

    let start_angle = (from.y - center.y).atan2(from.x - center.x);
    let end_angle = (to.y - center.y).atan2(to.x - center.x);
    
    let angle_diff = (end_angle - start_angle).abs();

    // Cap the number of segments to prevent hang
    let steps = ((radius * angle_diff) as i32).max(10).min(1000) as usize;

    let mut prev_x = (from.x - min_x) * scale + 20.0;
    let mut prev_y = height - (from.y - min_y) * scale - 20.0;

    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let angle = start_angle + (end_angle - start_angle) * t;

        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();

        let screen_x = (x - min_x) * scale + 20.0;
        let screen_y = height - (y - min_y) * scale - 20.0;

        let prev_x_i32 = safe_to_i32(prev_x);
        let prev_y_i32 = safe_to_i32(prev_y);
        let screen_x_i32 = safe_to_i32(screen_x);
        let screen_y_i32 = safe_to_i32(screen_y);

        draw_line(
            img,
            prev_x_i32,
            prev_y_i32,
            screen_x_i32,
            screen_y_i32,
            color,
        );

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
            eprintln!("PNG encoding error: {}", e);
            Vec::new()
        }
    }
}
