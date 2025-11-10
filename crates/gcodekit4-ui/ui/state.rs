//! UI State Management - Task 66
//!
//! Manages application state and data flow through UI components

use std::collections::HashMap;

/// Application UI state
#[derive(Debug, Clone)]
pub struct UiState {
    /// Connection state
    pub connection_state: ConnectionState,
    /// Controller state (DRO)
    pub controller_state: ControllerState,
    /// File state
    pub file_state: FileState,
    /// Machine state
    pub machine_state: MachineState,
    /// Settings
    pub settings: HashMap<String, String>,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Disconnected
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Connection error
    Error,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Connected => write!(f, "Connected"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Controller state (Digital Readout)
#[derive(Debug, Clone, Default)]
pub struct ControllerState {
    /// Current X position
    pub position_x: f32,
    /// Current Y position
    pub position_y: f32,
    /// Current Z position
    pub position_z: f32,
    /// Feed rate
    pub feed_rate: f32,
    /// Spindle speed
    pub spindle_speed: u16,
    /// Machine running
    pub is_running: bool,
}

/// File state
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct FileState {
    /// Current file path
    pub current_file: Option<String>,
    /// File loaded
    pub file_loaded: bool,
    /// Total lines in file
    pub total_lines: u32,
    /// Current line
    pub current_line: u32,
    /// File is dirty (unsaved)
    pub is_dirty: bool,
}


/// Machine state
#[derive(Debug, Clone, Default)]
pub struct MachineState {
    /// Machine is powered on
    pub powered_on: bool,
    /// Machine is homed
    pub is_homed: bool,
    /// Machine alarm state
    pub alarm: Option<String>,
    /// Last error
    pub last_error: Option<String>,
    /// Probe state
    pub probing: bool,
}

impl UiState {
    /// Create new UI state
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            controller_state: ControllerState::default(),
            file_state: FileState::default(),
            machine_state: MachineState::default(),
            settings: HashMap::new(),
        }
    }

    /// Set connection state
    pub fn set_connection_state(&mut self, state: ConnectionState) {
        self.connection_state = state;
    }

    /// Update controller position
    pub fn update_position(&mut self, x: f32, y: f32, z: f32) {
        self.controller_state.position_x = x;
        self.controller_state.position_y = y;
        self.controller_state.position_z = z;
    }

    /// Update feed rate
    pub fn update_feed_rate(&mut self, rate: f32) {
        self.controller_state.feed_rate = rate;
    }

    /// Update spindle speed
    pub fn update_spindle_speed(&mut self, speed: u16) {
        self.controller_state.spindle_speed = speed;
    }

    /// Set machine running state
    pub fn set_running(&mut self, running: bool) {
        self.controller_state.is_running = running;
    }

    /// Load file
    pub fn load_file(&mut self, path: String, total_lines: u32) {
        self.file_state.current_file = Some(path);
        self.file_state.file_loaded = true;
        self.file_state.total_lines = total_lines;
        self.file_state.current_line = 0;
    }

    /// Update current line
    pub fn set_current_line(&mut self, line: u32) {
        self.file_state.current_line = line;
    }

    /// Mark file as dirty
    pub fn mark_file_dirty(&mut self) {
        self.file_state.is_dirty = true;
    }

    /// Mark file as clean
    pub fn mark_file_clean(&mut self) {
        self.file_state.is_dirty = false;
    }

    /// Power on machine
    pub fn power_on(&mut self) {
        self.machine_state.powered_on = true;
    }

    /// Power off machine
    pub fn power_off(&mut self) {
        self.machine_state.powered_on = false;
    }

    /// Home machine
    pub fn home_machine(&mut self) {
        self.machine_state.is_homed = true;
    }

    /// Set alarm
    pub fn set_alarm(&mut self, alarm: Option<String>) {
        self.machine_state.alarm = alarm;
    }

    /// Set error
    pub fn set_error(&mut self, error: Option<String>) {
        self.machine_state.last_error = error;
    }

    /// Get setting
    pub fn get_setting(&self, key: &str) -> Option<&str> {
        self.settings.get(key).map(|s| s.as_str())
    }

    /// Set setting
    pub fn set_setting(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.settings.insert(key.into(), value.into());
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_state_creation() {
        let state = UiState::new();
        assert_eq!(state.connection_state, ConnectionState::Disconnected);
        assert!(!state.file_state.file_loaded);
    }

    #[test]
    fn test_connection_state() {
        let mut state = UiState::new();
        state.set_connection_state(ConnectionState::Connected);
        assert_eq!(state.connection_state, ConnectionState::Connected);
    }

    #[test]
    fn test_position_update() {
        let mut state = UiState::new();
        state.update_position(10.0, 20.0, 5.0);
        assert_eq!(state.controller_state.position_x, 10.0);
        assert_eq!(state.controller_state.position_y, 20.0);
        assert_eq!(state.controller_state.position_z, 5.0);
    }

    #[test]
    fn test_file_state() {
        let mut state = UiState::new();
        state.load_file("test.gcode".to_string(), 100);
        assert!(state.file_state.file_loaded);
        assert_eq!(state.file_state.total_lines, 100);
    }

    #[test]
    fn test_settings() {
        let mut state = UiState::new();
        state.set_setting("theme", "dark");
        assert_eq!(state.get_setting("theme"), Some("dark"));
    }
}
