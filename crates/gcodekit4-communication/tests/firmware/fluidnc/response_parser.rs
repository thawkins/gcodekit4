//! Tests for firmware::fluidnc::response_parser

use gcodekit4_communication::firmware::fluidnc::response_parser::*;

#[test]
fn test_parse_ok() {
    let mut parser = FluidNCResponseParser::new();
    assert_eq!(parser.parse_line("ok"), Some(FluidNCResponse::Ok));
}

#[test]
fn test_parse_error() {
    let mut parser = FluidNCResponseParser::new();
    if let Some(FluidNCResponse::Error { code, message }) =
        parser.parse_line("error: 5 Invalid G code")
    {
        assert_eq!(code, 5);
        assert!(message.contains("Invalid"));
    } else {
        panic!("Should parse error");
    }
}
