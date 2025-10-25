//! Utility functions and helpers

pub mod file_io;
pub mod processing;
pub mod export;
pub mod advanced;
pub mod phase6_extended;

pub use file_io::{
    FileEncoding, FileReadStats, FileValidation, GcodeFileReader, RecentFileEntry,
    RecentFilesManager,
};
pub use processing::{
    BoundingBox, FeedRateStats, FileProcessingPipeline, FileStatistics, ProcessedFile,
    SpindleStats,
};
pub use export::{
    DropEvent, DropFileType, DropIndicatorState, DropTarget, DropZone, ExportOptions,
    FileExporter, FileFormat,
};
pub use advanced::{
    AdvancedProber, BasicProber, BackupEntry, BackupManager, FileComparison,
    GcodeTemplate, TemplateLibrary, TemplateVariable, ValidationIssue, ValidationResult,
    ValidationSeverity, ProbePoint,
};
pub use phase6_extended::{
    ProbeMesh, HeightPoint, ToolInfo, ToolLibrary, ToolOffset, ToolOffsetManager,
    WorkOffset, WorkCoordinateSystem, SoftLimits, Simulator, SimulationPosition, Stepper,
    Bookmark, BookmarkManager, ProgramState, PerformanceMetrics, HistoryEntry, CommandHistory,
    CustomMacro, PendantButton, PendantConfig, CustomAction, AutoConnectConfig, NetworkConfig,
    LogEntry, DataLogger, Alarm, AlarmType, AlarmManager,
};

/// Format a float to a reasonable number of decimal places
pub fn format_float(value: f64, precision: usize) -> String {
    format!("{:.prec$}", value, prec = precision)
}

/// Convert degrees to radians
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

/// Convert radians to degrees
pub fn radians_to_degrees(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}
