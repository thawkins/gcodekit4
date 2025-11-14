//! File Export Module - Task 95
//! Drag and Drop Support - Task 96
//!
//! Task 95: File Export
//! - Export processed G-code
//! - Save modified files
//! - Add file format options
//!
//! Task 96: Drag and Drop Support
//! - Implement file drag and drop
//! - Support multiple file types
//! - Show drop indicators
//! - Handle drop events

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// File format options for export
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FileFormat {
    /// Standard G-code (.nc)
    #[default]
    GCode,
    /// Generic G-code (.gcode)
    GenericGCode,
    /// NGC format (.ngc)
    NGC,
    /// GCO format (.gco)
    GCO,
}

impl FileFormat {
    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            FileFormat::GCode => "nc",
            FileFormat::GenericGCode => "gcode",
            FileFormat::NGC => "ngc",
            FileFormat::GCO => "gco",
        }
    }

    /// Get MIME type
    pub fn mime_type(&self) -> &'static str {
        "text/plain"
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "nc" => Some(FileFormat::GCode),
            "gcode" => Some(FileFormat::GenericGCode),
            "ngc" => Some(FileFormat::NGC),
            "gco" => Some(FileFormat::GCO),
            _ => None,
        }
    }
}

/// Export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// File format
    pub format: FileFormat,
    /// Include comments in export
    pub include_comments: bool,
    /// Include empty lines
    pub include_empty_lines: bool,
    /// Add header with export info
    pub add_header: bool,
    /// Add timestamp in header
    pub add_timestamp: bool,
    /// Line ending: true = Unix (\n), false = Windows (\r\n)
    pub unix_line_endings: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: FileFormat::GCode,
            include_comments: true,
            include_empty_lines: false,
            add_header: true,
            add_timestamp: true,
            unix_line_endings: true,
        }
    }
}

/// File exporter
pub struct FileExporter;

impl FileExporter {
    /// Export content to file
    pub fn export(
        content: &str,
        dest_path: impl AsRef<Path>,
        options: &ExportOptions,
    ) -> Result<()> {
        let dest_path = dest_path.as_ref();

        // Validate path
        if let Some(parent) = dest_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Prepare content
        let mut output = String::new();

        // Add header if requested
        if options.add_header {
            output.push_str(&Self::generate_header(options));
        }

        // Process content
        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines if not including them
            if trimmed.is_empty() && !options.include_empty_lines {
                continue;
            }

            // Skip comments if not including them
            if !options.include_comments && (trimmed.starts_with(';') || trimmed.starts_with('(')) {
                continue;
            }

            output.push_str(trimmed);
            if options.unix_line_endings {
                output.push('\n');
            } else {
                output.push_str("\r\n");
            }
        }

        // Write file
        fs::write(dest_path, &output)?;

        Ok(())
    }

    /// Export with default options
    pub fn export_simple(content: &str, dest_path: impl AsRef<Path>) -> Result<()> {
        Self::export(content, dest_path, &ExportOptions::default())
    }

    /// Generate header for export
    fn generate_header(options: &ExportOptions) -> String {
        let mut header = String::new();
        header.push_str("; GCodeKit4 Exported File\n");

        if options.add_timestamp {
            let now = chrono::Local::now();
            header.push_str(&format!(
                "; Exported: {}\n",
                now.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        header.push_str("; Format options:\n");
        header.push_str(&format!(
            ";   Include comments: {}\n",
            options.include_comments
        ));
        header.push_str(&format!(
            ";   Include empty lines: {}\n",
            options.include_empty_lines
        ));
        header.push_str(&format!(
            ";   Line endings: {}\n",
            if options.unix_line_endings {
                "Unix"
            } else {
                "Windows"
            }
        ));
        header.push_str(";\n");

        header
    }
}

/// Drag and drop support
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropTarget {
    /// File drop on editor
    Editor,
    /// File drop on file browser
    FileBrowser,
    /// File drop on canvas
    Canvas,
    /// Generic drop area
    Generic,
}

/// Supported drop file types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropFileType {
    /// G-code file
    GCode,
    /// Image file (for reference)
    Image,
    /// Text file
    Text,
    /// All files
    All,
}

impl DropFileType {
    /// Check if file matches this type
    pub fn matches(&self, path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match self {
            DropFileType::GCode => {
                matches!(ext.as_str(), "nc" | "gcode" | "ngc" | "gco")
            }
            DropFileType::Image => {
                matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp")
            }
            DropFileType::Text => {
                matches!(ext.as_str(), "txt" | "gcode" | "nc" | "ngc" | "gco")
            }
            DropFileType::All => true,
        }
    }

    /// Get supported extensions
    pub fn extensions(&self) -> Vec<&'static str> {
        match self {
            DropFileType::GCode => vec!["nc", "gcode", "ngc", "gco"],
            DropFileType::Image => vec!["png", "jpg", "jpeg", "gif", "bmp"],
            DropFileType::Text => vec!["txt", "gcode", "nc", "ngc", "gco"],
            DropFileType::All => vec![],
        }
    }
}

/// Drop event data
#[derive(Debug, Clone)]
pub struct DropEvent {
    /// Files being dropped
    pub files: Vec<PathBuf>,
    /// Target for drop
    pub target: DropTarget,
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
}

impl DropEvent {
    /// Create new drop event
    pub fn new(files: Vec<PathBuf>, target: DropTarget) -> Self {
        Self {
            files,
            target,
            x: 0.0,
            y: 0.0,
        }
    }

    /// Set position
    pub fn with_position(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Get first file
    pub fn first_file(&self) -> Option<&Path> {
        self.files.first().map(|p| p.as_path())
    }

    /// Get all G-code files
    pub fn gcode_files(&self) -> Vec<&Path> {
        self.files
            .iter()
            .filter(|p| DropFileType::GCode.matches(p))
            .map(|p| p.as_path())
            .collect()
    }

    /// Check if drop is valid for target
    pub fn is_valid_for_target(&self, file_type: DropFileType) -> bool {
        self.files.iter().any(|f| file_type.matches(f))
    }
}

/// Drop indicator visual feedback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropIndicatorState {
    /// No drag in progress
    None,
    /// Dragging over area
    Over,
    /// Dragging with valid files
    Valid,
    /// Dragging with invalid files
    Invalid,
}

/// Drop zone for drag and drop
#[derive(Debug, Clone)]
pub struct DropZone {
    /// Zone ID
    pub id: String,
    /// Supported file types
    pub file_type: DropFileType,
    /// Current state
    pub state: DropIndicatorState,
    /// Is enabled
    pub enabled: bool,
}

impl DropZone {
    /// Create new drop zone
    pub fn new(id: impl Into<String>, file_type: DropFileType) -> Self {
        Self {
            id: id.into(),
            file_type,
            state: DropIndicatorState::None,
            enabled: true,
        }
    }

    /// Update state on drag over
    pub fn on_drag_over(&mut self, event: &DropEvent) {
        if !self.enabled {
            self.state = DropIndicatorState::None;
            return;
        }

        if event.is_valid_for_target(self.file_type) {
            self.state = DropIndicatorState::Valid;
        } else {
            self.state = DropIndicatorState::Invalid;
        }
    }

    /// Update state on drag leave
    pub fn on_drag_leave(&mut self) {
        self.state = DropIndicatorState::None;
    }

    /// Update state on drop
    pub fn on_drop(&mut self) {
        self.state = DropIndicatorState::None;
    }

    /// Get CSS class for indicator
    pub fn indicator_class(&self) -> &'static str {
        match self.state {
            DropIndicatorState::None => "",
            DropIndicatorState::Over => "drop-over",
            DropIndicatorState::Valid => "drop-valid",
            DropIndicatorState::Invalid => "drop-invalid",
        }
    }

    /// Get visual color for state
    pub fn indicator_color(&self) -> &'static str {
        match self.state {
            DropIndicatorState::None => "transparent",
            DropIndicatorState::Over => "rgba(100, 100, 100, 0.3)",
            DropIndicatorState::Valid => "rgba(0, 255, 0, 0.2)",
            DropIndicatorState::Invalid => "rgba(255, 0, 0, 0.2)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_format_extensions() {
        assert_eq!(FileFormat::GCode.extension(), "nc");
        assert_eq!(FileFormat::GenericGCode.extension(), "gcode");
        assert_eq!(FileFormat::NGC.extension(), "ngc");
        assert_eq!(FileFormat::GCO.extension(), "gco");
    }

    #[test]
    fn test_file_format_detection() {
        assert_eq!(FileFormat::from_extension("nc"), Some(FileFormat::GCode));
        assert_eq!(
            FileFormat::from_extension("gcode"),
            Some(FileFormat::GenericGCode)
        );
        assert_eq!(FileFormat::from_extension("ngc"), Some(FileFormat::NGC));
        assert_eq!(FileFormat::from_extension("gco"), Some(FileFormat::GCO));
        assert_eq!(FileFormat::from_extension("txt"), None);
    }

    #[test]
    fn test_export_options_default() {
        let opts = ExportOptions::default();
        assert_eq!(opts.format, FileFormat::GCode);
        assert!(opts.include_comments);
        assert!(!opts.include_empty_lines);
        assert!(opts.add_header);
        assert!(opts.add_timestamp);
        assert!(opts.unix_line_endings);
    }

    #[test]
    fn test_drop_file_type_gcode() {
        let ft = DropFileType::GCode;
        assert!(ft.matches(Path::new("test.nc")));
        assert!(ft.matches(Path::new("test.gcode")));
        assert!(ft.matches(Path::new("test.ngc")));
        assert!(ft.matches(Path::new("test.gco")));
        assert!(!ft.matches(Path::new("test.txt")));
    }

    #[test]
    fn test_drop_file_type_image() {
        let ft = DropFileType::Image;
        assert!(ft.matches(Path::new("test.png")));
        assert!(ft.matches(Path::new("test.jpg")));
        assert!(ft.matches(Path::new("test.jpeg")));
        assert!(!ft.matches(Path::new("test.nc")));
    }

    #[test]
    fn test_drop_event_creation() {
        let files = vec![PathBuf::from("test.nc")];
        let event = DropEvent::new(files.clone(), DropTarget::Editor);
        assert_eq!(event.files.len(), 1);
        assert_eq!(event.target, DropTarget::Editor);
    }

    #[test]
    fn test_drop_event_first_file() {
        let files = vec![PathBuf::from("test1.nc"), PathBuf::from("test2.nc")];
        let event = DropEvent::new(files, DropTarget::Editor);
        assert_eq!(event.first_file(), Some(Path::new("test1.nc")));
    }

    #[test]
    fn test_drop_event_gcode_files() {
        let files = vec![
            PathBuf::from("test1.nc"),
            PathBuf::from("test2.txt"),
            PathBuf::from("test3.gcode"),
        ];
        let event = DropEvent::new(files, DropTarget::Editor);
        let gcode_files = event.gcode_files();
        assert_eq!(gcode_files.len(), 2);
    }

    #[test]
    fn test_drop_zone_creation() {
        let zone = DropZone::new("editor", DropFileType::GCode);
        assert_eq!(zone.id, "editor");
        assert_eq!(zone.file_type, DropFileType::GCode);
        assert!(zone.enabled);
        assert_eq!(zone.state, DropIndicatorState::None);
    }

    #[test]
    fn test_drop_zone_drag_over() {
        let mut zone = DropZone::new("editor", DropFileType::GCode);
        let event = DropEvent::new(vec![PathBuf::from("test.nc")], DropTarget::Editor);

        zone.on_drag_over(&event);
        assert_eq!(zone.state, DropIndicatorState::Valid);
    }

    #[test]
    fn test_drop_zone_invalid_drop() {
        let mut zone = DropZone::new("editor", DropFileType::GCode);
        let event = DropEvent::new(vec![PathBuf::from("test.txt")], DropTarget::Editor);

        zone.on_drag_over(&event);
        assert_eq!(zone.state, DropIndicatorState::Invalid);
    }

    #[test]
    fn test_drop_zone_indicator_colors() {
        let mut zone = DropZone::new("editor", DropFileType::GCode);
        assert_eq!(zone.indicator_color(), "transparent");

        zone.state = DropIndicatorState::Over;
        assert_eq!(zone.indicator_color(), "rgba(100, 100, 100, 0.3)");

        zone.state = DropIndicatorState::Valid;
        assert_eq!(zone.indicator_color(), "rgba(0, 255, 0, 0.2)");

        zone.state = DropIndicatorState::Invalid;
        assert_eq!(zone.indicator_color(), "rgba(255, 0, 0, 0.2)");
    }

    #[test]
    fn test_drop_zone_disabled() {
        let mut zone = DropZone::new("editor", DropFileType::GCode);
        zone.enabled = false;

        let event = DropEvent::new(vec![PathBuf::from("test.nc")], DropTarget::Editor);
        zone.on_drag_over(&event);
        assert_eq!(zone.state, DropIndicatorState::None);
    }

    #[test]
    fn test_drop_zone_leave() {
        let mut zone = DropZone::new("editor", DropFileType::GCode);
        zone.state = DropIndicatorState::Valid;

        zone.on_drag_leave();
        assert_eq!(zone.state, DropIndicatorState::None);
    }

    #[test]
    fn test_drop_target_equality() {
        assert_eq!(DropTarget::Editor, DropTarget::Editor);
        assert_ne!(DropTarget::Editor, DropTarget::FileBrowser);
    }
}
