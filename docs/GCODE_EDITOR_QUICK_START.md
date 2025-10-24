# G-Code Editor Quick Start Guide

## Basic Usage

### Creating an Editor Instance
```rust
use gcodekit4::ui::GcodeEditor;

let editor = GcodeEditor::new();
```

### Loading a File
```rust
// Option 1: Load from path directly
editor.load_file(Path::new("program.gcode"))?;

// Option 2: User selects file via dialog
let path = editor.open_and_load_file()?;
```

### Getting Content
```rust
// Plain text without formatting
let content = editor.get_plain_content();

// Formatted with line numbers and execution markers
let formatted = editor.get_display_content();
```

### Saving Work
```rust
// Save to current path
editor.save_file()?;

// Save as (new path)
editor.save_as(Path::new("new_name.gcode"))?;

// Save as with dialog
let new_path = editor.save_as_with_dialog()?;

// Export copy (doesn't change tracked path)
editor.export_to(Path::new("backup.gcode"))?;
```

## Editing Operations

### Insert Text
```rust
let success = editor.insert_text(
    line_number,  // 1-indexed
    position,     // character position
    "G01 Z5"
)?;
```

### Delete Character
```rust
let success = editor.delete_char(line_number, position)?;
```

### Find and Replace
```rust
// Find all occurrences (case-insensitive)
let matches = editor.search("G00");

// Replace all
let count = editor.replace_all("X10", "X20");

// Replace at specific location
let success = editor.replace_at(line_num, pos, "old", "new")?;
```

## Execution Tracking

### Track Program Execution
```rust
for line_num in 1..=editor.get_line_count() {
    // Set as current line (shows execution marker)
    editor.set_current_line(line_num);
    
    // Execute the line...
    
    // Mark as completed
    editor.mark_line_executed(line_num);
}
```

### Display Execution State
```rust
let content = editor.get_display_content();
// Shows: ▶ for current, ✓ for executed, blanks for pending
```

### Reset for New Run
```rust
editor.clear_execution_state();
```

## Read-Only Mode

### Disable Editing During Execution
```rust
editor.set_read_only(true);
// Now insert/delete operations fail silently

// Re-enable editing
editor.set_read_only(false);
```

## File Management

### Track Current File
```rust
if let Some(path) = editor.get_file_path() {
    println!("Editing: {}", path);
}

// Set path manually
editor.set_file_path(Some("path/to/file.gcode".to_string()));
```

## Error Handling

### Graceful Error Handling
```rust
match editor.load_file(path) {
    Ok(()) => println!("File loaded successfully"),
    Err(e) => eprintln!("Error: {}", e),
}

// Or use ? operator
editor.load_file(path)?;
```

## Common Patterns

### Full Workflow Example
```rust
use gcodekit4::ui::GcodeEditor;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let editor = GcodeEditor::new();
    
    // Load file
    editor.open_and_load_file()?;
    
    // Show content
    println!("{}", editor.get_display_content());
    
    // Enable editing
    editor.set_editable(true);
    
    // Modify
    editor.insert_text(1, 0, "; Header\n")?;
    
    // Save
    editor.save_file()?;
    
    Ok(())
}
```

### Search and Modify
```rust
// Find all rapid moves
let matches = editor.search("G00");
println!("Found {} rapid move commands", matches.len());

// Convert to linear moves
editor.replace_all("G00", "G01");

// Save changes
editor.save_file()?;
```

### Program Execution Simulation
```rust
editor.load_file(path)?;
editor.set_read_only(true);

let current = editor.get_current_line();
for i in current..=editor.get_line_count() {
    editor.set_current_line(i);
    
    // Get line content for execution
    if let Some(line_text) = editor.get_line_text(i) {
        println!("Executing: {}", line_text);
        // ... execute ...
    }
    
    editor.mark_line_executed(i);
}

editor.set_read_only(false);
```

## Thread-Safe Usage

The editor is safe to use across threads:

```rust
use std::sync::Arc;
use std::thread;

let editor = Arc::new(GcodeEditor::new());

// Clone for thread
let editor_clone = Arc::clone(&editor);
thread::spawn(move || {
    editor_clone.load_content("G00 X10").ok();
});

// Can still use in main thread
let content = editor.get_plain_content();
```

## API Quick Reference

| Method | Purpose |
|--------|---------|
| `open_file_dialog()` | Browse and select file |
| `load_file(path)` | Load from path |
| `open_and_load_file()` | Dialog + load |
| `save_file()` | Save to current path |
| `save_as_dialog()` | Browse save location |
| `save_as(path)` | Save to new path |
| `save_as_with_dialog()` | Dialog + save as |
| `export_to(path)` | Export without tracking |
| `get_file_path()` | Get current file path |
| `set_file_path()` | Set file path |
| `load_content(text)` | Load from string |
| `get_plain_content()` | Get text content |
| `get_display_content()` | Get formatted content |
| `insert_text()` | Insert text |
| `delete_char()` | Delete character |
| `search(query)` | Find text |
| `replace_all()` | Replace all |
| `replace_at()` | Replace specific |
| `set_current_line()` | Set execution line |
| `mark_line_executed()` | Mark line done |
| `clear_execution_state()` | Reset markers |
| `set_read_only()` | Lock/unlock editing |
| `get_line_count()` | Number of lines |
| `get_line_text()` | Get specific line |

## Tips

1. **Always check errors** - File operations can fail
2. **Use read-only mode** - During execution to prevent accidental changes
3. **Track paths** - Use `set_file_path()` after "Save As" operations
4. **Search is case-insensitive** - For G-Code compatibility
5. **Content is in-memory** - No automatic disk sync after edits
6. **Thread-safe design** - Safe to use with tokio tasks

## Troubleshooting

### File Not Found
```rust
// Check if path exists first
if std::path::Path::new("file.gcode").exists() {
    editor.load_file(path)?;
}
```

### No File Path Set
```rust
// Use save_as instead of save_file
editor.save_as(Path::new("output.gcode"))?;
```

### Read-Only Errors
```rust
// Check if in read-only mode
if editor.is_read_only() {
    editor.set_read_only(false);
}
```

### Permission Denied
```rust
// On Linux/macOS, check file permissions
// chmod +rw file.gcode

// Or save to different location
editor.save_as(Path::new("/tmp/output.gcode"))?;
```
