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
pub mod optimizer;
pub mod stats;
pub mod toolpath;
pub mod validator;

pub use advanced_features::{
    CommandHistory, ProbingSystem, SimulationMode, SoftLimits, ToolLibrary, WorkCoordinateManager,
};
pub use arc_expander::ArcExpander;
pub use comment_processor::CommentProcessor;
pub use core_infrastructure::{AppConfig, ApplicationState, Logger, TelemetryData};
pub use optimizer::GCodeOptimizer;
pub use stats::StatsCalculator;
pub use toolpath::{Segment, SegmentType, Toolpath};
pub use validator::GCodeValidator;
