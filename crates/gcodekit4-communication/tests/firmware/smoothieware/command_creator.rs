//! Tests for firmware::smoothieware::command_creator

use gcodekit4_communication::firmware::smoothieware::command_creator::*;

#[test]
fn test_jog_command() {
    let creator = SmoothiewareCommandCreator::new();
    let cmd = creator.jog_command('X', 10.0, 1000.0);
    assert!(cmd.contains("X:10.00"));
    assert!(cmd.contains("1000"));
}

#[test]
fn test_spindle_commands() {
    let creator = SmoothiewareCommandCreator::new();
    assert_eq!(creator.spindle_on_cw(1000), "M3 S1000");
    assert_eq!(creator.spindle_on_ccw(500), "M4 S500");
    assert_eq!(creator.spindle_off(), "M5");
}

#[test]
fn test_home_command() {
    let creator = SmoothiewareCommandCreator::new();
    assert_eq!(creator.home_command(None), "G28.2");
    assert_eq!(creator.home_command(Some("X Y Z")), "G28.2 X Y Z");
}
