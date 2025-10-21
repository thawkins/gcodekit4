//! GRBL Protocol Utilities
//!
//! This module provides utility functions for working with GRBL including
//! response validation, command formatting, and state lookups.

use crate::gcode::GcodeCommand;
use std::collections::HashMap;

/// Validates a GRBL response line
pub fn is_valid_response(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }

    let trimmed = line.trim();

    // OK
    if trimmed == "ok" {
        return true;
    }

    // Error responses
    if trimmed.starts_with("error:") {
        let error_code = trimmed[6..].trim();
        return error_code.parse::<u8>().is_ok();
    }

    // Alarm responses
    if trimmed.starts_with("alarm:") {
        let alarm_code = trimmed[6..].trim();
        return alarm_code.parse::<u8>().is_ok();
    }

    // Status reports
    if trimmed.starts_with('<') && trimmed.ends_with('>') {
        return trimmed.contains('|');
    }

    // Settings
    if trimmed.starts_with('$') && trimmed.contains('=') {
        return true;
    }

    // Version strings
    if trimmed.starts_with("Grbl ") {
        return true;
    }

    // Build info
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return true;
    }

    // Accept as a generic message if it contains recognizable GRBL markers
    false
}

/// Format a G-code command for GRBL transmission
pub fn format_command(cmd: &GcodeCommand) -> String {
    format!("{}\n", cmd.command)
}

/// Get the human-readable state name from GRBL state string
pub fn get_state_name(state: &str) -> &'static str {
    match state {
        "Idle" => "Idle",
        "Run" => "Running",
        "Hold" => "Hold",
        "Jog" => "Jogging",
        "Alarm" => "Alarm",
        "Check" => "Check",
        "Door" => "Door",
        "Sleep" => "Sleep",
        _ => "Unknown",
    }
}

/// Check if GRBL is in an error state
pub fn is_error_state(state: &str) -> bool {
    matches!(state, "Alarm" | "Check" | "Door")
}

/// Check if GRBL is running
pub fn is_running_state(state: &str) -> bool {
    matches!(state, "Run" | "Jog")
}

/// Check if GRBL is idle
pub fn is_idle_state(state: &str) -> bool {
    state == "Idle"
}

/// Check if GRBL is held/paused
pub fn is_held_state(state: &str) -> bool {
    state == "Hold"
}

/// Get error code lookup map
pub fn get_error_codes() -> HashMap<u8, &'static str> {
    let mut map = HashMap::new();
    map.insert(1, "Expected command letter");
    map.insert(2, "Bad number format");
    map.insert(3, "Invalid statement");
    map.insert(4, "Negative value");
    map.insert(5, "Setting disabled");
    map.insert(20, "Unsupported or invalid g-code command");
    map.insert(21, "Modal group violation");
    map.insert(22, "Undefined feed rate");
    map.insert(23, "Failed to execute startup block");
    map.insert(24, "EEPROM read failed");
    map
}

/// Get alarm code lookup map
pub fn get_alarm_codes() -> HashMap<u8, &'static str> {
    let mut map = HashMap::new();
    map.insert(1, "Hard limit triggered");
    map.insert(2, "Soft limit exceeded");
    map.insert(3, "Abort during cycle");
    map.insert(4, "Probe fail");
    map.insert(5, "Probe not triggered");
    map.insert(6, "Homing fail");
    map.insert(7, "Homing fail pulloff");
    map.insert(8, "Spindle control failure");
    map.insert(9, "Cooling mist control failure");
    map
}

/// Get setting name from setting number
pub fn get_setting_name(setting_num: u8) -> &'static str {
    match setting_num {
        110 => "X max rate",
        111 => "Y max rate",
        112 => "Z max rate",
        113 => "X accel",
        114 => "Y accel",
        115 => "Z accel",
        120 => "X max travel",
        121 => "Y max travel",
        122 => "Z max travel",
        130 => "Step pulse duration",
        131 => "Step idle delay",
        132 => "Step port invert mask",
        133 => "Direction port invert mask",
        134 => "Step enable invert",
        135 => "Limit pins invert",
        136 => "Probe pin invert",
        140 => "Status report mask",
        160 => "Junction deviation",
        161 => "Arc tolerance",
        162 => "Report inches",
        _ => "Unknown setting",
    }
}

/// Format a position value with appropriate precision
pub fn format_position(value: f64) -> String {
    format!("{:.3}", value)
}

/// Format multiple positions
pub fn format_positions(x: f64, y: f64, z: f64) -> String {
    format!(
        "{},{},{}",
        format_position(x),
        format_position(y),
        format_position(z)
    )
}

/// Parse GRBL settings string response into number and value
pub fn parse_setting_response(line: &str) -> Option<(u8, String)> {
    if !line.starts_with('$') {
        return None;
    }

    let line = &line[1..];
    let parts: Vec<&str> = line.split('=').collect();

    if parts.len() != 2 {
        return None;
    }

    let setting_num = parts[0].trim().parse::<u8>().ok()?;
    let value = parts[1].trim().to_string();

    Some((setting_num, value))
}

/// Get buffer status as human-readable string
pub fn format_buffer_state(plan: u8, rx: u8) -> String {
    format!("Plan: {}/127, RX: {}/256", plan, rx)
}

/// Check if a response indicates command accepted
pub fn is_command_accepted(response: &str) -> bool {
    response.trim() == "ok"
}

/// Check if a response indicates an error
pub fn is_command_error(response: &str) -> bool {
    let trimmed = response.trim();
    trimmed.starts_with("error:") || trimmed.starts_with("alarm:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_response_ok() {
        assert!(is_valid_response("ok"));
        assert!(is_valid_response("  ok  "));
    }

    #[test]
    fn test_is_valid_response_error() {
        assert!(is_valid_response("error:1"));
        assert!(is_valid_response("error:23"));
    }

    #[test]
    fn test_is_valid_response_alarm() {
        assert!(is_valid_response("alarm:1"));
        assert!(is_valid_response("alarm:6"));
    }

    #[test]
    fn test_is_valid_response_status() {
        assert!(is_valid_response("<Idle|MPos:0,0,0|WPos:0,0,0>"));
        assert!(is_valid_response("<Run|MPos:10,20,30|WPos:10,20,30>"));
    }

    #[test]
    fn test_is_valid_response_setting() {
        assert!(is_valid_response("$110=1000"));
    }

    #[test]
    fn test_is_valid_response_version() {
        assert!(is_valid_response("Grbl 1.1h"));
    }

    #[test]
    fn test_is_valid_response_invalid() {
        assert!(!is_valid_response(""));
        assert!(!is_valid_response("invalid"));
    }

    #[test]
    fn test_get_state_name() {
        assert_eq!(get_state_name("Idle"), "Idle");
        assert_eq!(get_state_name("Run"), "Running");
        assert_eq!(get_state_name("Hold"), "Hold");
        assert_eq!(get_state_name("Jog"), "Jogging");
        assert_eq!(get_state_name("Alarm"), "Alarm");
        assert_eq!(get_state_name("Unknown"), "Unknown");
    }

    #[test]
    fn test_is_error_state() {
        assert!(is_error_state("Alarm"));
        assert!(is_error_state("Check"));
        assert!(is_error_state("Door"));
        assert!(!is_error_state("Idle"));
        assert!(!is_error_state("Run"));
    }

    #[test]
    fn test_is_running_state() {
        assert!(is_running_state("Run"));
        assert!(is_running_state("Jog"));
        assert!(!is_running_state("Idle"));
        assert!(!is_running_state("Alarm"));
    }

    #[test]
    fn test_is_idle_state() {
        assert!(is_idle_state("Idle"));
        assert!(!is_idle_state("Run"));
        assert!(!is_idle_state("Hold"));
    }

    #[test]
    fn test_is_held_state() {
        assert!(is_held_state("Hold"));
        assert!(!is_held_state("Idle"));
        assert!(!is_held_state("Run"));
    }

    #[test]
    fn test_get_error_codes() {
        let codes = get_error_codes();
        assert_eq!(codes.get(&1), Some(&"Expected command letter"));
        assert_eq!(codes.get(&23), Some(&"Failed to execute startup block"));
        assert!(codes.get(&99).is_none());
    }

    #[test]
    fn test_get_alarm_codes() {
        let codes = get_alarm_codes();
        assert_eq!(codes.get(&1), Some(&"Hard limit triggered"));
        assert_eq!(codes.get(&6), Some(&"Homing fail"));
        assert!(codes.get(&99).is_none());
    }

    #[test]
    fn test_get_setting_name() {
        assert_eq!(get_setting_name(110), "X max rate");
        assert_eq!(get_setting_name(160), "Junction deviation");
        assert_eq!(get_setting_name(200), "Unknown setting");
    }

    #[test]
    fn test_format_position() {
        assert_eq!(format_position(10.1234), "10.123");
        assert_eq!(format_position(0.0), "0.000");
    }

    #[test]
    fn test_format_positions() {
        let result = format_positions(10.0, 20.0, 30.0);
        assert_eq!(result, "10.000,20.000,30.000");
    }

    #[test]
    fn test_parse_setting_response() {
        let (num, val) = parse_setting_response("$110=1000.000").unwrap();
        assert_eq!(num, 110);
        assert_eq!(val, "1000.000");
    }

    #[test]
    fn test_parse_setting_response_invalid() {
        assert!(parse_setting_response("invalid").is_none());
        assert!(parse_setting_response("110=1000").is_none());
    }

    #[test]
    fn test_format_buffer_state() {
        let result = format_buffer_state(10, 100);
        assert!(result.contains("Plan: 10"));
        assert!(result.contains("RX: 100"));
    }

    #[test]
    fn test_is_command_accepted() {
        assert!(is_command_accepted("ok"));
        assert!(is_command_accepted("  ok  "));
        assert!(!is_command_accepted("error:1"));
    }

    #[test]
    fn test_is_command_error() {
        assert!(is_command_error("error:1"));
        assert!(is_command_error("alarm:2"));
        assert!(!is_command_error("ok"));
    }
}
