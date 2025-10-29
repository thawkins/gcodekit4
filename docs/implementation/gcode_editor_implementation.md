# G-Code Editor Implementation

## Overview

The G-Code Editor is a unified, full-featured editor for G-Code files that combines the functionality of the previous gcode_viewer and gcode_editor modules into a single comprehensive implementation. It provides:

- **Syntax Highlighting**: Color-coded tokens for G-Code commands
- **Line Number Support**: Automatic line numbering for easy reference
- **Execution Tracking**: Visual markers for currently executing and completed lines
- **Search & Replace**: Find and replace text with case-insensitive matching
- **Text Editing**: Insert/delete operations with read-only mode
- **File Management**: Load, display, and manage G-Code content
- **Real-time Validation**: Automatic token parsing and classification

This module unifies the previous gcode_viewer (search/replace, cursor management) 
and gcode_editor (syntax highlighting, execution tracking) into a single, 
comprehensive editor implementation.

## Architecture

### Core Components

#### 1. Token Types (`TokenType`)
Represents different types of G-Code tokens for syntax highlighting:

```rust
pub enum TokenType {
    GCommand,      // G00, G01, G02, G03, etc.
    MCommand,      // M03, M04, M05, M30, etc.
    Coordinate,    // X, Y, Z, A, B, C parameters
    Parameter,     // F (feedrate), S (spindle), T (tool), etc.
    Comment,       // Lines starting with ;
    Normal,        // Other text
}
```

#### 2. GcodeEditor (`GcodeEditor`)
High-level editor manager with thread-safe access:

```rust
pub struct GcodeEditor {
    file: Arc<Mutex<GcodeFile>>,
    editable: Arc<Mutex<bool>>,
}
```

**Core Methods:**
- `load_content(content: &str)` - Load G-Code content
- `get_display_content()` - Get formatted content with line numbers and markers
- `get_plain_content()` - Get plain text without formatting
- `mark_line_executed(line_number)` - Track execution progress
- `set_current_line(line_number)` - Update current executing line
- `clear_execution_state()` - Reset all markers
- `search(query: &str)` - Find text (case-insensitive)
- `replace_all(old, new)` - Replace all occurrences
- `replace_at(line, pos, old, new)` - Replace at specific location
- `insert_text(line, pos, text)` - Insert text at position
- `delete_char(line, pos)` - Delete character at position
- `get_line_text(line)` - Get specific line text
- `get_all_lines()` - Get all lines as strings
- `set_editable(bool)` - Control edit mode
- `set_read_only(bool)` - Set read-only mode
- `is_read_only()` - Check read-only status
- `get_line_count()` - Get total lines
- `get_current_line()` - Get current line number

## Features

### Syntax Highlighting

The `GcodeLine::tokenize()` method analyzes each token and classifies it:

1. **G-Commands**: Start with 'G' (case-insensitive)
2. **M-Commands**: Start with 'M'
3. **Coordinates**: X, Y, Z, A, B, C
4. **Parameters**: F, S, T, H, P
5. **Comments**: Lines starting with ';'
6. **Normal**: All other text

### Execution Tracking

- `▶` marks the currently executing line
- `✓` marks completed lines
- Two spaces mark pending lines

### Search & Replace

```rust
// Case-insensitive search
let results = editor.search("G00");

// Replace all
let count = editor.replace_all("Y10", "Y30");

// Replace specific
editor.replace_at(line_number, position, "old", "new");
```

### Text Editing

```rust
editor.set_editable(true);
editor.insert_text(line, pos, "text");
editor.delete_char(line, pos);
```

### Display Format

```
1 | ▶ ; Sample G-Code
2 |   G00 X10 Y10
3 | ✓ G01 Z-5 F100
4 |   G00 Z5
5 |   M30
```

## Testing

### Unit Tests (12 tests, all passing)

```bash
cargo test --lib gcode_editor
```

**Coverage:**
- Token classification
- File loading and parsing
- Execution state tracking
- Search functionality (case-insensitive)
- Replace operations
- Text insertion and deletion
- Read-only mode

### Sample Tests

```rust
#[test]
fn test_search() {
    let editor = GcodeEditor::new();
    editor.load_content("G00 X10\nG01 Y20\nG00 X30").unwrap();
    let results = editor.search("G00");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_replace_all() {
    let editor = GcodeEditor::new();
    editor.load_content("X10 Y10\nX20 Y10").unwrap();
    let count = editor.replace_all("Y10", "Y30");
    assert_eq!(count, 2);
}
```

## Performance

- **File Size**: Handles thousands of lines efficiently
- **Memory**: ~200 bytes per line (text + tokens)
- **Search**: O(n*m) where n = total characters, m = query length
- **Thread Safety**: Arc<Mutex<>> for concurrent access

## Integration Example

```rust
use gcodekit4::GcodeEditor;

// Initialize
let editor = GcodeEditor::new();

// Load content
editor.load_content("G00 X10\nG01 Y20\nM30")?;

// Track execution
editor.set_current_line(1);
editor.mark_line_executed(1);

// Search
let results = editor.search("G00");

// Replace
let count = editor.replace_all("Y10", "Y20");

// Get display
let content = editor.get_display_content();
```

## Future Enhancements

1. **Color Syntax Highlighting** - Full color support in Slint UI
2. **Line Background Highlighting** - Visual highlighting of current/executed lines
3. **G-Code Validation** - Validate against controller types
4. **Code Folding** - Collapse/expand sections
5. **Undo/Redo** - Full undo/redo stack
6. **Breakpoints** - Set and manage breakpoints
7. **Macro Support** - Macro expansion
8. **Performance Metrics** - Estimate runtime

## Files

- `src/ui/gcode_editor.rs` - Main implementation (merged gcode_viewer + gcode_editor)
- `docs/gcode_editor_implementation.md` - This documentation

## Changes from Previous Implementation

This version merges two separate modules:

**Previous Architecture:**
- `gcode_viewer.rs` - Search, replace, cursor management, editing
- `gcode_editor.rs` - Syntax highlighting, execution tracking, line display

**New Unified Architecture:**
- `gcode_editor.rs` - All functionality combined

**Benefits:**
- Single source of truth
- No code duplication
- Improved consistency
- Easier maintenance
- Better test coverage (12 tests)
