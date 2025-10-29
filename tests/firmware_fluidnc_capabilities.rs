//! Tests for firmware::fluidnc::capabilities

use gcodekit4::firmware::fluidnc::capabilities::*;

#[test]
fn test_default_capabilities() {
    let caps = FluidNCCapabilities::default();
    assert_eq!(caps.max_spindle_speed, 10000);
    assert!(caps.supports_probing);
    assert!(caps.supports_wifi);
}

#[test]
fn test_axis_support() {
    let caps = FluidNCCapabilities::default();
    assert!(caps.supports_axis('x'));
    assert!(caps.supports_axis('Y'));
    assert!(caps.supports_axis('Z'));
    assert!(caps.supports_axis('A'));
    assert!(caps.supports_axis('B'));
    assert!(caps.supports_axis('C'));
    assert!(caps.supports_axis('U'));
}
