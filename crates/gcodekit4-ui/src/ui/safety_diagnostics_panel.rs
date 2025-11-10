//! Safety & Diagnostics UI Panel - Phase 7 Features
//!
//! Displays safety status and diagnostic information including:
//! - Task 121: Emergency stop, motion interlock, feed hold
//! - Task 125: Communication/buffer/performance diagnostics

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Emergency stop state display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergencyStopDisplay {
    /// Armed and ready
    Armed,
    /// Triggered - emergency stop active
    Triggered,
    /// Resetting
    Resetting,
    /// Stopped and locked
    Stopped,
}

impl std::fmt::Display for EmergencyStopDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmergencyStopDisplay::Armed => write!(f, "ARMED"),
            EmergencyStopDisplay::Triggered => write!(f, "TRIGGERED"),
            EmergencyStopDisplay::Resetting => write!(f, "RESETTING"),
            EmergencyStopDisplay::Stopped => write!(f, "STOPPED"),
        }
    }
}

/// Motion interlock state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionInterlockState {
    /// Motion allowed
    Enabled,
    /// Motion blocked
    Disabled,
    /// Waiting for user action
    Waiting,
}

/// Feed hold state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedHoldState {
    /// Normal operation
    Normal,
    /// Feed hold active
    Held,
    /// Resuming
    Resuming,
}

/// Communication diagnostic info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationDiagnostics {
    /// Total bytes sent
    pub total_bytes_sent: u64,
    /// Total bytes received
    pub total_bytes_received: u64,
    /// Current baud rate
    pub baud_rate: u32,
    /// Port name
    pub port_name: String,
    /// Connection status
    pub connected: bool,
    /// Last error (if any)
    pub last_error: Option<String>,
    /// Errors in last minute
    pub error_count_1min: u32,
}

impl Default for CommunicationDiagnostics {
    fn default() -> Self {
        Self {
            total_bytes_sent: 0,
            total_bytes_received: 0,
            baud_rate: 115200,
            port_name: String::new(),
            connected: false,
            last_error: None,
            error_count_1min: 0,
        }
    }
}

/// Buffer diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferDiagnostics {
    /// Total buffer size
    pub total_buffer_size: usize,
    /// Used buffer size
    pub used_buffer_size: usize,
    /// Buffer usage percentage
    pub usage_percent: f64,
    /// Commands in buffer
    pub commands_in_buffer: usize,
    /// Peak usage percentage
    pub peak_usage_percent: f64,
    /// Buffer overflow count
    pub overflow_count: u32,
}

impl Default for BufferDiagnostics {
    fn default() -> Self {
        Self {
            total_buffer_size: 128,
            used_buffer_size: 0,
            usage_percent: 0.0,
            commands_in_buffer: 0,
            peak_usage_percent: 0.0,
            overflow_count: 0,
        }
    }
}

/// Performance diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDiagnostics {
    /// Commands per second
    pub commands_per_second: f64,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
    /// Max latency (ms)
    pub max_latency_ms: f64,
    /// Memory usage (MB)
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Uptime (seconds)
    pub uptime_seconds: u64,
}

impl Default for PerformanceDiagnostics {
    fn default() -> Self {
        Self {
            commands_per_second: 0.0,
            avg_latency_ms: 0.0,
            max_latency_ms: 0.0,
            memory_usage_mb: 0.0,
            cpu_percent: 0.0,
            uptime_seconds: 0,
        }
    }
}

/// Diagnostic event log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticEvent {
    /// Event timestamp (Unix seconds)
    pub timestamp: u64,
    /// Event level (INFO, WARN, ERROR)
    pub level: String,
    /// Event message
    pub message: String,
    /// Additional details
    pub details: Option<String>,
}

/// Safety and Diagnostics panel state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyDiagnosticsPanel {
    /// Emergency stop state
    pub estop_state: EmergencyStopDisplay,
    /// Motion interlock state
    pub motion_interlock: MotionInterlockState,
    /// Feed hold state
    pub feed_hold_state: FeedHoldState,
    /// Whether machine is in alarm state
    pub alarm_active: bool,
    /// Alarm description
    pub alarm_description: Option<String>,
    /// Communication diagnostics
    pub comm_diagnostics: CommunicationDiagnostics,
    /// Buffer diagnostics
    pub buffer_diagnostics: BufferDiagnostics,
    /// Performance diagnostics
    pub perf_diagnostics: PerformanceDiagnostics,
    /// Event log (keep last 100 events)
    pub event_log: VecDeque<DiagnosticEvent>,
    /// Max events to keep in log
    max_log_events: usize,
    /// Diagnostic mode enabled
    pub diagnostic_mode: bool,
}

impl SafetyDiagnosticsPanel {
    /// Create new safety diagnostics panel
    pub fn new() -> Self {
        Self {
            estop_state: EmergencyStopDisplay::Armed,
            motion_interlock: MotionInterlockState::Enabled,
            feed_hold_state: FeedHoldState::Normal,
            alarm_active: false,
            alarm_description: None,
            comm_diagnostics: CommunicationDiagnostics::default(),
            buffer_diagnostics: BufferDiagnostics::default(),
            perf_diagnostics: PerformanceDiagnostics::default(),
            event_log: VecDeque::new(),
            max_log_events: 100,
            diagnostic_mode: false,
        }
    }

    /// Trigger emergency stop
    pub fn trigger_estop(&mut self) {
        self.estop_state = EmergencyStopDisplay::Triggered;
        self.motion_interlock = MotionInterlockState::Disabled;
        self.log_event("WARN", "Emergency stop triggered");
    }

    /// Reset emergency stop
    pub fn reset_estop(&mut self) {
        self.estop_state = EmergencyStopDisplay::Resetting;
        self.log_event("INFO", "Emergency stop resetting");
    }

    /// Complete emergency stop reset
    pub fn complete_estop_reset(&mut self) {
        self.estop_state = EmergencyStopDisplay::Armed;
        self.motion_interlock = MotionInterlockState::Enabled;
        self.log_event("INFO", "Emergency stop reset complete");
    }

    /// Activate feed hold
    pub fn activate_feed_hold(&mut self) {
        self.feed_hold_state = FeedHoldState::Held;
        self.log_event("INFO", "Feed hold activated");
    }

    /// Resume feed
    pub fn resume_feed(&mut self) {
        self.feed_hold_state = FeedHoldState::Resuming;
        self.log_event("INFO", "Feed resuming");
    }

    /// Complete feed resume
    pub fn complete_feed_resume(&mut self) {
        self.feed_hold_state = FeedHoldState::Normal;
        self.log_event("INFO", "Feed resumed");
    }

    /// Set alarm state
    pub fn set_alarm(&mut self, description: String) {
        self.alarm_active = true;
        self.alarm_description = Some(description.clone());
        self.motion_interlock = MotionInterlockState::Disabled;
        self.log_event("ERROR", &format!("Alarm: {}", description));
    }

    /// Clear alarm
    pub fn clear_alarm(&mut self) {
        if self.alarm_active {
            self.alarm_active = false;
            self.alarm_description = None;
            self.log_event("INFO", "Alarm cleared");
        }
    }

    /// Update communication diagnostics
    pub fn update_comm_diagnostics(&mut self, diagnostics: CommunicationDiagnostics) {
        if let Some(ref error) = diagnostics.last_error {
            if self.comm_diagnostics.last_error.as_ref() != diagnostics.last_error.as_ref() {
                self.log_event("WARN", &format!("Comm error: {}", error));
            }
        }
        self.comm_diagnostics = diagnostics;
    }

    /// Update buffer diagnostics
    pub fn update_buffer_diagnostics(&mut self, diagnostics: BufferDiagnostics) {
        if diagnostics.overflow_count > self.buffer_diagnostics.overflow_count {
            self.log_event("ERROR", "Buffer overflow detected");
        }
        self.buffer_diagnostics = diagnostics;
    }

    /// Update performance diagnostics
    pub fn update_perf_diagnostics(&mut self, diagnostics: PerformanceDiagnostics) {
        self.perf_diagnostics = diagnostics;
    }

    /// Add event to log
    pub fn log_event(&mut self, level: &str, message: &str) {
        let event = DiagnosticEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            level: level.to_string(),
            message: message.to_string(),
            details: None,
        };

        self.event_log.push_back(event);
        while self.event_log.len() > self.max_log_events {
            self.event_log.pop_front();
        }
    }

    /// Get diagnostics summary
    pub fn get_summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&"Safety Status:\n".to_string());
        summary.push_str(&format!("  E-Stop: {}\n", self.estop_state));
        summary.push_str(&format!(
            "  Motion: {}\n",
            match self.motion_interlock {
                MotionInterlockState::Enabled => "ENABLED",
                MotionInterlockState::Disabled => "DISABLED",
                MotionInterlockState::Waiting => "WAITING",
            }
        ));
        summary.push_str(&format!(
            "  Feed Hold: {}\n",
            match self.feed_hold_state {
                FeedHoldState::Normal => "NORMAL",
                FeedHoldState::Held => "HELD",
                FeedHoldState::Resuming => "RESUMING",
            }
        ));

        summary.push_str(&"\nDiagnostics:\n".to_string());
        summary.push_str(&format!(
            "  Comm: {} ({}% errors in 1min)\n",
            if self.comm_diagnostics.connected {
                "Connected"
            } else {
                "Disconnected"
            },
            self.comm_diagnostics.error_count_1min
        ));
        summary.push_str(&format!(
            "  Buffer: {:.1}% used\n",
            self.buffer_diagnostics.usage_percent
        ));
        summary.push_str(&format!(
            "  Performance: {:.1} cmd/s\n",
            self.perf_diagnostics.commands_per_second
        ));

        summary
    }
}

impl Default for SafetyDiagnosticsPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_diagnostics_new() {
        let panel = SafetyDiagnosticsPanel::new();
        assert_eq!(panel.estop_state, EmergencyStopDisplay::Armed);
        assert_eq!(panel.motion_interlock, MotionInterlockState::Enabled);
        assert!(!panel.alarm_active);
    }

    #[test]
    fn test_trigger_estop() {
        let mut panel = SafetyDiagnosticsPanel::new();
        panel.trigger_estop();
        assert_eq!(panel.estop_state, EmergencyStopDisplay::Triggered);
        assert_eq!(panel.motion_interlock, MotionInterlockState::Disabled);
    }

    #[test]
    fn test_feed_hold_cycle() {
        let mut panel = SafetyDiagnosticsPanel::new();
        panel.activate_feed_hold();
        assert_eq!(panel.feed_hold_state, FeedHoldState::Held);

        panel.resume_feed();
        assert_eq!(panel.feed_hold_state, FeedHoldState::Resuming);

        panel.complete_feed_resume();
        assert_eq!(panel.feed_hold_state, FeedHoldState::Normal);
    }

    #[test]
    fn test_alarm_management() {
        let mut panel = SafetyDiagnosticsPanel::new();
        panel.set_alarm("Test alarm".to_string());
        assert!(panel.alarm_active);
        assert_eq!(panel.motion_interlock, MotionInterlockState::Disabled);

        panel.clear_alarm();
        assert!(!panel.alarm_active);
    }

    #[test]
    fn test_event_logging() {
        let mut panel = SafetyDiagnosticsPanel::new();
        panel.log_event("INFO", "Test event");
        assert_eq!(panel.event_log.len(), 1);
        assert_eq!(panel.event_log[0].level, "INFO");
    }

    #[test]
    fn test_event_log_max_size() {
        let mut panel = SafetyDiagnosticsPanel::new();
        for i in 0..150 {
            panel.log_event("INFO", &format!("Event {}", i));
        }
        assert_eq!(panel.event_log.len(), 100);
    }

    #[test]
    fn test_get_summary() {
        let panel = SafetyDiagnosticsPanel::new();
        let summary = panel.get_summary();
        assert!(summary.contains("Safety Status"));
        assert!(summary.contains("ARMED"));
    }

    #[test]
    fn test_communication_error_logging() {
        let mut panel = SafetyDiagnosticsPanel::new();
        let mut diag = CommunicationDiagnostics::default();
        diag.last_error = Some("Connection timeout".to_string());

        panel.update_comm_diagnostics(diag.clone());
        assert_eq!(panel.event_log.len(), 1);
    }

    #[test]
    fn test_buffer_overflow_detection() {
        let mut panel = SafetyDiagnosticsPanel::new();
        let mut diag = BufferDiagnostics::default();
        diag.overflow_count = 1;

        panel.update_buffer_diagnostics(diag);
        assert_eq!(panel.event_log.len(), 1);
        assert!(panel.event_log[0].message.contains("Buffer overflow"));
    }
}
