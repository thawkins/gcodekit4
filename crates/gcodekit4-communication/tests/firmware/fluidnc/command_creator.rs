//! Tests for firmware::fluidnc::command_creator

use gcodekit4_communication::firmware::fluidnc::command_creator::*;

#[test]
fn test_jog_command() {
    let creator = FluidNCCommandCreator::new();
    let cmd = creator.jog_command('X', 10.0, 1000.0);
    assert!(cmd.contains("X10"));
    assert!(cmd.contains("1000"));
}

#[test]
fn test_spindle_commands() {
    let creator = FluidNCCommandCreator::new();
    assert_eq!(creator.spindle_on_cw(1000), "M3 S1000");
    assert_eq!(creator.spindle_on_ccw(500), "M4 S500");
    assert_eq!(creator.spindle_off(), "M5");
}

#[test]
fn test_reset_command() {
    let creator = FluidNCCommandCreator::new();
    assert_eq!(creator.reset(), "\x18");
}
