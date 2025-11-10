//! Progress indicators for file streaming and execution tracking
//!
//! Provides progress bar, time tracking, and completion statistics display.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Progress tracking for streaming operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamProgress {
    /// Total lines to send
    pub total_lines: usize,
    /// Lines sent so far
    pub lines_sent: usize,
    /// Total bytes in file
    pub total_bytes: usize,
    /// Bytes sent so far
    pub bytes_sent: usize,
    /// When streaming started
    pub start_time: Option<Duration>,
    /// Streaming start instant (for runtime tracking)
    #[serde(skip)]
    start_instant: Option<Instant>,
}

impl StreamProgress {
    /// Create new progress tracker
    pub fn new(total_lines: usize, total_bytes: usize) -> Self {
        Self {
            total_lines,
            lines_sent: 0,
            total_bytes,
            bytes_sent: 0,
            start_time: None,
            start_instant: None,
        }
    }

    /// Start tracking
    pub fn start(&mut self) {
        self.start_instant = Some(Instant::now());
    }

    /// Update progress
    pub fn update(&mut self, lines_sent: usize, bytes_sent: usize) {
        self.lines_sent = lines_sent;
        self.bytes_sent = bytes_sent;
    }

    /// Get completion percentage (0-100)
    pub fn percentage(&self) -> f32 {
        if self.total_lines == 0 {
            100.0
        } else {
            (self.lines_sent as f32 / self.total_lines as f32) * 100.0
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_instant
            .map(|inst| inst.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Estimate remaining time
    pub fn estimated_remaining(&self) -> Option<Duration> {
        let elapsed = self.elapsed();
        if elapsed.is_zero() || self.lines_sent == 0 {
            return None;
        }

        let rate = self.lines_sent as f64 / elapsed.as_secs_f64();
        if rate > 0.0 {
            let remaining_lines = self.total_lines - self.lines_sent;
            let remaining_secs = remaining_lines as f64 / rate;
            Some(Duration::from_secs_f64(remaining_secs))
        } else {
            None
        }
    }

    /// Get lines remaining
    pub fn lines_remaining(&self) -> usize {
        self.total_lines.saturating_sub(self.lines_sent)
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.lines_sent >= self.total_lines && self.total_lines > 0
    }

    /// Reset progress
    pub fn reset(&mut self) {
        self.lines_sent = 0;
        self.bytes_sent = 0;
        self.start_time = None;
        self.start_instant = None;
    }
}

/// Progress display information
#[derive(Debug, Clone)]
pub struct ProgressDisplay {
    /// Percentage complete (0-100)
    pub percentage: f32,
    /// Formatted elapsed time
    pub elapsed_str: String,
    /// Formatted remaining time
    pub remaining_str: String,
    /// Lines sent / total lines
    pub lines_display: String,
    /// Status text
    pub status: String,
}

impl ProgressDisplay {
    /// Create from StreamProgress
    pub fn from_progress(progress: &StreamProgress) -> Self {
        let elapsed = progress.elapsed();
        let elapsed_str = format_duration(elapsed);

        let remaining_str = progress
            .estimated_remaining()
            .map(format_duration)
            .unwrap_or_else(|| "-- :-- ".to_string());

        let lines_display = format!("{} / {}", progress.lines_sent, progress.total_lines);

        let status = if progress.is_complete() {
            "Complete".to_string()
        } else if progress.lines_sent == 0 {
            "Ready".to_string()
        } else {
            "Streaming".to_string()
        };

        Self {
            percentage: progress.percentage(),
            elapsed_str,
            remaining_str,
            lines_display,
            status,
        }
    }
}

/// Format duration as HH:MM:SS
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_creation() {
        let progress = StreamProgress::new(100, 5000);
        assert_eq!(progress.total_lines, 100);
        assert_eq!(progress.total_bytes, 5000);
        assert_eq!(progress.lines_sent, 0);
        assert_eq!(progress.percentage(), 0.0);
    }

    #[test]
    fn test_progress_percentage() {
        let mut progress = StreamProgress::new(100, 5000);
        progress.update(50, 2500);
        assert_eq!(progress.percentage(), 50.0);
    }

    #[test]
    fn test_progress_display() {
        let progress = StreamProgress::new(100, 5000);
        let display = ProgressDisplay::from_progress(&progress);
        assert_eq!(display.percentage, 0.0);
        assert_eq!(display.status, "Ready");
    }

    #[test]
    fn test_progress_lines_remaining() {
        let mut progress = StreamProgress::new(100, 5000);
        progress.update(30, 1500);
        assert_eq!(progress.lines_remaining(), 70);
    }

    #[test]
    fn test_progress_completion() {
        let mut progress = StreamProgress::new(100, 5000);
        progress.update(100, 5000);
        assert!(progress.is_complete());
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0)), "00:00:00");
        assert_eq!(format_duration(Duration::from_secs(65)), "00:01:05");
        assert_eq!(format_duration(Duration::from_secs(3661)), "01:01:01");
    }
}
