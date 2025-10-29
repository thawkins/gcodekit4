# G-Code Editor Features

## Overview

The G-Code editor in GCodeKit4 provides a full-featured editing and file management system for G-Code files. It's built in Rust using the Slint UI framework and integrates with the `rfd` crate for native file dialogs.

## Architecture

The editor is implemented in `src/ui/gcode_editor.rs` and consists of several key components:

### TokenType Enum
Defines syntax highlighting categories:
- **GCommand**: G-Code commands (G00, G01, etc.)
- **MCommand**: M-Code commands (M03, M04, etc.)
- **Coordinate**: Coordinate axes (X, Y, Z, A, B, C)
- **Parameter**: Parameters (F, S, T, H, P)
- **Comment**: Comments (prefixed with `;`)
- **Normal**: Regular text

### Token Struct
Represents a single token in the source:
- `token_type`: Classification of the token
- `text`: The token content
- `start`: Starting position in line
- `end`: Ending position in line

### GcodeLine Struct
Represents a single line in a G-Code file:
- `line_number`: 1-indexed line number
- `text`: Raw text content
- `tokens`: Parsed tokens for syntax highlighting
- `executed`: Whether the line has been executed
- `is_current`: Whether this is the currently executing line

### GcodeFile Struct
Internal file representation:
- `path`: Optional file path
- `lines`: Vector of GcodeLine objects
- `current_line`: Track current execution position

### GcodeEditor Struct
Main editor interface - thread-safe with Arc<Mutex<>>:
- `file`: Arc-protected GcodeFile
- `editable`: Arc-protected editability flag

## File Operations

### Opening Files

#### Open File Dialog
```rust
let path = editor.open_file_dialog()?;
```
Opens a native file dialog for selecting a G-Code file. Returns the selected path or an error if cancelled.

**Supported file types:**
- `.gcode` - Standard G-Code files
- `.gc` - Generic G-Code
- `.ngc` - RS274 NGC format
- `.tap` - Tap files
- `.txt` - Text files
- `*` - All files

#### Load from Path
```rust
editor.load_file(&path)?;
```
Loads G-Code content from a file path. Automatically sets the editor's file path.

#### Open and Load
```rust
let path = editor.open_and_load_file()?;
```
Combined operation: Opens file dialog and loads the selected file.

### Saving Files

#### Save File
```rust
editor.save_file()?;
```
Saves the current content to the tracked file path. Returns an error if no path is set (use `save_as` instead).

#### Save As Dialog
```rust
let path = editor.save_as_dialog()?;
```
Opens a native "Save As" dialog. Returns the selected path or an error if cancelled.

#### Save As
```rust
editor.save_as(&path)?;
```
Saves content to a new file path and updates the editor's tracked path.

#### Save As with Dialog
```rust
let path = editor.save_as_with_dialog()?;
```
Combined operation: Opens save dialog and saves to the selected location.

### Additional File Operations

#### Get File Path
```rust
let path = editor.get_file_path();
```
Returns the currently tracked file path (if any).

#### Set File Path
```rust
editor.set_file_path(Some("/path/to/file.gcode".to_string()));
```
Manually sets the tracked file path without saving.

#### Export To
```rust
editor.export_to(&path)?;
```
Exports current content to a file without updating the tracked path (useful for "Save Copy" functionality).

## Content Management

### Loading Content
```rust
editor.load_content("G00 X10\nG01 Y20")?;
```
Loads G-Code content from a string. Automatically tokenizes for syntax highlighting.

### Getting Content

#### Plain Content
```rust
let content = editor.get_plain_content();
```
Returns content as plain text without line numbers or execution markers.

#### Display Content
```rust
let formatted = editor.get_display_content();
```
Returns formatted content with line numbers and execution state markers:
- `▶` - Currently executing line
- `✓` - Previously executed line
- `  ` - Not yet executed

## Syntax Highlighting & Tokenization

### Token Classification
The editor automatically tokenizes lines for syntax highlighting:

```rust
// Example: "G00 X10 Y20 ; Move to position"
// Tokens:
// - G00 → GCommand
// - X10 → Coordinate  
// - Y20 → Coordinate
// - Move to position → Comment
```

### Line Tokenization
```rust
let line = GcodeLine::new(1, "G00 X10 Y20");
// Automatically tokenized on creation
```

## Editing Operations

### Text Insertion
```rust
editor.insert_text(line_number, position, "text")?;
```
Inserts text at a specific position. Respects read-only mode.

### Character Deletion
```rust
editor.delete_char(line_number, position)?;
```
Deletes a character at a specific position. Respects read-only mode.

### Text Replacement
```rust
// Replace at specific position
editor.replace_at(line_number, position, "old", "new")?;

// Replace all occurrences
let count = editor.replace_all("old", "new");
```

## Search Functionality

### Search for Text
```rust
let results = editor.search("G00");
// Returns Vec<(line_number, position)>
```
Case-insensitive search returns all occurrences in the file.

## Execution Tracking

### Mark Line as Executed
```rust
editor.mark_line_executed(line_number);
```

### Set Current Line
```rust
editor.set_current_line(line_number);
```
Sets the currently executing line. Automatically clears previous current line marker.

### Get Current Line
```rust
let current = editor.get_current_line();
```

### Clear Execution State
```rust
editor.clear_execution_state();
```
Resets all executed and current line markers.

## Read-Only Mode

### Set Editable State
```rust
editor.set_editable(true);   // Enable editing
editor.set_editable(false);  // Disable editing
```

### Set Read-Only Mode
```rust
editor.set_read_only(true);   // Enable read-only
editor.set_read_only(false);  // Disable read-only
```

### Check State
```rust
let is_editable = editor.is_editable();
let is_readonly = editor.is_read_only();
```

## Thread Safety

All editor operations are thread-safe:
- Uses `Arc<Mutex<>>` for interior mutability
- Multiple threads can safely read/write editor state
- Automatic lock management through Mutex

## Integration with Java UGS

The implementation follows patterns from the Java Universal G-Code Sender:

| Java Feature | Rust Equivalent |
|---|---|
| EditorUtils.openFile() | editor.open_and_load_file() |
| EditorUtils.unloadFile() | clear_execution_state() |
| GcodeFileListener | (handled by watcher component) |
| PopupEditor | (UI layer in Slint) |

## Error Handling

All file operations return `anyhow::Result<T>` for proper error propagation:

```rust
match editor.load_file(&path) {
    Ok(()) => println!("File loaded"),
    Err(e) => eprintln!("Failed to load: {}", e),
}
```

Common errors:
- File not found
- Permission denied
- Invalid file format
- Disk full (on write operations)
- Dialog cancelled by user

## Performance Considerations

1. **Tokenization**: Performed on content load and during edits
2. **String Operations**: Efficient Rust string handling
3. **Memory**: Content stored in memory (suitable for typical G-Code files)
4. **Thread Safety**: Mutex overhead minimal for typical use cases

## Examples

### Complete Workflow

```rust
let editor = GcodeEditor::new();

// Open and load file
let path = editor.open_and_load_file()?;

// Set to read-only during execution
editor.set_read_only(true);

// Track execution
for i in 1..=editor.get_line_count() {
    editor.set_current_line(i);
    // Execute line...
    editor.mark_line_executed(i);
}

// Clear execution state for next run
editor.clear_execution_state();
editor.set_read_only(false);

// Save modifications
editor.save_file()?;
```

### Search and Replace

```rust
let editor = GcodeEditor::new();
editor.load_content("G00 X10\nG01 X20\nG00 X30")?;

// Find all G00 commands
let matches = editor.search("G00");
println!("Found {} occurrences", matches.len());

// Replace all
let replaced = editor.replace_all("X", "Y");
println!("Replaced {} lines", replaced);
```

### Export Workflow

```rust
// Load from one format, export to another
editor.load_file(Path::new("program.gcode"))?;

// Make modifications...

// Export as backup
editor.export_to(Path::new("program.backup"))?;

// Save as new version
editor.save_as(Path::new("program_v2.gcode"))?;
```
