//! G-Code stream management for reading and tracking position
//!
//! This module provides:
//! - Stream reader trait for different input sources
//! - File-based stream reader for reading from disk
//! - String-based stream reader for in-memory G-Code
//! - Stream position tracking and pause/resume capabilities

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Trait for reading G-Code streams from various sources
pub trait GcodeStreamReader: Send + Sync {
    /// Read the next line from the stream
    ///
    /// Returns `Some(line)` if a line is available, `None` if at end of stream
    fn read_line(&mut self) -> Option<String>;

    /// Get the current line number (0-indexed)
    fn current_line_number(&self) -> usize;

    /// Get the total number of lines (if known)
    fn total_lines(&self) -> Option<usize>;

    /// Reset stream to beginning
    fn reset(&mut self) -> std::io::Result<()>;

    /// Seek to a specific line number
    fn seek_to_line(&mut self, line_number: usize) -> std::io::Result<()>;

    /// Check if stream is at end
    fn is_eof(&self) -> bool;
}

/// File-based G-Code stream reader
///
/// Reads G-Code from a file on disk, supporting position tracking
/// and pause/resume functionality.
pub struct FileStreamReader {
    reader: BufReader<File>,
    file_path: std::path::PathBuf,
    current_line: usize,
    total_lines: Option<usize>,
    is_eof: bool,
}

impl FileStreamReader {
    /// Create a new file stream reader
    ///
    /// # Arguments
    /// * `path` - Path to the G-Code file
    ///
    /// # Errors
    /// Returns an error if the file cannot be opened
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        // Count total lines
        let file_for_count = File::open(&path)?;
        let count_reader = BufReader::new(file_for_count);
        let total_lines = Some(count_reader.lines().count());

        Ok(Self {
            reader,
            file_path: path.as_ref().to_path_buf(),
            current_line: 0,
            total_lines,
            is_eof: false,
        })
    }

    /// Get the file path
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Get the progress as a percentage (0-100)
    pub fn progress_percent(&self) -> f64 {
        if let Some(total) = self.total_lines {
            if total == 0 {
                100.0
            } else {
                (self.current_line as f64 / total as f64) * 100.0
            }
        } else {
            0.0
        }
    }
}

impl GcodeStreamReader for FileStreamReader {
    fn read_line(&mut self) -> Option<String> {
        if self.is_eof {
            return None;
        }

        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => {
                self.is_eof = true;
                None
            }
            Ok(_) => {
                self.current_line += 1;
                Some(line)
            }
            Err(_) => {
                self.is_eof = true;
                None
            }
        }
    }

    fn current_line_number(&self) -> usize {
        self.current_line
    }

    fn total_lines(&self) -> Option<usize> {
        self.total_lines
    }

    fn reset(&mut self) -> std::io::Result<()> {
        let file = File::open(&self.file_path)?;
        self.reader = BufReader::new(file);
        self.current_line = 0;
        self.is_eof = false;
        Ok(())
    }

    fn seek_to_line(&mut self, line_number: usize) -> std::io::Result<()> {
        self.reset()?;
        let mut current = 0;
        let mut line = String::new();

        while current < line_number && self.reader.read_line(&mut line)? > 0 {
            current += 1;
            line.clear();
        }

        self.current_line = current;
        self.is_eof = current >= line_number && line_number > 0;
        Ok(())
    }

    fn is_eof(&self) -> bool {
        self.is_eof
    }
}

/// String-based G-Code stream reader
///
/// Reads G-Code from an in-memory string, supporting position tracking
/// and pause/resume functionality.
pub struct StringStreamReader {
    lines: Vec<String>,
    current_index: usize,
}

impl StringStreamReader {
    /// Create a new string stream reader
    ///
    /// # Arguments
    /// * `content` - The G-Code content as a string
    pub fn new(content: &str) -> Self {
        let lines = content
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        let total = lines.len();
        tracing::debug!("Created StringStreamReader with {} lines", total);

        Self {
            lines,
            current_index: 0,
        }
    }

    /// Get the progress as a percentage (0-100)
    pub fn progress_percent(&self) -> f64 {
        if self.lines.is_empty() {
            100.0
        } else {
            (self.current_index as f64 / self.lines.len() as f64) * 100.0
        }
    }
}

impl GcodeStreamReader for StringStreamReader {
    fn read_line(&mut self) -> Option<String> {
        if self.current_index < self.lines.len() {
            let line = self.lines[self.current_index].clone();
            self.current_index += 1;
            Some(line)
        } else {
            None
        }
    }

    fn current_line_number(&self) -> usize {
        self.current_index
    }

    fn total_lines(&self) -> Option<usize> {
        Some(self.lines.len())
    }

    fn reset(&mut self) -> std::io::Result<()> {
        self.current_index = 0;
        Ok(())
    }

    fn seek_to_line(&mut self, line_number: usize) -> std::io::Result<()> {
        if line_number <= self.lines.len() {
            self.current_index = line_number;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Line number {} exceeds total lines {}", line_number, self.lines.len()),
            ))
        }
    }

    fn is_eof(&self) -> bool {
        self.current_index >= self.lines.len()
    }
}

/// Pausable G-Code stream wrapper
///
/// Wraps any stream reader and adds pause/resume functionality.
pub struct PausableStream {
    inner: Box<dyn GcodeStreamReader>,
    is_paused: Arc<AtomicUsize>, // 0 = running, 1 = paused
    pause_line: Arc<AtomicUsize>,
}

impl PausableStream {
    /// Create a new pausable stream wrapper
    pub fn new(inner: Box<dyn GcodeStreamReader>) -> Self {
        Self {
            inner,
            is_paused: Arc::new(AtomicUsize::new(0)),
            pause_line: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Pause the stream
    pub fn pause(&self) {
        self.is_paused.store(1, Ordering::SeqCst);
        let line = self.inner.current_line_number();
        self.pause_line.store(line, Ordering::SeqCst);
        tracing::info!("Stream paused at line {}", line);
    }

    /// Resume the stream
    pub fn resume(&self) {
        self.is_paused.store(0, Ordering::SeqCst);
        tracing::info!("Stream resumed");
    }

    /// Check if stream is paused
    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst) == 1
    }

    /// Get the pause line number
    pub fn pause_line(&self) -> usize {
        self.pause_line.load(Ordering::SeqCst)
    }

    /// Get current line number
    pub fn current_line(&self) -> usize {
        self.inner.current_line_number()
    }

    /// Get total lines if known
    pub fn total_lines(&self) -> Option<usize> {
        self.inner.total_lines()
    }

    /// Check if at end of stream
    pub fn is_eof(&self) -> bool {
        self.inner.is_eof()
    }
}

impl GcodeStreamReader for PausableStream {
    fn read_line(&mut self) -> Option<String> {
        // Don't read while paused
        if self.is_paused.load(Ordering::SeqCst) == 1 {
            return None;
        }
        self.inner.read_line()
    }

    fn current_line_number(&self) -> usize {
        self.inner.current_line_number()
    }

    fn total_lines(&self) -> Option<usize> {
        self.inner.total_lines()
    }

    fn reset(&mut self) -> std::io::Result<()> {
        self.is_paused.store(0, Ordering::SeqCst);
        self.pause_line.store(0, Ordering::SeqCst);
        self.inner.reset()
    }

    fn seek_to_line(&mut self, line_number: usize) -> std::io::Result<()> {
        self.is_paused.store(0, Ordering::SeqCst);
        self.pause_line.store(0, Ordering::SeqCst);
        self.inner.seek_to_line(line_number)
    }

    fn is_eof(&self) -> bool {
        self.inner.is_eof()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_stream_reader_creation() {
        let content = "G28\nG0 X10 Y10\nG1 Z5 F100\n";
        let reader = StringStreamReader::new(content);
        assert_eq!(reader.current_index, 0);
        assert_eq!(reader.total_lines(), Some(3));
    }

    #[test]
    fn test_string_stream_read_lines() {
        let content = "G28\nG0 X10 Y10\nG1 Z5 F100\n";
        let mut reader = StringStreamReader::new(content);

        assert_eq!(reader.read_line(), Some("G28".to_string()));
        assert_eq!(reader.current_line_number(), 1);

        assert_eq!(reader.read_line(), Some("G0 X10 Y10".to_string()));
        assert_eq!(reader.current_line_number(), 2);

        assert_eq!(reader.read_line(), Some("G1 Z5 F100".to_string()));
        assert_eq!(reader.current_line_number(), 3);

        assert_eq!(reader.read_line(), None);
        assert!(reader.is_eof());
    }

    #[test]
    fn test_string_stream_reset() {
        let content = "G28\nG0 X10 Y10\n";
        let mut reader = StringStreamReader::new(content);

        reader.read_line();
        reader.read_line();
        assert_eq!(reader.current_line_number(), 2);

        reader.reset().unwrap();
        assert_eq!(reader.current_line_number(), 0);
        assert_eq!(reader.read_line(), Some("G28".to_string()));
    }

    #[test]
    fn test_string_stream_seek() {
        let content = "Line0\nLine1\nLine2\nLine3\n";
        let mut reader = StringStreamReader::new(content);

        reader.seek_to_line(2).unwrap();
        assert_eq!(reader.current_line_number(), 2);
    }

    #[test]
    fn test_string_stream_progress() {
        let content = "L0\nL1\nL2\nL3\n";
        let mut reader = StringStreamReader::new(content);

        assert_eq!(reader.progress_percent(), 0.0);
        reader.read_line();
        assert_eq!(reader.progress_percent(), 25.0);
        reader.read_line();
        assert_eq!(reader.progress_percent(), 50.0);
    }

    #[test]
    fn test_pausable_stream_pause_resume() {
        let content = "G28\nG0 X10\nG1 Z5\n";
        let reader = StringStreamReader::new(content);
        let mut pausable = PausableStream::new(Box::new(reader));

        assert!(!pausable.is_paused());
        pausable.pause();
        assert!(pausable.is_paused());
        pausable.resume();
        assert!(!pausable.is_paused());
    }

    #[test]
    fn test_pausable_stream_no_read_when_paused() {
        let content = "G28\nG0 X10\n";
        let reader = StringStreamReader::new(content);
        let mut pausable = PausableStream::new(Box::new(reader));

        pausable.read_line(); // read first line
        pausable.pause();

        // Should not read while paused
        assert_eq!(pausable.read_line(), None);
        assert_eq!(pausable.current_line(), 1);

        pausable.resume();
        // Should be able to read after resume
        assert_eq!(pausable.read_line(), Some("G0 X10".to_string()));
    }

    #[test]
    fn test_pausable_stream_reset_clears_pause() {
        let content = "G28\nG0 X10\n";
        let reader = StringStreamReader::new(content);
        let mut pausable = PausableStream::new(Box::new(reader));

        pausable.pause();
        assert!(pausable.is_paused());

        pausable.reset().unwrap();
        assert!(!pausable.is_paused());
        assert_eq!(pausable.current_line(), 0);
    }
}
