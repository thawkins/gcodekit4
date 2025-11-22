//! Tests for firmware::grbl::override_manager

use gcodekit4_communication::firmware::grbl::override_manager::*;

#[test]
fn test_override_manager_creation() {
    let manager = OverrideManager::new();
    assert_eq!(manager.get_feed_override(), 100);
    assert_eq!(manager.get_rapid_override(), 100);
    assert_eq!(manager.get_spindle_override(), 100);
}

#[test]
fn test_set_feed_override_valid() {
    let mut manager = OverrideManager::new();
    assert!(manager.set_feed_override(50).is_ok());
    assert_eq!(manager.get_feed_override(), 50);

    assert!(manager.set_feed_override(200).is_ok());
    assert_eq!(manager.get_feed_override(), 200);

    assert!(manager.set_feed_override(0).is_ok());
    assert_eq!(manager.get_feed_override(), 0);
}

#[test]
fn test_set_feed_override_invalid() {
    let mut manager = OverrideManager::new();
    assert!(manager.set_feed_override(250).is_err());
    assert_eq!(manager.get_feed_override(), 100); // unchanged
}

#[test]
fn test_increase_feed_1() {
    let mut manager = OverrideManager::new();
    assert!(manager.increase_feed_1().is_ok());
    assert_eq!(manager.get_feed_override(), 101);
}

#[test]
fn test_decrease_feed_1() {
    let mut manager = OverrideManager::new();
    manager.set_feed_override(50).ok();
    assert!(manager.decrease_feed_1().is_ok());
    assert_eq!(manager.get_feed_override(), 49);
}

#[test]
fn test_increase_feed_10() {
    let mut manager = OverrideManager::new();
    assert!(manager.increase_feed_10().is_ok());
    assert_eq!(manager.get_feed_override(), 110);
}

#[test]
fn test_decrease_feed_10() {
    let mut manager = OverrideManager::new();
    manager.set_feed_override(50).ok();
    assert!(manager.decrease_feed_10().is_ok());
    assert_eq!(manager.get_feed_override(), 40);
}

#[test]
fn test_feed_override_clipping() {
    let mut manager = OverrideManager::new();
    manager.set_feed_override(199).ok();
    assert!(manager.increase_feed_10().is_ok());
    assert_eq!(manager.get_feed_override(), 200); // clamped

    let mut manager = OverrideManager::new();
    manager.set_feed_override(5).ok();
    assert!(manager.decrease_feed_10().is_ok());
    assert_eq!(manager.get_feed_override(), 0); // clamped
}

#[test]
fn test_set_rapid_override_valid() {
    let mut manager = OverrideManager::new();
    assert!(manager.set_rapid_override(25).is_ok());
    assert_eq!(manager.get_rapid_override(), 25);

    assert!(manager.set_rapid_override(50).is_ok());
    assert_eq!(manager.get_rapid_override(), 50);

    assert!(manager.set_rapid_override(100).is_ok());
    assert_eq!(manager.get_rapid_override(), 100);
}

#[test]
fn test_set_rapid_override_invalid() {
    let mut manager = OverrideManager::new();
    assert!(manager.set_rapid_override(75).is_err());
    assert_eq!(manager.get_rapid_override(), 100); // unchanged
}

#[test]
fn test_rapid_override_command() {
    let mut manager = OverrideManager::new();
    manager.set_rapid_override(25).ok();
    assert_eq!(
        manager.get_rapid_override_command(),
        RealTimeOverrideCommand::RapidOv25
    );

    manager.set_rapid_override(50).ok();
    assert_eq!(
        manager.get_rapid_override_command(),
        RealTimeOverrideCommand::RapidOv50
    );

    manager.set_rapid_override(100).ok();
    assert_eq!(
        manager.get_rapid_override_command(),
        RealTimeOverrideCommand::RapidOv100
    );
}

#[test]
fn test_set_spindle_override_valid() {
    let mut manager = OverrideManager::new();
    assert!(manager.set_spindle_override(50).is_ok());
    assert_eq!(manager.get_spindle_override(), 50);

    assert!(manager.set_spindle_override(200).is_ok());
    assert_eq!(manager.get_spindle_override(), 200);
}

#[test]
fn test_set_spindle_override_invalid() {
    let mut manager = OverrideManager::new();
    assert!(manager.set_spindle_override(250).is_err());
    assert_eq!(manager.get_spindle_override(), 100); // unchanged
}

#[test]
fn test_spindle_override_increment_decrement() {
    let mut manager = OverrideManager::new();
    manager.set_spindle_override(100).ok();

    assert!(manager.increase_spindle_1().is_ok());
    assert_eq!(manager.get_spindle_override(), 101);

    assert!(manager.decrease_spindle_1().is_ok());
    assert_eq!(manager.get_spindle_override(), 100);
}

#[test]
fn test_stop_spindle() {
    let mut manager = OverrideManager::new();
    manager.set_spindle_override(100).ok();
    assert!(manager.stop_spindle().is_ok());
    assert_eq!(manager.get_spindle_override(), 0);
}

#[test]
fn test_reset_all() {
    let mut manager = OverrideManager::new();
    manager.set_feed_override(50).ok();
    manager.set_rapid_override(25).ok();
    manager.set_spindle_override(150).ok();

    manager.reset_all();

    assert_eq!(manager.get_feed_override(), 100);
    assert_eq!(manager.get_rapid_override(), 100);
    assert_eq!(manager.get_spindle_override(), 100);
}

#[test]
fn test_is_overridden() {
    let mut manager = OverrideManager::new();
    assert!(!manager.is_overridden());

    manager.set_feed_override(50).ok();
    assert!(manager.is_overridden());

    manager.reset_all();
    assert!(!manager.is_overridden());

    manager.set_rapid_override(50).ok();
    assert!(manager.is_overridden());
}

#[test]
fn test_real_time_override_command_values() {
    assert_eq!(RealTimeOverrideCommand::FeedHold.as_byte(), 0x21);
    assert_eq!(RealTimeOverrideCommand::CycleStart.as_byte(), 0x7E);
    assert_eq!(RealTimeOverrideCommand::RapidOv100.as_byte(), 0x97);
    assert_eq!(RealTimeOverrideCommand::SpindleStop.as_byte(), 0x9D);
}

#[test]
fn test_feed_override_command_detection() {
    let mut manager = OverrideManager::new();
    manager.set_feed_override(100).ok();
    assert!(manager.get_feed_override_command().is_none());

    manager.increase_feed_10().ok();
    let cmd = manager.get_feed_override_command();
    assert!(cmd.is_some());
}
