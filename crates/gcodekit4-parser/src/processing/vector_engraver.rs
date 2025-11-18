//! Laser Vector Image Engraving Tool
//!
//! Converts vector image formats (SVG, DXF) to G-code for laser cutting/engraving.
//! Supports path stroking, fill patterns, and various vector formats.

use anyhow::{Context, Result};
use std::path::Path;

/// Vector path element
#[derive(Debug, Clone)]
pub struct PathElement {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

/// Vector engraving parameters
#[derive(Debug, Clone)]
pub struct VectorEngravingParameters {
    /// Feed rate for cutting moves (mm/min)
    pub feed_rate: f32,
    /// Travel feed rate for rapid moves (mm/min)
    pub travel_rate: f32,
    /// Laser power for cutting (0-100%)
    pub cut_power: f32,
    /// Laser power for engraving/marking (0-100%)
    pub engrave_power: f32,
    /// Laser power scale (0-1000 for GRBL S parameter)
    pub power_scale: f32,
    /// Whether to perform pass cuts for thick materials
    pub multi_pass: bool,
    /// Number of passes if multi_pass is enabled
    pub num_passes: u32,
    /// Z-axis depth increment per pass (mm)
    pub z_increment: f32,
    /// Invert cut and engrave power
    pub invert_power: bool,
    /// Desired output width in mm for scaling SVG/DXF
    pub desired_width: f32,
    /// X offset from machine origin
    pub offset_x: f32,
    /// Y offset from machine origin
    pub offset_y: f32,
}

impl Default for VectorEngravingParameters {
    fn default() -> Self {
        Self {
            feed_rate: 600.0,
            travel_rate: 3000.0,
            cut_power: 100.0,
            engrave_power: 50.0,
            power_scale: 1000.0,
            multi_pass: false,
            num_passes: 1,
            z_increment: 0.5,
            invert_power: false,
            desired_width: 100.0,
            offset_x: 10.0,
            offset_y: 10.0,
        }
    }
}

/// Vector engraver for SVG and DXF formats
#[derive(Debug)]
pub struct VectorEngraver {
    file_path: String,
    params: VectorEngravingParameters,
    paths: Vec<Vec<PathElement>>,
    /// Scale factor from SVG units to mm
    #[allow(dead_code)]
    scale_factor: f32,
}

impl VectorEngraver {
    /// Create a new vector engraver from a vector file
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        params: VectorEngravingParameters,
    ) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        // Validate file extension
        let ext = Path::new(&path_str)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .context("No file extension found")?;

        // Verify file exists
        if !Path::new(&path_str).exists() {
            anyhow::bail!("File not found: {}", path_str);
        }

        let (paths, scale_factor) = match ext.as_str() {
            "svg" => Self::parse_svg(&path_str)?,
            "dxf" => Self::parse_dxf(&path_str)?,
            _ => anyhow::bail!(
                "Unsupported file format: {}. Supported: SVG, DXF",
                ext
            ),
        };

        Ok(Self {
            file_path: path_str,
            params,
            paths,
            scale_factor,
        })
    }

    /// Parse SVG file and extract paths
    fn parse_svg(file_path: &str) -> Result<(Vec<Vec<PathElement>>, f32)> {
        use std::fs;

        let path = std::path::Path::new(file_path);
        if !path.exists() {
            anyhow::bail!("SVG file does not exist: {}", file_path);
        }

        if !path.is_file() {
            anyhow::bail!("SVG path is not a file: {}", file_path);
        }

        let content = fs::read_to_string(file_path).context("Failed to read SVG file")?;

        let mut all_paths = Vec::new();
        let mut viewbox_width = 100.0f32;
        let mut viewbox_height = 100.0f32;

        // Parse viewBox from SVG element
        if let Some(viewbox_start) = content.find("viewBox=\"") {
            if let Some(viewbox_end) = content[viewbox_start + 9..].find('"') {
                let viewbox_str = &content[viewbox_start + 9..viewbox_start + 9 + viewbox_end];
                let parts: Vec<&str> = viewbox_str.split_whitespace().collect();
                if parts.len() >= 4 {
                    viewbox_width = parts[2].parse().unwrap_or(100.0);
                    viewbox_height = parts[3].parse().unwrap_or(100.0);
                }
            }
        }

        tracing::info!(
            "SVG viewBox: width={:.2}, height={:.2}",
            viewbox_width, viewbox_height
        );

        // Extract group transform matrix
        let mut group_transform = None;
        if let Some(g_start) = content.find("<g") {
            if let Some(g_end) = content[g_start..].find('>') {
                let g_tag = &content[g_start..g_start + g_end];
                if let Some(transform_start) = g_tag.find("transform=\"") {
                    if let Some(transform_end) = g_tag[transform_start + 11..].find('"') {
                        let transform_str =
                            &g_tag[transform_start + 11..transform_start + 11 + transform_end];
                        group_transform = Self::parse_matrix_transform(transform_str);
                    }
                }
            }
        }

        // Extract all <path d="..."/> elements
        let mut search_pos = 0;
        while let Some(path_start) = content[search_pos..].find("<path") {
            let abs_path_start = search_pos + path_start;
            if let Some(path_end) = content[abs_path_start..].find('>') {
                let path_tag_end = abs_path_start + path_end;

                // Find d attribute
                if let Some(d_start) = content[abs_path_start..path_tag_end].find("d=\"") {
                    let abs_d_start = abs_path_start + d_start + 3;
                    if let Some(d_end) = content[abs_d_start..path_tag_end].find('"') {
                        let d_value = &content[abs_d_start..abs_d_start + d_end];

                        // Parse SVG path data
                        if let Ok(mut path_data) = Self::parse_path_data(d_value) {
                            // Apply group transform if present
                            if let Some((a, b, c, d_coeff, e, f)) = group_transform {
                                for point in &mut path_data {
                                    let new_x = a * point.x + c * point.y + e;
                                    let new_y = b * point.x + d_coeff * point.y + f;
                                    point.x = new_x;
                                    point.y = new_y;
                                }
                            }

                            if !path_data.is_empty() {
                                all_paths.push(path_data);
                            }
                        }
                    }
                }

                search_pos = path_tag_end + 1;
            } else {
                break;
            }
        }

        let scale_factor = if viewbox_width > 0.0 {
            100.0 / viewbox_width
        } else {
            0.1
        };

        tracing::info!(
            "Extracted {} paths, scale factor: {:.6}",
            all_paths.len(),
            scale_factor
        );

        Ok((all_paths, scale_factor))
    }

    /// Adaptive approximation tolerance for curves (in mm)
    const CURVE_TOLERANCE: f32 = 0.5;

    /// Parse matrix transform from SVG matrix(a,b,c,d,e,f) format
    fn parse_matrix_transform(transform_str: &str) -> Option<(f32, f32, f32, f32, f32, f32)> {
        let trimmed = transform_str.trim();
        if !trimmed.starts_with("matrix(") || !trimmed.ends_with(")") {
            return None;
        }

        let inner = &trimmed[7..trimmed.len() - 1];
        let values: Result<Vec<f32>, _> = inner
            .split(',')
            .map(|s| s.trim().parse::<f32>())
            .collect();

        if let Ok(vals) = values {
            if vals.len() == 6 {
                return Some((vals[0], vals[1], vals[2], vals[3], vals[4], vals[5]));
            }
        }
        None
    }

    /// Parse SVG path commands from svg::node::element::path::Data
    #[allow(dead_code)]
    fn parse_svg_path_commands(
        data: &svg::node::element::path::Data,
    ) -> Result<Vec<PathElement>> {
        let mut elements = Vec::new();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;

        for command in data.iter() {
            match command {
                svg::node::element::path::Command::Move(pos, params) => {
                    let mut param_iter = params.iter();
                    if let (Some(&x), Some(&y)) = (param_iter.next(), param_iter.next()) {
                        let x_f32 = x as f32;
                        let y_f32 = y as f32;
                        match pos {
                            svg::node::element::path::Position::Absolute => {
                                current_x = x_f32;
                                current_y = y_f32;
                            }
                            svg::node::element::path::Position::Relative => {
                                current_x += x_f32;
                                current_y += y_f32;
                            }
                        }
                        elements.push(PathElement { x: current_x, y: current_y });
                    }
                }
                svg::node::element::path::Command::Line(pos, params) => {
                    let mut param_iter = params.iter();
                    while let Some(&x) = param_iter.next() {
                        if let Some(&y) = param_iter.next() {
                            let x_f32 = x as f32;
                            let y_f32 = y as f32;
                            match pos {
                                svg::node::element::path::Position::Absolute => {
                                    current_x = x_f32;
                                    current_y = y_f32;
                                }
                                svg::node::element::path::Position::Relative => {
                                    current_x += x_f32;
                                    current_y += y_f32;
                                }
                            }
                            elements.push(PathElement { x: current_x, y: current_y });
                        }
                    }
                }
                svg::node::element::path::Command::HorizontalLine(pos, params) => {
                    for &x_val in params.iter() {
                        let x_f32 = x_val as f32;
                        match pos {
                            svg::node::element::path::Position::Absolute => {
                                current_x = x_f32;
                            }
                            svg::node::element::path::Position::Relative => {
                                current_x += x_f32;
                            }
                        }
                        elements.push(PathElement { x: current_x, y: current_y });
                    }
                }
                svg::node::element::path::Command::VerticalLine(pos, params) => {
                    for &y_val in params.iter() {
                        let y_f32 = y_val as f32;
                        match pos {
                            svg::node::element::path::Position::Absolute => {
                                current_y = y_f32;
                            }
                            svg::node::element::path::Position::Relative => {
                                current_y += y_f32;
                            }
                        }
                        elements.push(PathElement { x: current_x, y: current_y });
                    }
                }
                svg::node::element::path::Command::Close => {
                    // Close path
                }
                svg::node::element::path::Command::CubicCurve(pos, params) => {
                    let param_vec: Vec<_> = params.iter().copied().collect();
                    if param_vec.len() >= 6 {
                        let mut x1 = param_vec[0] as f32;
                        let mut y1 = param_vec[1] as f32;
                        let mut x2 = param_vec[2] as f32;
                        let mut y2 = param_vec[3] as f32;
                        let mut x = param_vec[4] as f32;
                        let mut y = param_vec[5] as f32;
                        
                        if matches!(pos, svg::node::element::path::Position::Relative) {
                            x1 += current_x;
                            y1 += current_y;
                            x2 += current_x;
                            y2 += current_y;
                            x += current_x;
                            y += current_y;
                        }
                        
                        // Adaptive curve approximation
                        let segments = Self::adaptive_cubic_segments(
                            current_x, current_y,
                            x1, y1,
                            x2, y2,
                            x, y,
                            Self::CURVE_TOLERANCE,
                        );
                        
                        for i in 1..=segments {
                            let t = i as f32 / segments as f32;
                            let t_inv = 1.0 - t;
                            let t3 = t * t * t;
                            let t3_inv = t_inv * t_inv * t_inv;
                            let t2 = t * t;
                            let t2_inv = t_inv * t_inv;
                            
                            let px = t3_inv * current_x + 3.0 * t * t2_inv * x1 + 3.0 * t2 * t_inv * x2 + t3 * x;
                            let py = t3_inv * current_y + 3.0 * t * t2_inv * y1 + 3.0 * t2 * t_inv * y2 + t3 * y;
                            
                            current_x = px;
                            current_y = py;
                            elements.push(PathElement { x: current_x, y: current_y });
                        }
                    }
                }
                svg::node::element::path::Command::QuadraticCurve(pos, params) => {
                    let param_vec: Vec<_> = params.iter().copied().collect();
                    if param_vec.len() >= 4 {
                        let mut x1 = param_vec[0] as f32;
                        let mut y1 = param_vec[1] as f32;
                        let mut x = param_vec[2] as f32;
                        let mut y = param_vec[3] as f32;
                        
                        if matches!(pos, svg::node::element::path::Position::Relative) {
                            x1 += current_x;
                            y1 += current_y;
                            x += current_x;
                            y += current_y;
                        }
                        
                        // Adaptive curve approximation
                        let segments = Self::adaptive_quad_segments(
                            current_x, current_y,
                            x1, y1,
                            x, y,
                            Self::CURVE_TOLERANCE,
                        );
                        
                        for i in 1..=segments {
                            let t = i as f32 / segments as f32;
                            let t_inv = 1.0 - t;
                            
                            let px = t_inv * t_inv * current_x + 2.0 * t * t_inv * x1 + t * t * x;
                            let py = t_inv * t_inv * current_y + 2.0 * t * t_inv * y1 + t * t * y;
                            
                            current_x = px;
                            current_y = py;
                            elements.push(PathElement { x: current_x, y: current_y });
                        }
                    }
                }
                _ => {
                    // Skip other commands
                }
            }
        }

        Ok(elements)
    }

    /// Calculate adaptive number of segments for cubic Bezier curve
    #[allow(dead_code)]
    fn adaptive_cubic_segments(x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, tolerance: f32) -> u32 {
        // Calculate chord length
        let dx = x3 - x0;
        let dy = y3 - y0;
        let chord_len = (dx * dx + dy * dy).sqrt();
        
        // Calculate control point distances
        let cp1_dist = ((x1 - x0).abs() + (y1 - y0).abs()).max((x1 - x3).abs() + (y1 - y3).abs());
        let cp2_dist = ((x2 - x0).abs() + (y2 - y0).abs()).max((x2 - x3).abs() + (y2 - y3).abs());
        let control_dist = cp1_dist.max(cp2_dist);
        
        // Estimate required segments based on curve complexity
        let error_estimate = (control_dist - chord_len / 2.0).max(0.0);
        let segments = (((error_estimate / tolerance).sqrt() * 2.0).ceil() as u32).max(2);
        
        segments.min(50) // Cap at 50 to prevent excessive segments
    }

    /// Calculate adaptive number of segments for quadratic Bezier curve
    fn adaptive_quad_segments(x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, tolerance: f32) -> u32 {
        // Calculate chord length
        let dx = x2 - x0;
        let dy = y2 - y0;
        let chord_len = (dx * dx + dy * dy).sqrt();
        
        // Distance from control point to the line between endpoints
        let line_dist = if chord_len > 0.01 {
            let t = ((x1 - x0) * dx + (y1 - y0) * dy) / (chord_len * chord_len);
            let t = t.clamp(0.0, 1.0);
            let closest_x = x0 + t * dx;
            let closest_y = y0 + t * dy;
            ((x1 - closest_x).powi(2) + (y1 - closest_y).powi(2)).sqrt()
        } else {
            ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt()
        };
        
        // Estimate required segments
        let segments = (((line_dist / tolerance).sqrt() * 2.0).ceil() as u32).max(1);
        
        segments.min(50) // Cap at 50 to prevent excessive segments
    }

    /// Parse SVG path data string into PathElement coordinates
    fn parse_path_data(data_str: &str) -> Result<Vec<PathElement>> {
        let mut elements = Vec::new();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;

        let commands = Self::tokenize_svg_path(data_str);
        let mut i = 0;

        while i < commands.len() {
            let cmd = &commands[i];

            match cmd.as_str() {
                "M" | "m" => {
                    if i + 2 < commands.len() {
                        let x: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        let y: f32 = commands[i + 2].parse().unwrap_or(0.0);

                        if cmd == "m" {
                            current_x += x;
                            current_y += y;
                        } else {
                            current_x = x;
                            current_y = y;
                        }

                        elements.push(PathElement {
                            x: current_x,
                            y: current_y,
                        });
                        i += 3;
                    } else {
                        i += 1;
                    }
                }
                "L" | "l" => {
                    // Handle multiple line segments in one command (SVG spec allows implicit repetition)
                    let mut j = i + 1;
                    while j + 1 < commands.len() {
                        let x: f32 = commands[j].parse().unwrap_or(0.0);
                        let y: f32 = commands[j + 1].parse().unwrap_or(0.0);

                        if cmd == "l" {
                            current_x += x;
                            current_y += y;
                        } else {
                            current_x = x;
                            current_y = y;
                        }

                        elements.push(PathElement {
                            x: current_x,
                            y: current_y,
                        });
                        j += 2;

                        // Check if next is a command letter or more line data
                        if j < commands.len() {
                            let next = &commands[j];
                            if next.len() == 1 && next.chars().all(|c| c.is_alphabetic()) {
                                break;
                            } else if next.parse::<f32>().is_err() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    i = j;
                }
                "H" | "h" => {
                    if i + 1 < commands.len() {
                        let x: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        if cmd == "h" {
                            current_x += x;
                        } else {
                            current_x = x;
                        }

                        elements.push(PathElement {
                            x: current_x,
                            y: current_y,
                        });
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "V" | "v" => {
                    if i + 1 < commands.len() {
                        let y: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        if cmd == "v" {
                            current_y += y;
                        } else {
                            current_y = y;
                        }

                        elements.push(PathElement {
                            x: current_x,
                            y: current_y,
                        });
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "C" | "c" => {
                    // Handle multiple cubic curve segments in one command
                    let mut j = i + 1;
                    while j + 5 < commands.len() {
                        let x1: f32 = commands[j].parse().unwrap_or(0.0);
                        let y1: f32 = commands[j + 1].parse().unwrap_or(0.0);
                        let x2: f32 = commands[j + 2].parse().unwrap_or(0.0);
                        let y2: f32 = commands[j + 3].parse().unwrap_or(0.0);
                        let x: f32 = commands[j + 4].parse().unwrap_or(0.0);
                        let y: f32 = commands[j + 5].parse().unwrap_or(0.0);

                        let mut cp1_x = x1;
                        let mut cp1_y = y1;
                        let mut cp2_x = x2;
                        let mut cp2_y = y2;
                        let mut end_x = x;
                        let mut end_y = y;

                        if cmd == "c" {
                            // Relative coordinates
                            cp1_x += current_x;
                            cp1_y += current_y;
                            cp2_x += current_x;
                            cp2_y += current_y;
                            end_x += current_x;
                            end_y += current_y;
                        }

                        // Approximate curve with line segments
                        let segments = Self::calculate_curve_segments(
                            current_x, current_y,
                            cp1_x, cp1_y,
                            cp2_x, cp2_y,
                            end_x, end_y,
                        );

                        for seg in 1..=segments {
                            let t = seg as f32 / segments as f32;
                            let t_inv = 1.0 - t;
                            let t3 = t * t * t;
                            let t3_inv = t_inv * t_inv * t_inv;
                            let t2 = t * t;
                            let t2_inv = t_inv * t_inv;

                            let px = t3_inv * current_x
                                + 3.0 * t * t2_inv * cp1_x
                                + 3.0 * t2 * t_inv * cp2_x
                                + t3 * end_x;
                            let py = t3_inv * current_y
                                + 3.0 * t * t2_inv * cp1_y
                                + 3.0 * t2 * t_inv * cp2_y
                                + t3 * end_y;

                            elements.push(PathElement { x: px, y: py });
                        }

                        current_x = end_x;
                        current_y = end_y;
                        j += 6;

                        // Check if next command is a digit (another curve segment) or a command letter
                        if j < commands.len() {
                            let next = &commands[j];
                            // If it looks like a number, continue; if it's a command letter, break
                            if next.len() == 1 && next.chars().all(|c| c.is_alphabetic()) {
                                break;
                            } else if next.parse::<f32>().is_err() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    i = j;
                }
                "Q" | "q" => {
                    // Handle multiple quadratic curve segments in one command
                    let mut j = i + 1;
                    while j + 3 < commands.len() {
                        let x1: f32 = commands[j].parse().unwrap_or(0.0);
                        let y1: f32 = commands[j + 1].parse().unwrap_or(0.0);
                        let x: f32 = commands[j + 2].parse().unwrap_or(0.0);
                        let y: f32 = commands[j + 3].parse().unwrap_or(0.0);

                        let mut cp_x = x1;
                        let mut cp_y = y1;
                        let mut end_x = x;
                        let mut end_y = y;

                        if cmd == "q" {
                            cp_x += current_x;
                            cp_y += current_y;
                            end_x += current_x;
                            end_y += current_y;
                        }

                        // Approximate quadratic curve with line segments
                        let segments = Self::adaptive_quad_segments(
                            current_x, current_y,
                            cp_x, cp_y,
                            end_x, end_y,
                            Self::CURVE_TOLERANCE,
                        );

                        for seg in 1..=segments {
                            let t = seg as f32 / segments as f32;
                            let t_inv = 1.0 - t;

                            let px = t_inv * t_inv * current_x
                                + 2.0 * t * t_inv * cp_x
                                + t * t * end_x;
                            let py = t_inv * t_inv * current_y
                                + 2.0 * t * t_inv * cp_y
                                + t * t * end_y;

                            elements.push(PathElement { x: px, y: py });
                        }

                        current_x = end_x;
                        current_y = end_y;
                        j += 4;

                        // Check if next is another segment or a new command
                        if j < commands.len() {
                            let next = &commands[j];
                            if next.len() == 1 && next.chars().all(|c| c.is_alphabetic()) {
                                break;
                            } else if next.parse::<f32>().is_err() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    i = j;
                }
                "Z" | "z" => {
                    // Path closing is handled by splitting into sub-paths
                    // We'll return the current path and start fresh on the next sub-path
                    // For now, just mark that we hit a close command
                    // The actual handling happens in parse_svg where we can detect this
                    i += 1;
                }
                _ => i += 1,
            }
        }

        Ok(elements)
    }

    /// Calculate number of segments for curve
    fn calculate_curve_segments(x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> u32 {
        let dx = x3 - x0;
        let dy = y3 - y0;
        let chord_len = (dx * dx + dy * dy).sqrt();

        let cp1_dist = ((x1 - x0).abs() + (y1 - y0).abs()).max((x1 - x3).abs() + (y1 - y3).abs());
        let cp2_dist = ((x2 - x0).abs() + (y2 - y0).abs()).max((x2 - x3).abs() + (y2 - y3).abs());
        let control_dist = cp1_dist.max(cp2_dist);

        let error_estimate = (control_dist - chord_len / 2.0).max(0.0);
        let segments = (((error_estimate / Self::CURVE_TOLERANCE).sqrt() * 2.0).ceil() as u32).max(2);

        segments.min(50)
    }

    /// Parse SVG path data (simplified M, L, Z commands)
    #[allow(dead_code)]
    fn parse_svg_path_data(path_data: &str) -> Result<Vec<PathElement>> {
        let mut elements = Vec::new();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;

        let commands = Self::tokenize_svg_path(path_data);
        let mut i = 0;

        while i < commands.len() {
            let cmd = &commands[i];

            match cmd.as_str() {
                "M" | "m" => {
                    if i + 2 < commands.len() {
                        let x: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        let y: f32 = commands[i + 2].parse().unwrap_or(0.0);

                        if cmd == "m" {
                            current_x += x;
                            current_y += y;
                        } else {
                            current_x = x;
                            current_y = y;
                        }

                        elements.push(PathElement {
                            x: current_x,
                            y: current_y,
                        });
                        i += 3;
                    } else {
                        i += 1;
                    }
                }
                "L" | "l" => {
                    if i + 2 < commands.len() {
                        let x: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        let y: f32 = commands[i + 2].parse().unwrap_or(0.0);

                        if cmd == "l" {
                            current_x += x;
                            current_y += y;
                        } else {
                            current_x = x;
                            current_y = y;
                        }

                        elements.push(PathElement {
                            x: current_x,
                            y: current_y,
                        });
                        i += 3;
                    } else {
                        i += 1;
                    }
                }
                "H" | "h" => {
                    if i + 1 < commands.len() {
                        let x: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        if cmd == "h" {
                            current_x += x;
                        } else {
                            current_x = x;
                        }

                        elements.push(PathElement {
                            x: current_x,
                            y: current_y,
                        });
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "V" | "v" => {
                    if i + 1 < commands.len() {
                        let y: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        if cmd == "v" {
                            current_y += y;
                        } else {
                            current_y = y;
                        }

                        elements.push(PathElement {
                            x: current_x,
                            y: current_y,
                        });
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "Z" | "z" => {
                    // Close path - already handled by returning to start
                    i += 1;
                }
                _ => i += 1,
            }
        }

        Ok(elements)
    }

    /// Tokenize SVG path data into commands and numbers
    fn tokenize_svg_path(path_data: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();

        for ch in path_data.chars() {
            match ch {
                'M' | 'm' | 'L' | 'l' | 'H' | 'h' | 'V' | 'v' | 'C' | 'c' | 'S' | 's' | 'Q'
                | 'q' | 'T' | 't' | 'A' | 'a' | 'Z' | 'z' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(ch.to_string());
                }
                ' ' | ',' | '\n' | '\r' | '\t' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                _ => current_token.push(ch),
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    /// Parse DXF file and extract entities
    fn parse_dxf(file_path: &str) -> Result<(Vec<Vec<PathElement>>, f32)> {
        use dxf::entities::EntityType;

        let mut file = std::fs::File::open(file_path)
            .context("Failed to open DXF file")?;

        let drawing = dxf::Drawing::load(&mut file)
            .context("Failed to parse DXF file")?;

        let mut all_paths = Vec::new();

        // Helper function to convert entity to PathElement Option
        fn entity_to_path(entity_type: &EntityType) -> Option<Vec<PathElement>> {
            match entity_type {
                EntityType::Line(line) => {
                    Some(vec![
                        PathElement {
                            x: line.p1.x as f32,
                            y: line.p1.y as f32,
                        },
                        PathElement {
                            x: line.p2.x as f32,
                            y: line.p2.y as f32,
                        },
                    ])
                }
                EntityType::Circle(circle) => {
                    // Approximate circle with 32 line segments
                    let radius = circle.radius as f32;
                    let center_x = circle.center.x as f32;
                    let center_y = circle.center.y as f32;
                    let segments = 32;

                    let mut circle_path = Vec::new();
                    for i in 0..=segments {
                        let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
                        let x = center_x + radius * angle.cos();
                        let y = center_y + radius * angle.sin();
                        circle_path.push(PathElement { x, y });
                    }
                    Some(circle_path)
                }
                EntityType::Arc(arc) => {
                    // Approximate arc with line segments
                    let radius = arc.radius as f32;
                    let center_x = arc.center.x as f32;
                    let center_y = arc.center.y as f32;
                    let start_angle = arc.start_angle as f32 * std::f32::consts::PI / 180.0;
                    let end_angle = arc.end_angle as f32 * std::f32::consts::PI / 180.0;
                    let segments = 16;

                    let mut arc_path = Vec::new();
                    for i in 0..=segments {
                        let angle = start_angle
                            + (end_angle - start_angle) * i as f32 / segments as f32;
                        let x = center_x + radius * angle.cos();
                        let y = center_y + radius * angle.sin();
                        arc_path.push(PathElement { x, y });
                    }
                    Some(arc_path)
                }
                EntityType::LwPolyline(polyline) => {
                    let poly_path: Vec<PathElement> = polyline
                        .vertices
                        .iter()
                        .map(|v| PathElement {
                            x: v.x as f32,
                            y: v.y as f32,
                        })
                        .collect();

                    if !poly_path.is_empty() {
                        Some(poly_path)
                    } else {
                        None
                    }
                }
                EntityType::Polyline(polyline) => {
                    // Regular polyline - vertices are stored separately
                    let poly_path: Vec<PathElement> = polyline
                        .vertices
                        .iter()
                        .map(|v| PathElement {
                            x: v.location.x as f32,
                            y: v.location.y as f32,
                        })
                        .collect();

                    if !poly_path.is_empty() {
                        Some(poly_path)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }

        // Extract entities from ENTITIES section
        for entity in &drawing.entities {
            if let Some(path) = entity_to_path(&entity.specific) {
                all_paths.push(path);
            }
        }

        // Extract entities from blocks as well
        for block in &drawing.blocks {
            for entity in &block.entities {
                if let Some(path) = entity_to_path(&entity.specific) {
                    all_paths.push(path);
                }
            }
        }

        // DXF uses drawing units which default to mm, but may vary
        // Default scale: 1 unit = 1 mm
        let scale_factor = 1.0;

        Ok((all_paths, scale_factor))
    }

    /// Get file information
    pub fn file_info(&self) -> (String, String) {
        let ext = Path::new(&self.file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        (self.file_path.clone(), ext.to_uppercase())
    }

    /// Estimate engraving time in seconds
    pub fn estimate_time(&self) -> f32 {
        // Calculate based on path lengths
        let total_distance: f32 = self
            .paths
            .iter()
            .map(|path| {
                path.windows(2)
                    .map(|w| {
                        let dx = w[1].x - w[0].x;
                        let dy = w[1].y - w[0].y;
                        (dx * dx + dy * dy).sqrt()
                    })
                    .sum::<f32>()
            })
            .sum();

        let cutting_time = (total_distance / self.params.feed_rate) * 60.0;
        let travel_time = (self.paths.len() as f32 * 10.0 / self.params.travel_rate) * 60.0;

        if self.params.multi_pass {
            (cutting_time + travel_time) * self.params.num_passes as f32
        } else {
            cutting_time + travel_time
        }
    }

    /// Calculate actual scale to apply based on desired width and current bounds
    fn calculate_actual_scale(&self) -> f32 {
        if self.paths.is_empty() {
            return 1.0;
        }

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;

        for path in &self.paths {
            for point in path {
                min_x = min_x.min(point.x);
                max_x = max_x.max(point.x);
            }
        }

        let current_width = max_x - min_x;
        if current_width > 0.0001 {
            let scale = self.params.desired_width / current_width;
            scale
        } else {
            1.0
        }
    }

    /// Generate G-code for vector engraving
    pub fn generate_gcode(&self) -> Result<String> {
        self.generate_gcode_with_progress(|_| {})
    }

    /// Generate G-code for vector engraving with progress callback
    pub fn generate_gcode_with_progress<F>(&self, mut progress_callback: F) -> Result<String>
    where
        F: FnMut(f32),
    {
        let mut gcode = String::new();

        gcode.push_str("; Laser Vector Engraving G-code\n");
        gcode.push_str(&format!(
            "; Generated: {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        let (file_path, file_type) = self.file_info();
        gcode.push_str(&format!("; Input file: {}\n", file_path));
        gcode.push_str(&format!("; File type: {}\n", file_type));
        gcode.push_str(&format!(
            "; Feed rate: {:.0} mm/min\n",
            self.params.feed_rate
        ));
        gcode.push_str(&format!(
            "; Travel rate: {:.0} mm/min\n",
            self.params.travel_rate
        ));
        gcode.push_str(&format!(
            "; Cut power: {:.0}%\n",
            self.params.cut_power
        ));
        gcode.push_str(&format!(
            "; Engrave power: {:.0}%\n",
            self.params.engrave_power
        ));
        if self.params.multi_pass {
            gcode.push_str(&format!(
                "; Multi-pass: {} passes, {:.2} mm per pass\n",
                self.params.num_passes, self.params.z_increment
            ));
        }
        gcode.push_str(&format!(
            "; Number of paths: {}\n",
            self.paths.len()
        ));
        gcode.push_str(&format!(
            "; Estimated time: {:.1} seconds\n",
            self.estimate_time()
        ));
        gcode.push_str(";\n");

        gcode.push_str("G21 ; Set units to millimeters\n");
        gcode.push_str("G90 ; Absolute positioning\n");
        gcode.push_str("G17 ; XY plane selection\n");
        gcode.push_str("\n");

        gcode.push_str("; Home and set work coordinate system\n");
        gcode.push_str("$H ; Home all axes (bottom-left corner)\n");
        gcode.push_str("G10 L2 P1 X0 Y0 Z0 ; Clear G54 offset\n");
        gcode.push_str("G54 ; Select work coordinate system 1\n");
        gcode.push_str(&format!(
            "G0 X{:.1} Y{:.1} ; Move to work origin\n",
            self.params.offset_x, self.params.offset_y
        ));
        gcode.push_str("G10 L20 P1 X0 Y0 Z0 ; Set current position as work zero\n");
        gcode.push_str(&format!(
            "G0 Z{:.2} F{:.0} ; Move to safe height\n",
            5.0, self.params.travel_rate
        ));
        gcode.push_str("\n");

        gcode.push_str("M5 ; Laser off\n");
        gcode.push_str("\n");

        progress_callback(0.1);

        let power_value = if self.params.invert_power {
            (self.params.engrave_power * self.params.power_scale / 100.0) as u32
        } else {
            (self.params.cut_power * self.params.power_scale / 100.0) as u32
        };

        let total_paths = self.paths.len() as f32;
        let scale = self.calculate_actual_scale();
        

        for (path_idx, path) in self.paths.iter().enumerate() {
            if path.is_empty() {
                continue;
            }

            // Travel to first point of path
            gcode.push_str(&format!(
                "G0 X{:.3} Y{:.3} ; Move to path start\n",
                path[0].x * scale, path[0].y * scale
            ));

            // Engage laser and cut path
            gcode.push_str(&format!(
                "G1 F{:.0} M3 S{} ; Enable laser at power level\n",
                self.params.feed_rate, power_value
            ));

            // Trace path
            let mut prev_x = path[0].x * scale;
            let mut prev_y = path[0].y * scale;
            
            for point in path.iter().skip(1) {
                let x = point.x * scale;
                let y = point.y * scale;
                
                // Detect discontinuities (e.g., from z/m commands) and handle with rapid move
                let dx = x - prev_x;
                let dy = y - prev_y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist > 5.0 {
                    // Large jump likely from path close (z) followed by path open (m)
                    // Turn off laser, rapid move to new position, turn laser back on
                    gcode.push_str("M5 ; Laser off for path break\n");
                    gcode.push_str(&format!(
                        "G0 X{:.3} Y{:.3} ; Rapid move to new segment\n",
                        x, y
                    ));
                    gcode.push_str(&format!(
                        "G1 F{:.0} M3 S{} ; Re-engage laser\n",
                        self.params.feed_rate, power_value
                    ));
                } else {
                    gcode.push_str(&format!("G1 X{:.3} Y{:.3}\n", x, y));
                }
                
                prev_x = x;
                prev_y = y;
            }

            gcode.push_str("M5 ; Laser off\n");

            let progress = 0.1 + (path_idx as f32 / total_paths) * 0.8;
            progress_callback(progress);
        }

        gcode.push_str("\n; End of engraving\n");
        gcode.push_str("M5 ; Laser off\n");
        gcode.push_str("G0 X0 Y0 ; Return to origin\n");

        progress_callback(1.0);

        Ok(gcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_parameters() {
        let params = VectorEngravingParameters::default();
        assert_eq!(params.feed_rate, 600.0);
        assert_eq!(params.cut_power, 100.0);
        assert_eq!(params.engrave_power, 50.0);
        assert!(!params.multi_pass);
    }

    #[test]
    fn test_svg_file_validation() {
        let result = VectorEngraver::from_file("test.txt", VectorEngravingParameters::default());
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Either unsupported format or file not found is acceptable
        assert!(
            error_msg.contains("Unsupported file format") || error_msg.contains("File not found")
        );
    }

    #[test]
    fn test_svg_path_tokenization() {
        let tokens = VectorEngraver::tokenize_svg_path("M 10 20 L 30 40 Z");
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0], "M");
        assert_eq!(tokens[1], "10");
    }

    #[test]
    fn test_estimate_time() {
        let mut params = VectorEngravingParameters::default();
        params.feed_rate = 100.0;

        let engraver = VectorEngraver {
            file_path: "test.svg".to_string(),
            params,
            paths: vec![vec![
                PathElement { x: 0.0, y: 0.0 },
                PathElement { x: 100.0, y: 0.0 },
            ]],
        };

        let time = engraver.estimate_time();
        assert!(time > 0.0);
    }

    #[test]
    fn test_svg_with_dtd_parsing() {
        // Test with actual SVG that may have DTD
        let tiger_path = "assets/svg/tiger_head_zhThh.svg";
        
        // Skip test if file doesn't exist
        if !std::path::Path::new(tiger_path).exists() {
            return;
        }
        
        let params = VectorEngravingParameters::default();
        let result = VectorEngraver::from_file(tiger_path, params);
        
        // Should successfully parse DTD SVG
        assert!(result.is_ok(), "Failed to parse SVG with DTD: {:?}", result);
    }
}

