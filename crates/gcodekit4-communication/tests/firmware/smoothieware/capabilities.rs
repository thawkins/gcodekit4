//! Tests for firmware::smoothieware::capabilities

use gcodekit4_communication::firmware::smoothieware::capabilities::*;

#[test]
fn test_default_capabilities() {
    let caps = SmoothiewareCapabilities::default();
    assert_eq!(caps.max_spindle_speed, 255);
    assert!(caps.supports_probing);
}

#[test]
fn test_axis_support() {
    let caps = SmoothiewareCapabilities::default();
    assert!(caps.supports_axis('x'));
    assert!(caps.supports_axis('Y'));
    assert!(caps.supports_axis('Z'));
    assert!(caps.supports_axis('A'));
    assert!(caps.supports_axis('B'));
    assert!(caps.supports_axis('C'));
}
