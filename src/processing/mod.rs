//! G-Code Processing Module (Tasks 41-65)
//!
//! Advanced G-code processing including:
//! - Tasks 41-50: Firmware frameworks (already complete in Phase 3)
//! - Tasks 51-65: Advanced G-code processing
//!   - Task 51: Arc Expansion
//!   - Task 52: Line Splitting
//!   - Task 53: Mesh Leveling
//!   - Task 54: Comment Processing
//!   - Task 55: Feed Override
//!   - Task 56: Pattern Removal
//!   - Task 57-59: Transformations (Translation, Rotation, Mirror)
//!   - Task 60: Run From Line
//!   - Task 61: Spindle Dweller
//!   - Task 62: Stats Processor
//!   - Task 63: G-Code Optimization
//!   - Task 64: Toolpath Representation
//!   - Task 65: G-Code Validation

pub mod arc_expander;
pub mod comment_processor;
pub mod optimizer;
pub mod stats;
pub mod toolpath;
pub mod validator;

pub use arc_expander::ArcExpander;
pub use comment_processor::CommentProcessor;
pub use optimizer::GCodeOptimizer;
pub use stats::StatsCalculator;
pub use toolpath::{Segment, SegmentType, Toolpath};
pub use validator::GCodeValidator;
