//! Firmware Settings Panel - Task 79
//!
//! Display and manage firmware-specific parameters with validation,
//! descriptions, save/restore functionality

use std::collections::HashMap;

/// Parameter data type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParameterType {
    /// Integer parameter
    Integer,
    /// Float parameter
    Float,
    /// Boolean parameter
    Boolean,
    /// String parameter
    String,
}

/// Firmware parameter definition
#[derive(Debug, Clone)]
pub struct FirmwareParameter {
    /// Parameter code/ID (e.g., "$0" for GRBL step pulse)
    pub code: String,
    /// Parameter display name
    pub name: String,
    /// Current value
    pub value: String,
    /// Default value
    pub default_value: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Description of what this parameter does
    pub description: Option<String>,
    /// Minimum value (for numeric types)
    pub min_value: Option<f64>,
    /// Maximum value (for numeric types)
    pub max_value: Option<f64>,
    /// Unit (e.g., "mm/min", "Hz")
    pub unit: Option<String>,
    /// Allowed string values (for enum-like parameters)
    pub allowed_values: Option<Vec<String>>,
    /// Whether parameter can be edited
    pub editable: bool,
    /// Whether parameter has been modified
    pub modified: bool,
}

impl FirmwareParameter {
    /// Create new firmware parameter
    pub fn new(code: impl Into<String>, name: impl Into<String>, value: impl Into<String>) -> Self {
        let value_str = value.into();
        Self {
            code: code.into(),
            name: name.into(),
            value: value_str.clone(),
            default_value: value_str,
            param_type: ParameterType::String,
            description: None,
            min_value: None,
            max_value: None,
            unit: None,
            allowed_values: None,
            editable: true,
            modified: false,
        }
    }

    /// Set parameter type
    pub fn with_type(mut self, param_type: ParameterType) -> Self {
        self.param_type = param_type;
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set min and max values
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min_value = Some(min);
        self.max_value = Some(max);
        self
    }

    /// Set unit
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set allowed values
    pub fn with_allowed_values(mut self, values: Vec<String>) -> Self {
        self.allowed_values = Some(values);
        self
    }

    /// Set as read-only
    pub fn read_only(mut self) -> Self {
        self.editable = false;
        self
    }

    /// Update value with validation
    pub fn set_value(&mut self, new_value: impl Into<String>) -> Result<(), String> {
        let value_str = new_value.into();

        if !self.editable {
            return Err("Parameter is read-only".to_string());
        }

        match self.param_type {
            ParameterType::Integer => {
                let val: i32 = value_str
                    .parse()
                    .map_err(|_| "Invalid integer value".to_string())?;
                if let Some(min) = self.min_value {
                    if (val as f64) < min {
                        return Err(format!("Value must be >= {}", min));
                    }
                }
                if let Some(max) = self.max_value {
                    if (val as f64) > max {
                        return Err(format!("Value must be <= {}", max));
                    }
                }
            }
            ParameterType::Float => {
                let val: f64 = value_str
                    .parse()
                    .map_err(|_| "Invalid float value".to_string())?;
                if let Some(min) = self.min_value {
                    if val < min {
                        return Err(format!("Value must be >= {}", min));
                    }
                }
                if let Some(max) = self.max_value {
                    if val > max {
                        return Err(format!("Value must be <= {}", max));
                    }
                }
            }
            ParameterType::Boolean => {
                if !["true", "false", "0", "1", "yes", "no"]
                    .contains(&value_str.to_lowercase().as_str())
                {
                    return Err("Invalid boolean value".to_string());
                }
            }
            ParameterType::String => {
                if let Some(ref allowed) = self.allowed_values {
                    if !allowed.contains(&value_str) {
                        return Err(format!("Invalid value. Allowed: {:?}", allowed));
                    }
                }
            }
        }

        self.value = value_str;
        self.modified = true;
        Ok(())
    }

    /// Reset to default value
    pub fn reset_to_default(&mut self) {
        self.value = self.default_value.clone();
        self.modified = false;
    }

    /// Check if value changed
    pub fn is_changed(&self) -> bool {
        self.value != self.default_value
    }
}

/// Firmware settings panel
#[derive(Debug, Clone)]
pub struct FirmwareSettingsPanel {
    /// All firmware parameters
    pub parameters: HashMap<String, FirmwareParameter>,
    /// Firmware type/version
    pub firmware_type: String,
    /// Firmware version
    pub firmware_version: String,
    /// Whether parameters are loaded
    pub loaded: bool,
    /// Backup of original parameters
    backup: Option<HashMap<String, FirmwareParameter>>,
}

impl FirmwareSettingsPanel {
    /// Create new firmware settings panel
    pub fn new(firmware_type: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            parameters: HashMap::new(),
            firmware_type: firmware_type.into(),
            firmware_version: version.into(),
            loaded: false,
            backup: None,
        }
    }

    /// Add parameter
    pub fn add_parameter(&mut self, param: FirmwareParameter) {
        self.parameters.insert(param.code.clone(), param);
    }

    /// Get parameter by code
    pub fn get_parameter(&self, code: &str) -> Option<&FirmwareParameter> {
        self.parameters.get(code)
    }

    /// Get mutable parameter
    pub fn get_parameter_mut(&mut self, code: &str) -> Option<&mut FirmwareParameter> {
        self.parameters.get_mut(code)
    }

    /// List all parameters
    pub fn list_parameters(&self) -> Vec<&FirmwareParameter> {
        let mut params: Vec<_> = self.parameters.values().collect();
        params.sort_by(|a, b| a.code.cmp(&b.code));
        params
    }

    /// Set parameter value with validation
    pub fn set_parameter_value(
        &mut self,
        code: &str,
        value: impl Into<String>,
    ) -> Result<(), String> {
        if let Some(param) = self.parameters.get_mut(code) {
            param.set_value(value)
        } else {
            Err(format!("Parameter {} not found", code))
        }
    }

    /// Get parameters with changes
    pub fn get_modified_parameters(&self) -> Vec<&FirmwareParameter> {
        self.parameters
            .values()
            .filter(|p| p.is_changed())
            .collect()
    }

    /// Create backup of current parameters
    pub fn create_backup(&mut self) {
        self.backup = Some(
            self.parameters
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        );
    }

    /// Restore from backup
    pub fn restore_backup(&mut self) -> Result<(), String> {
        if let Some(backup) = self.backup.take() {
            self.parameters = backup;
            Ok(())
        } else {
            Err("No backup available".to_string())
        }
    }

    /// Reset all parameters to defaults
    pub fn reset_all_to_defaults(&mut self) {
        for param in self.parameters.values_mut() {
            param.reset_to_default();
        }
    }

    /// Mark all as loaded from device
    pub fn mark_loaded(&mut self) {
        self.loaded = true;
        for param in self.parameters.values_mut() {
            param.default_value = param.value.clone();
            param.modified = false;
        }
    }

    /// Check if has changes
    pub fn has_changes(&self) -> bool {
        self.parameters.values().any(|p| p.is_changed())
    }

    /// Export parameters to JSON
    pub fn export_parameters(&self) -> Result<String, serde_json::Error> {
        let export_map: HashMap<String, String> = self
            .parameters
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect();
        serde_json::to_string_pretty(&export_map)
    }

    /// Import parameters from JSON (for loading device settings)
    pub fn import_parameters(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let import_map: HashMap<String, String> = serde_json::from_str(json)?;
        for (code, value) in import_map {
            if let Some(param) = self.parameters.get_mut(&code) {
                param.value = value;
                param.default_value = param.value.clone();
            }
        }
        self.mark_loaded();
        Ok(())
    }
}

impl Default for FirmwareSettingsPanel {
    fn default() -> Self {
        Self::new("Unknown", "0.0.0")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_creation() {
        let param = FirmwareParameter::new("$0", "Step Pulse", "10")
            .with_type(ParameterType::Integer)
            .with_range(1.0, 127.0)
            .with_description("Microseconds");

        assert_eq!(param.code, "$0");
        assert_eq!(param.value, "10");
        assert!(!param.is_changed());
    }

    #[test]
    fn test_parameter_validation() {
        let mut param = FirmwareParameter::new("$0", "Step Pulse", "10")
            .with_type(ParameterType::Integer)
            .with_range(1.0, 127.0);

        assert!(param.set_value("50").is_ok());
        assert!(param.set_value("200").is_err());
        assert!(param.set_value("invalid").is_err());
    }

    #[test]
    fn test_parameter_readonly() {
        let mut param = FirmwareParameter::new("$0", "Step Pulse", "10").read_only();

        assert!(param.set_value("20").is_err());
    }

    #[test]
    fn test_firmware_settings_panel() {
        let mut panel = FirmwareSettingsPanel::new("GRBL", "1.1");
        let param =
            FirmwareParameter::new("$0", "Step Pulse", "10").with_type(ParameterType::Integer);
        panel.add_parameter(param);

        assert_eq!(panel.list_parameters().len(), 1);
        assert!(panel.get_parameter("$0").is_some());
    }

    #[test]
    fn test_firmware_settings_backup() {
        let mut panel = FirmwareSettingsPanel::new("GRBL", "1.1");
        let param = FirmwareParameter::new("$0", "Step Pulse", "10");
        panel.add_parameter(param);

        panel.create_backup();
        panel.set_parameter_value("$0", "20").unwrap();
        assert!(panel.has_changes());

        panel.restore_backup().unwrap();
        assert_eq!(panel.get_parameter("$0").unwrap().value, "10");
    }

    #[test]
    fn test_firmware_settings_export_import() {
        let mut panel = FirmwareSettingsPanel::new("GRBL", "1.1");
        let param = FirmwareParameter::new("$0", "Step Pulse", "10");
        panel.add_parameter(param);

        let json = panel.export_parameters().unwrap();
        let mut panel2 = FirmwareSettingsPanel::new("GRBL", "1.1");
        let param2 = FirmwareParameter::new("$0", "Step Pulse", "5");
        panel2.add_parameter(param2);

        panel2.import_parameters(&json).unwrap();
        assert_eq!(panel2.get_parameter("$0").unwrap().value, "10");
    }
}
