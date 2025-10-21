/// G-Code Parser Tests
///
/// Comprehensive test suite for the G-Code parser covering:
/// - Command parsing
/// - Modal state tracking
/// - G/M/T code recognition
/// - Coordinate parsing
/// - Comment handling
/// - Error conditions
use gcodekit4::gcode::{
    CommandListener, CommandNumberGenerator, CommandResponse, CommandState, GcodeCommand,
    GcodeParser, GcodeState, ModalState,
};

#[test]
fn test_gcode_command_creation() {
    let cmd = GcodeCommand::new("G00 X10.0 Y20.0");
    assert_eq!(cmd.state, CommandState::Pending);
    assert_eq!(cmd.sequence_number, 0);
    assert_eq!(cmd.command, "G00 X10.0 Y20.0");
    assert!(cmd.line_number.is_none());
    assert!(cmd.response.is_none());
}

#[test]
fn test_gcode_command_with_sequence() {
    let cmd = GcodeCommand::with_sequence("G01 X5.0", 42);
    assert_eq!(cmd.sequence_number, 42);
}

#[test]
fn test_gcode_command_with_id() {
    let cmd = GcodeCommand::with_id("G00 X0.0", "test-id-123".to_string());
    assert_eq!(cmd.id, "test-id-123");
}

#[test]
fn test_gcode_command_mark_sent() {
    let mut cmd = GcodeCommand::new("G00 X10.0");
    cmd.mark_sent();
    assert_eq!(cmd.state, CommandState::Sent);
    assert!(cmd.sent_at.is_some());
}

#[test]
fn test_gcode_command_mark_ok() {
    let mut cmd = GcodeCommand::new("G00 X10.0");
    cmd.mark_ok();
    assert_eq!(cmd.state, CommandState::Ok);
    assert!(cmd.completed_at.is_some());
}

#[test]
fn test_gcode_command_mark_done() {
    let mut cmd = GcodeCommand::new("G00 X10.0");
    cmd.mark_done();
    assert_eq!(cmd.state, CommandState::Done);
    assert!(cmd.completed_at.is_some());
}

#[test]
fn test_gcode_command_mark_error() {
    let mut cmd = GcodeCommand::new("G00 X10.0");
    cmd.mark_error(Some(5), "Invalid G-code".to_string());
    assert_eq!(cmd.state, CommandState::Error);
    assert!(cmd.response.is_some());
    let resp = cmd.response.unwrap();
    assert_eq!(resp.error_code, Some(5));
    assert_eq!(resp.message, "Invalid G-code");
}

#[test]
fn test_gcode_command_mark_skipped() {
    let mut cmd = GcodeCommand::new("G00 X10.0");
    cmd.mark_skipped();
    assert_eq!(cmd.state, CommandState::Skipped);
}

#[test]
fn test_gcode_command_set_response() {
    let mut cmd = GcodeCommand::new("G00 X10.0");
    let resp = CommandResponse {
        success: true,
        message: "ok".to_string(),
        error_code: None,
        data: None,
    };
    cmd.set_response(resp);
    assert!(cmd.response.is_some());
}

#[test]
fn test_gcode_command_is_terminal() {
    let mut cmd = GcodeCommand::new("G00 X10.0");
    assert!(!cmd.is_terminal());

    cmd.mark_done();
    assert!(cmd.is_terminal());

    let mut cmd2 = GcodeCommand::new("G00 X10.0");
    cmd2.mark_error(None, "error".to_string());
    assert!(cmd2.is_terminal());

    let mut cmd3 = GcodeCommand::new("G00 X10.0");
    cmd3.mark_skipped();
    assert!(cmd3.is_terminal());
}

#[test]
fn test_gcode_command_is_sent() {
    let mut cmd = GcodeCommand::new("G00 X10.0");
    assert!(!cmd.is_sent());

    cmd.mark_sent();
    assert!(cmd.is_sent());
}

#[test]
fn test_gcode_command_duration() {
    let mut cmd = GcodeCommand::new("G00 X10.0");
    let _created = cmd.created_at;

    // Simulate some time passing
    std::thread::sleep(std::time::Duration::from_millis(10));

    cmd.mark_sent();
    let _sent = cmd.sent_at;

    std::thread::sleep(std::time::Duration::from_millis(10));

    cmd.mark_done();
    let _completed = cmd.completed_at;

    assert!(cmd.total_duration().is_some());
    assert!(cmd.total_duration().unwrap() >= 20);
    assert!(cmd.execution_duration().is_some());
    assert!(cmd.execution_duration().unwrap() >= 10);
}

#[test]
fn test_gcode_command_display() {
    let cmd = GcodeCommand::new("G00 X10.0");
    let display_str = format!("{}", cmd);
    assert!(display_str.contains("Pending"));
    assert!(display_str.contains("G00 X10.0"));
}

#[test]
fn test_gcode_command_default() {
    let cmd: GcodeCommand = Default::default();
    assert_eq!(cmd.state, CommandState::Pending);
}

#[test]
fn test_command_number_generator() {
    let gen = CommandNumberGenerator::new();
    assert_eq!(gen.current(), 0);
    assert_eq!(gen.next(), 0);
    assert_eq!(gen.next(), 1);
    assert_eq!(gen.next(), 2);
    assert_eq!(gen.current(), 3);
}

#[test]
fn test_command_number_generator_reset() {
    let gen = CommandNumberGenerator::new();
    gen.next();
    gen.next();
    assert_eq!(gen.current(), 2);
    gen.reset();
    assert_eq!(gen.current(), 0);
}

#[test]
fn test_command_number_generator_clone() {
    let gen1 = CommandNumberGenerator::new();
    let gen2 = gen1.clone();

    assert_eq!(gen1.next(), 0);
    assert_eq!(gen2.next(), 1); // Shared counter
    assert_eq!(gen1.current(), 2);
}

#[test]
fn test_modal_state_default() {
    let state = ModalState::default();
    assert_eq!(state.motion_mode, 0); // G00
    assert_eq!(state.plane, 17); // G17 (XY)
    assert_eq!(state.distance_mode, 90); // G90 (absolute)
    assert_eq!(state.feed_rate_mode, 94); // G94 (units per minute)
}

#[test]
fn test_gcode_parser_creation() {
    let parser = GcodeParser::new();
    let state = parser.get_modal_state();
    assert_eq!(state.motion_mode, 0);
    assert_eq!(state.plane, 17);
    assert_eq!(state.distance_mode, 90);
    assert_eq!(state.feed_rate_mode, 94);
}

#[test]
fn test_gcode_parser_default() {
    let parser: GcodeParser = Default::default();
    let state = parser.get_modal_state();
    assert_eq!(state.motion_mode, 0);
}

#[test]
fn test_gcode_parser_parse_simple_command() {
    let mut parser = GcodeParser::new();
    let result = parser.parse("G00 X10.0 Y20.0");
    assert!(result.is_ok());

    let cmd = result.unwrap();
    assert_eq!(cmd.command, "G00 X10.0 Y20.0");
    assert_eq!(cmd.sequence_number, 0);
}

#[test]
fn test_gcode_parser_parse_with_line_number() {
    let mut parser = GcodeParser::new();
    let result = parser.parse("N10 G00 X10.0");
    assert!(result.is_ok());
}

#[test]
fn test_gcode_parser_sequential_numbering() {
    let mut parser = GcodeParser::new();

    let cmd1 = parser.parse("G00 X10.0").unwrap();
    assert_eq!(cmd1.sequence_number, 0);

    let cmd2 = parser.parse("G01 Y20.0").unwrap();
    assert_eq!(cmd2.sequence_number, 1);

    let cmd3 = parser.parse("G02 X30.0 Y30.0 I5.0 J5.0").unwrap();
    assert_eq!(cmd3.sequence_number, 2);
}

#[test]
fn test_gcode_parser_empty_line() {
    let mut parser = GcodeParser::new();
    let result = parser.parse("");
    assert!(result.is_err());
}

#[test]
fn test_gcode_parser_whitespace_only() {
    let mut parser = GcodeParser::new();
    let result = parser.parse("   ");
    assert!(result.is_err());
}

#[test]
fn test_gcode_parser_comment_removal() {
    let mut parser = GcodeParser::new();

    // Semicolon comment
    let result1 = parser.parse("G00 X10.0 ; move to X=10");
    assert!(result1.is_ok());
    let cmd1 = result1.unwrap();
    assert_eq!(cmd1.command.trim(), "G00 X10.0");

    // Parentheses comment
    let result2 = parser.parse("G01 Y20.0 (rapid move)");
    assert!(result2.is_ok());
    let cmd2 = result2.unwrap();
    assert_eq!(cmd2.command.trim(), "G01 Y20.0");
}

#[test]
fn test_gcode_parser_only_comment() {
    let mut parser = GcodeParser::new();
    let result = parser.parse("; this is just a comment");
    assert!(result.is_err());
}

#[test]
fn test_gcode_parser_modal_state_retrieval() {
    let parser = GcodeParser::new();
    let state = parser.get_modal_state();
    assert_eq!(state.motion_mode, 0);
    assert_eq!(state.plane, 17);
}

#[test]
fn test_gcode_parser_command_generator() {
    let parser = GcodeParser::new();
    let gen = parser.command_generator();
    assert_eq!(gen.current(), 0);
}

#[test]
fn test_gcode_command_state_display() {
    assert_eq!(format!("{}", CommandState::Pending), "Pending");
    assert_eq!(format!("{}", CommandState::Sent), "Sent");
    assert_eq!(format!("{}", CommandState::Ok), "Ok");
    assert_eq!(format!("{}", CommandState::Done), "Done");
    assert_eq!(format!("{}", CommandState::Error), "Error");
    assert_eq!(format!("{}", CommandState::Skipped), "Skipped");
}

#[test]
fn test_gcode_command_response_default() {
    let resp: CommandResponse = Default::default();
    assert!(!resp.success);
    assert_eq!(resp.message, "");
    assert!(resp.error_code.is_none());
    assert!(resp.data.is_none());
}

#[test]
fn test_command_listener_noop() {
    use gcodekit4::gcode::NoOpCommandListener;

    let listener = NoOpCommandListener;
    let cmd = GcodeCommand::new("G00 X10.0");

    // These should not panic
    listener.on_command_created(&cmd);
    listener.on_command_sent(&cmd);
    listener.on_command_ok(&cmd);
    listener.on_command_completed(&cmd);
    listener.on_command_skipped(&cmd);

    let resp = CommandResponse::default();
    listener.on_command_error(&cmd, &resp);
    listener.on_command_state_changed(&cmd, CommandState::Pending);
}

#[test]
fn test_gcode_parser_parse_various_codes() {
    let mut parser = GcodeParser::new();

    // Rapid movement
    let cmd = parser.parse("G00 X10.0").unwrap();
    assert_eq!(cmd.sequence_number, 0);

    // Linear movement
    let cmd = parser.parse("G01 X20.0 F100").unwrap();
    assert_eq!(cmd.sequence_number, 1);

    // Arc clockwise
    let cmd = parser.parse("G02 X30.0 Y10.0 I5.0 J5.0").unwrap();
    assert_eq!(cmd.sequence_number, 2);

    // Arc counter-clockwise
    let cmd = parser.parse("G03 X40.0 Y20.0 I-5.0 J-5.0").unwrap();
    assert_eq!(cmd.sequence_number, 3);

    // Spindle on
    let cmd = parser.parse("M3 S1000").unwrap();
    assert_eq!(cmd.sequence_number, 4);

    // Spindle off
    let cmd = parser.parse("M5").unwrap();
    assert_eq!(cmd.sequence_number, 5);
}

#[test]
fn test_gcode_command_chaining() {
    let mut cmd = GcodeCommand::new("G00 X10.0");
    cmd.set_line_number(1);
    assert_eq!(cmd.line_number, Some(1));
}

#[test]
fn test_gcode_command_serialization() {
    let mut cmd = GcodeCommand::new("G00 X10.0 Y20.0");
    cmd.mark_sent();

    // Test serde serialization
    let json = serde_json::to_string(&cmd).unwrap();
    assert!(json.contains("G00 X10.0 Y20.0"));
    assert!(json.contains("Sent"));

    // Test deserialization
    let deserialized: GcodeCommand = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.command, cmd.command);
    assert_eq!(deserialized.state, cmd.state);
}

#[test]
fn test_modal_state_serialization() {
    let state = ModalState::default();
    let json = serde_json::to_string(&state).unwrap();
    let deserialized: ModalState = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.motion_mode, state.motion_mode);
    assert_eq!(deserialized.plane, state.plane);
    assert_eq!(deserialized.distance_mode, state.distance_mode);
    assert_eq!(deserialized.feed_rate_mode, state.feed_rate_mode);
}

#[test]
fn test_gcode_parser_with_multiple_coordinates() {
    let mut parser = GcodeParser::new();

    let cmd = parser
        .parse("G00 X10.5 Y20.3 Z0.0 A45.0 B30.0 C15.0 F100")
        .unwrap();
    assert_eq!(
        cmd.command.trim(),
        "G00 X10.5 Y20.3 Z0.0 A45.0 B30.0 C15.0 F100"
    );
}

#[test]
fn test_gcode_command_line_number_builder() {
    let mut cmd = GcodeCommand::new("G00 X10.0");
    cmd.set_line_number(42);
    assert_eq!(cmd.line_number, Some(42));
}

#[test]
fn test_command_number_generator_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let gen = Arc::new(CommandNumberGenerator::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let gen_clone = Arc::clone(&gen);
        let handle = thread::spawn(move || gen_clone.next());
        handles.push(handle);
    }

    let results: Vec<u32> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    assert_eq!(results.len(), 10);
    // All numbers should be unique
    for i in 0..10 {
        assert!(results.contains(&(i as u32)));
    }
}

#[test]
fn test_gcode_command_lowercase() {
    let mut parser = GcodeParser::new();
    let cmd = parser.parse("g00 x10.0 y20.0").unwrap();
    assert_eq!(cmd.command.trim(), "g00 x10.0 y20.0");
}

#[test]
fn test_gcode_command_with_negative_coordinates() {
    let mut parser = GcodeParser::new();
    let cmd = parser.parse("G00 X-10.5 Y-20.3 Z-5.0").unwrap();
    assert_eq!(cmd.command.trim(), "G00 X-10.5 Y-20.3 Z-5.0");
}

#[test]
fn test_gcode_command_with_feed_rate() {
    let mut parser = GcodeParser::new();
    let cmd = parser.parse("G01 X100.0 F1200").unwrap();
    assert!(cmd.command.contains("F1200"));
}

#[test]
fn test_gcode_command_with_spindle_speed() {
    let mut parser = GcodeParser::new();
    let cmd = parser.parse("M3 S5000").unwrap();
    assert!(cmd.command.contains("S5000"));
}

// ============================================================================
// GcodeState Tests - Task 12: G-Code Parser - State Machine
// ============================================================================

#[test]
fn test_gcode_state_default() {
    let state = GcodeState::default();
    assert_eq!(state.motion_mode, 0); // G00
    assert_eq!(state.plane_mode, 17); // G17 (XY)
    assert_eq!(state.distance_mode, 90); // G90 (absolute)
    assert_eq!(state.feed_rate_mode, 94); // G94 (units per minute)
    assert_eq!(state.units_mode, 21); // G21 (mm)
    assert_eq!(state.coordinate_system, 54); // G54 (first WCS)
    assert_eq!(state.tool_offset_mode, 49); // G49 (disabled)
    assert_eq!(state.compensation_mode, 40); // G40 (off)
    assert_eq!(state.feed_rate, 0.0);
    assert_eq!(state.spindle_speed, 0.0);
    assert_eq!(state.tool_number, 0);
}

#[test]
fn test_gcode_state_new() {
    let state = GcodeState::new();
    assert_eq!(state.motion_mode, 0);
    assert_eq!(state.plane_mode, 17);
}

#[test]
fn test_gcode_state_set_motion_mode() {
    let mut state = GcodeState::new();
    
    assert!(state.set_motion_mode(0).is_ok());
    assert_eq!(state.motion_mode, 0);
    
    assert!(state.set_motion_mode(1).is_ok());
    assert_eq!(state.motion_mode, 1);
    
    assert!(state.set_motion_mode(2).is_ok());
    assert_eq!(state.motion_mode, 2);
    
    assert!(state.set_motion_mode(3).is_ok());
    assert_eq!(state.motion_mode, 3);
    
    assert!(state.set_motion_mode(99).is_err());
}

#[test]
fn test_gcode_state_set_plane_mode() {
    let mut state = GcodeState::new();
    
    assert!(state.set_plane_mode(17).is_ok());
    assert_eq!(state.plane_mode, 17);
    
    assert!(state.set_plane_mode(18).is_ok());
    assert_eq!(state.plane_mode, 18);
    
    assert!(state.set_plane_mode(19).is_ok());
    assert_eq!(state.plane_mode, 19);
    
    assert!(state.set_plane_mode(20).is_err());
}

#[test]
fn test_gcode_state_set_distance_mode() {
    let mut state = GcodeState::new();
    
    assert!(state.set_distance_mode(90).is_ok());
    assert_eq!(state.distance_mode, 90);
    
    assert!(state.set_distance_mode(91).is_ok());
    assert_eq!(state.distance_mode, 91);
    
    assert!(state.set_distance_mode(92).is_err());
}

#[test]
fn test_gcode_state_set_feed_rate_mode() {
    let mut state = GcodeState::new();
    
    assert!(state.set_feed_rate_mode(93).is_ok());
    assert_eq!(state.feed_rate_mode, 93);
    
    assert!(state.set_feed_rate_mode(94).is_ok());
    assert_eq!(state.feed_rate_mode, 94);
    
    assert!(state.set_feed_rate_mode(95).is_ok());
    assert_eq!(state.feed_rate_mode, 95);
    
    assert!(state.set_feed_rate_mode(96).is_err());
}

#[test]
fn test_gcode_state_set_units_mode() {
    let mut state = GcodeState::new();
    
    assert!(state.set_units_mode(20).is_ok()); // Inches
    assert_eq!(state.units_mode, 20);
    
    assert!(state.set_units_mode(21).is_ok()); // Millimeters
    assert_eq!(state.units_mode, 21);
    
    assert!(state.set_units_mode(22).is_err());
}

#[test]
fn test_gcode_state_set_coordinate_system() {
    let mut state = GcodeState::new();
    
    for cs in 54..=59 {
        assert!(state.set_coordinate_system(cs).is_ok());
        assert_eq!(state.coordinate_system, cs);
    }
    
    assert!(state.set_coordinate_system(53).is_err());
    assert!(state.set_coordinate_system(60).is_err());
}

#[test]
fn test_gcode_state_set_tool_offset_mode() {
    let mut state = GcodeState::new();
    
    assert!(state.set_tool_offset_mode(43).is_ok());
    assert_eq!(state.tool_offset_mode, 43);
    
    assert!(state.set_tool_offset_mode(49).is_ok());
    assert_eq!(state.tool_offset_mode, 49);
    
    assert!(state.set_tool_offset_mode(44).is_err());
}

#[test]
fn test_gcode_state_set_compensation_mode() {
    let mut state = GcodeState::new();
    
    assert!(state.set_compensation_mode(40).is_ok());
    assert_eq!(state.compensation_mode, 40);
    
    assert!(state.set_compensation_mode(41).is_ok());
    assert_eq!(state.compensation_mode, 41);
    
    assert!(state.set_compensation_mode(42).is_ok());
    assert_eq!(state.compensation_mode, 42);
    
    assert!(state.set_compensation_mode(43).is_err());
}

#[test]
fn test_gcode_state_set_feed_rate() {
    let mut state = GcodeState::new();
    
    assert!(state.set_feed_rate(100.5).is_ok());
    assert_eq!(state.feed_rate, 100.5);
    
    assert!(state.set_feed_rate(0.0).is_ok());
    assert_eq!(state.feed_rate, 0.0);
    
    assert!(state.set_feed_rate(-10.0).is_err());
}

#[test]
fn test_gcode_state_set_spindle_speed() {
    let mut state = GcodeState::new();
    
    assert!(state.set_spindle_speed(5000.0).is_ok());
    assert_eq!(state.spindle_speed, 5000.0);
    
    assert!(state.set_spindle_speed(0.0).is_ok());
    assert_eq!(state.spindle_speed, 0.0);
    
    assert!(state.set_spindle_speed(-1000.0).is_err());
}

#[test]
fn test_gcode_state_set_tool_number() {
    let mut state = GcodeState::new();
    
    state.set_tool_number(5);
    assert_eq!(state.tool_number, 5);
    
    state.set_tool_number(0);
    assert_eq!(state.tool_number, 0);
    
    state.set_tool_number(999);
    assert_eq!(state.tool_number, 999);
}

#[test]
fn test_gcode_state_validate() {
    let state = GcodeState::default();
    assert!(state.validate().is_ok());
    
    let mut invalid_state = state;
    invalid_state.motion_mode = 99;
    assert!(invalid_state.validate().is_err());
}

#[test]
fn test_gcode_state_motion_mode_description() {
    let state = GcodeState::default();
    assert_eq!(state.motion_mode_description(), "Rapid positioning (G00)");
    
    let mut state = state;
    state.motion_mode = 1;
    assert_eq!(state.motion_mode_description(), "Linear interpolation (G01)");
    
    state.motion_mode = 2;
    assert_eq!(state.motion_mode_description(), "Clockwise arc (G02)");
    
    state.motion_mode = 3;
    assert_eq!(state.motion_mode_description(), "Counter-clockwise arc (G03)");
}

#[test]
fn test_gcode_state_plane_description() {
    let state = GcodeState::default();
    assert_eq!(state.plane_description(), "XY plane (G17)");
    
    let mut state = state;
    state.plane_mode = 18;
    assert_eq!(state.plane_description(), "XZ plane (G18)");
    
    state.plane_mode = 19;
    assert_eq!(state.plane_description(), "YZ plane (G19)");
}

#[test]
fn test_gcode_state_distance_mode_description() {
    let state = GcodeState::default();
    assert_eq!(
        state.distance_mode_description(),
        "Absolute positioning (G90)"
    );
    
    let mut state = state;
    state.distance_mode = 91;
    assert_eq!(
        state.distance_mode_description(),
        "Incremental positioning (G91)"
    );
}

#[test]
fn test_gcode_state_units_description() {
    let state = GcodeState::default();
    assert_eq!(state.units_description(), "Millimeters (G21)");
    
    let mut state = state;
    state.units_mode = 20;
    assert_eq!(state.units_description(), "Inches (G20)");
}

#[test]
fn test_gcode_state_serialization() {
    let mut state = GcodeState::new();
    state.set_motion_mode(1).unwrap();
    state.set_feed_rate(150.0).unwrap();
    state.set_spindle_speed(3000.0).unwrap();
    
    let json = serde_json::to_string(&state).unwrap();
    let deserialized: GcodeState = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.motion_mode, 1);
    assert_eq!(deserialized.feed_rate, 150.0);
    assert_eq!(deserialized.spindle_speed, 3000.0);
}

#[test]
fn test_gcode_parser_state_tracking() {
    let mut parser = GcodeParser::new();
    
    // Parse G00 (rapid)
    parser.parse("G00 X10.0").unwrap();
    let state = parser.get_state();
    assert_eq!(state.motion_mode, 0);
    
    // Parse G01 (linear)
    parser.parse("G01 Y20.0 F100").unwrap();
    let state = parser.get_state();
    assert_eq!(state.motion_mode, 1);
    assert_eq!(state.feed_rate, 100.0);
}

#[test]
fn test_gcode_parser_g_code_recognition() {
    let mut parser = GcodeParser::new();
    
    parser.parse("G17").unwrap(); // XY plane
    let state = parser.get_state();
    assert_eq!(state.plane_mode, 17);
    
    parser.parse("G18").unwrap(); // XZ plane
    let state = parser.get_state();
    assert_eq!(state.plane_mode, 18);
    
    parser.parse("G19").unwrap(); // YZ plane
    let state = parser.get_state();
    assert_eq!(state.plane_mode, 19);
}

#[test]
fn test_gcode_parser_coordinate_system_tracking() {
    let mut parser = GcodeParser::new();
    
    for cs in 54..=59 {
        parser.parse(&format!("G{}", cs)).unwrap();
        let state = parser.get_state();
        assert_eq!(state.coordinate_system, cs as u8);
    }
}

#[test]
fn test_gcode_parser_feed_and_spindle_parsing() {
    let mut parser = GcodeParser::new();
    
    parser.parse("G01 X100 F1200 S5000").unwrap();
    let state = parser.get_state();
    assert_eq!(state.feed_rate, 1200.0);
    assert_eq!(state.spindle_speed, 5000.0);
}

#[test]
fn test_gcode_parser_tool_number_parsing() {
    let mut parser = GcodeParser::new();
    
    parser.parse("T5 M6").unwrap();
    let state = parser.get_state();
    assert_eq!(state.tool_number, 5);
}

#[test]
fn test_gcode_parser_state_persistence() {
    let mut parser = GcodeParser::new();
    
    // Parse G01
    parser.parse("G01").unwrap();
    let state1 = parser.get_state();
    assert_eq!(state1.motion_mode, 1);
    
    // Parse without G code - should retain G01
    parser.parse("X100 Y200").unwrap();
    let state2 = parser.get_state();
    assert_eq!(state2.motion_mode, 1); // Still G01
}

#[test]
fn test_gcode_parser_set_state() {
    let mut parser = GcodeParser::new();
    
    let mut new_state = GcodeState::new();
    new_state.set_motion_mode(2).unwrap();
    new_state.set_feed_rate(500.0).unwrap();
    
    parser.set_state(new_state);
    
    let current_state = parser.get_state();
    assert_eq!(current_state.motion_mode, 2);
    assert_eq!(current_state.feed_rate, 500.0);
}

#[test]
fn test_gcode_parser_modal_state_compatibility() {
    let mut parser = GcodeParser::new();
    
    parser.parse("G01 G17 G90 G94").unwrap();
    
    // Test backward compatibility
    let modal_state = parser.get_modal_state();
    assert_eq!(modal_state.motion_mode, 1);
    assert_eq!(modal_state.plane, 17);
    assert_eq!(modal_state.distance_mode, 90);
    assert_eq!(modal_state.feed_rate_mode, 94);
}

#[test]
fn test_gcode_state_complex_command() {
    let mut parser = GcodeParser::new();
    
    parser
        .parse("G01 X50.5 Y100.25 Z-5.0 F500 S3000 T3")
        .unwrap();
    
    let state = parser.get_state();
    assert_eq!(state.motion_mode, 1);
    assert_eq!(state.feed_rate, 500.0);
    assert_eq!(state.spindle_speed, 3000.0);
    assert_eq!(state.tool_number, 3);
}

#[test]
fn test_gcode_state_all_modes() {
    let mut parser = GcodeParser::new();
    
    // Test a sequence that exercises all modal groups
    parser.parse("G20").unwrap(); // Inches
    parser.parse("G90").unwrap(); // Absolute
    parser.parse("G94").unwrap(); // Units per minute
    parser.parse("G17").unwrap(); // XY plane
    parser.parse("G54").unwrap(); // WCS 1
    parser.parse("G40").unwrap(); // Cutter compensation off
    parser.parse("G49").unwrap(); // Tool offset off
    
    let state = parser.get_state();
    assert_eq!(state.units_mode, 20);
    assert_eq!(state.distance_mode, 90);
    assert_eq!(state.feed_rate_mode, 94);
    assert_eq!(state.plane_mode, 17);
    assert_eq!(state.coordinate_system, 54);
    assert_eq!(state.compensation_mode, 40);
    assert_eq!(state.tool_offset_mode, 49);
}
