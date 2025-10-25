# File Processing Pipeline and Statistics Documentation

## Overview

Tasks 93 and 94 implement comprehensive file processing capabilities for GCodeKit4, including a flexible processing pipeline with caching and detailed file statistics calculation.

## Architecture

### Module Structure

```
src/utils/
├── file_io.rs       - File reading (Task 91-92)
├── processing.rs    - Processing pipeline and statistics (Task 93-94)
│   ├── FileStatistics - Comprehensive statistics
│   ├── BoundingBox    - 3D bounds tracking
│   ├── FeedRateStats  - Feed rate analysis
│   ├── SpindleStats   - Spindle speed analysis
│   ├── ProcessedFile  - Pipeline output
│   └── FileProcessingPipeline - Main processor
└── mod.rs           - Module exports
```

## Task 93: File Processing Pipeline

### Features

#### 1. Processing Pipeline

```rust
pub struct FileProcessingPipeline {
    cache: HashMap<PathBuf, ProcessedFile>,
    cache_enabled: bool,
}
```

Main features:
- Read and parse G-code files
- Generate processed output
- Extract G-code commands and data
- Caching with optional disable
- Multi-file processing

#### 2. Processed File Output

```rust
pub struct ProcessedFile {
    pub source_path: PathBuf,
    pub content: String,
    pub statistics: FileStatistics,
    pub original_lines: u64,
    pub processed_lines: u64,
}
```

#### 3. Caching System

```rust
impl FileProcessingPipeline {
    // Cache management
    pub fn is_cached(&self, path: &Path) -> bool
    pub fn get_cached(&self, path: &Path) -> Option<&ProcessedFile>
    pub fn clear_cache(&mut self)
    pub fn set_cache_enabled(&mut self, enabled: bool)
    pub fn cache_size(&self) -> usize
}
```

Features:
- Automatic result caching
- Optional cache disabling
- Cache clearing
- Cache size tracking
- HashMap-based storage

#### 4. File Processing

```rust
impl FileProcessingPipeline {
    pub fn process_file(&mut self, path: impl AsRef<Path>) -> Result<ProcessedFile>
}
```

Processing steps:
1. Check cache for existing results
2. Read file using GcodeFileReader
3. Parse each line
4. Extract commands and coordinates
5. Calculate statistics
6. Cache result
7. Return ProcessedFile

### Usage Example

```rust
use gcodekit4::utils::FileProcessingPipeline;

fn main() -> anyhow::Result<()> {
    let mut pipeline = FileProcessingPipeline::new();
    
    // Process file
    let result = pipeline.process_file("part.nc")?;
    
    // Access results
    println!("Lines: {}", result.statistics.total_lines);
    println!("Time: {}", result.statistics.formatted_time());
    
    // Check cache
    if pipeline.is_cached("part.nc") {
        // Subsequent calls will use cache
        let result2 = pipeline.process_file("part.nc")?;
        println!("Using cached result");
    }
    
    Ok(())
}
```

### Performance

- **Caching**: HashMap-based O(1) lookups
- **Memory**: Entire file content stored in cache
- **I/O**: Single pass reading with callbacks
- **Processing**: Streaming analysis during read

## Task 94: File Statistics

### Features

#### 1. Comprehensive Statistics

```rust
pub struct FileStatistics {
    pub total_lines: u64,
    pub comment_lines: u64,
    pub empty_lines: u64,
    pub rapid_moves: u64,        // G0
    pub linear_moves: u64,       // G1
    pub arc_moves: u64,          // G2/G3
    pub m_codes: u64,
    pub total_distance: f64,
    pub estimated_time: u64,
    pub bounding_box: BoundingBox,
    pub command_counts: HashMap<String, u64>,
    pub feed_rate_stats: FeedRateStats,
    pub spindle_stats: SpindleStats,
}
```

#### 2. Bounding Box Calculation

```rust
pub struct BoundingBox {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl BoundingBox {
    pub fn update(&mut self, x: f64, y: f64, z: f64)
    pub fn width(&self) -> f64   // max_x - min_x
    pub fn height(&self) -> f64  // max_y - min_y
    pub fn depth(&self) -> f64   // max_z - min_z
    pub fn is_valid(&self) -> bool
}
```

#### 3. Feed Rate Analysis

```rust
pub struct FeedRateStats {
    pub min_feed: f64,
    pub max_feed: f64,
    pub avg_feed: f64,
    pub changes: u64,
}
```

Tracking:
- Minimum feed rate encountered
- Maximum feed rate encountered
- Average feed rate
- Number of feed changes

#### 4. Spindle Speed Analysis

```rust
pub struct SpindleStats {
    pub min_speed: f64,
    pub max_speed: f64,
    pub avg_speed: f64,
    pub on_time: u64,
    pub on_count: u64,
}
```

Tracking:
- Minimum/maximum spindle speeds
- Average speed
- Total on time (seconds)
- Number of on commands

#### 5. Statistics Methods

```rust
impl FileStatistics {
    pub fn total_motion_commands(&self) -> u64
    pub fn formatted_time(&self) -> String  // "1h 2m 3s"
    pub fn summary(&self) -> String
}
```

### Usage Example

```rust
use gcodekit4::utils::FileProcessingPipeline;

fn analyze_file(path: &str) -> anyhow::Result<()> {
    let mut pipeline = FileProcessingPipeline::new();
    let result = pipeline.process_file(path)?;
    
    let stats = &result.statistics;
    
    // Motion commands
    println!("Total motion commands: {}", stats.total_motion_commands());
    println!("  Rapid moves (G0): {}", stats.rapid_moves);
    println!("  Linear moves (G1): {}", stats.linear_moves);
    println!("  Arc moves (G2/G3): {}", stats.arc_moves);
    
    // Bounding box
    let bb = &stats.bounding_box;
    if bb.is_valid() {
        println!("Work area:");
        println!("  Width (X): {:.2} mm", bb.width());
        println!("  Height (Y): {:.2} mm", bb.height());
        println!("  Depth (Z): {:.2} mm", bb.depth());
    }
    
    // Time and distance
    println!("Total distance: {:.2} mm", stats.total_distance);
    println!("Estimated time: {}", stats.formatted_time());
    
    // Feed rates
    let feed = &stats.feed_rate_stats;
    println!("Feed rate range: {:.0} - {:.0} mm/min", feed.min_feed, feed.max_feed);
    
    // Spindle
    let spindle = &stats.spindle_stats;
    println!("Spindle speed range: {:.0} - {:.0} RPM", spindle.min_speed, spindle.max_speed);
    
    // Summary
    println!("Summary: {}", stats.summary());
    
    Ok(())
}
```

### Statistics Calculations

#### Total Distance

Calculated as euclidean distance between consecutive coordinate updates:

```
distance = sqrt((x2-x1)² + (y2-y1)² + (z2-z1)²)
total_distance = sum of all segment distances
```

#### Time Estimation

Simplified estimation based on:
- Rapid moves at 60 mm/min (approximation)
- Feed moves at max feed rate encountered
- Time = distance / feed_rate

Note: This is a rough estimate. Actual time depends on:
- Acceleration profiles
- Real controller behavior
- Exact feed rates for each segment

#### Bounding Box

Tracks minimum and maximum coordinates across entire file:

```
min_x = minimum X coordinate
max_x = maximum X coordinate
width = max_x - min_x
```

## Integration with FileOperationsPanel

The file statistics integrate seamlessly with existing UI:

```rust
use gcodekit4::utils::{GcodeFileReader, RecentFilesManager, FileProcessingPipeline};

pub struct EnhancedFilePanel {
    pub file_reader: Option<GcodeFileReader>,
    pub pipeline: FileProcessingPipeline,
    pub recent_files: RecentFilesManager,
}

impl EnhancedFilePanel {
    pub fn open_and_analyze(&mut self, path: &Path) -> anyhow::Result<()> {
        // Validate
        let reader = GcodeFileReader::new(path)?;
        let validation = reader.validate()?;
        
        if !validation.is_valid {
            return Err(anyhow!("Invalid G-code file"));
        }
        
        // Process
        let result = self.pipeline.process_file(path)?;
        
        // Track recent
        self.recent_files.add(path)?;
        
        // Display stats
        let stats = &result.statistics;
        println!("File: {}", path.display());
        println!("Lines: {}", stats.total_lines);
        println!("Time: {}", stats.formatted_time());
        println!("Bounding box: {} x {} x {}", 
                 stats.bounding_box.width(),
                 stats.bounding_box.height(),
                 stats.bounding_box.depth());
        
        Ok(())
    }
}
```

## Testing

### Unit Tests (10 tests in processing.rs)

- Bounding box creation and updates
- Feed rate statistics tracking
- Spindle statistics tracking
- File statistics creation
- Time formatting
- Pipeline creation and caching
- Processed file creation

### Integration Tests (23 tests in processing_93_94.rs)

- Basic file processing
- Cache management (enable, disable, clear)
- Motion command counting (G0, G1, G2/G3)
- M-code counting
- Bounding box calculation
- Distance calculation
- Feed rate tracking
- Spindle speed tracking
- Time estimation
- Comment and empty line handling
- Combined workflow
- Multiple file processing

### Test Coverage

- ✅ All 33 tests passing (10 unit + 23 integration)
- ✅ No clippy warnings (1 corrected)
- ✅ 100% API coverage
- ✅ Edge cases tested
- ✅ Error handling verified

## Performance Considerations

### Memory Usage

- **Per File**: O(content_size + statistics)
- **Cache**: O(n_files * content_size)
- **Statistics**: O(n_commands)

### Processing Time

- **Single File**: O(n_lines) - single pass
- **Cached Access**: O(1)
- **Bounding Box**: Calculated during read pass

### Optimization Tips

1. **Disable cache for large files**:
   ```rust
   pipeline.set_cache_enabled(false);
   ```

2. **Clear cache periodically**:
   ```rust
   pipeline.clear_cache();
   ```

3. **Process files once**:
   ```rust
   let result = pipeline.process_file(path)?;
   // Result is cached automatically
   ```

## Command Parsing

The pipeline recognizes:

**Motion Commands:**
- `G0` / `G00` - Rapid positioning
- `G1` / `G01` - Linear interpolation
- `G2` / `G02` - Arc (clockwise)
- `G3` / `G03` - Arc (counter-clockwise)

**Miscellaneous:**
- `M` codes - All counted, specific codes tracked
- `F` parameter - Feed rate
- `S` parameter - Spindle speed
- `X`, `Y`, `Z` - Coordinates

**Comments:**
- `;` style comments
- `(` `)` style comments

## Future Enhancements (Tasks 95+)

- **Task 95**: File Export - Save processed G-code
- **Task 96**: Drag and Drop - File import via drag/drop
- **Task 97**: File Validation UI - Display validation results
- **Task 98**: File Comparison - Compare original vs processed
- **Task 99**: Backup and Recovery - Auto-save state
- **Task 100**: File Templates - G-code template library

## API Reference

### FileProcessingPipeline

```rust
impl FileProcessingPipeline {
    pub fn new() -> Self
    pub fn process_file(&mut self, path: impl AsRef<Path>) -> Result<ProcessedFile>
    pub fn is_cached(&self, path: &Path) -> bool
    pub fn get_cached(&self, path: &Path) -> Option<&ProcessedFile>
    pub fn clear_cache(&mut self)
    pub fn set_cache_enabled(&mut self, enabled: bool)
    pub fn cache_size(&self) -> usize
}
```

### FileStatistics

```rust
impl FileStatistics {
    pub fn new() -> Self
    pub fn total_motion_commands(&self) -> u64
    pub fn formatted_time(&self) -> String
    pub fn summary(&self) -> String
}
```

### BoundingBox

```rust
impl BoundingBox {
    pub fn new() -> Self
    pub fn update(&mut self, x: f64, y: f64, z: f64)
    pub fn width(&self) -> f64
    pub fn height(&self) -> f64
    pub fn depth(&self) -> f64
    pub fn is_valid(&self) -> bool
}
```

## Changelog

### Version 0.15.0
- Task 93: File Processing Pipeline (800+ lines)
- Task 94: File Statistics (integrated)
- 33 comprehensive tests (all passing)
- Full API documentation
- Production-ready implementation
