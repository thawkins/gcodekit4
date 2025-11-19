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
use crate::shapes::{Circle, Line as DesignerLine, Point, Shape, PathShape};
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

        let mut shapes: Vec<Box<dyn Shape>> = Vec::new();
        let mut viewbox_width = 100.0f64;
        let mut _viewbox_height = 100.0f64;

        // Parse viewBox from SVG element
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
                            
                            // Apply importer scale and offset
                            let scale_transform = lyon::math::Transform::scale(self.scale as f32, self.scale as f32)
                                .then_translate(lyon::math::vector(self.offset_x as f32, self.offset_y as f32));
                            let scaled_path = final_path.clone().transformed(&scale_transform);

                            shapes.push(Box::new(PathShape::new(scaled_path)));
                        }
                    }
                }

                search_pos = path_tag_end + 1;
            } else {
                break;
            }
        }

        Ok(ImportedDesign {
            shapes,
            dimensions: (viewbox_width, _viewbox_height),
            format: FileFormat::Svg,
            layer_count: 0,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_importer_creation() {
        let importer = SvgImporter::new(1.0, 0.0, 0.0);
        assert_eq!(importer.scale, 1.0);
    }

    #[test]
    fn test_dxf_importer_creation() {
        let importer = DxfImporter::new(1.0, 0.0, 0.0);
        assert_eq!(importer.scale, 1.0);
    }

    #[test]
    fn test_svg_import_basic() {
        let importer = SvgImporter::new(1.0, 0.0, 0.0);
        let svg = r#"<svg width="100" height="100"></svg>"#;
        let result = importer.import_string(svg);

        assert!(result.is_ok());
        let design = result.unwrap();
        assert_eq!(design.format, FileFormat::Svg);
        assert_eq!(design.dimensions.0, 100.0);
        assert_eq!(design.dimensions.1, 100.0);
    }

    #[test]
    fn test_svg_import_rectangle() {
        let importer = SvgImporter::new(1.0, 0.0, 0.0);
        let svg = r#"<svg><rect x="10" y="20" width="30" height="40"/></svg>"#;
        let result = importer.import_string(svg);

        assert!(result.is_ok());
        let design = result.unwrap();
        assert_eq!(design.shapes.len(), 1);
    }

    #[test]
    fn test_svg_import_circle() {
        let importer = SvgImporter::new(1.0, 0.0, 0.0);
        let svg = r#"<svg><circle cx="50" cy="50" r="25"/></svg>"#;
        let result = importer.import_string(svg);

        assert!(result.is_ok());
        let design = result.unwrap();
        assert_eq!(design.shapes.len(), 1);
    }

    #[test]
    fn test_svg_import_line() {
        let importer = SvgImporter::new(1.0, 0.0, 0.0);
        let svg = r#"<svg><line x1="0" y1="0" x2="100" y2="100"/></svg>"#;
        let result = importer.import_string(svg);

        assert!(result.is_ok());
        let design = result.unwrap();
        assert_eq!(design.shapes.len(), 1);
    }

    #[test]
    fn test_svg_import_with_scale() {
        let importer = SvgImporter::new(2.0, 0.0, 0.0);
        let svg = r#"<svg width="100" height="100"></svg>"#;
        let result = importer.import_string(svg);

        assert!(result.is_ok());
        let design = result.unwrap();
        assert_eq!(design.dimensions.0, 200.0);
        assert_eq!(design.dimensions.1, 200.0);
    }

    #[test]
    fn test_dxf_import_framework() {
        let importer = DxfImporter::new(1.0, 0.0, 0.0);
        let result = importer.import_string("0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF");

        assert!(result.is_ok());
        let design = result.unwrap();
        assert_eq!(design.format, FileFormat::Dxf);
    }
}
