//! Tests for GRBL Firmware Settings (Task 32)
//!
//! Tests GRBL firmware settings management including:
//! - Settings query/parsing
//! - Settings validation
//! - Settings backup/restore
//! - Settings import/export

#[cfg(test)]
mod tests {
    use gcodekit4::firmware::grbl::{Setting, SettingsManager};
    use std::fs;
    use std::path::PathBuf;

    fn get_temp_dir() -> PathBuf {
        PathBuf::from("target/temp")
    }

    fn setup_test_settings() -> SettingsManager {
        let mut manager = SettingsManager::new();

        // Add some test settings
        let settings = vec![
            Setting {
                number: 110,
                name: "X-axis travel resolution".to_string(),
                value: "250.000".to_string(),
                numeric_value: Some(250.0),
                description: "Step/mm".to_string(),
                range: Some((1.0, 10000.0)),
                read_only: false,
            },
            Setting {
                number: 120,
                name: "X-axis maximum rate".to_string(),
                value: "500.000".to_string(),
                numeric_value: Some(500.0),
                description: "mm/min".to_string(),
                range: Some((1.0, 10000.0)),
                read_only: false,
            },
            Setting {
                number: 130,
                name: "X-axis acceleration".to_string(),
                value: "25.000".to_string(),
                numeric_value: Some(25.0),
                description: "mm/sec^2".to_string(),
                range: Some((1.0, 1000.0)),
                read_only: false,
            },
        ];

        for setting in settings {
            manager.set_setting(setting);
        }

        manager
    }

    #[test]
    fn test_settings_manager_creation() {
        let manager = SettingsManager::new();
        assert_eq!(manager.count(), 0);
        assert!(!manager.is_dirty());
    }

    #[test]
    fn test_add_and_retrieve_setting() {
        let mut manager = SettingsManager::new();
        let setting = Setting {
            number: 110,
            name: "Test Setting".to_string(),
            value: "123".to_string(),
            numeric_value: Some(123.0),
            description: "A test setting".to_string(),
            range: None,
            read_only: false,
        };

        manager.set_setting(setting);
        assert_eq!(manager.count(), 1);
        assert!(manager.is_dirty());

        let retrieved = manager.get_setting(110);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Setting");
    }

    #[test]
    fn test_parse_setting_line_valid() {
        let result = SettingsManager::parse_setting_line("$110=500.000");
        assert!(result.is_some());
        let (num, val) = result.unwrap();
        assert_eq!(num, 110);
        assert_eq!(val, "500.000");
    }

    #[test]
    fn test_parse_setting_line_invalid() {
        let result = SettingsManager::parse_setting_line("invalid");
        assert!(result.is_none());
    }

    #[test]
    fn test_settings_backup() {
        let mut manager = setup_test_settings();
        assert_eq!(manager.count(), 3);

        manager.reset_dirty();
        manager.backup();

        // Should have backup now
        // Clear settings
        manager.clear();
        assert_eq!(manager.count(), 0);

        // Restore
        let result = manager.restore();
        assert!(result.is_ok());
        assert_eq!(manager.count(), 3);
    }

    #[test]
    fn test_settings_restore_without_backup() {
        let mut manager = SettingsManager::new();
        let result = manager.restore();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_setting_read_only() {
        let mut manager = SettingsManager::new();
        let setting = Setting {
            number: 110,
            name: "Read-Only Setting".to_string(),
            value: "123".to_string(),
            numeric_value: Some(123.0),
            description: "Test".to_string(),
            range: None,
            read_only: true,
        };

        manager.set_setting(setting);
        let result = manager.validate_setting(110, "456");
        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .contains("read-only"));
    }

    #[test]
    fn test_validate_setting_range() {
        let mut manager = SettingsManager::new();
        let setting = Setting {
            number: 120,
            name: "Rate Setting".to_string(),
            value: "500".to_string(),
            numeric_value: Some(500.0),
            description: "Test".to_string(),
            range: Some((100.0, 1000.0)),
            read_only: false,
        };

        manager.set_setting(setting);

        // Valid value
        assert!(manager.validate_setting(120, "500").is_ok());

        // Out of range
        let result = manager.validate_setting(120, "2000");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_sorted_settings() {
        let manager = setup_test_settings();
        let sorted = manager.get_sorted_settings();

        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].number, 110);
        assert_eq!(sorted[1].number, 120);
        assert_eq!(sorted[2].number, 130);
    }

    #[test]
    fn test_find_by_name() {
        let manager = setup_test_settings();
        let results = manager.find_by_name("rate");

        assert!(results.len() >= 1);
        assert!(results.iter().any(|s| s.number == 120));
    }

    #[test]
    fn test_export_import_settings() {
        // Ensure temp directory exists
        let temp_dir = get_temp_dir();
        let _ = fs::create_dir_all(&temp_dir);

        let file_path = temp_dir.join("test_settings.json");

        let mut manager = setup_test_settings();
        let export_result = manager.export_to_file(&file_path);
        assert!(export_result.is_ok(), "Export failed: {:?}", export_result);

        // Verify file exists
        assert!(file_path.exists());

        // Import into new manager
        let mut new_manager = SettingsManager::new();
        let import_result = new_manager.import_from_file(&file_path);
        assert!(import_result.is_ok(), "Import failed: {:?}", import_result);

        // Verify settings were imported
        assert_eq!(new_manager.count(), 3);
        assert!(new_manager.get_setting(110).is_some());
        assert!(new_manager.get_setting(120).is_some());
        assert!(new_manager.get_setting(130).is_some());

        // Cleanup
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_get_value_numeric() {
        let manager = setup_test_settings();
        let value = manager.get_value(110);
        assert!(value.is_some());
        assert_eq!(value.unwrap(), 250.0);
    }

    #[test]
    fn test_get_string_value() {
        let manager = setup_test_settings();
        let value = manager.get_string_value(110);
        assert!(value.is_some());
        assert_eq!(value.unwrap(), "250.000");
    }

    #[test]
    fn test_get_all_settings() {
        let manager = setup_test_settings();
        let all_settings = manager.get_all_settings();
        assert_eq!(all_settings.len(), 3);
    }

    #[test]
    fn test_clear_settings() {
        let mut manager = setup_test_settings();
        assert_eq!(manager.count(), 3);

        manager.clear();
        assert_eq!(manager.count(), 0);
        assert!(manager.is_dirty());
    }

    #[test]
    fn test_dirty_flag_management() {
        let mut manager = SettingsManager::new();
        assert!(!manager.is_dirty());

        let setting = Setting {
            number: 110,
            name: "Test".to_string(),
            value: "123".to_string(),
            numeric_value: Some(123.0),
            description: "Test".to_string(),
            range: None,
            read_only: false,
        };

        manager.set_setting(setting);
        assert!(manager.is_dirty());

        manager.reset_dirty();
        assert!(!manager.is_dirty());
    }
}
