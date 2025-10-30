//! # Designer Module
//!
//! Provides a 2D CAD/CAM design tool for creating CNC projects and generating G-code.
//! 
//! The designer module includes:
//! - Shape drawing and manipulation
//! - Canvas with zoom/pan controls with proper coordinate mapping
//! - Viewport management for pixel-to-world coordinate conversion
//! - Toolpath generation from design shapes
//! - CAM operations: pocket milling, drilling patterns, multi-pass depth control
//! - Tool library management
//! - Toolpath simulation and visualization
//! - G-code export to the G-Code Editor
//!
//! This module is organized into sub-modules:
//! - `shapes` - Geometric shape definitions and operations
//! - `canvas` - Canvas state and drawing operations
//! - `viewport` - Coordinate transformation and viewport management
//! - `toolpath` - Toolpath generation from shapes
//! - `tool_library` - Tool definitions and management
//! - `pocket_operations` - Pocket milling with island detection
//! - `drilling_patterns` - Drilling pattern generation
//! - `multipass` - Multi-pass depth control and ramping
//! - `toolpath_simulation` - Toolpath preview and analysis
//! - `gcode_gen` - G-code generation from toolpaths

pub mod canvas;
pub mod gcode_gen;
pub mod renderer;
pub mod shapes;
pub mod toolpath;
pub mod viewport;
pub mod tool_library;
pub mod pocket_operations;
pub mod drilling_patterns;
pub mod multipass;
pub mod toolpath_simulation;

pub use canvas::{Canvas, CanvasPoint, DrawingMode};
pub use gcode_gen::ToolpathToGcode;
pub use shapes::{Circle, Line, Point, Rectangle, Shape, ShapeType};
pub use toolpath::{Toolpath, ToolpathGenerator, ToolpathSegment, ToolpathSegmentType};
pub use viewport::Viewport;
pub use tool_library::{Tool, ToolLibrary, ToolType, CoolantType, MaterialProfile};
pub use pocket_operations::{PocketOperation, PocketGenerator, Island};
pub use drilling_patterns::{DrillOperation, DrillingPattern, DrillingPatternGenerator, PatternType};
pub use multipass::{MultiPassConfig, MultiPassToolpathGenerator, DepthStrategy};
pub use toolpath_simulation::{ToolpathSimulator, ToolpathAnalyzer, SimulationState, ToolPosition};
