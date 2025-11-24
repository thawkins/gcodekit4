//! File Operations Panel - Task 71
//!
//! G-Code file browser, open dialog, and file information display

use std::path::{Path, PathBuf};

/// File type filter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFilter {
    /// G-Code files (.nc, .gcode, .ngc, .gco)
    GCode,
    /// All files
    All,
}

impl FileFilter {
    /// Get file extensions for this filter
    pub fn extensions(&self) -> Vec<&str> {
        match self {
            Self::GCode => vec!["nc", "gcode", "ngc", "gco"],
            Self::All => vec![],
        }
    }

    /// Check if file matches this filter
    pub fn matches(&self, path: &Path) -> bool {
        match self {
            Self::GCode => path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| self.extensions().contains(&ext))
                .unwrap_or(false),
            Self::All => true,
        }
    }
}

/// File information
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// File path
    pub path: PathBuf,
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// Line count
    pub line_count: u32,
    /// Last modified timestamp
    pub modified: String,
}

impl FileInfo {
    /// Create new file info
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let metadata = std::fs::metadata(&path)?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| {
                time.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .map(|d| format!("{}", d.as_secs()))
            })
            .unwrap_or_else(|| "unknown".to_string());

        Ok(Self {
            path,
            name,
            size: metadata.len(),
            line_count: 0,
            modified,
        })
    }

    /// Get file size in KB
    pub fn size_kb(&self) -> f64 {
        self.size as f64 / 1024.0
    }

    /// Get formatted size string
    pub fn formatted_size(&self) -> String {
        if self.size < 1024 {
            format!("{} B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{:.2} KB", self.size_kb())
        } else {
            format!("{:.2} MB", self.size as f64 / (1024.0 * 1024.0))
        }
    }

    /// Set line count
    pub fn set_line_count(&mut self, count: u32) {
        self.line_count = count;
    }
}

/// File statistics
#[derive(Debug, Clone, Default)]
pub struct FileStatistics {
    /// Total lines
    pub total_lines: u32,
    /// G0 rapid moves count
    pub rapid_moves: u32,
    /// G1 linear moves count
    pub linear_moves: u32,
    /// G2/G3 arc moves count
    pub arc_moves: u32,
    /// M-code count
    pub m_codes: u32,
    /// Comment lines count
    pub comments: u32,
    /// Estimated run time (seconds)
    pub estimated_time: u32,
}

impl FileStatistics {
    /// Get estimated time formatted
    pub fn formatted_time(&self) -> String {
        let hours = self.estimated_time / 3600;
        let minutes = (self.estimated_time % 3600) / 60;
        let seconds = self.estimated_time % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

/// Recent file entry
#[derive(Debug, Clone)]
pub struct RecentFile {
    /// File path
    pub path: PathBuf,
    /// File name
    pub name: String,
    /// Last opened timestamp
    pub timestamp: u64,
}

impl RecentFile {
    /// Create new recent file entry
    pub fn new(path: impl AsRef<Path>, timestamp: u64) -> Self {
        let path = path.as_ref().to_path_buf();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
            path,
            name,
            timestamp,
        }
    }
}

/// File operations panel
#[derive(Debug)]
pub struct FileOperationsPanel {
    /// Current directory
    pub current_dir: PathBuf,
    /// Current file filter
    pub filter: FileFilter,
    /// Available files in current directory
    pub files: Vec<FileInfo>,
    /// Selected file
    pub selected_file: Option<FileInfo>,
    /// Recent files list
    pub recent_files: Vec<RecentFile>,
    /// Current file info
    pub current_file: Option<FileInfo>,
    /// File statistics
    pub file_stats: Option<FileStatistics>,
    /// Maximum recent files to keep
    pub max_recent: usize,
}

impl FileOperationsPanel {
    /// Create new file operations panel
    pub fn new() -> Self {
        Self {
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            filter: FileFilter::GCode,
            files: Vec::new(),
            selected_file: None,
            recent_files: Vec::new(),
            current_file: None,
            file_stats: None,
            max_recent: 10,
        }
    }

    /// Set current directory and refresh file list
    pub fn set_directory(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory"));
        }

        self.current_dir = path.to_path_buf();
        self.refresh_file_list()?;
        Ok(())
    }

    /// Refresh file list from current directory
    pub fn refresh_file_list(&mut self) -> anyhow::Result<()> {
        self.files.clear();

        for entry in std::fs::read_dir(&self.current_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && self.filter.matches(&path) {
                if let Ok(info) = FileInfo::new(&path) {
                    self.files.push(info);
                }
            }
        }

        // Sort by name
        self.files.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(())
    }

    /// Select a file
    pub fn select_file(&mut self, index: usize) {
        if let Some(file) = self.files.get(index) {
            self.selected_file = Some(file.clone());
        }
    }

    /// Open selected file
    pub fn open_file(&mut self) -> anyhow::Result<()> {
        if let Some(file) = &self.selected_file {
            // Read and count lines
            let content = std::fs::read_to_string(&file.path)?;
            let line_count = content.lines().count() as u32;

            let mut file_info = file.clone();
            file_info.set_line_count(line_count);

            // Add to recent files
            self.add_recent_file(file.path.clone());

            self.current_file = Some(file_info);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No file selected"))
        }
    }

    /// Add file to recent list
    pub fn add_recent_file(&mut self, path: PathBuf) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let recent = RecentFile::new(&path, now);

        // Remove if already exists
        self.recent_files.retain(|f| f.path != path);

        // Add to front
        self.recent_files.insert(0, recent);

        // Trim to max size
        self.recent_files.truncate(self.max_recent);
    }

    /// Get recent files list
    pub fn get_recent_files(&self) -> Vec<&RecentFile> {
        self.recent_files.iter().collect()
    }

    /// Set file statistics
    pub fn set_file_statistics(&mut self, stats: FileStatistics) {
        self.file_stats = Some(stats);
    }

    /// Set file filter
    pub fn set_filter(&mut self, filter: FileFilter) -> anyhow::Result<()> {
        self.filter = filter;
        self.refresh_file_list()?;
        Ok(())
    }

    /// Get directory listing as strings
    pub fn get_file_list_strings(&self) -> Vec<String> {
        self.files
            .iter()
            .map(|f| format!("{} ({})", f.name, f.formatted_size()))
            .collect()
    }

    /// Get current file summary
    pub fn current_file_summary(&self) -> Option<String> {
        self.current_file.as_ref().map(|file| {
            let stats = self.file_stats.as_ref();
            match stats {
                Some(s) => {
                    format!(
                        "{}: {} lines, {}, Est. time: {}",
                        file.name,
                        file.line_count,
                        file.formatted_size(),
                        s.formatted_time()
                    )
                }
                None => {
                    format!(
                        "{}: {} lines, {}",
                        file.name,
                        file.line_count,
                        file.formatted_size()
                    )
                }
            }
        })
    }
}

impl Default for FileOperationsPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_filter() {
        let filter = FileFilter::GCode;
        assert!(filter.matches(Path::new("test.nc")));
        assert!(filter.matches(Path::new("test.gcode")));
        assert!(!filter.matches(Path::new("test.txt")));
    }

    #[test]
    fn test_file_filter_all() {
        let filter = FileFilter::All;
        assert!(filter.matches(Path::new("test.nc")));
        assert!(filter.matches(Path::new("test.txt")));
        assert!(filter.matches(Path::new("any.file")));
    }

    #[test]
    fn test_file_statistics() {
        let mut stats = FileStatistics::default();
        stats.estimated_time = 3661;
        assert_eq!(stats.formatted_time(), "1h 1m 1s");
    }

    #[test]
    fn test_file_statistics_short() {
        let mut stats = FileStatistics::default();
        stats.estimated_time = 125;
        assert_eq!(stats.formatted_time(), "2m 5s");
    }

    #[test]
    fn test_recent_file() {
        let recent = RecentFile::new("/path/to/file.nc", 1000);
        assert_eq!(recent.name, "file.nc");
        assert_eq!(recent.timestamp, 1000);
    }

    #[test]
    fn test_file_operations_panel() {
        let panel = FileOperationsPanel::new();
        assert_eq!(panel.max_recent, 10);
        assert!(panel.selected_file.is_none());
    }

    #[test]
    fn test_add_recent_file() {
        let mut panel = FileOperationsPanel::new();
        panel.add_recent_file(PathBuf::from("test1.nc"));
        panel.add_recent_file(PathBuf::from("test2.nc"));
        assert_eq!(panel.recent_files.len(), 2);
    }

    #[test]
    fn test_recent_file_dedup() {
        let mut panel = FileOperationsPanel::new();
        panel.add_recent_file(PathBuf::from("test.nc"));
        panel.add_recent_file(PathBuf::from("test.nc"));
        assert_eq!(panel.recent_files.len(), 1);
    }

    #[test]
    fn test_file_size_formatting() {
        let info = FileInfo {
            path: PathBuf::from("test.nc"),
            name: "test.nc".to_string(),
            size: 2048,
            line_count: 0,
            modified: String::new(),
        };
        assert!(info.formatted_size().contains("KB"));
    }
}
