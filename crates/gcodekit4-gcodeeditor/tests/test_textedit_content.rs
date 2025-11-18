use gcodekit4::ui::GcodeEditor;
use tempfile::NamedTempFile;

#[test]
fn test_display_content_has_line_numbers() {
    let editor = GcodeEditor::new();

    // Create test content
    let test_content = "; Test Program\nG00 X10 Y10\nG01 Z-5 F100\nG00 Z5\nM30";
    editor.load_content(test_content).unwrap();

    // Get display content (what will be shown in TextEdit)
    let display = editor.get_display_content();

    println!("Display content length: {}", display.len());
    println!("Display content:\n{}", display);

    // Verify content has line numbers
    assert!(display.contains("1 |"), "Should have line 1");
    assert!(display.contains("2 |"), "Should have line 2");
    assert!(display.contains("3 |"), "Should have line 3");
    assert!(display.contains("4 |"), "Should have line 4");
    assert!(display.contains("5 |"), "Should have line 5");

    // Verify original content is present
    assert!(display.contains("; Test Program"), "Should have comment");
    assert!(display.contains("G00 X10 Y10"), "Should have first command");
    assert!(display.contains("M30"), "Should have end command");

    // Verify it's not empty
    assert!(display.len() > 50, "Should have substantial content");
}

#[test]
fn test_display_content_from_file() {
    let editor = GcodeEditor::new();

    // Create temporary file
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path();

    let content = "; File Test\nG00 X10\nG01 Y20\nM30";
    std::fs::write(path, content).unwrap();

    // Load from file
    editor.load_file(path).unwrap();

    // Get display content
    let display = editor.get_display_content();

    println!("File display content length: {}", display.len());
    println!("File display content:\n{}", display);

    // Verify it has content
    assert!(display.len() > 0, "Display content should not be empty");
    assert!(display.contains("G00 X10"), "Should have file content");
    assert!(display.contains("1 |"), "Should have line numbers");
}

#[test]
fn test_plain_vs_display_content() {
    let editor = GcodeEditor::new();

    let test_content = "G00 X10\nG01 Y20";
    editor.load_content(test_content).unwrap();

    // Get both versions
    let plain = editor.get_plain_content();
    let display = editor.get_display_content();

    println!("Plain content: '{}'", plain);
    println!("Display content:\n{}", display);

    // Plain should match input
    assert_eq!(plain, test_content);

    // Display should have line numbers but include the content
    assert!(display.contains("G00 X10"));
    assert!(display.contains("G01 Y20"));
    assert!(display.contains("1 |"));
    assert!(display.contains("2 |"));

    // Display should be longer due to line numbers
    assert!(display.len() > plain.len());
}
