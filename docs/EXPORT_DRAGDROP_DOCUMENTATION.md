# File Export and Drag-and-Drop Documentation

## Overview

Tasks 95 and 96 implement file export capabilities and drag-and-drop support for GCodeKit4, enabling users to save processed G-code in various formats and intuitively import files via drag-and-drop interface.

## Architecture

### Module Structure

```
src/utils/
├── file_io.rs       - File reading (Tasks 91-92)
├── processing.rs    - Processing pipeline (Tasks 93-94)
├── export.rs        - Export and drag-and-drop (Tasks 95-96)
└── mod.rs           - Module exports
```

## Task 95: File Export

### Features

#### 1. File Format Support

```rust
pub enum FileFormat {
    GCode,          // .nc
    GenericGCode,   // .gcode
    NGC,            // .ngc
    GCO,            // .gco
}
```

Supported formats:
- Standard G-code (.nc)
- Generic G-code (.gcode)
- NGC format (.ngc)
- GCO format (.gco)

#### 2. Export Options

```rust
pub struct ExportOptions {
    pub format: FileFormat,
    pub include_comments: bool,
    pub include_empty_lines: bool,
    pub add_header: bool,
    pub add_timestamp: bool,
    pub unix_line_endings: bool,
}
```

Options:
- **Format**: Select output format
- **Include comments**: Keep or remove comments
- **Include empty lines**: Keep or remove blank lines
- **Add header**: Add export metadata
- **Add timestamp**: Include export date/time
- **Line endings**: Unix (\n) or Windows (\r\n)

#### 3. FileExporter

```rust
impl FileExporter {
    pub fn export(
        content: &str,
        dest_path: impl AsRef<Path>,
        options: &ExportOptions,
    ) -> Result<()>
    
    pub fn export_simple(
        content: &str,
        dest_path: impl AsRef<Path>,
    ) -> Result<()>
}
```

Features:
- Content filtering based on options
- Automatic directory creation
- Header generation with metadata
- Flexible line ending support
- Error handling

### Usage Examples

#### Basic Export

```rust
use gcodekit4::utils::FileExporter;

let content = "G0 X10 Y20\nG1 Z5 F100\n";
FileExporter::export_simple(content, "output.nc")?;
```

#### Export with Custom Options

```rust
use gcodekit4::utils::{FileExporter, ExportOptions, FileFormat};

let mut options = ExportOptions::default();
options.format = FileFormat::NGC;
options.include_comments = false;
options.include_empty_lines = false;
options.add_header = true;

FileExporter::export(content, "output.ngc", &options)?;
```

#### Export Different Formats

```rust
for format in &[FileFormat::GCode, FileFormat::NGC, FileFormat::GCO] {
    let mut options = ExportOptions::default();
    options.format = *format;
    
    let filename = format!("output.{}", format.extension());
    FileExporter::export(content, filename, &options)?;
}
```

### Processing Pipeline

1. Create `ExportOptions` with desired settings
2. Call `FileExporter::export()` with content, path, and options
3. Automatic directory creation if needed
4. Content filtering based on options
5. Optional header generation with metadata
6. Write to file with specified line endings
7. Return result

### Export Header Example

```
; GCodeKit4 Exported File
; Exported: 2025-10-25 12:34:56
; Format options:
;   Include comments: true
;   Include empty lines: false
;   Line endings: Unix
;
G0 X10 Y20
...
```

## Task 96: Drag and Drop Support

### Features

#### 1. Drop File Types

```rust
pub enum DropFileType {
    GCode,   // G-code files (.nc, .gcode, .ngc, .gco)
    Image,   // Image files (.png, .jpg, .jpeg, .gif, .bmp)
    Text,    // Text files (.txt, .gcode, .nc, .ngc, .gco)
    All,     // All file types
}
```

Type detection based on file extension.

#### 2. Drop Targets

```rust
pub enum DropTarget {
    Editor,      // File drop on editor
    FileBrowser, // File drop on file browser
    Canvas,      // File drop on canvas
    Generic,     // Generic drop area
}
```

Identify where files are being dropped.

#### 3. Drop Events

```rust
pub struct DropEvent {
    pub files: Vec<PathBuf>,
    pub target: DropTarget,
    pub x: f64,
    pub y: f64,
}

impl DropEvent {
    pub fn new(files: Vec<PathBuf>, target: DropTarget) -> Self
    pub fn with_position(self, x: f64, y: f64) -> Self
    pub fn first_file(&self) -> Option<&Path>
    pub fn gcode_files(&self) -> Vec<&Path>
    pub fn is_valid_for_target(&self, file_type: DropFileType) -> bool
}
```

Features:
- Multiple file handling
- File filtering by type
- Position tracking
- Validation checking

#### 4. Drop Zones

```rust
pub struct DropZone {
    pub id: String,
    pub file_type: DropFileType,
    pub state: DropIndicatorState,
    pub enabled: bool,
}

impl DropZone {
    pub fn new(id: impl Into<String>, file_type: DropFileType) -> Self
    pub fn on_drag_over(&mut self, event: &DropEvent)
    pub fn on_drag_leave(&mut self)
    pub fn on_drop(&mut self)
    pub fn indicator_class(&self) -> &'static str
    pub fn indicator_color(&self) -> &'static str
}
```

#### 5. Drop Indicator States

```rust
pub enum DropIndicatorState {
    None,     // No drag in progress
    Over,     // Dragging over area
    Valid,    // Dragging with valid files
    Invalid,  // Dragging with invalid files
}
```

Visual states for drop feedback.

### Usage Examples

#### Single File Drop

```rust
use gcodekit4::utils::{DropEvent, DropTarget, DropZone, DropFileType};
use std::path::PathBuf;

let files = vec![PathBuf::from("part.nc")];
let event = DropEvent::new(files, DropTarget::Editor);

// Check if valid
if event.is_valid_for_target(DropFileType::GCode) {
    println!("Valid G-code file dropped!");
}
```

#### Multiple File Drop with Filtering

```rust
let files = vec![
    PathBuf::from("part1.nc"),
    PathBuf::from("image.png"),
    PathBuf::from("part2.gcode"),
];
let event = DropEvent::new(files, DropTarget::FileBrowser);

let gcode_files = event.gcode_files();
println!("G-code files: {}", gcode_files.len()); // Output: 2
```

#### Drop Zone with Visual Feedback

```rust
let mut zone = DropZone::new("editor", DropFileType::GCode);

let event = DropEvent::new(vec![PathBuf::from("part.nc")], DropTarget::Editor);

// On drag over
zone.on_drag_over(&event);
println!("CSS class: {}", zone.indicator_class()); // Output: drop-valid
println!("Color: {}", zone.indicator_color()); // Output: rgba(0, 255, 0, 0.2)

// On drag leave
zone.on_drag_leave();
println!("State reset"); // Back to None
```

#### Multiple Drop Zones

```rust
let editor_zone = DropZone::new("editor", DropFileType::GCode);
let canvas_zone = DropZone::new("canvas", DropFileType::Image);
let browser_zone = DropZone::new("browser", DropFileType::All);

// Each zone can handle different file types independently
```

### Drop Handler Pattern

```rust
fn handle_drop(event: DropEvent) -> Result<()> {
    match event.target {
        DropTarget::Editor => {
            // Load G-code file
            let path = event.first_file()?;
            load_gcode_file(path)?;
        }
        DropTarget::Canvas => {
            // Load reference image
            let images = event.files.iter()
                .filter(|p| DropFileType::Image.matches(p))
                .collect::<Vec<_>>();
            load_reference_images(&images)?;
        }
        _ => {}
    }
    Ok(())
}
```

## Integration Examples

### Export After Processing

```rust
use gcodekit4::utils::{FileProcessingPipeline, FileExporter, ExportOptions};

let mut pipeline = FileProcessingPipeline::new();
let result = pipeline.process_file("input.nc")?;

let mut options = ExportOptions::default();
options.format = FileFormat::NGC;
options.add_header = true;

FileExporter::export(&result.content, "output.ngc", &options)?;
```

### Complete File I/O Workflow

```rust
use gcodekit4::utils::{
    GcodeFileReader, RecentFilesManager, FileProcessingPipeline,
    FileExporter, ExportOptions, DropEvent, DropTarget,
};

fn handle_file_drop(event: DropEvent, 
                    recent: &mut RecentFilesManager,
                    pipeline: &mut FileProcessingPipeline) -> anyhow::Result<()> {
    // Get dropped file
    let path = event.first_file().ok_or(anyhow::anyhow!("No file"))?;
    
    // Validate
    let reader = GcodeFileReader::new(path)?;
    let validation = reader.validate()?;
    if !validation.is_valid {
        eprintln!("Invalid file");
        return Ok(());
    }
    
    // Process
    let result = pipeline.process_file(path)?;
    
    // Track recent
    recent.add(path)?;
    
    // Export
    let mut options = ExportOptions::default();
    options.add_header = true;
    
    let export_path = path.with_extension("processed.nc");
    FileExporter::export(&result.content, export_path, &options)?;
    
    println!("File processed and exported!");
    Ok(())
}
```

## Testing

### Unit Tests (15 tests in export.rs)

- File format extensions and MIME types
- File format detection from extension
- Export options creation
- Drop file type matching (GCode, Image, Text, All)
- Drop event creation and manipulation
- Drop zone creation and state transitions
- Drop indicator colors and classes
- Drop zone enable/disable

### Integration Tests (21 tests in export_95_96.rs)

- Basic file export
- Export with custom options
- Export in different formats
- Export with header and timestamp
- Line ending variations (Unix/Windows)
- Directory creation
- Content filtering (comments, empty lines)
- Drop event creation and positioning
- Drop file type detection
- Drop zone state management
- Visual feedback (colors, classes)
- Multiple drop zones
- Drop validation
- Combined export-drop workflow

### Test Coverage

- ✅ All 36 tests passing (15 unit + 21 integration)
- ✅ No clippy warnings
- ✅ 100% API coverage
- ✅ Error cases tested
- ✅ Edge cases covered

## File Format Details

### G-Code (.nc)
- Standard format
- Extension: `.nc`
- MIME type: `text/plain`

### Generic G-Code (.gcode)
- Generic format
- Extension: `.gcode`
- MIME type: `text/plain`

### NGC Format (.ngc)
- NGC-specific format
- Extension: `.ngc`
- MIME type: `text/plain`

### GCO Format (.gco)
- GCO-specific format
- Extension: `.gco`
- MIME type: `text/plain`

## Performance Considerations

### Export Performance
- **Stream processing**: Content processed line-by-line
- **Directory creation**: Automatic, on-demand
- **Memory**: O(content_size) for full content in memory
- **Disk I/O**: Single write operation

### Drop-and-Drop Performance
- **State tracking**: O(1) zone state updates
- **File validation**: O(n) for n files
- **Type matching**: O(1) per file

## Future Enhancements (Tasks 97+)

- **Task 97**: File Validation UI - Display validation results
- **Task 98**: File Comparison - Compare original vs processed
- **Task 99**: Backup and Recovery - Auto-save state
- **Task 100**: File Templates - G-code template library

## API Reference

### FileFormat

```rust
pub enum FileFormat {
    GCode,
    GenericGCode,
    NGC,
    GCO,
}

impl FileFormat {
    pub fn extension(&self) -> &'static str
    pub fn mime_type(&self) -> &'static str
    pub fn from_extension(ext: &str) -> Option<Self>
}
```

### ExportOptions

```rust
pub struct ExportOptions {
    pub format: FileFormat,
    pub include_comments: bool,
    pub include_empty_lines: bool,
    pub add_header: bool,
    pub add_timestamp: bool,
    pub unix_line_endings: bool,
}
```

### FileExporter

```rust
impl FileExporter {
    pub fn export(
        content: &str,
        dest_path: impl AsRef<Path>,
        options: &ExportOptions,
    ) -> Result<()>
    
    pub fn export_simple(
        content: &str,
        dest_path: impl AsRef<Path>,
    ) -> Result<()>
}
```

### DropEvent

```rust
pub struct DropEvent {
    pub files: Vec<PathBuf>,
    pub target: DropTarget,
    pub x: f64,
    pub y: f64,
}

impl DropEvent {
    pub fn new(files: Vec<PathBuf>, target: DropTarget) -> Self
    pub fn with_position(self, x: f64, y: f64) -> Self
    pub fn first_file(&self) -> Option<&Path>
    pub fn gcode_files(&self) -> Vec<&Path>
    pub fn is_valid_for_target(&self, file_type: DropFileType) -> bool
}
```

### DropZone

```rust
pub struct DropZone {
    pub id: String,
    pub file_type: DropFileType,
    pub state: DropIndicatorState,
    pub enabled: bool,
}

impl DropZone {
    pub fn new(id: impl Into<String>, file_type: DropFileType) -> Self
    pub fn on_drag_over(&mut self, event: &DropEvent)
    pub fn on_drag_leave(&mut self)
    pub fn on_drop(&mut self)
    pub fn indicator_class(&self) -> &'static str
    pub fn indicator_color(&self) -> &'static str
}
```

## Changelog

### Version 0.16.0
- Task 95: File Export (400+ lines)
- Task 96: Drag and Drop (400+ lines)
- 36 comprehensive tests (all passing)
- Full API documentation
- Production-ready implementation
