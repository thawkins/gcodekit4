use gcodekit4_communication::firmware::grbl::status_parser::*;
use gcodekit4_core::Units;

#[test]
fn test_parse_machine_position() {
    let mpos = MachinePosition::parse("10.000,20.000,30.000").unwrap();
    assert_eq!(mpos.x, 10.0);
    assert_eq!(mpos.y, 20.0);
    assert_eq!(mpos.z, 30.0);
    assert_eq!(mpos.a, None);
}

#[test]
fn test_parse_machine_position_multiaxis() {
    let mpos = MachinePosition::parse("10.000,20.000,30.000,5.000,2.000").unwrap();
    assert_eq!(mpos.x, 10.0);
    assert_eq!(mpos.a, Some(5.0));
    assert_eq!(mpos.b, Some(2.0));
}

#[test]
fn test_parse_work_position() {
    let wpos = WorkPosition::parse("0.000,0.000,0.000").unwrap();
    assert_eq!(wpos.x, 0.0);
    assert_eq!(wpos.y, 0.0);
    assert_eq!(wpos.z, 0.0);
}

#[test]
fn test_parse_work_coordinate_offset() {
    let wco = WorkCoordinateOffset::parse("10.000,0.000,5.000").unwrap();
    assert_eq!(wco.x, 10.0);
    assert_eq!(wco.y, 0.0);
    assert_eq!(wco.z, 5.0);
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
fn test_status_parser_feed_spindle_state() {
    let status = "<Run|MPos:0,0,0|WPos:0,0,0|F:1500.0|S:1200>";
    let state = StatusParser::parse_feed_spindle(status).unwrap();
    assert_eq!(state.feed_rate, 1500.0);
    assert_eq!(state.spindle_speed, 1200);
}

#[test]
fn test_status_parser_machine_state() {
    let idle_status = "<Idle|MPos:10.000,20.000,30.000>";
    assert_eq!(
        StatusParser::parse_machine_state(idle_status).unwrap(),
        "Idle"
    );

    let run_status = "<Run|MPos:10,20,30|F:1500>";
        assert_eq!(
            StatusParser::parse_machine_state(run_status).unwrap(),
            "Run"
        );

        let hold_status = "<Hold:0|MPos:10,20,30>";
        assert_eq!(
            StatusParser::parse_machine_state(hold_status).unwrap(),
            "Hold:0"
        );

        let alarm_status = "<Alarm|MPos:0,0,0>";
        assert_eq!(
            StatusParser::parse_machine_state(alarm_status).unwrap(),
            "Alarm"
        );
    }

    #[test]
    fn test_status_parser_full_parse() {
        let status = "<Run|MPos:10,20,30|WPos:5,8,2|Buf:10:100|F:1500|S:1000>";
        let full = StatusParser::parse_full(status);

        assert_eq!(full.machine_state.as_deref(), Some("Run"));
        assert!(full.mpos.is_some());
        assert!(full.wpos.is_some());
        assert!(full.buffer.is_some());
        assert_eq!(full.feed_rate, Some(1500.0));
        assert_eq!(full.spindle_speed, Some(1000));
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
