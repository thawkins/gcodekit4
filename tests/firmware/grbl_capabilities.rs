use gcodekit4::firmware::grbl::capabilities::*;

#[test]
fn test_grbl_version_parsing() {
    let version = GrblVersion::parse("Grbl 1.1h ['$' for help]").unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 1);
    assert_eq!(version.build, Some("h".to_string()));
}

#[test]
fn test_grbl_version_comparison() {
    let v1_0 = GrblVersion::new(1, 0, 0);
    let v1_1 = GrblVersion::new(1, 1, 0);
    let v1_2 = GrblVersion::new(1, 2, 0);

    assert!(v1_0 < v1_1);
    assert!(v1_1 < v1_2);
    assert!(v1_2 > v1_1);
}

#[test]
fn test_grbl_version_parsing_1_2() {
    let version = GrblVersion::parse("Grbl 1.2h ['$' for help]").unwrap();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.build, Some("h".to_string()));
}

#[test]
fn test_grbl_is_1_2_or_later() {
    let v1_1 = GrblVersion::new(1, 1, 0);
    let v1_2 = GrblVersion::new(1, 2, 0);
    let v1_3 = GrblVersion::new(1, 3, 0);

    assert!(!v1_1.is_1_2_or_later());
    assert!(v1_2.is_1_2_or_later());
    assert!(v1_3.is_1_2_or_later());
}

#[test]
fn test_grbl_version_minimum_check() {
    let v1_1 = GrblVersion::new(1, 1, 0);
    let v1_0 = GrblVersion::new(1, 0, 0);

    assert!(v1_1.meets_minimum(&v1_0));
    assert!(!v1_0.meets_minimum(&v1_1));
}

#[test]
fn test_grbl_feature_set_1_1() {
    let features = GrblFeatureSet::grbl_1_1();
    assert!(features.jog_command);
    assert!(features.safety_door);
    assert!(features.coolant_control);
}

#[test]
fn test_grbl_feature_set_1_2() {
    let features = GrblFeatureSet::grbl_1_2();
    assert!(features.jog_command);
    assert!(features.safety_door);
    assert!(features.coolant_control);
    // 1.2 has same features as 1.1
    let features_1_1 = GrblFeatureSet::grbl_1_1();
    assert_eq!(features, features_1_1);
}

#[test]
fn test_grbl_capabilities_creation() {
    let caps = GrblCapabilities::for_version(GrblVersion::new(1, 1, 0));
    assert_eq!(caps.version.major, 1);
    assert_eq!(caps.version.minor, 1);
    assert!(caps.supports(GrblFeature::JogCommand));
}

#[test]
fn test_grbl_capabilities_1_2_creation() {
    let caps = GrblCapabilities::for_version(GrblVersion::new(1, 2, 0));
    assert_eq!(caps.version.major, 1);
    assert_eq!(caps.version.minor, 2);
    assert!(caps.supports(GrblFeature::JogCommand));
    assert!(caps.supports(GrblFeature::SafetyDoor));
    assert!(caps.supports(GrblFeature::CoolantControl));
}
