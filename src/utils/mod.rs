//! Utility functions and helpers

pub mod file_io;
pub mod processing;

pub use file_io::{
    FileEncoding, FileReadStats, FileValidation, GcodeFileReader, RecentFileEntry,
    RecentFilesManager,
};
pub use processing::{
    BoundingBox, FeedRateStats, FileProcessingPipeline, FileStatistics, ProcessedFile,
    SpindleStats,
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
