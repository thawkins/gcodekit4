//! GRBL Firmware Settings Management
//!
//! Provides comprehensive firmware settings management for GRBL controllers,
//! including query, parsing, updating, validation, and backup/restore functionality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::warn;

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
        self.backup = Some(self.settings.clone());
    }

    /// Restore settings from backup
    pub fn restore(&mut self) -> anyhow::Result<()> {
        if let Some(backup) = self.backup.take() {
            self.settings = backup;
            self.dirty = true;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No settings backup available"))
        }
    }

    /// Export settings to JSON file
    pub fn export_to_file(&self, path: &Path) -> anyhow::Result<()> {
        let settings_list: Vec<_> = self.settings.values().collect();
        let json = serde_json::to_string_pretty(&settings_list)?;

        fs::write(path, json)?;

        Ok(())
    }

    /// Import settings from JSON file
    pub fn import_from_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let contents = fs::read_to_string(path)?;
        let settings_list: Vec<Setting> = serde_json::from_str(&contents)?;

        self.backup();
        self.settings.clear();

        for setting in settings_list {
            self.settings.insert(setting.number, setting);
        }

        self.dirty = true;

        Ok(())
    }

    /// Clear all cached settings
    pub fn clear(&mut self) {
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

