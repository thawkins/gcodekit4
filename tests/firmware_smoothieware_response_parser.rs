//! Tests for firmware::smoothieware::response_parser

use gcodekit4::firmware::smoothieware::response_parser::*;

#[test]
fn test_parse_ok() {
    let mut parser = SmoothiewareResponseParser::new();
    assert_eq!(parser.parse_line("ok"), Some(SmoothiewareResponse::Ok));
}

#[test]
fn test_parse_error() {
    let mut parser = SmoothiewareResponseParser::new();
    if let Some(SmoothiewareResponse::Error(msg)) = parser.parse_line("Error: Invalid command") {
        assert!(msg.contains("Invalid"));
    } else {
        panic!("Should parse error");
    }
}

#[test]
fn test_parse_position() {
    let mut parser = SmoothiewareResponseParser::new();
    if let Some(SmoothiewareResponse::Position { x, y, z }) =
        parser.parse_line("X:10.0 Y:20.0 Z:5.0")
    {
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
        assert_eq!(z, 5.0);
    } else {
        panic!("Should parse position");
    }
}
