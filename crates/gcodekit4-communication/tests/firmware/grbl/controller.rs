use gcodekit4_communication::firmware::grbl::controller::*;
use gcodekit4_core::ControllerState;

#[test]
fn test_grbl_controller_state_default() {
    let state = GrblControllerState::default();
    assert_eq!(state.state, ControllerState::Disconnected);
    assert_eq!(state.poll_rate_ms, 100);
    assert!(!state.is_streaming);
}
