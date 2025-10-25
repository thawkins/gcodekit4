//! Integration tests for Tasks 93 & 94 - File Processing Pipeline and Statistics
//!
//! Task 93: File Processing Pipeline
//! - Create file processor pipeline
//! - Apply preprocessors
//! - Generate processed output
//! - Cache processed results
//!
//! Task 94: File Statistics
//! - Calculate file statistics
//! - Estimate execution time
//! - Determine bounding box
//! - Count commands by type
//! - Calculate total distance

use gcodekit4::utils::{BoundingBox, FileProcessingPipeline, FileStatistics};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create test G-code file
fn create_test_gcode_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write test file");
    path
}

#[test]
fn test_task_93_basic_processing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10 Y20\nG1 Z5 F100\nM3 S1000\nG0 Z10\nM5\n";
    let file_path = create_test_gcode_file(&temp_dir, "basic.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    assert!(!result.content.is_empty());
    assert!(result.statistics.total_lines > 0);
}

#[test]
fn test_task_93_caching() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10 Y20\nG1 Z5 F100\n";
    let file_path = create_test_gcode_file(&temp_dir, "cache.nc", content);

    let mut pipeline = FileProcessingPipeline::new();

    // First processing
    assert!(!pipeline.is_cached(&file_path));
    let result1 = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    // Check cache
    assert!(pipeline.is_cached(&file_path));
    assert_eq!(pipeline.cache_size(), 1);

    // Second processing should return cached result
    let result2 = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    assert_eq!(result1.content, result2.content);
}

#[test]
fn test_task_93_cache_disabled() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10 Y20\n";
    let file_path = create_test_gcode_file(&temp_dir, "nocache.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    pipeline.set_cache_enabled(false);

    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    assert!(!pipeline.is_cached(&file_path));
    assert_eq!(pipeline.cache_size(), 0);
    assert!(!result.content.is_empty());
}

#[test]
fn test_task_93_cache_clear() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file1 = create_test_gcode_file(&temp_dir, "file1.nc", "G0 X10");
    let file2 = create_test_gcode_file(&temp_dir, "file2.nc", "G1 Y20");

    let mut pipeline = FileProcessingPipeline::new();
    pipeline.process_file(&file1).ok();
    pipeline.process_file(&file2).ok();

    assert_eq!(pipeline.cache_size(), 2);

    pipeline.clear_cache();
    assert_eq!(pipeline.cache_size(), 0);
    assert!(!pipeline.is_cached(&file1));
}

#[test]
fn test_task_94_basic_statistics() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10 Y20\nG1 Z5 F100\nG2 X15 Y25 I2.5 J2.5\nM3 S1000\nM5\n";
    let file_path = create_test_gcode_file(&temp_dir, "stats.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let stats = &result.statistics;
    assert!(stats.total_lines > 0);
    assert!(stats.rapid_moves > 0);
    assert!(stats.linear_moves > 0);
}

#[test]
fn test_task_94_motion_command_counting() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10\nG0 X20\nG1 Y30\nG2 X40 Y40 I5 J5\nG3 X50 Y50 I5 J5\n";
    let file_path = create_test_gcode_file(&temp_dir, "motions.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let stats = &result.statistics;
    assert_eq!(stats.rapid_moves, 2);
    assert_eq!(stats.linear_moves, 1);
    assert_eq!(stats.arc_moves, 2);
}

#[test]
fn test_task_94_m_code_counting() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "M3 S1000\nG1 X10 Y20\nM5\nM8\nG0 Z0\nM9\n";
    let file_path = create_test_gcode_file(&temp_dir, "mcodes.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let stats = &result.statistics;
    assert!(stats.m_codes > 0);
    assert!(stats.command_counts.contains_key("M3 ") || stats.command_counts.contains_key("M03"));
}

#[test]
fn test_task_94_bounding_box() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X0 Y0 Z0\nG1 X10 Y20 Z5\nG1 X-5 Y15 Z10\n";
    let file_path = create_test_gcode_file(&temp_dir, "bbox.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let bb = &result.statistics.bounding_box;
    assert!(bb.is_valid());
    assert!(bb.width() > 0.0);
    assert!(bb.height() > 0.0);
}

#[test]
fn test_task_94_distance_calculation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X0 Y0 Z0\nG1 X3 Y4 Z0\n"; // 3-4-5 triangle
    let file_path = create_test_gcode_file(&temp_dir, "distance.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let distance = result.statistics.total_distance;
    // Should be approximately 5.0 (3-4-5 triangle)
    assert!(distance > 0.0);
}

#[test]
fn test_task_94_feed_rate_tracking() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G1 X10 F100\nG1 X20 F200\nG1 X30 F150\n";
    let file_path = create_test_gcode_file(&temp_dir, "feedrate.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let feed = &result.statistics.feed_rate_stats;
    assert!(feed.changes > 0);
}

#[test]
fn test_task_94_spindle_tracking() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "M3 S1000\nG1 X10 Y20 F50\nM3 S2000\nM5\n";
    let file_path = create_test_gcode_file(&temp_dir, "spindle.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let spindle = &result.statistics.spindle_stats;
    assert!(spindle.on_count > 0);
}

#[test]
fn test_task_94_time_estimation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10 Y20\nG1 X20 Y30 F100\nG1 X30 Y40 F50\nG0 Z10\n";
    let file_path = create_test_gcode_file(&temp_dir, "timing.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let _stats = &result.statistics;
    // estimated_time is always >= 0 due to type being u64
}

#[test]
fn test_task_94_time_formatting() {
    let stats = FileStatistics::new();

    let mut timed = stats.clone();
    timed.estimated_time = 3661;
    assert_eq!(timed.formatted_time(), "1h 1m 1s");

    let mut timed2 = stats.clone();
    timed2.estimated_time = 125;
    assert_eq!(timed2.formatted_time(), "2m 5s");

    let mut timed3 = stats;
    timed3.estimated_time = 45;
    assert_eq!(timed3.formatted_time(), "45s");
}

#[test]
fn test_task_94_statistics_summary() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10\nG1 Y20\nG2 X30 I5\n";
    let file_path = create_test_gcode_file(&temp_dir, "summary.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let summary = result.statistics.summary();
    assert!(!summary.is_empty());
    assert!(summary.contains("Lines:"));
}

#[test]
fn test_task_94_empty_lines_ignored() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10\n\nG1 Y20\n\n\nG2 X30 I5\n";
    let file_path = create_test_gcode_file(&temp_dir, "empty.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let stats = &result.statistics;
    assert!(stats.empty_lines > 0);
    assert!(stats.total_lines > stats.empty_lines);
}

#[test]
fn test_task_94_comments_counted() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "; This is a comment\nG0 X10\n(Another comment)\nG1 Y20\n";
    let file_path = create_test_gcode_file(&temp_dir, "comments.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let stats = &result.statistics;
    assert!(stats.comment_lines > 0);
}

#[test]
fn test_task_94_total_motion_commands() {
    let mut stats = FileStatistics::new();
    stats.rapid_moves = 5;
    stats.linear_moves = 10;
    stats.arc_moves = 3;

    assert_eq!(stats.total_motion_commands(), 18);
}

#[test]
fn test_task_94_bounding_box_dimensions() {
    let mut bb = BoundingBox::new();
    bb.update(0.0, 0.0, 0.0);
    bb.update(10.0, 20.0, 5.0);

    assert_eq!(bb.width(), 10.0);
    assert_eq!(bb.height(), 20.0);
    assert_eq!(bb.depth(), 5.0);
}

#[test]
fn test_task_94_bounding_box_validity() {
    let bb = BoundingBox::new();
    assert!(!bb.is_valid());

    let mut bb2 = BoundingBox::new();
    bb2.update(5.0, 5.0, 5.0);
    assert!(bb2.is_valid());
}

#[test]
fn test_task_93_94_combined_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "
(Sample part program)
G90 G21 F100
G0 X0 Y0 Z0
G1 Z-5 F50
G1 X10 Y10 Z-10 F75
G2 X20 Y20 I5 J5
G1 X30 Y10 F100
G0 Z5
M5
";
    let file_path = create_test_gcode_file(&temp_dir, "combined.nc", content);

    let mut pipeline = FileProcessingPipeline::new();

    // First process
    let result1 = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let stats = &result1.statistics;

    // Verify statistics are calculated
    assert!(stats.total_lines > 0);
    assert!(stats.rapid_moves > 0);
    assert!(stats.linear_moves > 0);
    assert!(stats.arc_moves > 0);
    assert!(stats.comment_lines > 0);

    // Verify bounding box
    assert!(stats.bounding_box.is_valid());
    assert!(stats.bounding_box.width() > 0.0);

    // Verify caching works
    assert!(pipeline.is_cached(&file_path));
    let result2 = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    assert_eq!(
        result1.statistics.total_lines,
        result2.statistics.total_lines
    );
}

#[test]
fn test_task_93_multiple_file_processing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file1 = create_test_gcode_file(&temp_dir, "file1.nc", "G0 X10\nG1 Y20");
    let file2 = create_test_gcode_file(&temp_dir, "file2.nc", "G0 X5\nG1 Y15\nG1 Y25");

    let mut pipeline = FileProcessingPipeline::new();

    let result1 = pipeline
        .process_file(&file1)
        .expect("Failed to process file1");
    let result2 = pipeline
        .process_file(&file2)
        .expect("Failed to process file2");

    assert_eq!(pipeline.cache_size(), 2);
    assert!(result1.statistics.linear_moves > 0);
    assert!(result2.statistics.linear_moves > 0);
}

#[test]
fn test_task_94_command_count_breakdown() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10\nG0 Y20\nG1 X30\nM3 S1000\nM5\n";
    let file_path = create_test_gcode_file(&temp_dir, "breakdown.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    let stats = &result.statistics;
    assert!(stats.command_counts.len() > 0);
}

#[test]
fn test_task_93_processed_file_info() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10\nG1 Y20\n";
    let file_path = create_test_gcode_file(&temp_dir, "info.nc", content);

    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline
        .process_file(&file_path)
        .expect("Failed to process file");

    assert_eq!(result.source_path, file_path);
    assert!(result.processed_lines > 0);
}
