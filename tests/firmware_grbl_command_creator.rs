//! Tests for firmware::grbl::command_creator

use gcodekit4::data::{CNCPoint, Units};
use gcodekit4::firmware::grbl::command_creator::*;

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
    assert_eq!(cmd.description(), "Feed Hold");
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
fn test_jog_command_xy() {
    let target = CNCPoint::new(Units::MM);
    let jog = JogCommand::new(JogMode::XY, target, 1000.0);
    let gcode = jog.to_gcode();
    assert!(gcode.contains("$J=G91 G0"));
    assert!(gcode.contains("X"));
    assert!(gcode.contains("Y"));
}

#[test]
fn test_probe_command() {
    let target = CNCPoint::with_axes(0.0, 0.0, -10.0, 0.0, 0.0, 0.0, Units::MM);
    let probe = ProbeCommand::new(ProbeType::Touching, target, 100.0);
    let gcode = probe.to_gcode();
    assert!(gcode.contains("G38.2"));
    assert!(gcode.contains("Z"));
}

#[test]
fn test_command_creator_soft_reset() {
    let cmd = CommandCreator::soft_reset();
    assert_eq!(cmd[0], 0x18);
}

#[test]
fn test_command_creator_home_all() {
    let cmd = CommandCreator::home_all();
    assert_eq!(cmd.trim(), "$H");
}

#[test]
fn test_command_creator_spindle_on() {
    let cmd = CommandCreator::spindle_on(1200);
    assert!(cmd.contains("M3"));
    assert!(cmd.contains("S1200"));
}

#[test]
fn test_command_creator_rapid_move() {
    let cmd = CommandCreator::rapid_move(Some(10.0), Some(20.0), None);
    assert!(cmd.contains("G0"));
    assert!(cmd.contains("X10.000"));
    assert!(cmd.contains("Y20.000"));
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
fn test_command_creator_program_end() {
    let cmd = CommandCreator::program_end();
    assert_eq!(cmd.trim(), "M2");
}

#[test]
fn test_probe_type_descriptions() {
    assert_eq!(ProbeType::Touching.gcode_command(), "G38.2");
    assert_eq!(ProbeType::Backing.gcode_command(), "G38.4");
}
