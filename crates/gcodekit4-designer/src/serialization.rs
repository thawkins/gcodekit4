//! Serialization and deserialization for designer files.
//!
//! Implements save/load functionality for .gck4 (GCodeKit4) design files
//! using JSON format with complete design state preservation.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::canvas::DrawingObject;
use super::shapes::*;

/// Design file format version
const FILE_FORMAT_VERSION: &str = "1.0";

/// Complete design file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignFile {
    pub version: String,
    pub metadata: DesignMetadata,
    pub viewport: ViewportState,
    pub shapes: Vec<ShapeData>,
    #[serde(default)]
    pub toolpath_params: ToolpathParameters,
}

/// Design metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignMetadata {
    pub name: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
}

/// Viewport state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportState {
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
}

/// Serialized shape data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeData {
    pub id: i32,
    pub shape_type: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub points: Vec<(f64, f64)>,
    pub selected: bool,
    #[serde(default)]
    pub operation_type: String,
    #[serde(default)]
    pub pocket_depth: f64,
    #[serde(default)]
    pub text_content: String,
    #[serde(default)]
    pub font_size: f64,
}

/// Toolpath generation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolpathParameters {
    #[serde(default = "default_feed_rate")]
    pub feed_rate: f64,
    #[serde(default = "default_spindle_speed")]
    pub spindle_speed: f64,
    #[serde(default = "default_tool_diameter")]
    pub tool_diameter: f64,
    #[serde(default = "default_cut_depth")]
    pub cut_depth: f64,
}

fn default_feed_rate() -> f64 {
    1000.0
}
fn default_spindle_speed() -> f64 {
    3000.0
}
fn default_tool_diameter() -> f64 {
    3.175
}
fn default_cut_depth() -> f64 {
    -5.0
}

impl Default for ToolpathParameters {
    fn default() -> Self {
        Self {
            feed_rate: default_feed_rate(),
            spindle_speed: default_spindle_speed(),
            tool_diameter: default_tool_diameter(),
            cut_depth: default_cut_depth(),
        }
    }
}

impl DesignFile {
    /// Create a new design file with default values
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            version: FILE_FORMAT_VERSION.to_string(),
            metadata: DesignMetadata {
                name: name.into(),
                created: now,
                modified: now,
                author: String::new(),
                description: String::new(),
            },
            viewport: ViewportState {
                zoom: 1.0,
                pan_x: 0.0,
                pan_y: 0.0,
            },
            shapes: Vec::new(),
            toolpath_params: ToolpathParameters::default(),
        }
    }

    /// Save design to file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string_pretty(self).context("Failed to serialize design")?;

        std::fs::write(path.as_ref(), json).context("Failed to write design file")?;

        Ok(())
    }

    /// Load design from file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content =
            std::fs::read_to_string(path.as_ref()).context("Failed to read design file")?;

        let mut design: DesignFile =
            serde_json::from_str(&content).context("Failed to parse design file")?;

        // Update modified timestamp
        design.metadata.modified = Utc::now();

        Ok(design)
    }

    /// Convert DrawingObject to ShapeData
    pub fn from_drawing_object(obj: &DrawingObject) -> ShapeData {
        let (x, y, x2, y2) = obj.shape.bounding_box();
        let width = x2 - x;
        let height = y2 - y;

        let shape_type = match obj.shape.shape_type() {
            ShapeType::Rectangle => "rectangle",
            ShapeType::Circle => "circle",
            ShapeType::Line => "line",
            ShapeType::Ellipse => "ellipse",
            ShapeType::Polygon => "polygon",
            ShapeType::Path => "path",
            ShapeType::Text => "text",
        };

        let (text_content, font_size) = if let Some(text_shape) = obj.shape.as_any().downcast_ref::<TextShape>() {
             (text_shape.text.clone(), text_shape.font_size)
        } else {
             (String::new(), 0.0)
        };

        ShapeData {
            id: obj.id as i32,
            shape_type: shape_type.to_string(),
            x,
            y,
            width,
            height,
            points: Vec::new(),
            selected: false,
            operation_type: match obj.operation_type {
                OperationType::Profile => "profile".to_string(),
                OperationType::Pocket => "pocket".to_string(),
            },
            pocket_depth: obj.pocket_depth,
            text_content,
            font_size,
        }
    }

    /// Convert ShapeData to DrawingObject
    pub fn to_drawing_object(data: &ShapeData, next_id: i32) -> Result<DrawingObject> {
        let shape: Box<dyn Shape> = match data.shape_type.as_str() {
            "rectangle" => Box::new(Rectangle::new(data.x, data.y, data.width, data.height)),
            "circle" => {
                let radius = data.width.min(data.height) / 2.0;
                let center = Point::new(data.x + radius, data.y + radius);
                Box::new(Circle::new(center, radius))
            }
            "line" => {
                let start = Point::new(data.x, data.y);
                let end = Point::new(data.x + data.width, data.y + data.height);
                Box::new(Line::new(start, end))
            }
            "ellipse" => {
                let center = Point::new(data.x + data.width / 2.0, data.y + data.height / 2.0);
                Box::new(Ellipse::new(center, data.width / 2.0, data.height / 2.0))
            }
            "polygon" => {
                let center = Point::new(data.x + data.width / 2.0, data.y + data.height / 2.0);
                let radius = data.width.min(data.height) / 2.0;
                Box::new(Polygon::regular(center, radius, 6))
            }
            "text" => Box::new(TextShape::new(
                data.text_content.clone(),
                data.x,
                data.y,
                data.font_size,
            )),
            _ => anyhow::bail!("Unknown shape type: {}", data.shape_type),
        };

        let operation_type = match data.operation_type.as_str() {
            "pocket" => OperationType::Pocket,
            _ => OperationType::Profile,
        };

        Ok(DrawingObject {
            id: next_id as u64,
            shape,
            selected: data.selected,
            operation_type,
            pocket_depth: data.pocket_depth,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new_design() {
        let design = DesignFile::new("Test Design");
        assert_eq!(design.version, "1.0");
        assert_eq!(design.metadata.name, "Test Design");
        assert_eq!(design.shapes.len(), 0);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_design.gck4");

        let mut design = DesignFile::new("Test");
        design.shapes.push(ShapeData {
            id: 1,
            shape_type: "rectangle".to_string(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            points: Vec::new(),
            selected: false,
        });

        design.save_to_file(&file_path).unwrap();
        let loaded = DesignFile::load_from_file(&file_path).unwrap();

        assert_eq!(loaded.shapes.len(), 1);
        assert_eq!(loaded.shapes[0].width, 100.0);

        std::fs::remove_file(&file_path).ok();
    }
}
