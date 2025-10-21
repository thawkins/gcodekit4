//! core module integration tests

use gcodekit4::core::{Controller, ControllerState};

#[test]
fn test_controller_creation() {
    let controller = Controller::new();
    assert_eq!(controller.get_state(), ControllerState::Disconnected);
}

#[test]
fn test_state_transitions() {
    let controller = Controller::new();

    controller.set_state(ControllerState::Idle);
    assert_eq!(controller.get_state(), ControllerState::Idle);

    controller.set_state(ControllerState::Executing);
    assert_eq!(controller.get_state(), ControllerState::Executing);
}
