//! G-Code Processing Module (Tasks 41-150)
//!
//! Advanced G-code processing and advanced features:
//! - Tasks 51-65: Advanced G-code processing (Phase 4)
//! - Tasks 101-125: Advanced features (probing, tools, simulation, etc.)
//! - Tasks 126-150: Core infrastructure and polish

pub mod arc_expander;
pub mod comment_processor;
pub mod optimizer;
pub mod stats;
pub mod toolpath;
pub mod validator;
pub mod advanced_features;
pub mod core_infrastructure;

pub use arc_expander::ArcExpander;
pub use comment_processor::CommentProcessor;
pub use optimizer::GCodeOptimizer;
pub use stats::StatsCalculator;
pub use advanced_features::{ProbingSystem, ToolLibrary, WorkCoordinateManager, SoftLimits, SimulationMode, CommandHistory};
pub use core_infrastructure::{AppConfig, Logger, TelemetryData, ApplicationState};
pub use toolpath::{Segment, SegmentType, Toolpath};
pub use validator::GCodeValidator;
