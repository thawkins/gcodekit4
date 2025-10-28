//! firmware module integration tests

pub mod capabilities;
pub mod g2core;
pub mod grbl;
pub mod grbl_communicator;
pub mod grbl_controller;
pub mod grbl_override_manager;
pub mod grbl_settings;
pub mod tinyg;

use gcodekit4::firmware::*;

#[test]
fn test_controller_type_display() {
    assert_eq!(ControllerType::Grbl.to_string(), "GRBL");
    assert_eq!(ControllerType::TinyG.to_string(), "TinyG");
}

#[test]
fn test_grbl_capabilities() {
    let caps = FirmwareCapabilities::grbl();
    assert_eq!(caps.controller_type, ControllerType::Grbl);
    assert_eq!(caps.max_axes, 5);
    assert!(caps.supports_probing);
}

#[test]
fn test_fluidnc_capabilities() {
    let caps = FirmwareCapabilities::fluidnc();
    assert_eq!(caps.controller_type, ControllerType::FluidNC);
    assert_eq!(caps.max_axes, 6);
    assert!(caps.supports_tool_change);
}
