//! G-Code Processing Module (Tasks 41-150)
//!
//! Advanced G-code processing and advanced features:
//! - Tasks 51-65: Advanced G-code processing (Phase 4)
//! - Tasks 101-125: Advanced features (probing, tools, simulation, etc.)
//! - Tasks 126-150: Core infrastructure and polish

pub mod advanced_features;
pub mod arc_expander;
pub mod comment_processor;
pub mod core_infrastructure;
pub mod jigsaw_puzzle;
pub mod laser_engraver;
pub mod optimizer;
pub mod stats;
pub mod tabbed_box;
pub mod toolpath;
pub mod validator;

pub use advanced_features::{
    CommandHistory, ProbingSystem, SimulationMode, SoftLimits, ToolLibrary, WorkCoordinateManager,
};
pub use arc_expander::ArcExpander;
pub use comment_processor::CommentProcessor;
pub use core_infrastructure::{AppConfig, ApplicationState, Logger, TelemetryData};
pub use jigsaw_puzzle::{JigsawPuzzleMaker, PuzzleParameters};
pub use laser_engraver::{EngravingParameters, LaserEngraver, ScanDirection};
pub use optimizer::GCodeOptimizer;
pub use stats::StatsCalculator;
pub use tabbed_box::{BoxParameters, BoxType, FingerJointSettings, FingerStyle, TabbedBoxMaker};
pub use toolpath::{Segment, SegmentType, Toolpath};
pub use validator::GCodeValidator;
