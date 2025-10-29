//! Tests for firmware::grbl::utils

use gcodekit4::firmware::grbl::utils::*;

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
