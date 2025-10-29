//! Tests for firmware::g2core::capabilities

use gcodekit4::firmware::g2core::capabilities::*;

#[test]
fn test_g2core_version_parse() {
    let version = G2CoreVersion::parse("100.00").unwrap();
    assert_eq!(version.major, 100);
    assert_eq!(version.minor, 0);
    assert_eq!(version.build, None);
}

#[test]
fn test_g2core_version_parse_with_build() {
    let version = G2CoreVersion::parse("100.10.05").unwrap();
    assert_eq!(version.major, 100);
    assert_eq!(version.minor, 10);
    assert_eq!(version.build, Some(5));
}

#[test]
fn test_g2core_version_display() {
    let version = G2CoreVersion::new(100, 10);
    assert_eq!(version.to_string(), "100.10");
}

#[test]
fn test_g2core_version_comparison() {
    let v1 = G2CoreVersion::new(100, 10);
    let v2 = G2CoreVersion::new(100, 5);
    assert!(v1 > v2);
}
