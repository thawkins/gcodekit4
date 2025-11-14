//! Tests for GRBL firmware configuration backup and restore

use chrono::Utc;
use gcodekit4::firmware::grbl::settings::{
    ConfigBackup, Setting, SettingCategory, SettingsManager,
};
use std::fs;

#[test]
fn test_create_and_export_backup() {
    let mut manager = SettingsManager::new();

    // Add some test settings
    manager.set_setting(Setting {
        number: 100,
        name: "X-axis steps per mm".to_string(),
        value: "80.0".to_string(),
        numeric_value: Some(80.0),
        description: "X-axis steps per millimeter".to_string(),
        unit: Some("steps/mm".to_string()),
        category: SettingCategory::StepsPerUnit,
        range: Some((1.0, 1000.0)),
        read_only: false,
    });

    manager.set_setting(Setting {
        number: 110,
        name: "X-axis max rate".to_string(),
        value: "5000.0".to_string(),
        numeric_value: Some(5000.0),
        description: "X-axis maximum rate".to_string(),
        unit: Some("mm/min".to_string()),
        category: SettingCategory::MaxRate,
        range: Some((1.0, 20000.0)),
        read_only: false,
    });

    // Create backup
    let backup = manager.create_backup(
        "GRBL 1.1h".to_string(),
        "Test Machine".to_string(),
        "Test configuration".to_string(),
    );

    assert_eq!(backup.firmware_version, "GRBL 1.1h");
    assert_eq!(backup.machine_name, "Test Machine");
    assert_eq!(backup.notes, "Test configuration");
    assert_eq!(backup.settings.len(), 2);
}

#[test]
fn test_export_and_load_backup() {
    let temp_dir = std::env::temp_dir();
    let backup_path = temp_dir.join("test_grbl_backup.json");

    let mut manager = SettingsManager::new();

    // Add test settings
    manager.set_setting(Setting {
        number: 100,
        name: "X-axis steps per mm".to_string(),
        value: "80.0".to_string(),
        numeric_value: Some(80.0),
        description: "X-axis steps per millimeter".to_string(),
        unit: Some("steps/mm".to_string()),
        category: SettingCategory::StepsPerUnit,
        range: Some((1.0, 1000.0)),
        read_only: false,
    });

    // Export to file
    let result = manager.export_backup(
        &backup_path,
        "GRBL 1.1h".to_string(),
        "Test Machine".to_string(),
        "Test backup".to_string(),
    );
    assert!(result.is_ok());

    // Verify file exists
    assert!(backup_path.exists());

    // Load backup from file
    let loaded_backup = SettingsManager::load_backup(&backup_path);
    assert!(loaded_backup.is_ok());

    let backup = loaded_backup.unwrap();
    assert_eq!(backup.firmware_version, "GRBL 1.1h");
    assert_eq!(backup.machine_name, "Test Machine");
    assert_eq!(backup.settings.len(), 1);
    assert_eq!(backup.settings[0].number, 100);

    // Clean up
    fs::remove_file(&backup_path).ok();
}

#[test]
fn test_apply_backup() {
    let mut manager = SettingsManager::new();

    // Create a backup manually
    let backup = ConfigBackup {
        timestamp: Utc::now(),
        firmware_version: "GRBL 1.1h".to_string(),
        machine_name: "Test Machine".to_string(),
        settings: vec![
            Setting {
                number: 100,
                name: "X-axis steps per mm".to_string(),
                value: "80.0".to_string(),
                numeric_value: Some(80.0),
                description: "X-axis steps per millimeter".to_string(),
                unit: Some("steps/mm".to_string()),
                category: SettingCategory::StepsPerUnit,
                range: Some((1.0, 1000.0)),
                read_only: false,
            },
            Setting {
                number: 110,
                name: "X-axis max rate".to_string(),
                value: "5000.0".to_string(),
                numeric_value: Some(5000.0),
                description: "X-axis maximum rate".to_string(),
                unit: Some("mm/min".to_string()),
                category: SettingCategory::MaxRate,
                range: Some((1.0, 20000.0)),
                read_only: false,
            },
        ],
        notes: "Test backup".to_string(),
    };

    // Apply backup
    manager.apply_backup(&backup);

    // Verify settings were applied
    assert_eq!(manager.count(), 2);
    assert!(manager.get_setting(100).is_some());
    assert!(manager.get_setting(110).is_some());

    let setting_100 = manager.get_setting(100).unwrap();
    assert_eq!(setting_100.value, "80.0");
}

#[test]
fn test_enrich_settings() {
    let mut manager = SettingsManager::new();

    // Add a setting without metadata
    manager.set_setting(Setting {
        number: 100,
        name: "Unknown".to_string(),
        value: "80.0".to_string(),
        numeric_value: Some(80.0),
        description: "".to_string(),
        unit: None,
        category: SettingCategory::Other,
        range: None,
        read_only: false,
    });

    // Enrich with metadata
    manager.enrich_settings();

    // Verify metadata was added
    let setting = manager.get_setting(100).unwrap();
    assert_eq!(setting.name, "X-axis steps per mm");
    assert_eq!(setting.description, "X-axis steps per millimeter");
    assert_eq!(setting.unit, Some("steps/mm".to_string()));
    assert_eq!(setting.category, SettingCategory::StepsPerUnit);
    assert_eq!(setting.range, Some((1.0, 1000.0)));
}

#[test]
fn test_backup_json_format() {
    let temp_dir = std::env::temp_dir();
    let backup_path = temp_dir.join("test_grbl_format.json");

    let mut manager = SettingsManager::new();
    manager.set_setting(Setting {
        number: 100,
        name: "X-axis steps per mm".to_string(),
        value: "80.0".to_string(),
        numeric_value: Some(80.0),
        description: "X-axis steps per millimeter".to_string(),
        unit: Some("steps/mm".to_string()),
        category: SettingCategory::StepsPerUnit,
        range: Some((1.0, 1000.0)),
        read_only: false,
    });

    // Export
    manager
        .export_backup(
            &backup_path,
            "GRBL 1.1h".to_string(),
            "Test".to_string(),
            "Test notes".to_string(),
        )
        .unwrap();

    // Read and verify JSON structure
    let json_content = fs::read_to_string(&backup_path).unwrap();
    assert!(json_content.contains("firmware_version"));
    assert!(json_content.contains("GRBL 1.1h"));
    assert!(json_content.contains("machine_name"));
    assert!(json_content.contains("timestamp"));
    assert!(json_content.contains("settings"));
    assert!(json_content.contains("X-axis steps per mm"));

    // Clean up
    fs::remove_file(&backup_path).ok();
}
