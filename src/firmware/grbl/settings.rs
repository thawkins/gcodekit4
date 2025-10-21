//! GRBL Firmware Settings Management
//!
//! Provides comprehensive firmware settings management for GRBL controllers,
//! including query, parsing, updating, validation, and backup/restore functionality.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// GRBL firmware setting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    /// Setting number/ID
    pub number: u8,
    /// Setting name
    pub name: String,
    /// Current value as string
    pub value: String,
    /// Parsed numeric value
    pub numeric_value: Option<f64>,
    /// Setting description
    pub description: String,
    /// Valid value range (min, max) if applicable
    pub range: Option<(f64, f64)>,
    /// Is this setting read-only
    pub read_only: bool,
}

/// GRBL Settings Manager
///
/// Handles querying, parsing, updating, and managing GRBL firmware settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsManager {
    /// All loaded settings
    settings: HashMap<u8, Setting>,
    /// Backup settings
    backup: Option<HashMap<u8, Setting>>,
    /// Settings changed since last load
    dirty: bool,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
            backup: None,
            dirty: false,
        }
    }

    /// Add or update a setting
    pub fn set_setting(&mut self, setting: Setting) {
        debug!(
            "Adding/updating setting {} ({}): {}",
            setting.number, setting.name, setting.value
        );
        self.dirty = true;
        self.settings.insert(setting.number, setting);
    }

    /// Get a setting by number
    pub fn get_setting(&self, number: u8) -> Option<&Setting> {
        self.settings.get(&number)
    }

    /// Get all settings
    pub fn get_all_settings(&self) -> Vec<&Setting> {
        self.settings.values().collect()
    }

    /// Get a setting's numeric value
    pub fn get_value(&self, number: u8) -> Option<f64> {
        self.settings.get(&number).and_then(|s| s.numeric_value)
    }

    /// Get a setting's string value
    pub fn get_string_value(&self, number: u8) -> Option<String> {
        self.settings.get(&number).map(|s| s.value.clone())
    }

    /// Parse a GRBL settings response line ($=value format)
    ///
    /// # Arguments
    /// * `line` - The settings line from GRBL (e.g., "$110=500.000")
    ///
    /// # Returns
    /// * Setting if parsed successfully, None otherwise
    pub fn parse_setting_line(line: &str) -> Option<(u8, String)> {
        if !line.starts_with('$') {
            return None;
        }

        let line = &line[1..];
        if let Some(eq_pos) = line.find('=') {
            let num_str = &line[..eq_pos];
            let value_str = &line[eq_pos + 1..];

            if let Ok(number) = num_str.parse::<u8>() {
                return Some((number, value_str.to_string()));
            }
        }

        None
    }

    /// Validate a setting value
    pub fn validate_setting(&self, number: u8, value: &str) -> anyhow::Result<()> {
        if let Some(setting) = self.settings.get(&number) {
            if setting.read_only {
                return Err(anyhow::anyhow!("Setting {} is read-only", number));
            }

            if let Some((min, max)) = setting.range {
                if let Ok(numeric_val) = value.parse::<f64>() {
                    if numeric_val < min || numeric_val > max {
                        return Err(anyhow::anyhow!(
                            "Setting {} value {} is out of range [{}, {}]",
                            number,
                            numeric_val,
                            min,
                            max
                        ));
                    }
                } else {
                    return Err(anyhow::anyhow!(
                        "Setting {} value '{}' is not numeric",
                        number,
                        value
                    ));
                }
            }
        } else {
            warn!("Setting {} not found in manager", number);
        }

        Ok(())
    }

    /// Create a backup of current settings
    pub fn backup(&mut self) {
        debug!("Creating settings backup");
        self.backup = Some(self.settings.clone());
        info!("Settings backup created with {} entries", self.settings.len());
    }

    /// Restore settings from backup
    pub fn restore(&mut self) -> anyhow::Result<()> {
        if let Some(backup) = self.backup.take() {
            debug!("Restoring settings from backup");
            self.settings = backup;
            self.dirty = true;
            info!("Settings restored from backup");
            Ok(())
        } else {
            Err(anyhow::anyhow!("No settings backup available"))
        }
    }

    /// Export settings to JSON file
    pub fn export_to_file(&self, path: &Path) -> anyhow::Result<()> {
        debug!("Exporting settings to file: {:?}", path);

        let settings_list: Vec<_> = self.settings.values().collect();
        let json = serde_json::to_string_pretty(&settings_list)?;

        fs::write(path, json)?;
        info!("Settings exported to {:?}", path);

        Ok(())
    }

    /// Import settings from JSON file
    pub fn import_from_file(&mut self, path: &Path) -> anyhow::Result<()> {
        debug!("Importing settings from file: {:?}", path);

        let contents = fs::read_to_string(path)?;
        let settings_list: Vec<Setting> = serde_json::from_str(&contents)?;

        self.backup();
        self.settings.clear();

        for setting in settings_list {
            self.settings.insert(setting.number, setting);
        }

        self.dirty = true;
        info!(
            "Settings imported from {:?} ({} entries)",
            path,
            self.settings.len()
        );

        Ok(())
    }

    /// Clear all cached settings
    pub fn clear(&mut self) {
        debug!("Clearing all settings");
        self.settings.clear();
        self.dirty = true;
    }

    /// Get settings count
    pub fn count(&self) -> usize {
        self.settings.len()
    }

    /// Check if settings have been modified
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Reset dirty flag
    pub fn reset_dirty(&mut self) {
        self.dirty = false;
    }

    /// Get a sorted list of all settings
    pub fn get_sorted_settings(&self) -> Vec<&Setting> {
        let mut settings: Vec<_> = self.settings.values().collect();
        settings.sort_by_key(|s| s.number);
        settings
    }

    /// Find settings by name pattern
    pub fn find_by_name(&self, pattern: &str) -> Vec<&Setting> {
        self.settings
            .values()
            .filter(|s| s.name.contains(pattern))
            .collect()
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_setting_line_valid() {
        let result = SettingsManager::parse_setting_line("$110=500.000");
        assert!(result.is_some());
        let (num, val) = result.unwrap();
        assert_eq!(num, 110);
        assert_eq!(val, "500.000");
    }

    #[test]
    fn test_parse_setting_line_invalid_format() {
        let result = SettingsManager::parse_setting_line("invalid");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_setting_line_no_dollar() {
        let result = SettingsManager::parse_setting_line("110=500");
        assert!(result.is_none());
    }

    #[test]
    fn test_settings_manager_creation() {
        let manager = SettingsManager::new();
        assert_eq!(manager.count(), 0);
        assert!(!manager.is_dirty());
    }

    #[test]
    fn test_add_setting() {
        let mut manager = SettingsManager::new();
        let setting = Setting {
            number: 110,
            name: "Baud Rate".to_string(),
            value: "115200".to_string(),
            numeric_value: Some(115200.0),
            description: "Serial communication speed".to_string(),
            range: Some((9600.0, 115200.0)),
            read_only: false,
        };

        manager.set_setting(setting);
        assert_eq!(manager.count(), 1);
        assert!(manager.is_dirty());

        let retrieved = manager.get_setting(110);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Baud Rate");
    }

    #[test]
    fn test_backup_and_restore() {
        let mut manager = SettingsManager::new();
        let setting = Setting {
            number: 110,
            name: "Baud Rate".to_string(),
            value: "115200".to_string(),
            numeric_value: Some(115200.0),
            description: "Serial communication speed".to_string(),
            range: Some((9600.0, 115200.0)),
            read_only: false,
        };

        manager.set_setting(setting);
        manager.reset_dirty();
        manager.backup();

        // Modify
        manager.clear();
        assert_eq!(manager.count(), 0);

        // Restore
        let result = manager.restore();
        assert!(result.is_ok());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_validate_setting_read_only() {
        let mut manager = SettingsManager::new();
        let setting = Setting {
            number: 110,
            name: "Baud Rate".to_string(),
            value: "115200".to_string(),
            numeric_value: Some(115200.0),
            description: "Serial communication speed".to_string(),
            range: Some((9600.0, 115200.0)),
            read_only: true,
        };

        manager.set_setting(setting);
        let result = manager.validate_setting(110, "57600");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_setting_range() {
        let mut manager = SettingsManager::new();
        let setting = Setting {
            number: 110,
            name: "Baud Rate".to_string(),
            value: "115200".to_string(),
            numeric_value: Some(115200.0),
            description: "Serial communication speed".to_string(),
            range: Some((9600.0, 115200.0)),
            read_only: false,
        };

        manager.set_setting(setting);

        // Valid value
        assert!(manager.validate_setting(110, "57600").is_ok());

        // Out of range
        assert!(manager.validate_setting(110, "500000").is_err());
    }

    #[test]
    fn test_get_sorted_settings() {
        let mut manager = SettingsManager::new();
        for i in [120u8, 110, 130].iter() {
            let setting = Setting {
                number: *i,
                name: format!("Setting {}", i),
                value: "test".to_string(),
                numeric_value: None,
                description: "Test setting".to_string(),
                range: None,
                read_only: false,
            };
            manager.set_setting(setting);
        }

        let sorted = manager.get_sorted_settings();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].number, 110);
        assert_eq!(sorted[1].number, 120);
        assert_eq!(sorted[2].number, 130);
    }

    #[test]
    fn test_find_by_name() {
        let mut manager = SettingsManager::new();
        let setting1 = Setting {
            number: 110,
            name: "Step Pulse Duration".to_string(),
            value: "10".to_string(),
            numeric_value: Some(10.0),
            description: "Pulse duration".to_string(),
            range: None,
            read_only: false,
        };

        let setting2 = Setting {
            number: 111,
            name: "Idle Delay".to_string(),
            value: "25".to_string(),
            numeric_value: Some(25.0),
            description: "Idle delay".to_string(),
            range: None,
            read_only: false,
        };

        manager.set_setting(setting1);
        manager.set_setting(setting2);

        let results = manager.find_by_name("Pulse");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].number, 110);
    }
}
