use gcodekit4::ui::gcode_editor::GcodeEditor;
use tempfile::NamedTempFile;

#[test]
fn test_save_as_file() {
    let editor = GcodeEditor::new();
    editor.load_content("G00 X10").unwrap();

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let result = editor.save_as(path);
    assert!(result.is_ok());
    assert_eq!(
        editor.get_file_path(),
        Some(path.to_string_lossy().to_string())
    );

    let content = std::fs::read_to_string(path).unwrap();
    assert_eq!(content, "G00 X10");
}

#[test]
fn test_file_path_tracking() {
    let editor = GcodeEditor::new();
    editor.load_content("G00 X10").unwrap();

    let path = Some("/path/to/file.gcode".to_string());
    editor.set_file_path(path.clone());

    assert_eq!(editor.get_file_path(), path);
}

#[test]
fn test_export_content() {
    let editor = GcodeEditor::new();
    editor.load_content("G00 X10\nG01 Y20").unwrap();

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let result = editor.export_to(path);
    assert!(result.is_ok());

    let content = std::fs::read_to_string(path).unwrap();
    assert_eq!(content, "G00 X10\nG01 Y20");
}

#[test]
fn test_load_file() {
    let editor = GcodeEditor::new();

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    std::fs::write(path, "G00 X10\nG01 Y20").unwrap();

    let result = editor.load_file(path);
    assert!(result.is_ok());

    let content = editor.get_plain_content();
    assert_eq!(content, "G00 X10\nG01 Y20");
    assert_eq!(
        editor.get_file_path(),
        Some(path.to_string_lossy().to_string())
    );
}
