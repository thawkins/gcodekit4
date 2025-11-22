//! # File Import Module
//!
//! Provides functionality to import design files (SVG, DXF) into Designer shapes.
//!
//! This module provides importers for converting external file formats into Designer shapes.
//! Includes full SVG path parsing and DXF entity conversion.
//!
//! Supports:
//! - File format detection and validation
//! - SVG path parsing (lines, circles, rectangles, ellipses, paths)
//! - DXF entity conversion (lines, circles, arcs, polylines)
//! - Coordinate system transformation
//! - Scale and offset adjustment

use crate::dxf_parser::{DxfEntity, DxfFile, DxfParser};
use crate::shapes::{Shape, PathShape, Rectangle, Circle, Line, Ellipse, Polyline, Point};
use anyhow::{anyhow, Result};
use lyon::path::Path;
use lyon::math::point;
use lyon::geom::Arc;

/// Represents an imported design from a file
#[derive(Debug)]
pub struct ImportedDesign {
    /// Imported shapes as trait objects
    pub shapes: Vec<Box<dyn Shape>>,
    /// Original file dimensions (width, height)
    pub dimensions: (f64, f64),
    /// Source file format
    pub format: FileFormat,
    /// Number of layers imported
    pub layer_count: usize,
}

/// Supported import file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// SVG (Scalable Vector Graphics)
    Svg,
    /// DXF (Drawing Exchange Format)
    Dxf,
}

/// SVG importer for converting SVG files to Designer shapes
///
/// Currently provides basic framework for SVG import.
/// Full implementation requires SVG parsing library integration.
pub struct SvgImporter {
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

enum ImportedShape {
    Rect(Rectangle),
    Circle(Circle),
    Line(Line),
    Ellipse(Ellipse),
    Polyline(Polyline),
    Path(PathShape),
}

impl ImportedShape {
    fn bounding_box(&self) -> (f64, f64, f64, f64) {
        match self {
            Self::Rect(s) => s.bounding_box(),
            Self::Circle(s) => s.bounding_box(),
            Self::Line(s) => s.bounding_box(),
            Self::Ellipse(s) => s.bounding_box(),
            Self::Polyline(s) => s.bounding_box(),
            Self::Path(s) => s.bounding_box(),
        }
    }

    fn convert(self, center_y: f64, offset_x: f64, offset_y: f64) -> Box<dyn Shape> {
        match self {
            Self::Rect(r) => {
                // y' = -y + 2c
                // New min_y is -(old_max_y) + 2c = -(r.y + r.height) + 2c
                let new_y = -(r.y + r.height) + 2.0 * center_y + offset_y;
                let new_x = r.x + offset_x;
                Box::new(Rectangle::new(new_x, new_y, r.width, r.height))
            },
            Self::Circle(c) => {
                let new_y = -c.center.y + 2.0 * center_y + offset_y;
                let new_x = c.center.x + offset_x;
                Box::new(Circle::new(Point::new(new_x, new_y), c.radius))
            },
            Self::Line(l) => {
                let start_y = -l.start.y + 2.0 * center_y + offset_y;
                let start_x = l.start.x + offset_x;
                let end_y = -l.end.y + 2.0 * center_y + offset_y;
                let end_x = l.end.x + offset_x;
                Box::new(Line::new(Point::new(start_x, start_y), Point::new(end_x, end_y)))
            },
            Self::Ellipse(e) => {
                let new_y = -e.center.y + 2.0 * center_y + offset_y;
                let new_x = e.center.x + offset_x;
                Box::new(Ellipse::new(Point::new(new_x, new_y), e.rx, e.ry))
            },
            Self::Polyline(p) => {
                let new_vertices = p.vertices.into_iter().map(|v| {
                    Point::new(v.x + offset_x, -v.y + 2.0 * center_y + offset_y)
                }).collect();
                Box::new(Polyline::new(new_vertices))
            },
            Self::Path(p) => {
                // Transform: Translate(0, -c) -> Scale(1, -1) -> Translate(0, c) -> Translate(off_x, off_y)
                // y' = -y + 2c + off_y
                // x' = x + off_x
                
                let transform = lyon::math::Transform::new(
                    1.0, 0.0,
                    0.0, -1.0,
                    offset_x as f32, (2.0 * center_y + offset_y) as f32
                );
                Box::new(PathShape::new(p.path.transformed(&transform)))
            }
        }
    }
}

impl SvgImporter {
    /// Create a new SVG importer with optional scaling
    pub fn new(scale: f64, offset_x: f64, offset_y: f64) -> Self {
        Self {
            scale,
            offset_x,
            offset_y,
        }
    }

    /// Import SVG from string content
    pub fn import_string(&self, svg_content: &str) -> Result<ImportedDesign> {
        // Validate SVG structure by checking for basic tags
        if !svg_content.contains("<svg") {
            anyhow::bail!("Invalid SVG: missing <svg> element");
        }

        let mut imported_shapes: Vec<ImportedShape> = Vec::new();
        let mut viewbox_width = 100.0f64;
        let mut _viewbox_height = 100.0f64;

        // Parse width and height from SVG element
        if let Some(svg_start) = svg_content.find("<svg") {
            if let Some(svg_end) = svg_content[svg_start..].find('>') {
                let svg_tag = &svg_content[svg_start..svg_start + svg_end];
                
                if let Some(w) = Self::extract_attr_f64(svg_tag, "width") {
                    viewbox_width = w;
                }
                if let Some(h) = Self::extract_attr_f64(svg_tag, "height") {
                    _viewbox_height = h;
                }
            }
        }

        // Parse viewBox from SVG element (overrides width/height for logical dimensions if present)
        if let Some(viewbox_start) = svg_content.find("viewBox=\"") {
            if let Some(viewbox_end) = svg_content[viewbox_start + 9..].find('"') {
                let viewbox_str = &svg_content[viewbox_start + 9..viewbox_start + 9 + viewbox_end];
                let parts: Vec<&str> = viewbox_str.split_whitespace().collect();
                if parts.len() >= 4 {
                    viewbox_width = parts[2].parse().unwrap_or(100.0);
                    _viewbox_height = parts[3].parse().unwrap_or(100.0);
                }
            }
        }

        // Extract group transform matrix
        let mut group_transform = None;
        if let Some(g_start) = svg_content.find("<g") {
            if let Some(g_end) = svg_content[g_start..].find('>') {
                let g_tag = &svg_content[g_start..g_start + g_end];
                if let Some(transform_start) = g_tag.find("transform=\"") {
                    if let Some(transform_end) = g_tag[transform_start + 11..].find('"') {
                        let transform_str =
                            &g_tag[transform_start + 11..transform_start + 11 + transform_end];
                        group_transform = Self::parse_matrix_transform(transform_str);
                    }
                }
            }
        }

        // Extract all <rect .../> elements
        let mut search_pos = 0;
        while let Some(tag_start) = svg_content[search_pos..].find("<rect") {
            let abs_tag_start = search_pos + tag_start;
            if let Some(tag_end) = svg_content[abs_tag_start..].find('>') {
                let tag_content = &svg_content[abs_tag_start..abs_tag_start + tag_end];
                
                let x = Self::extract_attr_f64(tag_content, "x").unwrap_or(0.0);
                let y = Self::extract_attr_f64(tag_content, "y").unwrap_or(0.0);
                let width = Self::extract_attr_f64(tag_content, "width").unwrap_or(0.0);
                let height = Self::extract_attr_f64(tag_content, "height").unwrap_or(0.0);
                
                if width > 0.0 && height > 0.0 {
                    let rect = Rectangle::new(
                        x * self.scale,
                        y * self.scale,
                        width * self.scale,
                        height * self.scale
                    );
                    imported_shapes.push(ImportedShape::Rect(rect));
                }
                search_pos = abs_tag_start + tag_end + 1;
            } else {
                break;
            }
        }

        // Extract all <circle .../> elements
        let mut search_pos = 0;
        while let Some(tag_start) = svg_content[search_pos..].find("<circle") {
            let abs_tag_start = search_pos + tag_start;
            if let Some(tag_end) = svg_content[abs_tag_start..].find('>') {
                let tag_content = &svg_content[abs_tag_start..abs_tag_start + tag_end];
                
                let cx = Self::extract_attr_f64(tag_content, "cx").unwrap_or(0.0);
                let cy = Self::extract_attr_f64(tag_content, "cy").unwrap_or(0.0);
                let r = Self::extract_attr_f64(tag_content, "r").unwrap_or(0.0);
                
                if r > 0.0 {
                    let circle = Circle::new(
                        Point::new(cx * self.scale, cy * self.scale),
                        r * self.scale
                    );
                    imported_shapes.push(ImportedShape::Circle(circle));
                }
                search_pos = abs_tag_start + tag_end + 1;
            } else {
                break;
            }
        }

        // Extract all <line .../> elements
        let mut search_pos = 0;
        while let Some(tag_start) = svg_content[search_pos..].find("<line") {
            let abs_tag_start = search_pos + tag_start;
            if let Some(tag_end) = svg_content[abs_tag_start..].find('>') {
                let tag_content = &svg_content[abs_tag_start..abs_tag_start + tag_end];
                
                let x1 = Self::extract_attr_f64(tag_content, "x1").unwrap_or(0.0);
                let y1 = Self::extract_attr_f64(tag_content, "y1").unwrap_or(0.0);
                let x2 = Self::extract_attr_f64(tag_content, "x2").unwrap_or(0.0);
                let y2 = Self::extract_attr_f64(tag_content, "y2").unwrap_or(0.0);
                
                let line = Line::new(
                    Point::new(x1 * self.scale, y1 * self.scale),
                    Point::new(x2 * self.scale, y2 * self.scale)
                );
                imported_shapes.push(ImportedShape::Line(line));
                
                search_pos = abs_tag_start + tag_end + 1;
            } else {
                break;
            }
        }

        // Extract all <ellipse .../> elements
        let mut search_pos = 0;
        while let Some(tag_start) = svg_content[search_pos..].find("<ellipse") {
            let abs_tag_start = search_pos + tag_start;
            if let Some(tag_end) = svg_content[abs_tag_start..].find('>') {
                let tag_content = &svg_content[abs_tag_start..abs_tag_start + tag_end];
                
                let cx = Self::extract_attr_f64(tag_content, "cx").unwrap_or(0.0);
                let cy = Self::extract_attr_f64(tag_content, "cy").unwrap_or(0.0);
                let rx = Self::extract_attr_f64(tag_content, "rx").unwrap_or(0.0);
                let ry = Self::extract_attr_f64(tag_content, "ry").unwrap_or(0.0);
                
                if rx > 0.0 && ry > 0.0 {
                    let ellipse = Ellipse::new(
                        Point::new(cx * self.scale, cy * self.scale),
                        rx * self.scale,
                        ry * self.scale
                    );
                    imported_shapes.push(ImportedShape::Ellipse(ellipse));
                }
                search_pos = abs_tag_start + tag_end + 1;
            } else {
                break;
            }
        }

        // Extract all <polyline .../> elements
        let mut search_pos = 0;
        while let Some(tag_start) = svg_content[search_pos..].find("<polyline") {
            let abs_tag_start = search_pos + tag_start;
            if let Some(tag_end) = svg_content[abs_tag_start..].find('>') {
                let tag_content = &svg_content[abs_tag_start..abs_tag_start + tag_end];
                
                if let Some(points_str) = Self::extract_attr_str(tag_content, "points") {
                    let points: Vec<Point> = points_str
                        .split(|c| c == ' ' || c == ',')
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<&str>>()
                        .chunks(2)
                        .filter_map(|chunk| {
                            if chunk.len() == 2 {
                                let x = chunk[0].parse::<f64>().ok()?;
                                let y = chunk[1].parse::<f64>().ok()?;
                                Some(Point::new(
                                    x * self.scale,
                                    y * self.scale
                                ))
                            } else {
                                None
                            }
                        })
                        .collect();
                    
                    if !points.is_empty() {
                        imported_shapes.push(ImportedShape::Polyline(Polyline::new(points)));
                    }
                }
                search_pos = abs_tag_start + tag_end + 1;
            } else {
                break;
            }
        }

        // Extract all <polygon .../> elements
        let mut search_pos = 0;
        while let Some(tag_start) = svg_content[search_pos..].find("<polygon") {
            let abs_tag_start = search_pos + tag_start;
            if let Some(tag_end) = svg_content[abs_tag_start..].find('>') {
                let tag_content = &svg_content[abs_tag_start..abs_tag_start + tag_end];
                
                if let Some(points_str) = Self::extract_attr_str(tag_content, "points") {
                    let points: Vec<Point> = points_str
                        .split(|c| c == ' ' || c == ',')
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<&str>>()
                        .chunks(2)
                        .filter_map(|chunk| {
                            if chunk.len() == 2 {
                                let x = chunk[0].parse::<f64>().ok()?;
                                let y = chunk[1].parse::<f64>().ok()?;
                                Some(Point::new(
                                    x * self.scale,
                                    y * self.scale
                                ))
                            } else {
                                None
                            }
                        })
                        .collect();
                    
                    if !points.is_empty() {
                        imported_shapes.push(ImportedShape::Polyline(Polyline::new(points)));
                    }
                }
                search_pos = abs_tag_start + tag_end + 1;
            } else {
                break;
            }
        }

        // Extract all <path d="..."/> elements
        let mut search_pos = 0;
        while let Some(path_start) = svg_content[search_pos..].find("<path") {
            let abs_path_start = search_pos + path_start;
            if let Some(path_end) = svg_content[abs_path_start..].find('>') {
                let path_tag_end = abs_path_start + path_end;

                // Find d attribute
                if let Some(d_start) = svg_content[abs_path_start..path_tag_end].find("d=\"") {
                    let abs_d_start = abs_path_start + d_start + 3;
                    if let Some(d_end) = svg_content[abs_d_start..path_tag_end].find('"') {
                        let d_value = &svg_content[abs_d_start..abs_d_start + d_end];

                        // Parse SVG path data
                        if let Ok(path) = Self::build_path_from_svg_data(d_value) {
                            // Apply group transform if present
                            let final_path = if let Some((a, b, c, d_coeff, e, f)) = group_transform {
                                let transform = lyon::math::Transform::new(
                                    a, b, c, d_coeff, e, f
                                );
                                path.clone().transformed(&transform)
                            } else {
                                path
                            };
                            
                            // Apply importer scale only
                            let scale_transform = lyon::math::Transform::scale(self.scale as f32, self.scale as f32);
                            let scaled_path = final_path.clone().transformed(&scale_transform);

                            imported_shapes.push(ImportedShape::Path(PathShape::new(scaled_path)));
                        }
                    }
                }

                search_pos = path_tag_end + 1;
            } else {
                break;
            }
        }

        // Calculate bounds and mirror
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;
        
        for shape in &imported_shapes {
            let (_, s_min_y, _, s_max_y) = shape.bounding_box();
            if s_min_y < min_y { min_y = s_min_y; }
            if s_max_y > max_y { max_y = s_max_y; }
        }
        
        let center_y = if min_y == f64::MAX { 0.0 } else { (min_y + max_y) / 2.0 };
        
        let shapes: Vec<Box<dyn Shape>> = imported_shapes
            .into_iter()
            .map(|s| s.convert(center_y, self.offset_x, self.offset_y))
            .collect();

        Ok(ImportedDesign {
            shapes,
            dimensions: (viewbox_width * self.scale, _viewbox_height * self.scale),
            format: FileFormat::Svg,
            layer_count: 0,
        })
    }

    fn extract_attr_str<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
        let pattern = format!("{}=\"", attr);
        if let Some(start) = tag.find(&pattern) {
            let val_start = start + pattern.len();
            if let Some(end) = tag[val_start..].find('"') {
                return Some(&tag[val_start..val_start + end]);
            }
        }
        None
    }

    fn extract_attr_f64(tag: &str, attr: &str) -> Option<f64> {
        Self::extract_attr_str(tag, attr).and_then(|s| s.parse().ok())
    }

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

    /// Build lyon Path from SVG path data string
    fn build_path_from_svg_data(data_str: &str) -> Result<Path> {
        let mut builder = Path::builder();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;
        let mut start_x = 0.0f32;
        let mut start_y = 0.0f32;
        let mut subpath_active = false;

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
                        
                        if subpath_active {
                            builder.end(false);
                        }
                        
                        start_x = current_x;
                        start_y = current_y;
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                        i += 3;
                    } else {
                        i += 1;
                    }
                }
                "L" | "l" => {
                    if !subpath_active {
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                    }
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

                        builder.line_to(point(current_x, current_y));
                        j += 2;

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
                    if !subpath_active {
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                    }
                    if i + 1 < commands.len() {
                        let x: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        if cmd == "h" {
                            current_x += x;
                        } else {
                            current_x = x;
                        }
                        builder.line_to(point(current_x, current_y));
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "V" | "v" => {
                    if !subpath_active {
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                    }
                    if i + 1 < commands.len() {
                        let y: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        if cmd == "v" {
                            current_y += y;
                        } else {
                            current_y = y;
                        }
                        builder.line_to(point(current_x, current_y));
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "C" | "c" => {
                    if !subpath_active {
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                    }
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
                            cp1_x += current_x;
                            cp1_y += current_y;
                            cp2_x += current_x;
                            cp2_y += current_y;
                            end_x += current_x;
                            end_y += current_y;
                        }

                        builder.cubic_bezier_to(
                            point(cp1_x, cp1_y),
                            point(cp2_x, cp2_y),
                            point(end_x, end_y)
                        );

                        current_x = end_x;
                        current_y = end_y;
                        j += 6;

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
                "Q" | "q" => {
                    if !subpath_active {
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                    }
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

                        builder.quadratic_bezier_to(
                            point(cp_x, cp_y),
                            point(end_x, end_y)
                        );

                        current_x = end_x;
                        current_y = end_y;
                        j += 4;

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
                    if subpath_active {
                        builder.close();
                        subpath_active = false;
                    }
                    current_x = start_x;
                    current_y = start_y;
                    i += 1;
                }
                _ => i += 1,
            }
        }
        
        if subpath_active {
            builder.end(false);
        }
        Ok(builder.build())
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
}

/// DXF importer for converting DXF files to Designer shapes
pub struct DxfImporter {
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

impl DxfImporter {
    /// Create a new DXF importer with optional scaling
    pub fn new(scale: f64, offset_x: f64, offset_y: f64) -> Self {
        Self {
            scale,
            offset_x,
            offset_y,
        }
    }

    /// Import DXF from file path
    ///
    /// # Arguments
    /// * `path` - Path to DXF file
    ///
    /// # Returns
    /// Imported design with converted shapes
    pub fn import_file(&self, path: &str) -> Result<ImportedDesign> {
        let content =
            std::fs::read_to_string(path).map_err(|e| anyhow!("Failed to read DXF file: {}", e))?;

        self.import_string(&content)
    }

    /// Import DXF from string content
    ///
    /// # Arguments
    /// * `content` - DXF file content as string
    ///
    /// # Returns
    /// Imported design with converted shapes
    pub fn import_string(&self, content: &str) -> Result<ImportedDesign> {
        let mut dxf_file = DxfParser::parse(content)?;

        // Apply scaling
        dxf_file.scale(self.scale);

        // Convert DXF entities to Designer shapes
        let shapes = self.convert_entities_to_shapes(&dxf_file)?;

        // Calculate dimensions from bounding box
        let (min, max) = dxf_file.bounding_box();
        let dimensions = ((max.x - min.x).abs(), (max.y - min.y).abs());

        Ok(ImportedDesign {
            shapes,
            dimensions,
            format: FileFormat::Dxf,
            layer_count: dxf_file.layer_names().len(),
        })
    }

    /// Convert DXF entities to Designer shapes
    ///
    /// Note: DXF coordinates are negated on X-axis to correct for coordinate system difference.
    /// DXF uses right-handed coordinate system, Designer uses left-handed with Y-up.
    fn convert_entities_to_shapes(&self, dxf_file: &DxfFile) -> Result<Vec<Box<dyn Shape>>> {
        let mut shapes: Vec<Box<dyn Shape>> = Vec::new();

        // Transform to apply: negate X and add offset
        // Note: dxf_file is already scaled by self.scale
        let transform = lyon::math::Transform::scale(-1.0, 1.0)
            .then_translate(lyon::math::vector(self.offset_x as f32, self.offset_y as f32));

        for entity in &dxf_file.entities {
            let path_opt = match entity {
                DxfEntity::Line(line) => {
                    let mut builder = Path::builder();
                    builder.begin(point(line.start.x as f32, line.start.y as f32));
                    builder.line_to(point(line.end.x as f32, line.end.y as f32));
                    builder.end(false);
                    Some(builder.build())
                }
                DxfEntity::Circle(circle) => {
                    let mut builder = Path::builder();
                    let center = point(circle.center.x as f32, circle.center.y as f32);
                    let radius = circle.radius as f32;
                    builder.add_ellipse(
                        center,
                        lyon::math::vector(radius, radius),
                        lyon::math::Angle::radians(0.0),
                        lyon::path::Winding::Positive,
                    );
                    Some(builder.build())
                }
                DxfEntity::Arc(arc) => {
                    let mut builder = Path::builder();
                    let center = point(arc.center.x as f32, arc.center.y as f32);
                    let radius = arc.radius as f32;
                    let start_angle = lyon::math::Angle::degrees(arc.start_angle as f32);
                    let end_angle = lyon::math::Angle::degrees(arc.end_angle as f32);
                    let sweep_angle = end_angle - start_angle;
                    
                    let start_point = center + lyon::math::vector(
                        radius * start_angle.radians.cos(), 
                        radius * start_angle.radians.sin()
                    );

                    builder.begin(start_point);
                    
                    let arc_geom = Arc {
                        center,
                        radii: lyon::math::vector(radius, radius),
                        x_rotation: lyon::math::Angle::radians(0.0),
                        start_angle,
                        sweep_angle,
                    };
                    
                    arc_geom.for_each_cubic_bezier(&mut |ctrl| {
                        builder.cubic_bezier_to(ctrl.ctrl1, ctrl.ctrl2, ctrl.to);
                    });

                    builder.end(false);
                    Some(builder.build())
                }
                DxfEntity::Polyline(polyline) => {
                    if polyline.vertices.is_empty() { None }
                    else {
                        let mut builder = Path::builder();
                        let start = polyline.vertices[0];
                        builder.begin(point(start.x as f32, start.y as f32));
                        for v in polyline.vertices.iter().skip(1) {
                            builder.line_to(point(v.x as f32, v.y as f32));
                        }
                        if polyline.closed {
                            builder.close();
                        } else {
                            builder.end(false);
                        }
                        Some(builder.build())
                    }
                }
                _ => None,
            };

            if let Some(path) = path_opt {
                let transformed_path = path.clone().transformed(&transform);
                shapes.push(Box::new(PathShape::new(transformed_path)));
            }
        }

        Ok(shapes)
    }
}


