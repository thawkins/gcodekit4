use gcodekit4_communication::firmware::capabilities_db::*;
use gcodekit4_communication::firmware::firmware_version::{FirmwareType, SemanticVersion};

#[test]
fn test_capabilities_database_new() {
    let db = CapabilitiesDatabase::new();
    assert!(!db.supported_firmware_types().is_empty());
}

#[test]
fn test_grbl_1_1_capabilities() {
    let db = CapabilitiesDatabase::new();
    let caps = db
        .get_capabilities(FirmwareType::Grbl, &SemanticVersion::new(1, 1, 0))
        .unwrap();

    assert_eq!(caps.max_axes, 3);
    assert!(caps.arc_support);
    assert!(caps.variable_spindle);
    assert!(caps.tool_change);
    assert!(caps.probing);
    assert!(caps.status_reports);
    assert!(caps.realtime_commands);
    assert_eq!(caps.coordinate_systems, 6);
}

#[test]
fn test_grbl_1_2_capabilities() {
    let db = CapabilitiesDatabase::new();
    let caps = db
        .get_capabilities(FirmwareType::Grbl, &SemanticVersion::new(1, 2, 0))
        .unwrap();

    assert_eq!(caps.max_axes, 3);
    assert!(caps.arc_support);
    assert!(caps.variable_spindle);
    assert!(caps.tool_change);
    assert!(caps.probing);
    assert!(caps.probe_away);
    assert!(caps.status_reports);
    assert!(caps.realtime_commands);
    assert_eq!(caps.coordinate_systems, 6);
}

#[test]
fn test_grbl_1_3_capabilities() {
    let db = CapabilitiesDatabase::new();
    let caps = db
        .get_capabilities(FirmwareType::Grbl, &SemanticVersion::new(1, 3, 0))
        .unwrap();

    assert_eq!(caps.max_axes, 3);
    assert!(caps.arc_support);
    assert!(caps.variable_spindle);
    assert!(caps.tool_change);
    assert!(caps.probing);
    assert!(caps.probe_away);
    assert!(caps.status_reports);
    assert!(caps.realtime_commands);
    assert_eq!(caps.coordinate_systems, 6);
}

#[test]
fn test_grbl_0_9_limited_capabilities() {
    let db = CapabilitiesDatabase::new();
    let caps = db
        .get_capabilities(FirmwareType::Grbl, &SemanticVersion::new(0, 9, 0))
        .unwrap();

    assert!(!caps.arc_support);
    assert!(!caps.tool_change);
    assert!(!caps.probing);
    assert!(!caps.status_reports);
}

#[test]
fn test_tinyg_full_capabilities() {
    let db = CapabilitiesDatabase::new();
    let caps = db
        .get_capabilities(FirmwareType::TinyG, &SemanticVersion::new(2, 0, 0))
        .unwrap();

    assert_eq!(caps.max_axes, 4);
    assert!(caps.arc_support);
    assert!(caps.macro_support);
    assert!(caps.conditional_blocks);
    assert!(caps.variable_support);
    assert_eq!(caps.coordinate_systems, 9);
}

#[test]
fn test_g2core_advanced_capabilities() {
    let db = CapabilitiesDatabase::new();
    let caps = db
        .get_capabilities(FirmwareType::G2Core, &SemanticVersion::new(3, 0, 0))
        .unwrap();

    assert_eq!(caps.max_axes, 6);
    assert!(caps.arc_support);
    assert!(caps.tool_diameter_offset);
    assert!(caps.probe_away);
}

#[test]
fn test_fluidnc_max_capabilities() {
    let db = CapabilitiesDatabase::new();
    let caps = db
        .get_capabilities(FirmwareType::FluidNC, &SemanticVersion::new(3, 0, 0))
        .unwrap();

    assert_eq!(caps.max_axes, 9);
    assert!(caps.arc_support);
    assert!(caps.conditional_blocks);
}

#[test]
fn test_supports_capability() {
    let db = CapabilitiesDatabase::new();

    assert!(db.supports_capability(FirmwareType::Grbl, &SemanticVersion::new(1, 1, 0), "arc"));
    assert!(!db.supports_capability(FirmwareType::Grbl, &SemanticVersion::new(0, 9, 0), "arc"));
}

#[test]
fn test_supported_firmware_types() {
    let db = CapabilitiesDatabase::new();
    let types = db.supported_firmware_types();

    assert!(types.contains(&FirmwareType::Grbl));
    assert!(types.contains(&FirmwareType::TinyG));
    assert!(types.contains(&FirmwareType::G2Core));
    assert!(types.contains(&FirmwareType::Smoothieware));
    assert!(types.contains(&FirmwareType::FluidNC));
}

#[test]
fn test_capability_info_is_available() {
    let cap = CapabilityInfo {
        enabled: true,
        min_version: SemanticVersion::new(1, 0, 0),
        max_version: None,
        notes: "Test".to_string(),
    };

    assert!(cap.is_available_for(&SemanticVersion::new(1, 0, 0)));
    assert!(cap.is_available_for(&SemanticVersion::new(1, 1, 0)));
    assert!(!cap.is_available_for(&SemanticVersion::new(0, 9, 0)));
}

#[test]
fn test_smoothieware_capabilities() {
    let db = CapabilitiesDatabase::new();
    let caps = db
        .get_capabilities(FirmwareType::Smoothieware, &SemanticVersion::new(1, 0, 0))
        .unwrap();

    assert_eq!(caps.max_axes, 5);
    assert!(caps.arc_support);
    assert!(caps.variable_spindle);
    assert!(caps.tool_change);
    assert!(caps.tool_length_offset);
    assert!(caps.probing);
    assert!(caps.status_reports);
    assert!(caps.realtime_commands);
    assert_eq!(caps.coordinate_systems, 9);
    assert!(caps.macro_support);
    assert!(caps.conditional_blocks);
}
