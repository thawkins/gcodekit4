//! Integration tests for Tasks 95 & 96 - File Export and Drag-and-Drop
//!
//! Task 95: File Export
//! - Export processed G-code
//! - Save modified files
//! - Add file format options
//!
//! Task 96: Drag and Drop Support
//! - Implement file drag and drop
//! - Support multiple file types
//! - Show drop indicators
//! - Handle drop events

use gcodekit4::utils::{
    DropEvent, DropFileType, DropIndicatorState, DropTarget, DropZone, ExportOptions, FileExporter,
    FileFormat,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_task_95_basic_export() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dest = temp_dir.path().join("exported.nc");
    let content = "G0 X10 Y20\nG1 Z5 F100\nM3 S1000\n";

    let result = FileExporter::export_simple(content, &dest);
    assert!(result.is_ok());
    assert!(dest.exists());

    let exported = fs::read_to_string(&dest).expect("Failed to read exported file");
    assert!(exported.contains("G0 X10"));
}

#[test]
fn test_task_95_export_with_options() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dest = temp_dir.path().join("custom.nc");
    let content = "G0 X10 Y20\n; Comment\nG1 Z5 F100\n\nM3 S1000\n";

    let mut options = ExportOptions::default();
    options.include_comments = false;
    options.include_empty_lines = false;

    let result = FileExporter::export(content, &dest, &options);
    assert!(result.is_ok());

    let exported = fs::read_to_string(&dest).expect("Failed to read exported file");
    assert!(!exported.contains("; Comment"));
}

#[test]
fn test_task_95_export_formats() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10\n";

    // Test different formats
    for format in &[
        FileFormat::GCode,
        FileFormat::GenericGCode,
        FileFormat::NGC,
        FileFormat::GCO,
    ] {
        let mut options = ExportOptions::default();
        options.format = *format;
        let dest = temp_dir.path().join(format!("test.{}", format.extension()));

        let result = FileExporter::export(content, &dest, &options);
        assert!(result.is_ok());
        assert!(dest.exists());
    }
}

#[test]
fn test_task_95_export_with_header() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dest = temp_dir.path().join("with_header.nc");
    let content = "G0 X10\n";

    let mut options = ExportOptions::default();
    options.add_header = true;
    options.add_timestamp = false;

    let result = FileExporter::export(content, &dest, &options);
    assert!(result.is_ok());

    let exported = fs::read_to_string(&dest).expect("Failed to read exported file");
    assert!(exported.contains("GCodeKit4"));
}

#[test]
fn test_task_95_export_unix_line_endings() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dest = temp_dir.path().join("unix.nc");
    let content = "G0 X10\nG1 Y20\n";

    let mut options = ExportOptions::default();
    options.unix_line_endings = true;

    let result = FileExporter::export(content, &dest, &options);
    assert!(result.is_ok());

    let exported = fs::read_to_string(&dest).expect("Failed to read exported file");
    assert!(!exported.contains("\r\n"));
}

#[test]
fn test_task_95_export_windows_line_endings() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dest = temp_dir.path().join("windows.nc");
    let content = "G0 X10\nG1 Y20\n";

    let mut options = ExportOptions::default();
    options.unix_line_endings = false;
    options.add_header = false;

    let result = FileExporter::export(content, &dest, &options);
    assert!(result.is_ok());

    let exported = fs::read_to_string(&dest).expect("Failed to read exported file");
    assert!(exported.contains("\r\n"));
}

#[test]
fn test_task_95_export_creates_directories() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dest = temp_dir
        .path()
        .join("subdir1")
        .join("subdir2")
        .join("file.nc");
    let content = "G0 X10\n";

    let result = FileExporter::export_simple(content, &dest);
    assert!(result.is_ok());
    assert!(dest.exists());
}

#[test]
fn test_task_96_drop_event_creation() {
    let files = vec![PathBuf::from("test.nc")];
    let event = DropEvent::new(files, DropTarget::Editor);

    assert_eq!(event.files.len(), 1);
    assert_eq!(event.target, DropTarget::Editor);
    assert_eq!(event.x, 0.0);
    assert_eq!(event.y, 0.0);
}

#[test]
fn test_task_96_drop_event_with_position() {
    let files = vec![PathBuf::from("test.nc")];
    let event = DropEvent::new(files, DropTarget::Canvas).with_position(100.5, 200.5);

    assert_eq!(event.x, 100.5);
    assert_eq!(event.y, 200.5);
}

#[test]
fn test_task_96_drop_event_file_filtering() {
    let files = vec![
        PathBuf::from("test1.nc"),
        PathBuf::from("test2.txt"),
        PathBuf::from("test3.gcode"),
        PathBuf::from("image.png"),
    ];
    let event = DropEvent::new(files, DropTarget::FileBrowser);

    let gcode_files = event.gcode_files();
    assert_eq!(gcode_files.len(), 2);
    assert_eq!(gcode_files[0].to_string_lossy(), "test1.nc");
    assert_eq!(gcode_files[1].to_string_lossy(), "test3.gcode");
}

#[test]
fn test_task_96_drop_file_type_detection() {
    let types = vec![
        (DropFileType::GCode, "test.nc", true),
        (DropFileType::GCode, "test.gcode", true),
        (DropFileType::GCode, "test.txt", false),
        (DropFileType::Image, "test.png", true),
        (DropFileType::Image, "test.jpg", true),
        (DropFileType::Image, "test.nc", false),
        (DropFileType::Text, "test.txt", true),
        (DropFileType::All, "anything.xyz", true),
    ];

    for (drop_type, filename, expected) in types {
        let path = std::path::Path::new(filename);
        assert_eq!(
            drop_type.matches(path),
            expected,
            "Failed for {} with type {:?}",
            filename,
            drop_type
        );
    }
}

#[test]
fn test_task_96_drop_zone_state_transitions() {
    let mut zone = DropZone::new("editor", DropFileType::GCode);

    // Initial state
    assert_eq!(zone.state, DropIndicatorState::None);

    // Drag valid files
    let valid_event = DropEvent::new(vec![PathBuf::from("test.nc")], DropTarget::Editor);
    zone.on_drag_over(&valid_event);
    assert_eq!(zone.state, DropIndicatorState::Valid);

    // Leave
    zone.on_drag_leave();
    assert_eq!(zone.state, DropIndicatorState::None);

    // Drag invalid files
    let invalid_event = DropEvent::new(vec![PathBuf::from("test.txt")], DropTarget::Editor);
    zone.on_drag_over(&invalid_event);
    assert_eq!(zone.state, DropIndicatorState::Invalid);

    // Leave
    zone.on_drag_leave();
    assert_eq!(zone.state, DropIndicatorState::None);
}

#[test]
fn test_task_96_drop_zone_disabled() {
    let mut zone = DropZone::new("editor", DropFileType::GCode);
    zone.enabled = false;

    let event = DropEvent::new(vec![PathBuf::from("test.nc")], DropTarget::Editor);
    zone.on_drag_over(&event);

    // Should not change state when disabled
    assert_eq!(zone.state, DropIndicatorState::None);
}

#[test]
fn test_task_96_drop_zone_visual_feedback() {
    let mut zone = DropZone::new("editor", DropFileType::GCode);

    zone.state = DropIndicatorState::None;
    assert_eq!(zone.indicator_class(), "");
    assert_eq!(zone.indicator_color(), "transparent");

    zone.state = DropIndicatorState::Valid;
    assert_eq!(zone.indicator_class(), "drop-valid");
    assert_eq!(zone.indicator_color(), "rgba(0, 255, 0, 0.2)");

    zone.state = DropIndicatorState::Invalid;
    assert_eq!(zone.indicator_class(), "drop-invalid");
    assert_eq!(zone.indicator_color(), "rgba(255, 0, 0, 0.2)");
}

#[test]
fn test_task_96_multiple_drop_zones() {
    let editor_zone = DropZone::new("editor", DropFileType::GCode);
    let browser_zone = DropZone::new("browser", DropFileType::All);
    let canvas_zone = DropZone::new("canvas", DropFileType::Image);

    assert_eq!(editor_zone.id, "editor");
    assert_eq!(browser_zone.id, "browser");
    assert_eq!(canvas_zone.id, "canvas");

    let gcode_event = DropEvent::new(vec![PathBuf::from("test.nc")], DropTarget::Editor);

    assert!(editor_zone
        .file_type
        .matches(gcode_event.first_file().unwrap()));
    assert!(browser_zone
        .file_type
        .matches(gcode_event.first_file().unwrap()));
    assert!(!canvas_zone
        .file_type
        .matches(gcode_event.first_file().unwrap()));
}

#[test]
fn test_task_95_96_combined_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create drop event with files
    let files = vec![temp_dir.path().join("input.nc")];
    fs::write(&files[0], "G0 X10\nG1 Y20\n").expect("Failed to write input file");

    let drop_event =
        DropEvent::new(files.clone(), DropTarget::FileBrowser).with_position(100.0, 200.0);

    // Verify drop event
    assert!(drop_event.is_valid_for_target(DropFileType::GCode));
    assert_eq!(drop_event.gcode_files().len(), 1);

    // Export the file
    let input_content = fs::read_to_string(&drop_event.files[0]).expect("Failed to read file");
    let export_path = temp_dir.path().join("exported.nc");

    let mut options = ExportOptions::default();
    options.add_header = true;

    let result = FileExporter::export(&input_content, &export_path, &options);
    assert!(result.is_ok());

    let exported = fs::read_to_string(&export_path).expect("Failed to read exported file");
    assert!(exported.contains("G0 X10"));
}

#[test]
fn test_task_95_export_filters_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dest = temp_dir.path().join("filtered.nc");
    let content = "G0 X10 Y20\n(Comment 1)\nG1 Z5\n; Comment 2\n\nM3 S1000\n";

    let mut options = ExportOptions::default();
    options.include_comments = false;
    options.include_empty_lines = false;
    options.add_header = false;

    FileExporter::export(content, &dest, &options).expect("Export failed");

    let exported = fs::read_to_string(&dest).expect("Read failed");
    assert!(!exported.contains("Comment"));
    assert!(!exported.contains(";\n"));

    // Should still have the G-code
    assert!(exported.contains("G0 X10"));
    assert!(exported.contains("G1 Z5"));
    assert!(exported.contains("M3"));
}

#[test]
fn test_task_96_drop_event_validation() {
    let files = vec![
        PathBuf::from("test1.nc"),
        PathBuf::from("test2.gcode"),
        PathBuf::from("readme.txt"),
    ];
    let event = DropEvent::new(files, DropTarget::Editor);

    // Check validation
    assert!(event.is_valid_for_target(DropFileType::GCode));
    assert!(event.is_valid_for_target(DropFileType::All));
    assert!(event.is_valid_for_target(DropFileType::Text));
    assert!(!event.is_valid_for_target(DropFileType::Image));
}

#[test]
fn test_task_96_drop_zone_multiple_files() {
    let files = vec![
        PathBuf::from("test1.nc"),
        PathBuf::from("test2.nc"),
        PathBuf::from("test3.nc"),
    ];
    let event = DropEvent::new(files, DropTarget::Editor);
    let mut zone = DropZone::new("editor", DropFileType::GCode);

    zone.on_drag_over(&event);
    assert_eq!(zone.state, DropIndicatorState::Valid);
}

#[test]
fn test_task_95_file_format_detection() {
    assert_eq!(FileFormat::from_extension("nc"), Some(FileFormat::GCode));
    assert_eq!(FileFormat::from_extension("NGC"), Some(FileFormat::NGC));
    assert_eq!(FileFormat::from_extension("unknown"), None);
}

#[test]
fn test_task_95_export_options_combinations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let content = "G0 X10\n(C)\nG1 Y20\n\n";

    let mut options = ExportOptions::default();

    // Test all combinations
    options.include_comments = true;
    options.include_empty_lines = true;
    options.add_header = false;
    let dest1 = temp_dir.path().join("full.nc");
    FileExporter::export(content, &dest1, &options).expect("Export 1 failed");

    options.include_comments = false;
    options.include_empty_lines = true;
    let dest2 = temp_dir.path().join("no_comments.nc");
    FileExporter::export(content, &dest2, &options).expect("Export 2 failed");

    options.include_comments = true;
    options.include_empty_lines = false;
    let dest3 = temp_dir.path().join("no_empty.nc");
    FileExporter::export(content, &dest3, &options).expect("Export 3 failed");

    options.include_comments = false;
    options.include_empty_lines = false;
    let dest4 = temp_dir.path().join("minimal.nc");
    FileExporter::export(content, &dest4, &options).expect("Export 4 failed");

    // All should exist
    assert!(dest1.exists());
    assert!(dest2.exists());
    assert!(dest3.exists());
    assert!(dest4.exists());
}
