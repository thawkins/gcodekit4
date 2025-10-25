//! Integration tests for Tasks 91 & 92 - File I/O and Recent Files
//!
//! Task 91: File I/O - Reading
//! - G-code file reader with UTF-8/ASCII support
//! - Efficient large file handling with streaming
//! - File validation capabilities
//!
//! Task 92: File I/O - Recent Files
//! - Track recently opened files
//! - Provide recent files menu functionality
//! - Persist recent files to disk

use gcodekit4::utils::{FileEncoding, GcodeFileReader, RecentFilesManager};
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
fn test_task_91_basic_file_reading() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10 Y20\nG1 Z5 F100\nM3 S1000\n";
    let file_path = create_test_gcode_file(&temp_dir, "test.nc", content);

    let reader = GcodeFileReader::new(&file_path).expect("Failed to create reader");
    assert_eq!(reader.file_size(), content.len() as u64);

    let text = reader.read_all().expect("Failed to read file");
    assert_eq!(text, content);
}

#[test]
fn test_task_91_file_reading_streaming() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lines = vec!["G0 X10 Y20", "G1 Z5 F100", "M3 S1000"];
    let content = lines.join("\n");
    let file_path = create_test_gcode_file(&temp_dir, "stream.nc", &content);

    let reader = GcodeFileReader::new(&file_path).expect("Failed to create reader");

    let mut collected_lines = Vec::new();
    let stats = reader
        .read_lines(|line| {
            collected_lines.push(line.to_string());
            Ok(())
        })
        .expect("Failed to read lines");

    assert_eq!(collected_lines.len(), 3);
    assert_eq!(stats.lines_read, 3);
    assert!(stats.bytes_read > 0);
    assert_eq!(stats.encoding, FileEncoding::Utf8);
}

#[test]
fn test_task_91_file_encoding_detection() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Test UTF-8 content
    let utf8_content = "G0 X10 Y20\nG1 Z5 F100\n";
    let utf8_path = create_test_gcode_file(&temp_dir, "utf8.nc", utf8_content);
    let reader = GcodeFileReader::new(&utf8_path).expect("Failed to create reader");

    let mut encoding = FileEncoding::Utf8;
    reader
        .read_lines(|_line| {
            encoding = FileEncoding::detect(_line.as_bytes());
            Ok(())
        })
        .expect("Failed to detect encoding");

    assert_eq!(encoding, FileEncoding::Utf8);
}

#[test]
fn test_task_91_file_validation_simple() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10 Y20\nG1 Z5 F100\nG2 X20 Y30 I5 J5\nM3 S1000\n";
    let file_path = create_test_gcode_file(&temp_dir, "validate.nc", content);

    let reader = GcodeFileReader::new(&file_path).expect("Failed to create reader");
    let validation = reader.validate().expect("Failed to validate");

    assert!(validation.is_valid);
    assert_eq!(validation.total_lines, 4);
    assert_eq!(validation.rapid_moves, 1);
    assert_eq!(validation.linear_moves, 1);
    assert_eq!(validation.arc_moves, 1);
    assert_eq!(validation.total_motion_commands(), 3);
}

#[test]
fn test_task_91_file_validation_empty_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = create_test_gcode_file(&temp_dir, "empty.nc", "");

    let reader = GcodeFileReader::new(&file_path).expect("Failed to create reader");
    let validation = reader.validate().expect("Failed to validate");

    assert!(!validation.is_valid);
    assert!(validation.errors.contains(&"File is empty".to_string()));
}

#[test]
fn test_task_91_file_validation_no_motion() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "(This is just a comment)\n(Another comment)\n";
    let file_path = create_test_gcode_file(&temp_dir, "comments.nc", content);

    let reader = GcodeFileReader::new(&file_path).expect("Failed to create reader");
    let validation = reader.validate().expect("Failed to validate");

    assert!(!validation.is_valid);
    assert!(validation
        .warnings
        .iter()
        .any(|w| w.contains("no motion commands")));
}

#[test]
fn test_task_91_file_validation_long_lines() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let long_line = "G0 ".to_string() + &"X10.123456789 ".repeat(20);
    let content = format!("{}\nG1 Z5\n", long_line);
    let file_path = create_test_gcode_file(&temp_dir, "longlines.nc", &content);

    let reader = GcodeFileReader::new(&file_path).expect("Failed to create reader");
    let validation = reader.validate().expect("Failed to validate");

    assert!(!validation.warnings.is_empty());
}

#[test]
fn test_task_91_read_lines_limited() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let lines = vec!["G0 X10", "G1 Y20", "G2 Z30", "M3 S1000", "M5"];
    let content = lines.join("\n");
    let file_path = create_test_gcode_file(&temp_dir, "limited.nc", &content);

    let reader = GcodeFileReader::new(&file_path).expect("Failed to create reader");
    let (collected, stats) = reader
        .read_lines_limited(3)
        .expect("Failed to read limited lines");

    assert_eq!(collected.len(), 3);
    assert!(stats.lines_read <= 3);
}

#[test]
fn test_task_92_recent_files_add() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = create_test_gcode_file(&temp_dir, "recent1.nc", "G0 X10");

    let mut manager = RecentFilesManager::new(10);
    manager.add(&file_path).expect("Failed to add recent file");

    assert_eq!(manager.count(), 1);
    let list = manager.list();
    assert_eq!(list[0].name, "recent1.nc");
}

#[test]
fn test_task_92_recent_files_order() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file1 = create_test_gcode_file(&temp_dir, "file1.nc", "G0 X10");
    let file2 = create_test_gcode_file(&temp_dir, "file2.nc", "G1 Y20");
    let file3 = create_test_gcode_file(&temp_dir, "file3.nc", "M3 S1000");

    let mut manager = RecentFilesManager::new(10);
    manager.add(&file1).expect("Failed to add file1");
    manager.add(&file2).expect("Failed to add file2");
    manager.add(&file3).expect("Failed to add file3");

    assert_eq!(manager.count(), 3);
    let list = manager.list();
    // Most recent should be first
    assert_eq!(list[0].name, "file3.nc");
    assert_eq!(list[1].name, "file2.nc");
    assert_eq!(list[2].name, "file1.nc");
}

#[test]
fn test_task_92_recent_files_duplicate() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = create_test_gcode_file(&temp_dir, "dup.nc", "G0 X10");

    let mut manager = RecentFilesManager::new(10);
    manager.add(&file_path).expect("Failed to add first time");
    manager.add(&file_path).expect("Failed to add second time");

    // Should only have one entry (moved to front)
    assert_eq!(manager.count(), 1);
}

#[test]
fn test_task_92_recent_files_max_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut manager = RecentFilesManager::new(3);

    for i in 1..=5 {
        let file_path = create_test_gcode_file(&temp_dir, &format!("file{}.nc", i), "G0 X10");
        manager
            .add(&file_path)
            .expect(&format!("Failed to add file{}", i));
    }

    // Should only keep 3 most recent
    assert_eq!(manager.count(), 3);
    let list = manager.list();
    assert_eq!(list[0].name, "file5.nc");
    assert_eq!(list[1].name, "file4.nc");
    assert_eq!(list[2].name, "file3.nc");
}

#[test]
fn test_task_92_recent_files_remove() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file1 = create_test_gcode_file(&temp_dir, "remove1.nc", "G0 X10");
    let file2 = create_test_gcode_file(&temp_dir, "remove2.nc", "G1 Y20");

    let mut manager = RecentFilesManager::new(10);
    manager.add(&file1).expect("Failed to add file1");
    manager.add(&file2).expect("Failed to add file2");

    assert_eq!(manager.count(), 2);
    manager.remove(&file1).expect("Failed to remove file");
    assert_eq!(manager.count(), 1);
}

#[test]
fn test_task_92_recent_files_clear() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file1 = create_test_gcode_file(&temp_dir, "clear1.nc", "G0 X10");
    let file2 = create_test_gcode_file(&temp_dir, "clear2.nc", "G1 Y20");

    let mut manager = RecentFilesManager::new(10);
    manager.add(&file1).expect("Failed to add file1");
    manager.add(&file2).expect("Failed to add file2");

    assert_eq!(manager.count(), 2);
    manager.clear().expect("Failed to clear");
    assert_eq!(manager.count(), 0);
}

#[test]
fn test_task_92_recent_files_find() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = create_test_gcode_file(&temp_dir, "find.nc", "G0 X10");

    let mut manager = RecentFilesManager::new(10);
    manager.add(&file_path).expect("Failed to add recent file");

    let found = manager.find_by_path(&file_path);
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "find.nc");
}

#[test]
fn test_task_92_recent_files_touch() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file1 = create_test_gcode_file(&temp_dir, "touch1.nc", "G0 X10");
    let file2 = create_test_gcode_file(&temp_dir, "touch2.nc", "G1 Y20");

    let mut manager = RecentFilesManager::new(10);
    manager.add(&file1).expect("Failed to add file1");
    manager.add(&file2).expect("Failed to add file2");

    let list_before = manager.list();
    assert_eq!(list_before[0].name, "touch2.nc");

    // Touch file1 to bring it to front
    manager.touch(&file1).expect("Failed to touch file");

    let list_after = manager.list();
    assert_eq!(list_after[0].name, "touch1.nc");
}

#[test]
fn test_task_92_recent_files_persistence() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let persist_dir = TempDir::new().expect("Failed to create persist dir");
    let persist_path = persist_dir.path().join("recent.json");

    let file_path = create_test_gcode_file(&temp_dir, "persist.nc", "G0 X10");

    // Create and save
    {
        let mut manager = RecentFilesManager::new(10);
        manager.set_persist_path(&persist_path);
        manager.add(&file_path).expect("Failed to add recent file");
    }

    assert!(persist_path.exists(), "Persist file should exist");

    // Load and verify
    {
        let mut manager = RecentFilesManager::new(10);
        manager.set_persist_path(&persist_path);
        manager.load().expect("Failed to load recent files");

        assert_eq!(manager.count(), 1);
        let list = manager.list();
        assert_eq!(list[0].name, "persist.nc");
    }
}

#[test]
fn test_task_91_file_size_formatting() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = create_test_gcode_file(&temp_dir, "size.nc", "G0 X10");

    let mut manager = RecentFilesManager::new(10);
    manager.add(&file_path).expect("Failed to add file");

    let list = manager.list();
    let formatted = list[0].formatted_size();
    assert!(formatted.contains("B") || formatted.contains("KB"));
}

#[test]
fn test_task_91_file_read_stats() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10 Y20\nG1 Z5 F100\nM3 S1000\n";
    let file_path = create_test_gcode_file(&temp_dir, "stats.nc", content);

    let reader = GcodeFileReader::new(&file_path).expect("Failed to create reader");
    let stats = reader.read_lines(|_| Ok(())).expect("Failed to read lines");

    assert!(stats.bytes_read > 0);
    assert_eq!(stats.lines_read, 3);
    assert!(stats.progress_percent() > 0.0);
}

#[test]
fn test_task_92_recent_files_get_by_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file1 = create_test_gcode_file(&temp_dir, "idx1.nc", "G0 X10");
    let file2 = create_test_gcode_file(&temp_dir, "idx2.nc", "G1 Y20");

    let mut manager = RecentFilesManager::new(10);
    manager.add(&file1).expect("Failed to add file1");
    manager.add(&file2).expect("Failed to add file2");

    let entry = manager.get(0);
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().name, "idx2.nc");

    let entry2 = manager.get(1);
    assert!(entry2.is_some());
    assert_eq!(entry2.unwrap().name, "idx1.nc");

    let out_of_bounds = manager.get(10);
    assert!(out_of_bounds.is_none());
}

#[test]
fn test_task_91_validate_complete_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "
(Test G-code file)
G90 G21 F100
G0 X0 Y0 Z0
G1 X10 Y10 Z-5 F50
G2 X20 Y20 I5 J5
G0 Z10
M3 S1000
G1 Z0 F30
G2 X0 Y0 I-10 J-10
M5
G0 Z10
";
    let file_path = create_test_gcode_file(&temp_dir, "complete.nc", content);

    let reader = GcodeFileReader::new(&file_path).expect("Failed to create reader");

    // Validate
    let validation = reader.validate().expect("Failed to validate");
    assert!(validation.is_valid);

    // Read all
    let text = reader.read_all().expect("Failed to read all");
    assert!(text.contains("G90"));
    assert!(text.contains("M5"));

    // Stream with callback
    let mut line_count = 0;
    reader
        .read_lines(|_line| {
            line_count += 1;
            Ok(())
        })
        .expect("Failed to stream read");
    assert!(line_count > 0);
}
