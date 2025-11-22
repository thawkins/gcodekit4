//! Tests for firmware::file_service

use gcodekit4_communication::firmware::file_service::*;

#[test]
fn test_storage_info_usage() {
    let info = StorageInfo {
        total_size: 1000,
        used_size: 500,
        available_size: 500,
    };
    assert_eq!(info.usage_percent(), 50.0);
}

#[test]
fn test_file_info() {
    let file = FileInfo {
        name: "test.gcode".to_string(),
        size: 1024,
        is_directory: false,
        modified: Some(1234567890),
    };
    assert_eq!(file.name, "test.gcode");
    assert_eq!(file.size, 1024);
    assert!(!file.is_directory);
}
