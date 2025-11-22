//! Firmware capability manager for UI integration
//!
//! Manages firmware capabilities and provides UI-friendly state.
//! Tracks current controller capabilities and notifies UI of changes.

use super::capabilities_db::{CapabilitiesDatabase, FirmwareCapabilities};
use super::firmware_version::{FirmwareType, SemanticVersion};
use std::sync::{Arc, Mutex};

/// UI-friendly capability state
#[derive(Debug, Clone)]
pub struct CapabilityState {
    /// Whether capabilities have been detected
    pub detected: bool,

    /// Current firmware type
    pub firmware_type: Option<FirmwareType>,

    /// Current firmware version
    pub version: Option<SemanticVersion>,

    /// Maximum number of axes
    pub max_axes: u8,

    /// Arc motion (G2/G3) support
    pub supports_arcs: bool,

    /// Probing (G38.x) support
    pub supports_probing: bool,

    /// Tool change (M6) support
    pub supports_tool_change: bool,

    /// Variable spindle speed support
    pub supports_variable_spindle: bool,

    /// Coolant control support
    pub supports_coolant: bool,

    /// Homing cycle support
    pub supports_homing: bool,

    /// Number of work coordinate systems (G54-G59)
    pub coordinate_systems: u8,

    /// Status reports support
    pub supports_status_reports: bool,

    /// Real-time override commands support
    pub supports_overrides: bool,

    /// Soft limits support
    pub supports_soft_limits: bool,

    /// Hard limits support
    pub supports_hard_limits: bool,

    /// Macro support
    pub supports_macros: bool,

    /// Laser mode support (M3/M4 with dynamic power)
    pub supports_laser: bool,

    /// Custom capabilities
    pub custom_capabilities: Vec<(String, bool)>,
}

impl Default for CapabilityState {
    fn default() -> Self {
        Self {
            detected: false,
            firmware_type: None,
            version: None,
            max_axes: 3,
            supports_arcs: false,
            supports_probing: false,
            supports_tool_change: false,
            supports_variable_spindle: false,
            supports_coolant: false,
            supports_homing: false,
            coordinate_systems: 1,
            supports_status_reports: false,
            supports_overrides: false,
            supports_soft_limits: false,
            supports_hard_limits: false,
            supports_macros: false,
            supports_laser: false,
            custom_capabilities: Vec::new(),
        }
    }
}

impl CapabilityState {
    /// Create capability state from firmware capabilities
    fn from_capabilities(
        firmware_type: FirmwareType,
        version: SemanticVersion,
        caps: &FirmwareCapabilities,
    ) -> Self {
        let custom_capabilities: Vec<(String, bool)> =
            caps.custom.iter().map(|(k, v)| (k.clone(), *v)).collect();

        Self {
            detected: true,
            firmware_type: Some(firmware_type),
            version: Some(version),
            max_axes: caps.max_axes,
            supports_arcs: caps.arc_support,
            supports_probing: caps.probing,
            supports_tool_change: caps.tool_change,
            supports_variable_spindle: caps.variable_spindle,
            supports_coolant: caps.coolant_control,
            supports_homing: caps.homing_cycle,
            coordinate_systems: caps.coordinate_systems,
            supports_status_reports: caps.status_reports,
            supports_overrides: caps.realtime_commands,
            supports_soft_limits: caps.soft_limits,
            supports_hard_limits: caps.hard_limits,
            supports_macros: caps.macro_support,
            supports_laser: caps.laser_mode,
            custom_capabilities,
        }
    }

    /// Get a human-readable description of current capabilities
    pub fn get_summary(&self) -> String {
        if !self.detected {
            return "No firmware detected".to_string();
        }

        let firmware_name = self
            .firmware_type
            .map(|f| format!("{}", f))
            .unwrap_or_else(|| "Unknown".to_string());

        let version_str = self
            .version
            .as_ref()
            .map(|v| format!("v{}.{}.{}", v.major, v.minor, v.patch))
            .unwrap_or_else(|| "Unknown".to_string());

        let features = vec![
            if self.supports_arcs { "Arcs" } else { "" },
            if self.supports_probing { "Probing" } else { "" },
            if self.supports_tool_change {
                "Tool Change"
            } else {
                ""
            },
            if self.supports_variable_spindle {
                "Variable Spindle"
            } else {
                ""
            },
            if self.supports_overrides {
                "Overrides"
            } else {
                ""
            },
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(", ");

        format!(
            "{} {} - {} axes | Features: {}",
            firmware_name,
            version_str,
            self.max_axes,
            if features.is_empty() {
                "Basic"
            } else {
                &features
            }
        )
    }
}

/// Manages firmware capabilities for UI integration
pub struct CapabilityManager {
    /// Capabilities database
    database: CapabilitiesDatabase,

    /// Current capability state
    state: Arc<Mutex<CapabilityState>>,
}

impl CapabilityManager {
    /// Create a new capability manager
    pub fn new() -> Self {
        Self {
            database: CapabilitiesDatabase::new(),
            state: Arc::new(Mutex::new(CapabilityState::default())),
        }
    }

    /// Update capabilities based on detected firmware
    pub fn update_firmware(&self, firmware_type: FirmwareType, version: SemanticVersion) {
        if let Some(caps) = self.database.get_capabilities(firmware_type, &version) {
            let new_state = CapabilityState::from_capabilities(firmware_type, version, &caps);

            if let Ok(mut state) = self.state.lock() {
                *state = new_state;
            }
        }
    }

    /// Get current capability state
    pub fn get_state(&self) -> CapabilityState {
        self.state.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// Check if a specific capability is supported
    pub fn supports(&self, capability: &str) -> bool {
        if let Ok(state) = self.state.lock() {
            match capability {
                "arcs" => state.supports_arcs,
                "probing" => state.supports_probing,
                "tool_change" => state.supports_tool_change,
                "variable_spindle" => state.supports_variable_spindle,
                "coolant" => state.supports_coolant,
                "homing" => state.supports_homing,
                "status_reports" => state.supports_status_reports,
                "overrides" => state.supports_overrides,
                "soft_limits" => state.supports_soft_limits,
                "hard_limits" => state.supports_hard_limits,
                "macros" => state.supports_macros,
                _ => state
                    .custom_capabilities
                    .iter()
                    .any(|(k, v)| k == capability && *v),
            }
        } else {
            false
        }
    }

    /// Get maximum number of axes
    pub fn get_max_axes(&self) -> u8 {
        self.state.lock().map(|s| s.max_axes).unwrap_or(3)
    }

    /// Get number of coordinate systems
    pub fn get_coordinate_systems(&self) -> u8 {
        self.state.lock().map(|s| s.coordinate_systems).unwrap_or(1)
    }

    /// Reset to default (disconnected) state
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            *state = CapabilityState::default();
        }
    }

    /// Get shared state reference for thread-safe access
    pub fn get_state_ref(&self) -> Arc<Mutex<CapabilityState>> {
        Arc::clone(&self.state)
    }
}

impl Default for CapabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

