//! Tests for GRBL Protocol Constants and Capabilities

use gcodekit4::firmware::grbl::{
    GrblCapabilities, GrblFeature, GrblFeatureSet, GrblVersion, VersionComparison,
    constants::*,
};

#[test]
fn test_grbl_version_parsing_full_version() {
    let version = GrblVersion::parse("Grbl 1.1h ['$' for help]").unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 1);
    assert_eq!(version.patch, 0);
    assert_eq!(version.build, Some("h".to_string()));
}

#[test]
fn test_grbl_version_parsing_v0_9() {
    let version = GrblVersion::parse("Grbl 0.9i").unwrap();
    assert_eq!(version.major, 0);
    assert_eq!(version.minor, 9);
    assert_eq!(version.build, Some("i".to_string()));
}

#[test]
fn test_grbl_version_parsing_invalid() {
    assert!(GrblVersion::parse("Invalid").is_none());
    assert!(GrblVersion::parse("1.1.0").is_none());
}

#[test]
fn test_grbl_version_comparison() {
    let v1_0 = GrblVersion::new(1, 0, 0);
    let v1_1 = GrblVersion::new(1, 1, 0);
    let v1_2 = GrblVersion::new(1, 2, 0);

    assert!(v1_0 < v1_1);
    assert!(v1_1 < v1_2);
    assert!(v1_1 > v1_0);
    assert_eq!(v1_1, v1_1);
}

#[test]
fn test_grbl_version_meets_minimum() {
    let v1_1 = GrblVersion::new(1, 1, 0);
    let v1_0 = GrblVersion::new(1, 0, 0);

    assert!(v1_1.meets_minimum(&v1_0));
    assert!(!v1_0.meets_minimum(&v1_1));
    assert!(v1_1.meets_minimum(&v1_1));
}

#[test]
fn test_grbl_version_is_1_1_or_later() {
    let v1_0 = GrblVersion::new(1, 0, 0);
    let v1_1 = GrblVersion::new(1, 1, 0);
    let v2_0 = GrblVersion::new(2, 0, 0);

    assert!(!v1_0.is_1_1_or_later());
    assert!(v1_1.is_1_1_or_later());
    assert!(v2_0.is_1_1_or_later());
}

#[test]
fn test_grbl_version_is_0_9_or_later() {
    let v0_8 = GrblVersion::new(0, 8, 0);
    let v0_9 = GrblVersion::new(0, 9, 0);
    let v1_0 = GrblVersion::new(1, 0, 0);

    assert!(!v0_8.is_0_9_or_later());
    assert!(v0_9.is_0_9_or_later());
    assert!(v1_0.is_0_9_or_later());
}

#[test]
fn test_grbl_version_to_summary() {
    let v1 = GrblVersion::new(1, 1, 0);
    assert_eq!(v1.to_summary(), "1.1");

    let v2 = GrblVersion::with_build(1, 1, 0, "h".to_string());
    assert_eq!(v2.to_summary(), "1.1h");
}

#[test]
fn test_grbl_version_display() {
    let v1 = GrblVersion::new(1, 1, 0);
    assert_eq!(v1.to_string(), "Grbl 1.1.0");

    let v2 = GrblVersion::with_build(1, 1, 0, "h".to_string());
    assert_eq!(v2.to_string(), "Grbl 1.1h");
}

#[test]
fn test_grbl_feature_set_0_9() {
    let features = GrblFeatureSet::grbl_0_9();
    assert!(features.status_reports);
    assert!(features.real_time_commands);
    assert!(features.comments);
    assert!(features.coordinate_systems);
    assert!(features.probing);
    assert!(features.spindle_control);
    assert!(!features.coolant_control);
    assert!(!features.safety_door);
    assert!(!features.jog_command);
}

#[test]
fn test_grbl_feature_set_1_1() {
    let features = GrblFeatureSet::grbl_1_1();
    assert!(features.status_reports);
    assert!(features.real_time_commands);
    assert!(features.comments);
    assert!(features.coordinate_systems);
    assert!(features.probing);
    assert!(features.spindle_control);
    assert!(features.coolant_control);
    assert!(features.safety_door);
    assert!(features.jog_command);
}

#[test]
fn test_grbl_feature_set_for_version() {
    let v0_9 = GrblVersion::new(0, 9, 0);
    let v1_1 = GrblVersion::new(1, 1, 0);

    let features_0_9 = GrblFeatureSet::for_version(&v0_9);
    assert!(!features_0_9.jog_command);

    let features_1_1 = GrblFeatureSet::for_version(&v1_1);
    assert!(features_1_1.jog_command);
}

#[test]
fn test_grbl_capabilities_for_version() {
    let version = GrblVersion::new(1, 1, 0);
    let caps = GrblCapabilities::for_version(version.clone());

    assert_eq!(caps.version, version);
    assert_eq!(caps.max_axes, 6);
    assert_eq!(caps.buffer_size, 128);
    assert_eq!(caps.max_block_size, 256);
}

#[test]
fn test_grbl_capabilities_from_startup_string() {
    let caps = GrblCapabilities::from_startup_string("Grbl 1.1h ['$' for help]").unwrap();
    assert_eq!(caps.version.major, 1);
    assert_eq!(caps.version.minor, 1);
}

#[test]
fn test_grbl_capabilities_supports() {
    let caps = GrblCapabilities::for_version(GrblVersion::new(1, 1, 0));

    assert!(caps.supports(GrblFeature::StatusReports));
    assert!(caps.supports(GrblFeature::JogCommand));
    assert!(caps.supports(GrblFeature::SafetyDoor));
}

#[test]
fn test_grbl_capabilities_max_speeds() {
    let caps = GrblCapabilities::for_version(GrblVersion::new(1, 1, 0));

    assert_eq!(caps.max_feed_rate(), 24000.0);
    assert_eq!(caps.max_rapid_rate(), 1000.0);
    assert_eq!(caps.max_spindle_speed(), 10000);
}

#[test]
fn test_grbl_feature_display() {
    assert_eq!(GrblFeature::StatusReports.to_string(), "Status Reports");
    assert_eq!(GrblFeature::JogCommand.to_string(), "Jog Command");
}

#[test]
fn test_version_comparison_too_old() {
    let version = GrblVersion::new(0, 8, 0);
    let minimum = GrblVersion::new(0, 9, 0);

    let result = VersionComparison::check(&version, &minimum, None);
    assert_eq!(result, VersionComparison::TooOld);
}

#[test]
fn test_version_comparison_compatible() {
    let version = GrblVersion::new(1, 1, 0);
    let minimum = GrblVersion::new(0, 9, 0);

    let result = VersionComparison::check(&version, &minimum, None);
    assert_eq!(result, VersionComparison::Compatible);
}

#[test]
fn test_version_comparison_too_new() {
    let version = GrblVersion::new(2, 0, 0);
    let minimum = GrblVersion::new(1, 0, 0);
    let maximum = GrblVersion::new(1, 9, 9);

    let result = VersionComparison::check(&version, &minimum, Some(&maximum));
    assert_eq!(result, VersionComparison::TooNew);
}

#[test]
fn test_grbl_constants_values() {
    assert_eq!(CMD_QUERY_STATUS, b'?');
    assert_eq!(CMD_FEED_HOLD, b'!');
    assert_eq!(CMD_CYCLE_START, b'~');
    assert_eq!(CMD_SOFT_RESET, 0x18);

    assert_eq!(STATUS_IDLE, "Idle");
    assert_eq!(STATUS_RUNNING, "Run");
    assert_eq!(STATUS_ALARM, "Alarm");
}

#[test]
fn test_grbl_error_constants() {
    assert_eq!(ERROR_EXPECTED_COMMAND_LETTER, 1);
    assert_eq!(ERROR_BAD_NUMBER_FORMAT, 2);
    assert_eq!(ERROR_INVALID_STATEMENT, 3);
}

#[test]
fn test_grbl_alarm_constants() {
    assert_eq!(ALARM_HARD_LIMIT, 1);
    assert_eq!(ALARM_SOFT_LIMIT, 2);
    assert_eq!(ALARM_ABORT_CYCLE, 3);
}

#[test]
fn test_grbl_default_capabilities() {
    let caps = GrblCapabilities::default();
    assert_eq!(caps.version.major, 1);
    assert_eq!(caps.version.minor, 1);
    assert!(caps.features.jog_command);
}

// Response parser tests
mod response_parser_tests {
    use gcodekit4::firmware::grbl::{
        GrblResponse, GrblResponseParser, StatusReport, BufferState,
    };

    #[test]
    fn test_parse_ok_response() {
        let parser = GrblResponseParser::new();
        assert_eq!(parser.parse("ok"), Some(GrblResponse::Ok));
    }

    #[test]
    fn test_parse_error_responses() {
        let parser = GrblResponseParser::new();
        assert_eq!(parser.parse("error:1"), Some(GrblResponse::Error(1)));
        assert_eq!(parser.parse("error:23"), Some(GrblResponse::Error(23)));
    }

    #[test]
    fn test_parse_alarm_responses() {
        let parser = GrblResponseParser::new();
        assert_eq!(parser.parse("alarm:1"), Some(GrblResponse::Alarm(1)));
        assert_eq!(parser.parse("alarm:6"), Some(GrblResponse::Alarm(6)));
    }

    #[test]
    fn test_parse_basic_status_report() {
        let parser = GrblResponseParser::new();
        let response = parser.parse("<Idle|MPos:0.000,0.000,0.000|WPos:0.000,0.000,0.000>");

        assert!(matches!(response, Some(GrblResponse::Status(_))));

        if let Some(GrblResponse::Status(status)) = response {
            assert_eq!(status.state, "Idle");
            assert_eq!(status.machine_pos.x, 0.0);
            assert_eq!(status.work_pos.y, 0.0);
        }
    }

    #[test]
    fn test_parse_status_with_buffer_state() {
        let parser = GrblResponseParser::new();
        let response =
            parser.parse("<Run|MPos:10.000,5.000,2.500|WPos:10.000,5.000,2.500|Buf:15:128>");

        if let Some(GrblResponse::Status(status)) = response {
            assert_eq!(status.state, "Run");
            assert_eq!(status.buffer_state, Some(BufferState { plan: 15, exec: 128 }));
        }
    }

    #[test]
    fn test_parse_status_with_feedrate_and_spindle() {
        let parser = GrblResponseParser::new();
        let response = parser.parse("<Run|MPos:0,0,0|WPos:0,0,0|F:1500.0|S:1200>");

        if let Some(GrblResponse::Status(status)) = response {
            assert_eq!(status.feed_rate, Some(1500.0));
            assert_eq!(status.spindle_speed, Some(1200));
        }
    }

    #[test]
    fn test_parse_setting_response() {
        let parser = GrblResponseParser::new();
        let response = parser.parse("$110=1000.000");

        assert!(matches!(response, Some(GrblResponse::Setting { .. })));

        if let Some(GrblResponse::Setting { number, value }) = response {
            assert_eq!(number, 110);
            assert_eq!(value, "1000.000");
        }
    }

    #[test]
    fn test_parse_version_response() {
        let parser = GrblResponseParser::new();
        assert!(matches!(
            parser.parse("Grbl 1.1h ['$' for help]"),
            Some(GrblResponse::Version(_))
        ));
    }

    #[test]
    fn test_parse_build_info() {
        let parser = GrblResponseParser::new();
        assert!(matches!(
            parser.parse("[GrblHAL 1.1 STM32F4xx]"),
            Some(GrblResponse::BuildInfo(_))
        ));
    }

    #[test]
    fn test_error_description_lookup() {
        assert_eq!(
            GrblResponseParser::error_description(1),
            "Expected command letter"
        );
        assert_eq!(
            GrblResponseParser::error_description(23),
            "Failed to execute startup block"
        );
        assert_eq!(
            GrblResponseParser::error_description(99),
            "Unknown error"
        );
    }

    #[test]
    fn test_alarm_description_lookup() {
        assert_eq!(
            GrblResponseParser::alarm_description(1),
            "Hard limit triggered"
        );
        assert_eq!(
            GrblResponseParser::alarm_description(6),
            "Homing fail"
        );
        assert_eq!(
            GrblResponseParser::alarm_description(99),
            "Unknown alarm"
        );
    }

    #[test]
    fn test_parse_empty_line() {
        let parser = GrblResponseParser::new();
        assert_eq!(parser.parse(""), None);
    }

    #[test]
    fn test_parse_whitespace_only() {
        let parser = GrblResponseParser::new();
        assert_eq!(parser.parse("   "), None);
    }

    #[test]
    fn test_parse_multiaxis_position() {
        let parser = GrblResponseParser::new();
        let response =
            parser.parse("<Idle|MPos:10.000,20.000,30.000,5.000|WPos:10.000,20.000,30.000,5.000>");

        if let Some(GrblResponse::Status(status)) = response {
            assert_eq!(status.machine_pos.x, 10.0);
            assert_eq!(status.machine_pos.y, 20.0);
            assert_eq!(status.machine_pos.z, 30.0);
            assert_eq!(status.machine_pos.a, 5.0);
        }
    }

    #[test]
    fn test_response_display() {
        assert_eq!(GrblResponse::Ok.to_string(), "ok");
        assert_eq!(GrblResponse::Error(1).to_string(), "error:1");
        assert_eq!(GrblResponse::Alarm(2).to_string(), "alarm:2");
    }
}

// Status parser tests
mod status_parser_tests {
    use gcodekit4::firmware::grbl::{
        StatusParser, MachinePosition, WorkPosition, BufferRxState, FeedSpindleState,
        WorkCoordinateOffset,
    };
    use gcodekit4::data::Units;

    #[test]
    fn test_parse_machine_position_basic() {
        let mpos = MachinePosition::parse("10.000,20.000,30.000").unwrap();
        assert_eq!(mpos.x, 10.0);
        assert_eq!(mpos.y, 20.0);
        assert_eq!(mpos.z, 30.0);
        assert_eq!(mpos.a, None);
    }

    #[test]
    fn test_parse_machine_position_with_axes() {
        let mpos = MachinePosition::parse("10.000,20.000,30.000,5.000,2.000").unwrap();
        assert_eq!(mpos.a, Some(5.0));
        assert_eq!(mpos.b, Some(2.0));
        assert_eq!(mpos.c, None);
    }

    #[test]
    fn test_parse_machine_position_invalid() {
        assert!(MachinePosition::parse("invalid").is_none());
        assert!(MachinePosition::parse("10.0,20.0").is_none());
    }

    #[test]
    fn test_parse_work_position() {
        let wpos = WorkPosition::parse("0.000,0.000,0.000").unwrap();
        assert_eq!(wpos.x, 0.0);
        assert_eq!(wpos.y, 0.0);
        assert_eq!(wpos.z, 0.0);
    }

    #[test]
    fn test_parse_work_position_multiaxis() {
        let wpos = WorkPosition::parse("5.0,8.0,2.0,1.5").unwrap();
        assert_eq!(wpos.z, 2.0);
        assert_eq!(wpos.a, Some(1.5));
    }

    #[test]
    fn test_parse_work_coordinate_offset() {
        let wco = WorkCoordinateOffset::parse("10.000,0.000,5.000").unwrap();
        assert_eq!(wco.x, 10.0);
        assert_eq!(wco.y, 0.0);
        assert_eq!(wco.z, 5.0);
    }

    #[test]
    fn test_work_coordinate_offset_to_cncpoint() {
        let wco = WorkCoordinateOffset::parse("10.000,20.000,30.000").unwrap();
        let point = wco.to_cncpoint(Units::MM);

        assert_eq!(point.x, 10.0);
        assert_eq!(point.y, 20.0);
        assert_eq!(point.z, 30.0);
        assert_eq!(point.unit, Units::MM);
    }

    #[test]
    fn test_parse_feed_spindle_state() {
        let state = FeedSpindleState::parse("1500.0", "1200").unwrap();
        assert_eq!(state.feed_rate, 1500.0);
        assert_eq!(state.spindle_speed, 1200);
    }

    #[test]
    fn test_parse_buffer_rx_state() {
        let buf = BufferRxState::parse("15:128").unwrap();
        assert_eq!(buf.plan, 15);
        assert_eq!(buf.rx, 128);
    }

    #[test]
    fn test_parse_buffer_rx_state_invalid() {
        assert!(BufferRxState::parse("15").is_none());
        assert!(BufferRxState::parse("invalid:data").is_none());
    }

    #[test]
    fn test_status_parser_extract_mpos() {
        let status = "<Idle|MPos:10.000,20.000,30.000|WPos:0.000,0.000,0.000>";
        let mpos = StatusParser::parse_mpos(status).unwrap();
        assert_eq!(mpos.x, 10.0);
        assert_eq!(mpos.y, 20.0);
        assert_eq!(mpos.z, 30.0);
    }

    #[test]
    fn test_status_parser_extract_wpos() {
        let status = "<Idle|MPos:10.000,20.000,30.000|WPos:5.000,8.000,2.000>";
        let wpos = StatusParser::parse_wpos(status).unwrap();
        assert_eq!(wpos.x, 5.0);
        assert_eq!(wpos.y, 8.0);
        assert_eq!(wpos.z, 2.0);
    }

    #[test]
    fn test_status_parser_extract_wco() {
        let status = "<Idle|MPos:0,0,0|WPos:0,0,0|WCO:10.000,0.000,5.000>";
        let wco = StatusParser::parse_wco(status).unwrap();
        assert_eq!(wco.x, 10.0);
        assert_eq!(wco.z, 5.0);
    }

    #[test]
    fn test_status_parser_extract_buffer() {
        let status = "<Run|MPos:0,0,0|WPos:0,0,0|Buf:12:96>";
        let buffer = StatusParser::parse_buffer(status).unwrap();
        assert_eq!(buffer.plan, 12);
        assert_eq!(buffer.rx, 96);
    }

    #[test]
    fn test_status_parser_extract_feed_rate() {
        let status = "<Run|MPos:0,0,0|WPos:0,0,0|F:2000.5>";
        let rate = StatusParser::parse_feed_rate(status).unwrap();
        assert_eq!(rate, 2000.5);
    }

    #[test]
    fn test_status_parser_extract_spindle_speed() {
        let status = "<Run|MPos:0,0,0|WPos:0,0,0|S:5000>";
        let speed = StatusParser::parse_spindle_speed(status).unwrap();
        assert_eq!(speed, 5000);
    }

    #[test]
    fn test_status_parser_feed_spindle_both() {
        let status = "<Run|MPos:0,0,0|WPos:0,0,0|F:1500.0|S:1200>";
        let state = StatusParser::parse_feed_spindle(status).unwrap();
        assert_eq!(state.feed_rate, 1500.0);
        assert_eq!(state.spindle_speed, 1200);
    }

    #[test]
    fn test_status_parser_full_complex_status() {
        let status = "<Run|MPos:10,20,30|WPos:5,8,2|WCO:10,0,5|Buf:10:100|F:1500|S:1000>";
        let full = StatusParser::parse_full(status);

        assert!(full.mpos.is_some());
        assert!(full.wpos.is_some());
        assert!(full.wco.is_some());
        assert!(full.buffer.is_some());
        assert_eq!(full.feed_rate, Some(1500.0));
        assert_eq!(full.spindle_speed, Some(1000));
    }

    #[test]
    fn test_status_parser_partial_status() {
        let status = "<Idle|MPos:0,0,0|WPos:0,0,0>";
        let full = StatusParser::parse_full(status);

        assert!(full.mpos.is_some());
        assert!(full.wpos.is_some());
        assert!(full.wco.is_none());
        assert!(full.buffer.is_none());
        assert!(full.feed_rate.is_none());
    }

    #[test]
    fn test_position_edge_cases() {
        // Test with zeros
        let mpos = MachinePosition::parse("0,0,0").unwrap();
        assert_eq!(mpos.x, 0.0);

        // Test with negative values
        let wpos = WorkPosition::parse("-10.5,20.5,-30.5").unwrap();
        assert_eq!(wpos.x, -10.5);
        assert_eq!(wpos.y, 20.5);
        assert_eq!(wpos.z, -30.5);
    }
}

// Utils tests
mod utils_tests {
    use gcodekit4::firmware::grbl::utils;

    #[test]
    fn test_is_valid_response_ok() {
        assert!(utils::is_valid_response("ok"));
        assert!(utils::is_valid_response("  ok  "));
    }

    #[test]
    fn test_is_valid_response_error() {
        assert!(utils::is_valid_response("error:1"));
        assert!(utils::is_valid_response("error:23"));
    }

    #[test]
    fn test_is_valid_response_alarm() {
        assert!(utils::is_valid_response("alarm:1"));
        assert!(utils::is_valid_response("alarm:6"));
    }

    #[test]
    fn test_is_valid_response_status() {
        assert!(utils::is_valid_response("<Idle|MPos:0,0,0|WPos:0,0,0>"));
        assert!(utils::is_valid_response("<Run|MPos:10,20,30|WPos:10,20,30>"));
    }

    #[test]
    fn test_is_valid_response_setting() {
        assert!(utils::is_valid_response("$110=1000"));
    }

    #[test]
    fn test_is_valid_response_version() {
        assert!(utils::is_valid_response("Grbl 1.1h"));
    }

    #[test]
    fn test_is_valid_response_invalid() {
        assert!(!utils::is_valid_response(""));
        assert!(!utils::is_valid_response("invalid"));
    }

    #[test]
    fn test_get_state_name() {
        assert_eq!(utils::get_state_name("Idle"), "Idle");
        assert_eq!(utils::get_state_name("Run"), "Running");
        assert_eq!(utils::get_state_name("Hold"), "Hold");
        assert_eq!(utils::get_state_name("Jog"), "Jogging");
        assert_eq!(utils::get_state_name("Alarm"), "Alarm");
    }

    #[test]
    fn test_is_error_state() {
        assert!(utils::is_error_state("Alarm"));
        assert!(utils::is_error_state("Check"));
        assert!(utils::is_error_state("Door"));
        assert!(!utils::is_error_state("Idle"));
    }

    #[test]
    fn test_is_running_state() {
        assert!(utils::is_running_state("Run"));
        assert!(utils::is_running_state("Jog"));
        assert!(!utils::is_running_state("Idle"));
    }

    #[test]
    fn test_is_idle_state() {
        assert!(utils::is_idle_state("Idle"));
        assert!(!utils::is_idle_state("Run"));
    }

    #[test]
    fn test_is_held_state() {
        assert!(utils::is_held_state("Hold"));
        assert!(!utils::is_held_state("Idle"));
    }

    #[test]
    fn test_get_error_codes() {
        let codes = utils::get_error_codes();
        assert!(codes.contains_key(&1));
        assert!(codes.contains_key(&23));
        assert!(!codes.contains_key(&99));
    }

    #[test]
    fn test_get_alarm_codes() {
        let codes = utils::get_alarm_codes();
        assert!(codes.contains_key(&1));
        assert!(codes.contains_key(&6));
        assert!(!codes.contains_key(&99));
    }

    #[test]
    fn test_get_setting_name() {
        assert_eq!(utils::get_setting_name(110), "X max rate");
        assert_eq!(utils::get_setting_name(160), "Junction deviation");
        assert_eq!(utils::get_setting_name(200), "Unknown setting");
    }

    #[test]
    fn test_format_position() {
        assert_eq!(utils::format_position(10.1234), "10.123");
        assert_eq!(utils::format_position(0.0), "0.000");
    }

    #[test]
    fn test_format_positions() {
        let result = utils::format_positions(10.0, 20.0, 30.0);
        assert_eq!(result, "10.000,20.000,30.000");
    }

    #[test]
    fn test_parse_setting_response() {
        let (num, val) = utils::parse_setting_response("$110=1000.000").unwrap();
        assert_eq!(num, 110);
        assert_eq!(val, "1000.000");
    }

    #[test]
    fn test_parse_setting_response_invalid() {
        assert!(utils::parse_setting_response("invalid").is_none());
        assert!(utils::parse_setting_response("110=1000").is_none());
    }

    #[test]
    fn test_format_buffer_state() {
        let result = utils::format_buffer_state(10, 100);
        assert!(result.contains("Plan: 10"));
        assert!(result.contains("RX: 100"));
    }

    #[test]
    fn test_is_command_accepted() {
        assert!(utils::is_command_accepted("ok"));
        assert!(utils::is_command_accepted("  ok  "));
        assert!(!utils::is_command_accepted("error:1"));
    }

    #[test]
    fn test_is_command_error() {
        assert!(utils::is_command_error("error:1"));
        assert!(utils::is_command_error("alarm:2"));
        assert!(!utils::is_command_error("ok"));
    }
}

// Command creator tests
mod command_creator_tests {
    use gcodekit4::firmware::grbl::{
        CommandCreator, RealTimeCommand, SystemCommand, JogCommand, JogMode, ProbeCommand, ProbeType,
    };
    use gcodekit4::data::{CNCPoint, Units};

    #[test]
    fn test_real_time_command_query_status() {
        let cmd = RealTimeCommand::QueryStatus;
        assert_eq!(cmd.to_byte(), b'?');
        assert_eq!(cmd.description(), "Query Status");
    }

    #[test]
    fn test_real_time_command_feed_hold() {
        let cmd = RealTimeCommand::FeedHold;
        assert_eq!(cmd.to_byte(), b'!');
    }

    #[test]
    fn test_real_time_command_cycle_start() {
        let cmd = RealTimeCommand::CycleStart;
        assert_eq!(cmd.to_byte(), b'~');
    }

    #[test]
    fn test_real_time_command_soft_reset() {
        let cmd = RealTimeCommand::SoftReset;
        assert_eq!(cmd.to_byte(), 0x18);
    }

    #[test]
    fn test_system_command_home_all() {
        let cmd = SystemCommand::HomeAll;
        assert_eq!(cmd.command(), "$H");
        assert_eq!(cmd.description(), "Home All Axes");
    }

    #[test]
    fn test_system_command_kill_alarm() {
        let cmd = SystemCommand::KillAlarmLock;
        assert_eq!(cmd.command(), "$X");
    }

    #[test]
    fn test_system_command_check_mode() {
        let cmd = SystemCommand::CheckMode;
        assert_eq!(cmd.command(), "$C");
    }

    #[test]
    fn test_jog_command_xy_plane() {
        let target = CNCPoint::with_axes(10.0, 20.0, 0.0, 0.0, 0.0, 0.0, Units::MM);
        let jog = JogCommand::new(JogMode::XY, target, 1000.0);
        let gcode = jog.to_gcode();
        
        assert!(gcode.contains("$J=G91 G0"));
        assert!(gcode.contains("X10.000"));
        assert!(gcode.contains("Y20.000"));
        assert!(gcode.contains("F1000"));
    }

    #[test]
    fn test_jog_command_xz_plane() {
        let target = CNCPoint::with_axes(10.0, 0.0, -5.0, 0.0, 0.0, 0.0, Units::MM);
        let jog = JogCommand::new(JogMode::XZ, target, 500.0);
        let gcode = jog.to_gcode();
        
        assert!(gcode.contains("X10.000"));
        assert!(gcode.contains("Z-5.000"));
    }

    #[test]
    fn test_probe_command_touching() {
        let target = CNCPoint::with_axes(0.0, 0.0, -10.0, 0.0, 0.0, 0.0, Units::MM);
        let probe = ProbeCommand::new(ProbeType::Touching, target, 100.0);
        let gcode = probe.to_gcode();
        
        assert!(gcode.contains("G38.2"));
        assert!(gcode.contains("Z-10.000"));
    }

    #[test]
    fn test_probe_command_backing() {
        let target = CNCPoint::new(Units::MM);
        let probe = ProbeCommand::new(ProbeType::Backing, target, 50.0);
        let gcode = probe.to_gcode();
        
        assert!(gcode.contains("G38.4"));
    }

    #[test]
    fn test_probe_type_gcode_commands() {
        assert_eq!(ProbeType::Touching.gcode_command(), "G38.2");
        assert_eq!(ProbeType::TouchingRequired.gcode_command(), "G38.3");
        assert_eq!(ProbeType::Backing.gcode_command(), "G38.4");
        assert_eq!(ProbeType::BackingRequired.gcode_command(), "G38.5");
    }

    #[test]
    fn test_command_creator_soft_reset() {
        let cmd = CommandCreator::soft_reset();
        assert_eq!(cmd[0], 0x18);
    }

    #[test]
    fn test_command_creator_query_status() {
        let cmd = CommandCreator::query_status();
        assert_eq!(cmd[0], b'?');
    }

    #[test]
    fn test_command_creator_feed_hold() {
        let cmd = CommandCreator::feed_hold();
        assert_eq!(cmd[0], b'!');
    }

    #[test]
    fn test_command_creator_cycle_start() {
        let cmd = CommandCreator::cycle_start();
        assert_eq!(cmd[0], b'~');
    }

    #[test]
    fn test_command_creator_home_all() {
        let cmd = CommandCreator::home_all();
        assert_eq!(cmd.trim(), "$H");
    }

    #[test]
    fn test_command_creator_kill_alarm_lock() {
        let cmd = CommandCreator::kill_alarm_lock();
        assert_eq!(cmd.trim(), "$X");
    }

    #[test]
    fn test_command_creator_spindle_on() {
        let cmd = CommandCreator::spindle_on(1200);
        assert!(cmd.contains("M3"));
        assert!(cmd.contains("S1200"));
    }

    #[test]
    fn test_command_creator_spindle_off() {
        let cmd = CommandCreator::spindle_off();
        assert_eq!(cmd.trim(), "M5");
    }

    #[test]
    fn test_command_creator_coolant_on() {
        let cmd = CommandCreator::coolant_on();
        assert_eq!(cmd.trim(), "M8");
    }

    #[test]
    fn test_command_creator_coolant_off() {
        let cmd = CommandCreator::coolant_off();
        assert_eq!(cmd.trim(), "M9");
    }

    #[test]
    fn test_command_creator_rapid_move_xy() {
        let cmd = CommandCreator::rapid_move(Some(10.0), Some(20.0), None);
        assert!(cmd.contains("G0"));
        assert!(cmd.contains("X10.000"));
        assert!(cmd.contains("Y20.000"));
    }

    #[test]
    fn test_command_creator_rapid_move_z_only() {
        let cmd = CommandCreator::rapid_move(None, None, Some(5.0));
        assert!(cmd.contains("G0"));
        assert!(cmd.contains("Z5.000"));
    }

    #[test]
    fn test_command_creator_linear_move() {
        let cmd = CommandCreator::linear_move(Some(10.0), None, Some(5.0), 1000.0);
        assert!(cmd.contains("G1"));
        assert!(cmd.contains("X10.000"));
        assert!(cmd.contains("Z5.000"));
        assert!(cmd.contains("F1000"));
    }

    #[test]
    fn test_command_creator_dwell() {
        let cmd = CommandCreator::dwell(1.5);
        assert!(cmd.contains("G4"));
        assert!(cmd.contains("P1.5"));
    }

    #[test]
    fn test_command_creator_program_pause() {
        let cmd = CommandCreator::program_pause();
        assert_eq!(cmd.trim(), "M0");
    }

    #[test]
    fn test_command_creator_program_end() {
        let cmd = CommandCreator::program_end();
        assert_eq!(cmd.trim(), "M2");
    }

    #[test]
    fn test_command_creator_tool_change() {
        let cmd = CommandCreator::tool_change(3);
        assert!(cmd.contains("T3"));
        assert!(cmd.contains("M6"));
    }

    #[test]
    fn test_command_creator_jog_incremental() {
        let cmd = CommandCreator::jog_incremental("X", 5.0, 500.0);
        assert!(cmd.contains("$J=G91 G0"));
        assert!(cmd.contains("X+5.000"));
        assert!(cmd.contains("F500"));
    }

    #[test]
    fn test_command_creator_set_work_offset() {
        let cmd = CommandCreator::set_work_offset(&["X", "Y", "Z"]);
        assert!(cmd.contains("G10"));
        assert!(cmd.contains("X0"));
        assert!(cmd.contains("Y0"));
        assert!(cmd.contains("Z0"));
    }
}
