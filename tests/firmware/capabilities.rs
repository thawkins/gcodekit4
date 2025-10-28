use gcodekit4::firmware::capabilities::*;

#[test]
fn test_default_capabilities() {
    let caps = DefaultCapabilities::default();
    assert!(caps.has_capability(Capability::Probing));
    assert!(caps.has_capability(Capability::AutoHome));
    assert!(!caps.has_capability(Capability::WiFi));
}

#[test]
fn test_axis_support() {
    let mut caps = DefaultCapabilities::default();
    caps.set_max_axes(3);
    assert!(caps.supports_axis('X'));
    assert!(caps.supports_axis('Y'));
    assert!(caps.supports_axis('Z'));
    assert!(!caps.supports_axis('A'));
}

#[test]
fn test_set_capability() {
    let mut caps = DefaultCapabilities::default();
    caps.set_capability(Capability::WiFi, true);
    assert!(caps.has_capability(Capability::WiFi));
}
