//! TinyG Controller Tests
//!
//! Tests for TinyG controller implementation

use gcodekit4::communication::ConnectionParams;
use gcodekit4::core::ControllerTrait;
use gcodekit4::data::ControllerState;
use gcodekit4::firmware::TinyGController;

#[test]
fn test_tinyg_controller_creation() {
    let params = ConnectionParams::default();
    let controller = TinyGController::new(params, Some("TestTinyG".to_string()));
    assert!(controller.is_ok());
    let ctrl = controller.unwrap();
    assert_eq!(ctrl.name(), "TestTinyG");
}

#[test]
fn test_tinyg_controller_default_name() {
    let params = ConnectionParams::default();
    let controller = TinyGController::new(params, None);
    assert!(controller.is_ok());
    let ctrl = controller.unwrap();
    assert_eq!(ctrl.name(), "TinyG");
}

#[test]
fn test_tinyg_controller_initial_state() {
    let params = ConnectionParams::default();
    let ctrl = TinyGController::new(params, None).unwrap();
    assert!(!ctrl.is_connected());
    let state = ctrl.get_state();
    assert_eq!(state, ControllerState::Disconnected);
}
