//! g2core Controller Tests
//!
//! Tests for g2core controller implementation

use gcodekit4::communication::ConnectionParams;
use gcodekit4::core::ControllerTrait;
use gcodekit4::data::ControllerState;
use gcodekit4::firmware::G2CoreController;

#[test]
fn test_g2core_controller_creation() {
    let params = ConnectionParams::default();
    let controller = G2CoreController::new(params, Some("TestG2Core".to_string()));
    assert!(controller.is_ok());
    let ctrl = controller.unwrap();
    assert_eq!(ctrl.name(), "TestG2Core");
}

#[test]
fn test_g2core_controller_default_name() {
    let params = ConnectionParams::default();
    let controller = G2CoreController::new(params, None);
    assert!(controller.is_ok());
    let ctrl = controller.unwrap();
    assert_eq!(ctrl.name(), "g2core");
}

#[test]
fn test_g2core_controller_initial_state() {
    let params = ConnectionParams::default();
    let ctrl = G2CoreController::new(params, None).unwrap();
    assert!(!ctrl.is_connected());
    let state = ctrl.get_state();
    assert_eq!(state, ControllerState::Disconnected);
}

#[test]
fn test_g2core_axes_management() {
    let params = ConnectionParams::default();
    let ctrl = G2CoreController::new(params, None).unwrap();
    assert_eq!(ctrl.get_active_axes(), 6);
    ctrl.set_active_axes(5);
    assert_eq!(ctrl.get_active_axes(), 5);
}

#[test]
fn test_g2core_kinematics_mode() {
    let params = ConnectionParams::default();
    let ctrl = G2CoreController::new(params, None).unwrap();
    assert_eq!(ctrl.get_kinematics_mode(), None);
    ctrl.set_kinematics_mode(Some("Cartesian".to_string()));
    assert_eq!(ctrl.get_kinematics_mode(), Some("Cartesian".to_string()));
}
