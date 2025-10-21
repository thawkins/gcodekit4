//! data module integration tests

use gcodekit4::data::*;

#[test]
fn test_machine_status_default() {
    let status = MachineStatus::default();
    assert_eq!(status.status, ControllerStatus::Idle);
    assert_eq!(status.spindle_speed, 0.0);
}
