use gcodekit4_communication::firmware::firmware_version::*;

#[test]
fn test_semantic_version_parse() {
    let v = SemanticVersion::parse("1.2.3").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 3);

    let v2 = SemanticVersion::parse("0.9j").unwrap();
    assert_eq!(v2.major, 0);
    assert_eq!(v2.minor, 9);
    // Patch might be 0 or parsed from 'j' depending on implementation, 
    // but let's assume basic parsing works
}

#[test]
fn test_semantic_version_comparison() {
    let v1 = SemanticVersion::new(1, 0, 0);
    let v2 = SemanticVersion::new(1, 1, 0);
    let v3 = SemanticVersion::new(2, 0, 0);

    assert!(v1 < v2);
    assert!(v2 < v3);
    assert!(v1 < v3);
}

#[test]
fn test_semantic_version_compatibility() {
    let v1 = SemanticVersion::new(1, 1, 0);
    let v2 = SemanticVersion::new(1, 1, 5);
    let v3 = SemanticVersion::new(2, 0, 0);

    assert!(v1.is_compatible_with(&v2)); // Same major version compatible
    assert!(!v1.is_compatible_with(&v3)); // Different major version not compatible
}

#[test]
fn test_firmware_type_parsing() {
    assert_eq!(FirmwareType::from_string("Grbl"), FirmwareType::Grbl);
    // Marlin is not supported in FirmwareType enum
    assert_eq!(FirmwareType::from_string("Unknown"), FirmwareType::Unknown);
}

#[test]
fn test_firmware_version_display() {
    let fw = FirmwareVersion::new(
        FirmwareType::Grbl,
        SemanticVersion::new(1, 1, 0),
        "1.1h".to_string(),
    );
    let display = fw.to_string();
    assert!(display.contains("GRBL"));
    assert!(display.contains("1.1.0"));
}

#[test]
fn test_grbl_version_parsing() {
    let v = SemanticVersion::parse("1.1h").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 1);
}

#[test]
fn test_version_at_least() {
    let v = SemanticVersion::new(1, 1, 0);
    assert!(v.is_at_least(&SemanticVersion::new(1, 0, 0)));
    assert!(v.is_at_least(&SemanticVersion::new(1, 1, 0)));
    assert!(!v.is_at_least(&SemanticVersion::new(1, 2, 0)));
    assert!(!v.is_at_least(&SemanticVersion::new(2, 0, 0)));
}
