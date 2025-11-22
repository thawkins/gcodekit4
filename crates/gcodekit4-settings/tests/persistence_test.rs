use gcodekit4_settings::{SettingsPersistence, SettingsDialog, SettingValue, SettingsCategory};

#[test]
fn test_persistence_new() {
    let persistence = SettingsPersistence::new();
    assert!(persistence.validate().is_ok());
}

#[test]
fn test_populate_dialog() {
    let persistence = SettingsPersistence::new();
    let mut dialog = SettingsDialog::new();
    persistence.populate_dialog(&mut dialog);

    assert!(!dialog.settings.is_empty());
    // assert!(dialog.get_setting("connection_type").is_some()); // Moved to DeviceDB
    assert!(dialog.get_setting("theme").is_some());
    assert!(!dialog.shortcuts.is_empty());
}

#[test]
fn test_load_from_dialog() {
    let mut persistence = SettingsPersistence::new();
    let mut dialog = SettingsDialog::new();
    persistence.populate_dialog(&mut dialog);

    // Modify a setting
    if let Some(setting) = dialog.get_setting_mut("theme") {
        setting.value = SettingValue::Enum("Light".to_string(), vec!["Light".to_string()]);
    }

    assert!(persistence.load_from_dialog(&dialog).is_ok());
}

#[test]
fn test_settings_dialog_integration() {
    let persistence = SettingsPersistence::new();
    let mut dialog = SettingsDialog::new();

    // Populate dialog
    persistence.populate_dialog(&mut dialog);

    // Check categories are populated
    let categories = dialog.get_categories();
    // assert!(categories.contains(&SettingsCategory::Controller)); // Moved to DeviceDB
    assert!(categories.contains(&SettingsCategory::UserInterface));
    assert!(categories.contains(&SettingsCategory::FileProcessing));
}

#[test]
fn test_keyboard_shortcuts() {
    let persistence = SettingsPersistence::new();
    let mut dialog = SettingsDialog::new();
    persistence.populate_dialog(&mut dialog);

    assert!(dialog.shortcuts.get("file_open").is_some());
    assert!(dialog.shortcuts.get("machine_home").is_some());
}
