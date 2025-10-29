//! Tests for firmware::settings

use gcodekit4::firmware::settings::*;

#[test]
fn test_add_and_get_setting() {
    let mut settings = DefaultFirmwareSettings::new();
    let setting = FirmwareSetting {
        id: "test".to_string(),
        value: "100".to_string(),
        description: "Test setting".to_string(),
        setting_type: SettingType::Numeric,
        min: Some(0.0),
        max: Some(200.0),
    };
    settings.add_setting(setting.clone());

    assert_eq!(settings.get_setting("test").unwrap().value, "100");
}

#[test]
fn test_validate_numeric_setting() {
    let mut settings = DefaultFirmwareSettings::new();
    settings.add_setting(FirmwareSetting {
        id: "test".to_string(),
        value: "100".to_string(),
        description: "Test".to_string(),
        setting_type: SettingType::Numeric,
        min: Some(0.0),
        max: Some(200.0),
    });

    assert!(settings.validate_setting("test", "50").is_ok());
    assert!(settings.validate_setting("test", "300").is_err());
}
