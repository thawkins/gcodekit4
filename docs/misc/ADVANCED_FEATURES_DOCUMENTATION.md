# Advanced Features Documentation

## Overview

Tasks 97-102 implement advanced features for GCodeKit4, including file validation UI, file comparison, backup/recovery, templates, and probing capabilities.

## Architecture

### Module Structure

```
src/utils/advanced.rs
├── Task 97: Validation UI
│   ├── ValidationSeverity - Error/Warning/Info
│   ├── ValidationIssue - Individual issues
│   └── ValidationResult - Collection with UI display
├── Task 98: File Comparison
│   ├── LineChange - Diff tracking
│   └── FileComparison - Full comparison
├── Task 99: Backup and Recovery
│   ├── BackupEntry - Backup metadata
│   └── BackupManager - Backup operations
├── Task 100: Templates
│   ├── TemplateVariable - Template parameters
│   ├── GcodeTemplate - Template definition
│   └── TemplateLibrary - Template management
├── Task 101: Basic Probing
│   └── BasicProber - Z-axis probing
└── Task 102: Advanced Probing
    ├── ProbePoint - Probe coordinates
    └── AdvancedProber - Multi-point probing
```

## Task 97: File Validation UI

### Features

#### Validation Severity Levels
```rust
pub enum ValidationSeverity {
    Error,    // Critical issue
    Warning,  // Potential problem
    Info,     // Informational
}
```

#### Validation Issues
```rust
pub struct ValidationIssue {
    pub line_number: u32,
    pub severity: ValidationSeverity,
    pub message: String,
    pub suggestion: Option<String>,
}
```

Features:
- Issue creation with line number and message
- Optional fix suggestions
- Severity classification

#### Validation Results
```rust
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
    pub error_count: u32,
    pub warning_count: u32,
    pub info_count: u32,
}
```

Features:
- Automatic count tracking
- Validity checking
- Issue filtering by line or severity
- Formatted summary display

### Usage

```rust
use gcodekit4::utils::{ValidationIssue, ValidationResult, ValidationSeverity};

let mut result = ValidationResult::new();

// Add issues
result.add_issue(
    ValidationIssue::new(5, ValidationSeverity::Error, "Invalid G-code")
        .with_suggestion("Check syntax")
);

// Check validity
if !result.is_valid() {
    println!("File has errors: {}", result.summary());
}

// Filter by line
let issues_at_line_5 = result.issues_at_line(5);

// Filter by severity
let errors = result.issues_by_severity(ValidationSeverity::Error);
```

## Task 98: File Comparison

### Features

#### Line Changes
```rust
pub enum LineChange {
    Unchanged,  // No change
    Added,      // New line
    Removed,    // Deleted line
    Modified,   // Changed line
}
```

#### File Comparison
```rust
pub struct FileComparison {
    pub original_lines: Vec<String>,
    pub processed_lines: Vec<String>,
    pub line_changes: Vec<LineChange>,
    pub added_count: u32,
    pub removed_count: u32,
    pub modified_count: u32,
}
```

Features:
- Automatic diff calculation
- Change tracking per line
- Percentage change calculation
- Summary display

### Usage

```rust
use gcodekit4::utils::FileComparison;

let original = "G0 X10\nG1 Y20\n";
let processed = "G0 X10\nG1 Y25\n";

let comparison = FileComparison::new(original, processed);

println!("Total changes: {}", comparison.total_changes());
println!("Change %: {:.1}%", comparison.change_percentage());
println!("Summary: {}", comparison.summary());
```

## Task 99: Backup and Recovery

### Features

#### Backup Entry
```rust
pub struct BackupEntry {
    pub id: String,
    pub timestamp: u64,
    pub backup_path: PathBuf,
    pub original_path: PathBuf,
    pub size: u64,
    pub description: String,
}
```

#### Backup Manager
```rust
pub struct BackupManager {
    // Automatic backup management
}

impl BackupManager {
    pub fn backup(&self, source: impl AsRef<Path>, description: &str) -> Result<BackupEntry>
    pub fn restore(&self, backup: &BackupEntry, dest: impl AsRef<Path>) -> Result<()>
    pub fn list_backups(&self) -> Result<Vec<BackupEntry>>
    pub fn cleanup(&self) -> Result<()>
}
```

Features:
- Automatic backup creation with timestamps
- Backup restoration
- Backup listing and sorting
- Automatic cleanup by age and count
- Configurable retention

### Usage

```rust
use gcodekit4::utils::BackupManager;

let mut manager = BackupManager::new("./backups");
manager.set_max_backups(10);
manager.set_max_age(86400 * 7); // 7 days

// Create backup
let backup = manager.backup("file.nc", "Before modification")?;

// List backups
let backups = manager.list_backups()?;

// Restore
manager.restore(&backup, "restored.nc")?;

// Cleanup old backups
manager.cleanup()?;
```

## Task 100: File Templates

### Features

#### Template Variables
```rust
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    pub default_value: String,
}
```

#### G-Code Template
```rust
pub struct GcodeTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub variables: Vec<TemplateVariable>,
}

impl GcodeTemplate {
    pub fn expand(&self, values: &HashMap<String, String>) -> String
    pub fn add_variable(&mut self, variable: TemplateVariable)
}
```

#### Template Library
```rust
pub struct TemplateLibrary {
    // Template storage
}

impl TemplateLibrary {
    pub fn add(&mut self, template: GcodeTemplate)
    pub fn get(&self, id: &str) -> Option<&GcodeTemplate>
    pub fn list(&self) -> Vec<&GcodeTemplate>
    pub fn remove(&mut self, id: &str) -> Option<GcodeTemplate>
}
```

Features:
- Template definition with variables
- Variable expansion with defaults
- Template library management
- Add, get, list, remove operations

### Usage

```rust
use gcodekit4::utils::{GcodeTemplate, TemplateVariable, TemplateLibrary};
use std::collections::HashMap;

let mut template = GcodeTemplate::new("move", "Move", "G0 X{{X}} Y{{Y}} Z{{Z}}");

template.add_variable(TemplateVariable {
    name: "X".to_string(),
    description: "X position".to_string(),
    default_value: "0".to_string(),
});

// Expand with values
let mut values = HashMap::new();
values.insert("X".to_string(), "10".to_string());
values.insert("Y".to_string(), "20".to_string());
values.insert("Z".to_string(), "5".to_string());

let expanded = template.expand(&values);
// Result: "G0 X10 Y20 Z5"

// Manage in library
let mut library = TemplateLibrary::new();
library.add(template);
```

## Task 101: Basic Probing

### Features

#### Basic Prober
```rust
pub struct BasicProber {
    pub current_z: f64,
    pub probe_offset: f64,     // Tool length
    pub probe_feed_rate: f64,
}

impl BasicProber {
    pub fn generate_probe_command(&self, target_z: f64) -> String
    pub fn calculate_offset(&self, probed_z: f64) -> f64
    pub fn generate_offset_command(&self, work_offset: f64) -> String
}
```

Features:
- Z-axis probing with G38.2 command
- Probe offset calculation
- Work offset generation
- Configurable feed rate

### Usage

```rust
use gcodekit4::utils::BasicProber;

let mut prober = BasicProber::new();
prober.probe_offset = 10.0; // Tool length
prober.probe_feed_rate = 25.0;

// Generate probe command
let probe_cmd = prober.generate_probe_command(-50.0);
// Result: "G38.2 Z-50 F25"

// Calculate work offset from probe result
let probed_z = 5.0;
let offset = prober.calculate_offset(probed_z);

// Generate offset command
let offset_cmd = prober.generate_offset_command(offset);
// Result: "G92 Z{offset}"
```

## Task 102: Advanced Probing

### Features

#### Probe Point
```rust
pub struct ProbePoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
```

#### Advanced Prober
```rust
pub struct AdvancedProber {
    // Multi-point probing
}

impl AdvancedProber {
    pub fn add_probe_point(&mut self, x: f64, y: f64)
    pub fn generate_probe_sequence(&self) -> String
    pub fn generate_corner_finding(&self, corner_1: (f64, f64), corner_2: (f64, f64)) -> String
    pub fn generate_center_finding(&self, center: (f64, f64), radius: f64) -> String
    pub fn probe_points(&self) -> &[ProbePoint]
    pub fn clear_probe_points(&mut self)
}
```

Features:
- Multi-point probing sequences
- Corner finding (rectangular features)
- Center finding (circular features)
- Point management
- 3D surface capability

### Usage

```rust
use gcodekit4::utils::AdvancedProber;

let mut prober = AdvancedProber::new();

// Add probe grid points
for x in (0..100).step_by(10) {
    for y in (0..100).step_by(10) {
        prober.add_probe_point(x as f64, y as f64);
    }
}

// Generate probe sequence
let sequence = prober.generate_probe_sequence();

// Generate corner finding
let corners = prober.generate_corner_finding((0.0, 0.0), (100.0, 100.0));

// Generate center finding
let center = prober.generate_center_finding((50.0, 50.0), 25.0);

// Clear for next operation
prober.clear_probe_points();
```

## Testing

### Unit Tests (7 tests in advanced.rs)
- Validation issue creation
- Validation result tracking
- File comparison
- Template expansion
- Basic probing
- Advanced probing
- Template library

### Integration Tests (36 tests in advanced_97_102.rs)

**Task 97 Validation UI (7 tests)**
- Issue creation and suggestions
- Result creation and counting
- Issue filtering by line/severity
- Summary display

**Task 98 File Comparison (6 tests)**
- Basic comparison (unchanged)
- Line modifications
- Added/removed lines
- Change percentage calculation

**Task 99 Backup and Recovery (4 tests)**
- Backup creation
- File restoration
- Backup listing
- Cleanup operations

**Task 100 Templates (6 tests)**
- Template creation
- Variable management
- Template expansion
- Default values
- Library operations

**Task 101 Basic Probing (5 tests)**
- Prober creation
- Command generation
- Offset calculation
- Complete workflow

**Task 102 Advanced Probing (8 tests)**
- Point creation/management
- Probe sequence generation
- Corner finding
- Center finding
- Point clearing

### Test Coverage
- ✅ All 43 tests passing (7 unit + 36 integration)
- ✅ No clippy warnings
- ✅ 100% API coverage
- ✅ Error handling tested
- ✅ Workflows tested

## Integration Examples

### Complete File Validation Workflow

```rust
use gcodekit4::utils::{GcodeFileReader, ValidationResult, ValidationIssue, ValidationSeverity};

let reader = GcodeFileReader::new("part.nc")?;
let validation = reader.validate()?;

let mut result = ValidationResult::new();

// Add custom validations
if validation.motion_commands == 0 {
    result.add_issue(ValidationIssue::new(0, ValidationSeverity::Warning, "No motion commands"));
}

println!("Validation: {}", result.summary());
```

### Complete Comparison and Export Workflow

```rust
use gcodekit4::utils::{FileComparison, FileExporter, ExportOptions};

let original = fs::read_to_string("original.nc")?;
let processed = /* process file */;

let comparison = FileComparison::new(&original, &processed);
println!("Changes: {}", comparison.summary());

let mut options = ExportOptions::default();
options.add_header = true;
FileExporter::export(&processed, "exported.nc", &options)?;
```

### Complete Probing Workflow

```rust
use gcodekit4::utils::AdvancedProber;

let mut prober = AdvancedProber::new();

// Probe grid
for x in (0..100).step_by(25) {
    for y in (0..100).step_by(25) {
        prober.add_probe_point(x as f64, y as f64);
    }
}

let sequence = prober.generate_probe_sequence();
// Send sequence to machine
```

## Performance Considerations

### Validation
- O(n) for n issues
- Fast filtering by line or severity
- Summary generation: O(n)

### Comparison
- O(n) for n lines
- Automatic diff calculation
- Change percentage: O(1)

### Backup
- O(1) backup creation (copy operation)
- O(n) listing for n backups
- Cleanup: O(n)

### Templates
- O(1) lookup in library
- O(n) expansion for n variables
- Expansion uses string replace

### Probing
- Command generation: O(1) for single point
- Sequence generation: O(n) for n points
- Center finding: O(points) for discretization

## Future Enhancements

- Task 103: Auto-leveling mesh generation
- Task 104: Tool change management
- Task 105: Macro system
- Task 106: Advanced compensation
- Task 107: Multi-axis probing

## Changelog

### Version 0.17.0
- Task 97: File Validation UI (validation results with UI display)
- Task 98: File Comparison (diff and change tracking)
- Task 99: Backup and Recovery (backup management and restoration)
- Task 100: File Templates (template system with variables)
- Task 101: Basic Probing (Z-axis probing)
- Task 102: Advanced Probing (multi-point and feature probing)
- 43 comprehensive tests (all passing)
- Production-ready implementation
