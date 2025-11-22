use gcodekit4_settings::{SettingsDialog, Setting, SettingValue, SettingsCategory, KeyboardShortcut};

#[test]
fn test_keyboard_shortcut() {
    let shortcut = KeyboardShortcut::new("file_open", "Open File", "Ctrl+O");
    assert_eq!(shortcut.action_id, "file_open");
    assert_eq!(shortcut.keys, "Ctrl+O");
    assert!(shortcut.customizable);
}

#[test]
fn test_setting_creation() {
    let setting = Setting::new("theme", "Theme", SettingValue::String("dark".to_string()))
        .with_description("Application theme")
        .with_category(SettingsCategory::UserInterface);

    assert_eq!(setting.id, "theme");
    assert!(!setting.is_changed());
}

#[test]
fn test_setting_value_change() {
    let mut setting = Setting::new("theme", "Theme", SettingValue::String("dark".to_string()));
    setting.value = SettingValue::String("light".to_string());
    assert!(setting.is_changed());

    setting.reset_to_default();
    assert!(!setting.is_changed());
}

#[test]
fn test_settings_dialog() {
    let mut dialog = SettingsDialog::new();
    let setting = Setting::new("theme", "Theme", SettingValue::String("dark".to_string()))
        .with_category(SettingsCategory::UserInterface);

    dialog.add_setting(setting);
    assert_eq!(
        dialog
            .get_settings_for_category(&SettingsCategory::UserInterface)
            .len(),
        1
    );
}

#[test]
fn test_settings_export_import() {
    let mut dialog = SettingsDialog::new();
    let setting = Setting::new("theme", "Theme", SettingValue::String("dark".to_string()));
    dialog.add_setting(setting);

    let json = dialog.export_settings().unwrap();
    let mut dialog2 = SettingsDialog::new();
    let setting2 = Setting::new("theme", "Theme", SettingValue::String("light".to_string()));
    dialog2.add_setting(setting2);
    dialog2.import_settings(&json).unwrap();

    assert_eq!(dialog2.get_setting("theme").unwrap().value.as_str(), "dark");
}
