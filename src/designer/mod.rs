//! # Designer Module
//!
//! Provides a 2D CAD/CAM design tool for creating CNC projects and generating G-code.
//! 
//! The designer module includes:
//! - Shape drawing and manipulation
//! - Canvas with zoom/pan controls
//! - Toolpath generation from design shapes
//! - G-code export to the G-Code Editor
//!
//! This module is organized into sub-modules:
//! - `shapes` - Geometric shape definitions and operations
//! - `canvas` - Canvas state and drawing operations
//! - `toolpath` - Toolpath generation from shapes
//! - `gcode_gen` - G-code generation from toolpaths

pub mod canvas;
pub mod gcode_gen;
pub mod shapes;
pub mod toolpath;

pub use canvas::{Canvas, CanvasPoint, DrawingMode};
pub use gcode_gen::ToolpathToGcode;
pub use shapes::{Circle, Line, Point, Rectangle, Shape, ShapeType};
pub use toolpath::{Toolpath, ToolpathGenerator, ToolpathSegment, ToolpathSegmentType};
