//! TinyG Protocol Support Tests
//!
//! Tests for TinyG protocol implementation including response parsing,
//! command creation, and utilities.

use gcodekit4::firmware::tinyg::{
    TinyGVersion, TinyGCapabilities, CommandCreator, MotionType, RealTimeCommand,
    TinyGResponseParser, TinyGResponseType,
};

#[test]
fn test_tinyg_version_parse() {
    let version = TinyGVersion::parse("440.20").unwrap();
    assert_eq!(version.major, 440);
    assert_eq!(version.minor, 20);
    assert_eq!(version.build, None);
}

#[test]
fn test_tinyg_version_parse_with_build() {
    let version = TinyGVersion::parse("440.20.10").unwrap();
    assert_eq!(version.major, 440);
    assert_eq!(version.minor, 20);
    assert_eq!(version.build, Some(10));
}

#[test]
fn test_tinyg_version_display() {
    let version = TinyGVersion::new(440, 20);
    assert_eq!(version.to_string(), "440.20");
}

#[test]
fn test_tinyg_version_comparison() {
    let v1 = TinyGVersion::new(440, 20);
    let v2 = TinyGVersion::new(440, 19);
    assert!(v1 > v2);
}

#[test]
fn test_tinyg_capabilities_minimum_version() {
    let caps = TinyGCapabilities::for_version(TinyGVersion::new(440, 0));
    assert!(caps.is_minimum_version());
}

#[test]
fn test_tinyg_command_creator_gcode() {
    let mut creator = CommandCreator::new();
    let cmd = creator.create_gcode_command("G0 X10 Y20");
    assert!(cmd.contains("N1"));
    assert!(cmd.contains("G0 X10 Y20"));
}

#[test]
fn test_tinyg_command_creator_status_query() {
    let creator = CommandCreator::new();
    let query = creator.create_status_query();
    assert_eq!(query, r#"{"sr":null}"#);
}

#[test]
fn test_tinyg_command_creator_motion_command() {
    let mut creator = CommandCreator::new();
    let cmd = creator.create_motion_command(
        MotionType::Linear,
        Some(10.0),
        Some(20.0),
        None,
        None,
        Some(1000.0),
    );
    assert!(cmd.contains("G1"));
    assert!(cmd.contains("X10"));
    assert!(cmd.contains("Y20"));
    assert!(cmd.contains("F1000"));
}

#[test]
fn test_tinyg_command_creator_jog_command() {
    let mut creator = CommandCreator::new();
    let cmd = creator.create_jog_command(Some(5.0), Some(5.0), None, 500.0);
    assert!(cmd.contains("G91"));
    assert!(cmd.contains("X5"));
    assert!(cmd.contains("Y5"));
    assert!(cmd.contains("F500"));
}

#[test]
fn test_tinyg_command_creator_spindle_command() {
    let mut creator = CommandCreator::new();
    let cmd = creator.create_spindle_command(true, Some(1000));
    assert!(cmd.contains("M3"));
    assert!(cmd.contains("S1000"));
}

#[test]
fn test_tinyg_command_creator_home_command() {
    let mut creator = CommandCreator::new();
    let cmd = creator.create_home_command(true);
    assert!(cmd.contains("G28.2"));
}

#[test]
fn test_tinyg_command_creator_line_number_increment() {
    let mut creator = CommandCreator::new();
    creator.create_gcode_command("G0 X0");
    assert_eq!(creator.get_line_number(), 1);
    creator.create_gcode_command("G0 Y0");
    assert_eq!(creator.get_line_number(), 2);
}

#[test]
fn test_tinyg_command_creator_reset_line_number() {
    let mut creator = CommandCreator::new();
    creator.create_gcode_command("G0 X0");
    creator.reset_line_number();
    assert_eq!(creator.get_line_number(), 0);
}

#[test]
fn test_tinyg_realtime_command_as_byte() {
    assert_eq!(RealTimeCommand::StatusRequest.as_byte(), b'?');
    assert_eq!(RealTimeCommand::FeedHold.as_byte(), b'!');
    assert_eq!(RealTimeCommand::CycleStart.as_byte(), b'~');
    assert_eq!(RealTimeCommand::SoftReset.as_byte(), 0x18);
}

#[test]
fn test_tinyg_response_parser_ok() {
    let mut parser = TinyGResponseParser::new();
    let response = r#"{"ok":true}"#;
    let result = parser.parse(response);
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.response_type, TinyGResponseType::Ok);
    assert!(resp.is_success());
}

#[test]
fn test_tinyg_response_parser_error() {
    let mut parser = TinyGResponseParser::new();
    let response = r#"{"er":{"code":1,"msg":"Generic error"}}"#;
    let result = parser.parse(response);
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert!(resp.is_error());
    assert_eq!(resp.error_code, Some(1));
}

#[test]
fn test_tinyg_response_parser_status_report() {
    let mut parser = TinyGResponseParser::new();
    let response = r#"{"sr":{"stat":{"state":"Idle"},"pos":{"x":0,"y":0,"z":0,"a":0},"mpos":{"x":0,"y":0,"z":0,"a":0},"feed":0,"speed":0,"unit":0,"line":0}}"#;
    let result = parser.parse(response);
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert!(resp.is_status_report());
}

#[test]
fn test_tinyg_utils_parse_json() {
    use gcodekit4::firmware::tinyg::utils;
    let response = r#"{"sr":{"stat":1,"pos":{"x":10.5,"y":20.3,"z":5.0,"a":0.0}}}"#;
    let result = utils::parse_json_response(response);
    assert!(result.is_ok());
}

#[test]
fn test_tinyg_utils_extract_position() {
    use gcodekit4::firmware::tinyg::utils;
    use serde_json::json;
    let sr = json!({"pos":{"x":10.5,"y":20.3,"z":5.0,"a":0.0}});
    let result = utils::extract_position(&sr);
    assert!(result.is_some());
    let (x, y, z, a) = result.unwrap();
    assert_eq!(x, 10.5);
    assert_eq!(y, 20.3);
    assert_eq!(z, 5.0);
    assert_eq!(a, 0.0);
}
