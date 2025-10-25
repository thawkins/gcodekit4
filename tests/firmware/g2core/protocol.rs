//! g2core Protocol Support Tests
//!
//! Tests for g2core protocol implementation including response parsing,
//! command creation with 6-axis support, and advanced features.

use gcodekit4::firmware::g2core::{
    CommandCreator, G2CoreCapabilities, G2CoreResponseParser, G2CoreResponseType, G2CoreVersion,
    KinematicMode, MotionType, RealTimeCommand,
};

#[test]
fn test_g2core_version_parse() {
    let version = G2CoreVersion::parse("100.00").unwrap();
    assert_eq!(version.major, 100);
    assert_eq!(version.minor, 0);
    assert_eq!(version.build, None);
}

#[test]
fn test_g2core_version_parse_with_build() {
    let version = G2CoreVersion::parse("100.10.05").unwrap();
    assert_eq!(version.major, 100);
    assert_eq!(version.minor, 10);
    assert_eq!(version.build, Some(5));
}

#[test]
fn test_g2core_version_display() {
    let version = G2CoreVersion::new(100, 10);
    assert_eq!(version.to_string(), "100.10");
}

#[test]
fn test_g2core_version_comparison() {
    let v1 = G2CoreVersion::new(100, 10);
    let v2 = G2CoreVersion::new(100, 5);
    assert!(v1 > v2);
}

#[test]
fn test_g2core_capabilities_minimum_version() {
    let caps = G2CoreCapabilities::for_version(G2CoreVersion::new(100, 0));
    assert!(caps.is_minimum_version());
}

#[test]
fn test_g2core_supports_advanced() {
    let caps = G2CoreCapabilities::for_version(G2CoreVersion::new(100, 10));
    assert!(caps.supports_advanced_features());
}

#[test]
fn test_g2core_max_axes() {
    let caps = G2CoreCapabilities::for_version(G2CoreVersion::new(100, 5));
    assert_eq!(caps.max_axes(), 6);
}

#[test]
fn test_g2core_command_creator_gcode() {
    let mut creator = CommandCreator::new();
    let cmd = creator.create_gcode_command("G0 X10 Y20");
    assert!(cmd.contains("N1"));
    assert!(cmd.contains("G0 X10 Y20"));
}

#[test]
fn test_g2core_command_creator_status_query() {
    let creator = CommandCreator::new();
    let query = creator.create_status_query();
    assert_eq!(query, r#"{"sr":null}"#);
}

#[test]
fn test_g2core_command_creator_motion_command_6axis() {
    let mut creator = CommandCreator::new();
    let cmd = creator.create_motion_command(
        MotionType::Linear,
        Some(10.0),
        Some(20.0),
        Some(5.0),
        Some(45.0),
        None,
        None,
        Some(1000.0),
    );
    assert!(cmd.contains("G1"));
    assert!(cmd.contains("X10"));
    assert!(cmd.contains("Y20"));
    assert!(cmd.contains("Z5"));
    assert!(cmd.contains("A45"));
    assert!(cmd.contains("F1000"));
}

#[test]
fn test_g2core_command_creator_jog_command() {
    let mut creator = CommandCreator::new();
    let cmd = creator.create_jog_command(Some(5.0), Some(5.0), None, None, 500.0);
    assert!(cmd.contains("G91"));
    assert!(cmd.contains("X5"));
    assert!(cmd.contains("Y5"));
    assert!(cmd.contains("F500"));
}

#[test]
fn test_g2core_command_creator_kinematics_mode() {
    let creator = CommandCreator::new();
    let cmd = creator.create_kinematics_mode(KinematicMode::Inverse);
    assert!(cmd.contains("kin"));
    assert!(cmd.contains("2"));
}

#[test]
fn test_g2core_command_creator_line_number_increment() {
    let mut creator = CommandCreator::new();
    creator.create_gcode_command("G0 X0");
    assert_eq!(creator.get_line_number(), 1);
    creator.create_gcode_command("G0 Y0");
    assert_eq!(creator.get_line_number(), 2);
}

#[test]
fn test_g2core_realtime_command_as_byte() {
    assert_eq!(RealTimeCommand::StatusRequest.as_byte(), b'?');
    assert_eq!(RealTimeCommand::FeedHold.as_byte(), b'!');
    assert_eq!(RealTimeCommand::CycleStart.as_byte(), b'~');
    assert_eq!(RealTimeCommand::SoftReset.as_byte(), 0x18);
}

#[test]
fn test_g2core_response_parser_ok() {
    let mut parser = G2CoreResponseParser::new();
    let response = r#"{"ok":true}"#;
    let result = parser.parse(response);
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.response_type, G2CoreResponseType::Ok);
    assert!(resp.is_success());
}

#[test]
fn test_g2core_response_parser_error() {
    let mut parser = G2CoreResponseParser::new();
    let response = r#"{"er":{"code":1,"msg":"Generic error"}}"#;
    let result = parser.parse(response);
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert!(resp.is_error());
    assert_eq!(resp.error_code, Some(1));
}

#[test]
fn test_g2core_response_parser_status_report() {
    let mut parser = G2CoreResponseParser::new();
    let response = r#"{"sr":{"stat":{"state":"Idle"},"pos":{"x":0,"y":0,"z":0,"a":0,"b":0,"c":0},"mpos":{"x":0,"y":0,"z":0,"a":0},"feed":0,"speed":0,"unit":0,"line":0}}"#;
    let result = parser.parse(response);
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert!(resp.is_status_report());
}
