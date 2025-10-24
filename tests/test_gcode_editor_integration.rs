use gcodekit4::ui::GcodeEditor;
use tempfile::NamedTempFile;

#[test]
fn test_open_and_load_file_integration() {
    let editor = GcodeEditor::new();
    
    // Create a temporary G-Code file
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();
    
    let content = "; Test Program\nG00 X10 Y10\nG01 Z-5 F100\nG00 Z5\nM30";
    std::fs::write(path, content).unwrap();
    
    // Load the file
    let result = editor.load_file(path);
    assert!(result.is_ok());
    
    // Verify content was loaded
    let loaded_content = editor.get_plain_content();
    assert_eq!(loaded_content, content);
    
    // Verify path was tracked
    assert_eq!(editor.get_file_path(), Some(path.to_string_lossy().to_string()));
    
    // Verify line count
    assert_eq!(editor.get_line_count(), 5);
}

#[test]
fn test_save_and_reload() {
    let editor = GcodeEditor::new();
    let original_content = "G00 X10 Y10\nG01 Z-5 F100\nG00 Z5\nM30";
    
    // Load content
    editor.load_content(original_content).unwrap();
    
    // Save to temporary file
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();
    
    editor.save_as(path).unwrap();
    
    // Create new editor and load the saved file
    let editor2 = GcodeEditor::new();
    editor2.load_file(path).unwrap();
    
    // Verify content matches
    assert_eq!(editor2.get_plain_content(), original_content);
}

#[test]
fn test_file_operations_workflow() {
    let editor = GcodeEditor::new();
    
    // Initial content
    let initial = "; Initial\nG00 X10";
    editor.load_content(initial).unwrap();
    
    // Save to file
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();
    editor.save_as(path).unwrap();
    
    // Verify file path is tracked
    assert_eq!(editor.get_file_path(), Some(path.to_string_lossy().to_string()));
    
    // Quick save should work now
    let save_result = editor.save_file();
    assert!(save_result.is_ok());
    
    // Reload and verify
    let content = std::fs::read_to_string(path).unwrap();
    assert_eq!(content, initial);
}

#[test]
fn test_display_content_vs_plain() {
    let editor = GcodeEditor::new();
    editor.load_content("G00 X10\nG01 Y20").unwrap();
    
    // Get plain content (no line numbers)
    let plain = editor.get_plain_content();
    assert_eq!(plain, "G00 X10\nG01 Y20");
    
    // Get display content (with line numbers)
    let display = editor.get_display_content();
    assert!(display.contains("1 |"));  // Has line numbers
    assert!(display.contains("2 |"));
    assert!(display.contains("G00 X10"));
}
