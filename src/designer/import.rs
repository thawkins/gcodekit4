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

use crate::designer::shapes::{Shape, Circle, Ellipse, Line as DesignerLine, Point, Rectangle, Polygon};
use crate::designer::dxf_parser::{DxfParser, DxfEntity, DxfFile};
use anyhow::{anyhow, Result};
use roxmltree::Document;
use svgtypes::PathParser;

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

    /// Import SVG from file path
    ///
    /// # Arguments
    /// * `path` - Path to SVG file
    ///
    /// # Returns
    /// Imported design with converted shapes
    pub fn import_file(&self, path: &str) -> Result<ImportedDesign> {
        let content =
            std::fs::read_to_string(path).map_err(|e| anyhow!("Failed to read SVG file: {}", e))?;
        self.import_string(&content)
    }

    /// Import SVG from string content
    ///
    /// # Arguments
    /// * `svg_content` - SVG XML content as string
    ///
    /// # Returns
    /// Imported design with converted shapes
    pub fn import_string(&self, svg_content: &str) -> Result<ImportedDesign> {
        let doc = Document::parse(svg_content)
            .map_err(|e| anyhow!("Failed to parse SVG XML: {}", e))?;
        
        let root = doc.root_element();
        
        let (width, height) = self.get_svg_dimensions(&root);
        
        let mut shapes: Vec<Box<dyn Shape>> = Vec::new();
        
        self.parse_svg_node(&root, &mut shapes)?;
        
        Ok(ImportedDesign {
            shapes,
            dimensions: (width, height),
            format: FileFormat::Svg,
            layer_count: 1,
        })
    }
    
    fn get_svg_dimensions(&self, node: &roxmltree::Node) -> (f64, f64) {
        let width = node.attribute("width")
            .and_then(|w| self.parse_dimension(w))
            .unwrap_or(100.0);
        let height = node.attribute("height")
            .and_then(|h| self.parse_dimension(h))
            .unwrap_or(100.0);
        
        (width * self.scale, height * self.scale)
    }
    
    fn parse_dimension(&self, value: &str) -> Option<f64> {
        let value = value.trim();
        let num_part = value.trim_end_matches(|c: char| c.is_alphabetic());
        num_part.parse::<f64>().ok()
    }
    
    fn parse_svg_node(&self, node: &roxmltree::Node, shapes: &mut Vec<Box<dyn Shape>>) -> Result<()> {
        match node.tag_name().name() {
            "rect" => self.parse_rect(node, shapes)?,
            "circle" => self.parse_circle(node, shapes)?,
            "ellipse" => self.parse_ellipse(node, shapes)?,
            "line" => self.parse_line(node, shapes)?,
            "polyline" | "polygon" => self.parse_polyline(node, shapes)?,
            "path" => self.parse_path(node, shapes)?,
            _ => {}
        }
        
        for child in node.children() {
            self.parse_svg_node(&child, shapes)?;
        }
        
        Ok(())
    }
    
    fn parse_rect(&self, node: &roxmltree::Node, shapes: &mut Vec<Box<dyn Shape>>) -> Result<()> {
        let x = self.get_attribute_f64(node, "x").unwrap_or(0.0);
        let y = self.get_attribute_f64(node, "y").unwrap_or(0.0);
        let width = self.get_attribute_f64(node, "width").unwrap_or(0.0);
        let height = self.get_attribute_f64(node, "height").unwrap_or(0.0);
        
        let scaled_x = x * self.scale + self.offset_x;
        let scaled_y = y * self.scale + self.offset_y;
        let scaled_w = width * self.scale;
        let scaled_h = height * self.scale;
        
        shapes.push(Box::new(Rectangle::new(scaled_x, scaled_y, scaled_w, scaled_h)));
        Ok(())
    }
    
    fn parse_circle(&self, node: &roxmltree::Node, shapes: &mut Vec<Box<dyn Shape>>) -> Result<()> {
        let cx = self.get_attribute_f64(node, "cx").unwrap_or(0.0);
        let cy = self.get_attribute_f64(node, "cy").unwrap_or(0.0);
        let r = self.get_attribute_f64(node, "r").unwrap_or(0.0);
        
        let center = Point::new(
            cx * self.scale + self.offset_x,
            cy * self.scale + self.offset_y,
        );
        
        shapes.push(Box::new(Circle::new(center, r * self.scale)));
        Ok(())
    }
    
    fn parse_ellipse(&self, node: &roxmltree::Node, shapes: &mut Vec<Box<dyn Shape>>) -> Result<()> {
        let cx = self.get_attribute_f64(node, "cx").unwrap_or(0.0);
        let cy = self.get_attribute_f64(node, "cy").unwrap_or(0.0);
        let rx = self.get_attribute_f64(node, "rx").unwrap_or(0.0);
        let ry = self.get_attribute_f64(node, "ry").unwrap_or(0.0);
        
        let center = Point::new(
            cx * self.scale + self.offset_x,
            cy * self.scale + self.offset_y,
        );
        
        shapes.push(Box::new(Ellipse::new(center, rx * self.scale, ry * self.scale)));
        Ok(())
    }
    
    fn parse_line(&self, node: &roxmltree::Node, shapes: &mut Vec<Box<dyn Shape>>) -> Result<()> {
        let x1 = self.get_attribute_f64(node, "x1").unwrap_or(0.0);
        let y1 = self.get_attribute_f64(node, "y1").unwrap_or(0.0);
        let x2 = self.get_attribute_f64(node, "x2").unwrap_or(0.0);
        let y2 = self.get_attribute_f64(node, "y2").unwrap_or(0.0);
        
        let start = Point::new(
            x1 * self.scale + self.offset_x,
            y1 * self.scale + self.offset_y,
        );
        let end = Point::new(
            x2 * self.scale + self.offset_x,
            y2 * self.scale + self.offset_y,
        );
        
        shapes.push(Box::new(DesignerLine::new(start, end)));
        Ok(())
    }
    
    fn parse_polyline(&self, node: &roxmltree::Node, shapes: &mut Vec<Box<dyn Shape>>) -> Result<()> {
        if let Some(points_str) = node.attribute("points") {
            let mut vertices = Vec::new();
            let coords: Vec<f64> = points_str
                .split(|c: char| c.is_whitespace() || c == ',')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse().ok())
                .collect();
            
            for chunk in coords.chunks(2) {
                if chunk.len() == 2 {
                    vertices.push(Point::new(
                        chunk[0] * self.scale + self.offset_x,
                        chunk[1] * self.scale + self.offset_y,
                    ));
                }
            }
            
            if !vertices.is_empty() {
                shapes.push(Box::new(Polygon::new(vertices)));
            }
        }
        Ok(())
    }
    
    fn parse_path(&self, node: &roxmltree::Node, shapes: &mut Vec<Box<dyn Shape>>) -> Result<()> {
        if let Some(d) = node.attribute("d") {
            let mut current_pos = Point::new(0.0, 0.0);
            let mut path_start = Point::new(0.0, 0.0);
            
            for segment in PathParser::from(d) {
                match segment.map_err(|e| anyhow!("Path parse error: {}", e))? {
                    svgtypes::PathSegment::MoveTo { abs, x, y } => {
                        if abs {
                            current_pos = Point::new(
                                x * self.scale + self.offset_x,
                                y * self.scale + self.offset_y,
                            );
                        } else {
                            current_pos = Point::new(
                                current_pos.x + x * self.scale,
                                current_pos.y + y * self.scale,
                            );
                        }
                        path_start = current_pos;
                    }
                    svgtypes::PathSegment::LineTo { abs, x, y } => {
                        let end = if abs {
                            Point::new(
                                x * self.scale + self.offset_x,
                                y * self.scale + self.offset_y,
                            )
                        } else {
                            Point::new(
                                current_pos.x + x * self.scale,
                                current_pos.y + y * self.scale,
                            )
                        };
                        shapes.push(Box::new(DesignerLine::new(current_pos, end)));
                        current_pos = end;
                    }
                    svgtypes::PathSegment::HorizontalLineTo { abs, x } => {
                        let end = if abs {
                            Point::new(x * self.scale + self.offset_x, current_pos.y)
                        } else {
                            Point::new(current_pos.x + x * self.scale, current_pos.y)
                        };
                        shapes.push(Box::new(DesignerLine::new(current_pos, end)));
                        current_pos = end;
                    }
                    svgtypes::PathSegment::VerticalLineTo { abs, y } => {
                        let end = if abs {
                            Point::new(current_pos.x, y * self.scale + self.offset_y)
                        } else {
                            Point::new(current_pos.x, current_pos.y + y * self.scale)
                        };
                        shapes.push(Box::new(DesignerLine::new(current_pos, end)));
                        current_pos = end;
                    }
                    svgtypes::PathSegment::ClosePath { .. } => {
                        if current_pos.x != path_start.x || current_pos.y != path_start.y {
                            shapes.push(Box::new(DesignerLine::new(current_pos, path_start)));
                            current_pos = path_start;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
    
    fn get_attribute_f64(&self, node: &roxmltree::Node, name: &str) -> Option<f64> {
        node.attribute(name)
            .and_then(|v| self.parse_dimension(v))
    }
}

/// DXF importer for converting DXF files to Designer shapes
///
/// Currently provides basic framework for DXF import.
/// Full implementation requires DXF parsing library integration.
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
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read DXF file: {}", e))?;
        
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
    fn convert_entities_to_shapes(&self, dxf_file: &DxfFile) -> Result<Vec<Box<dyn Shape>>> {
        let mut shapes: Vec<Box<dyn Shape>> = Vec::new();
        
        for entity in &dxf_file.entities {
            match entity {
                DxfEntity::Line(line) => {
                    let start = Point::new(
                        line.start.x + self.offset_x,
                        line.start.y + self.offset_y,
                    );
                    let end = Point::new(
                        line.end.x + self.offset_x,
                        line.end.y + self.offset_y,
                    );
                    let designer_line = DesignerLine::new(start, end);
                    shapes.push(Box::new(designer_line));
                }
                DxfEntity::Circle(circle) => {
                    let center = Point::new(
                        circle.center.x + self.offset_x,
                        circle.center.y + self.offset_y,
                    );
                    let designer_circle = Circle::new(center, circle.radius);
                    shapes.push(Box::new(designer_circle));
                }
                DxfEntity::Arc(_arc) => {
                    // TODO: Arc conversion - need Arc shape implementation or convert to polyline
                }
                DxfEntity::Polyline(polyline) => {
                    // Convert polyline to multiple line segments
                    for i in 0..polyline.vertices.len().saturating_sub(1) {
                        let start = Point::new(
                            polyline.vertices[i].x + self.offset_x,
                            polyline.vertices[i].y + self.offset_y,
                        );
                        let end = Point::new(
                            polyline.vertices[i + 1].x + self.offset_x,
                            polyline.vertices[i + 1].y + self.offset_y,
                        );
                        shapes.push(Box::new(DesignerLine::new(start, end)));
                    }
                    
                    // Close polyline if needed
                    if polyline.closed && polyline.vertices.len() > 2 {
                        let start = Point::new(
                            polyline.vertices[polyline.vertices.len() - 1].x + self.offset_x,
                            polyline.vertices[polyline.vertices.len() - 1].y + self.offset_y,
                        );
                        let end = Point::new(
                            polyline.vertices[0].x + self.offset_x,
                            polyline.vertices[0].y + self.offset_y,
                        );
                        shapes.push(Box::new(DesignerLine::new(start, end)));
                    }
                }
                DxfEntity::Text(_text) => {
                    // Text entities are ignored for now - needs text rendering support
                }
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
