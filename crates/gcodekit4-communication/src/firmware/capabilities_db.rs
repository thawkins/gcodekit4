//! Firmware capabilities database
//!
//! Track and query firmware capabilities by version.
//! Enables version-aware UI and G-code generation.

use super::firmware_version::{FirmwareType, SemanticVersion};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Individual capability information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityInfo {
    /// Whether capability is enabled
    pub enabled: bool,
    /// Minimum version where this capability is available
    pub min_version: SemanticVersion,
    /// Maximum version where this capability is available (None = current)
    pub max_version: Option<SemanticVersion>,
    /// Notes about this capability
    pub notes: String,
}

impl CapabilityInfo {
    /// Check if this capability is available for a given version
    pub fn is_available_for(&self, version: &SemanticVersion) -> bool {
        if !self.enabled {
            return false;
        }

        if !version.is_at_least(&self.min_version) {
            return false;
        }

        if let Some(max_ver) = self.max_version {
            if version > &max_ver {
                return false;
            }
        }

        true
    }
}

/// Complete firmware capabilities for a specific version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareCapabilities {
    /// Firmware type
    pub firmware_type: FirmwareType,
    /// Firmware version
    pub version: SemanticVersion,

    // Core Motion
    pub max_axes: u8,
    pub arc_support: bool,
    pub inverse_time_feed: bool,
    pub feed_per_revolution: bool,

    // Spindle
    pub variable_spindle: bool,
    pub spindle_direction: bool,
    pub spindle_css: bool,
    pub laser_mode: bool,

    // Tool Management
    pub tool_change: bool,
    pub tool_length_offset: bool,
    pub tool_diameter_offset: bool,

    // Probing
    pub probing: bool,
    pub probe_away: bool,

    // Coolant
    pub coolant_control: bool,
    pub mist_control: bool,

    // Homing
    pub homing_cycle: bool,
    pub soft_homing: bool,
    pub hard_homing: bool,

    // Offsets
    pub coordinate_systems: u8,
    pub local_offsets: bool,
    pub cutter_radius_compensation: bool,

    // Advanced
    pub macro_support: bool,
    pub conditional_blocks: bool,
    pub variable_support: bool,

    // Communication
    pub status_reports: bool,
    pub realtime_commands: bool,
    pub flow_control: bool,

    // Safety
    pub soft_limits: bool,
    pub hard_limits: bool,
    pub alarm_conditions: bool,
    pub door_interlock: bool,

    // Custom capabilities
    pub custom: HashMap<String, bool>,
}

impl FirmwareCapabilities {
    /// Create new capabilities with defaults
    pub fn new(firmware_type: FirmwareType, version: SemanticVersion) -> Self {
        Self {
            firmware_type,
            version,
            max_axes: 3,
            arc_support: false,
            inverse_time_feed: false,
            feed_per_revolution: false,
            variable_spindle: false,
            spindle_direction: false,
            spindle_css: false,
            laser_mode: false,
            tool_change: false,
            tool_length_offset: false,
            tool_diameter_offset: false,
            probing: false,
            probe_away: false,
            coolant_control: false,
            mist_control: false,
            homing_cycle: false,
            soft_homing: false,
            hard_homing: false,
            coordinate_systems: 1,
            local_offsets: false,
            cutter_radius_compensation: false,
            macro_support: false,
            conditional_blocks: false,
            variable_support: false,
            status_reports: false,
            realtime_commands: false,
            flow_control: false,
            soft_limits: false,
            hard_limits: false,
            alarm_conditions: false,
            door_interlock: false,
            custom: HashMap::new(),
        }
    }

    /// Check if a capability is supported
    pub fn supports(&self, capability: &str) -> bool {
        match capability {
            "arc" => self.arc_support,
            "spindle_variable" => self.variable_spindle,
            "spindle_direction" => self.spindle_direction,
            "tool_change" => self.tool_change,
            "probing" => self.probing,
            "coolant" => self.coolant_control,
            "homing" => self.homing_cycle,
            "status_reports" => self.status_reports,
            "realtime_commands" => self.realtime_commands,
            "soft_limits" => self.soft_limits,
            "hard_limits" => self.hard_limits,
            _ => self.custom.get(capability).copied().unwrap_or(false),
        }
    }
}

/// Firmware capabilities database
pub struct CapabilitiesDatabase {
    /// Predefined capabilities by firmware type and version
    database: HashMap<(FirmwareType, String), FirmwareCapabilities>,
}

impl CapabilitiesDatabase {
    /// Create new database with built-in capabilities
    pub fn new() -> Self {
        let mut db = Self {
            database: HashMap::new(),
        };

        // Initialize with built-in firmware profiles
        db.init_grbl_profiles();
        db.init_tinyg_profiles();
        db.init_g2core_profiles();
        db.init_smoothieware_profiles();
        db.init_fluidnc_profiles();

        db
    }

    /// Initialize GRBL capability profiles
    fn init_grbl_profiles(&mut self) {
        // GRBL 0.9
        let mut grbl_0_9 =
            FirmwareCapabilities::new(FirmwareType::Grbl, SemanticVersion::new(0, 9, 0));
        grbl_0_9.max_axes = 3;
        grbl_0_9.variable_spindle = true;
        grbl_0_9.spindle_direction = true;
        grbl_0_9.homing_cycle = true;
        grbl_0_9.soft_homing = true;
        grbl_0_9.soft_limits = true;
        grbl_0_9.alarm_conditions = true;
        self.database
            .insert((FirmwareType::Grbl, "0.9".to_string()), grbl_0_9);

        // GRBL 1.0
        let mut grbl_1_0 =
            FirmwareCapabilities::new(FirmwareType::Grbl, SemanticVersion::new(1, 0, 0));
        grbl_1_0.max_axes = 3;
        grbl_1_0.arc_support = true;
        grbl_1_0.variable_spindle = true;
        grbl_1_0.spindle_direction = true;
        grbl_1_0.tool_change = true;
        grbl_1_0.probing = true;
        grbl_1_0.homing_cycle = true;
        grbl_1_0.soft_homing = true;
        grbl_1_0.coordinate_systems = 6;
        grbl_1_0.soft_limits = true;
        grbl_1_0.alarm_conditions = true;
        grbl_1_0.flow_control = true;
        self.database
            .insert((FirmwareType::Grbl, "1.0".to_string()), grbl_1_0);

        // GRBL 1.1
        let mut grbl_1_1 =
            FirmwareCapabilities::new(FirmwareType::Grbl, SemanticVersion::new(1, 1, 0));
        grbl_1_1.max_axes = 3;
        grbl_1_1.arc_support = true;
        grbl_1_1.variable_spindle = true;
        grbl_1_1.spindle_direction = true;
        grbl_1_1.laser_mode = true;
        grbl_1_1.tool_change = true;
        grbl_1_1.probing = true;
        grbl_1_1.probe_away = true;
        grbl_1_1.homing_cycle = true;
        grbl_1_1.soft_homing = true;
        grbl_1_1.coordinate_systems = 6;
        grbl_1_1.cutter_radius_compensation = true;
        grbl_1_1.soft_limits = true;
        grbl_1_1.hard_limits = true;
        grbl_1_1.alarm_conditions = true;
        grbl_1_1.door_interlock = true;
        grbl_1_1.status_reports = true;
        grbl_1_1.realtime_commands = true;
        grbl_1_1.flow_control = true;
        self.database
            .insert((FirmwareType::Grbl, "1.1".to_string()), grbl_1_1);

        // GRBL 1.2
        let mut grbl_1_2 =
            FirmwareCapabilities::new(FirmwareType::Grbl, SemanticVersion::new(1, 2, 0));
        grbl_1_2.max_axes = 3;
        grbl_1_2.arc_support = true;
        grbl_1_2.variable_spindle = true;
        grbl_1_2.spindle_direction = true;
        grbl_1_2.laser_mode = true;
        grbl_1_2.tool_change = true;
        grbl_1_2.probing = true;
        grbl_1_2.probe_away = true;
        grbl_1_2.homing_cycle = true;
        grbl_1_2.soft_homing = true;
        grbl_1_2.coordinate_systems = 6;
        grbl_1_2.cutter_radius_compensation = true;
        grbl_1_2.soft_limits = true;
        grbl_1_2.hard_limits = true;
        grbl_1_2.alarm_conditions = true;
        grbl_1_2.door_interlock = true;
        grbl_1_2.status_reports = true;
        grbl_1_2.realtime_commands = true;
        grbl_1_2.flow_control = true;
        self.database
            .insert((FirmwareType::Grbl, "1.2".to_string()), grbl_1_2);

        // GRBL 1.3
        let mut grbl_1_3 =
            FirmwareCapabilities::new(FirmwareType::Grbl, SemanticVersion::new(1, 3, 0));
        grbl_1_3.max_axes = 3;
        grbl_1_3.arc_support = true;
        grbl_1_3.variable_spindle = true;
        grbl_1_3.spindle_direction = true;
        grbl_1_3.laser_mode = true;
        grbl_1_3.tool_change = true;
        grbl_1_3.probing = true;
        grbl_1_3.probe_away = true;
        grbl_1_3.homing_cycle = true;
        grbl_1_3.soft_homing = true;
        grbl_1_3.coordinate_systems = 6;
        grbl_1_3.cutter_radius_compensation = true;
        grbl_1_3.soft_limits = true;
        grbl_1_3.hard_limits = true;
        grbl_1_3.alarm_conditions = true;
        grbl_1_3.door_interlock = true;
        grbl_1_3.status_reports = true;
        grbl_1_3.realtime_commands = true;
        grbl_1_3.flow_control = true;
        self.database
            .insert((FirmwareType::Grbl, "1.3".to_string()), grbl_1_3);
    }

    /// Initialize TinyG capability profiles
    fn init_tinyg_profiles(&mut self) {
        let mut tinyg =
            FirmwareCapabilities::new(FirmwareType::TinyG, SemanticVersion::new(2, 0, 0));
        tinyg.max_axes = 4;
        tinyg.arc_support = true;
        tinyg.inverse_time_feed = true;
        tinyg.feed_per_revolution = true;
        tinyg.variable_spindle = true;
        tinyg.spindle_direction = true;
        tinyg.spindle_css = true;
        tinyg.tool_change = true;
        tinyg.tool_length_offset = true;
        tinyg.probing = true;
        tinyg.probe_away = true;
        tinyg.coolant_control = true;
        tinyg.mist_control = true;
        tinyg.homing_cycle = true;
        tinyg.soft_homing = true;
        tinyg.hard_homing = true;
        tinyg.coordinate_systems = 9;
        tinyg.local_offsets = true;
        tinyg.cutter_radius_compensation = true;
        tinyg.macro_support = true;
        tinyg.conditional_blocks = true;
        tinyg.variable_support = true;
        tinyg.status_reports = true;
        tinyg.realtime_commands = true;
        tinyg.flow_control = true;
        tinyg.soft_limits = true;
        tinyg.hard_limits = true;
        tinyg.alarm_conditions = true;
        tinyg.door_interlock = true;
        self.database
            .insert((FirmwareType::TinyG, "2.0".to_string()), tinyg);
    }

    /// Initialize g2core capability profiles
    fn init_g2core_profiles(&mut self) {
        let mut g2core =
            FirmwareCapabilities::new(FirmwareType::G2Core, SemanticVersion::new(3, 0, 0));
        g2core.max_axes = 6;
        g2core.arc_support = true;
        g2core.inverse_time_feed = true;
        g2core.feed_per_revolution = true;
        g2core.variable_spindle = true;
        g2core.spindle_direction = true;
        g2core.spindle_css = true;
        g2core.tool_change = true;
        g2core.tool_length_offset = true;
        g2core.tool_diameter_offset = true;
        g2core.probing = true;
        g2core.probe_away = true;
        g2core.coolant_control = true;
        g2core.mist_control = true;
        g2core.homing_cycle = true;
        g2core.soft_homing = true;
        g2core.hard_homing = true;
        g2core.coordinate_systems = 9;
        g2core.local_offsets = true;
        g2core.cutter_radius_compensation = true;
        g2core.macro_support = true;
        g2core.conditional_blocks = true;
        g2core.variable_support = true;
        g2core.status_reports = true;
        g2core.realtime_commands = true;
        g2core.flow_control = true;
        g2core.soft_limits = true;
        g2core.hard_limits = true;
        g2core.alarm_conditions = true;
        g2core.door_interlock = true;
        self.database
            .insert((FirmwareType::G2Core, "3.0".to_string()), g2core);
    }

    /// Initialize Smoothieware capability profiles
    fn init_smoothieware_profiles(&mut self) {
        let mut smoothieware =
            FirmwareCapabilities::new(FirmwareType::Smoothieware, SemanticVersion::new(1, 0, 0));
        smoothieware.max_axes = 5;
        smoothieware.arc_support = true;
        smoothieware.inverse_time_feed = true;
        smoothieware.feed_per_revolution = true;
        smoothieware.variable_spindle = true;
        smoothieware.spindle_direction = true;
        smoothieware.spindle_css = true;
        smoothieware.tool_change = true;
        smoothieware.tool_length_offset = true;
        smoothieware.tool_diameter_offset = true;
        smoothieware.probing = true;
        smoothieware.probe_away = true;
        smoothieware.coolant_control = true;
        smoothieware.mist_control = true;
        smoothieware.homing_cycle = true;
        smoothieware.soft_homing = true;
        smoothieware.hard_homing = true;
        smoothieware.coordinate_systems = 9;
        smoothieware.local_offsets = true;
        smoothieware.cutter_radius_compensation = true;
        smoothieware.macro_support = true;
        smoothieware.conditional_blocks = true;
        smoothieware.variable_support = true;
        smoothieware.status_reports = true;
        smoothieware.realtime_commands = true;
        smoothieware.flow_control = true;
        smoothieware.soft_limits = true;
        smoothieware.hard_limits = true;
        smoothieware.alarm_conditions = true;
        smoothieware.door_interlock = true;
        self.database.insert(
            (FirmwareType::Smoothieware, "1.0".to_string()),
            smoothieware,
        );
    }

    /// Initialize FluidNC capability profiles
    fn init_fluidnc_profiles(&mut self) {
        let mut fluidnc =
            FirmwareCapabilities::new(FirmwareType::FluidNC, SemanticVersion::new(3, 0, 0));
        fluidnc.max_axes = 9;
        fluidnc.arc_support = true;
        fluidnc.inverse_time_feed = true;
        fluidnc.feed_per_revolution = true;
        fluidnc.variable_spindle = true;
        fluidnc.spindle_direction = true;
        fluidnc.spindle_css = true;
        fluidnc.tool_change = true;
        fluidnc.tool_length_offset = true;
        fluidnc.tool_diameter_offset = true;
        fluidnc.probing = true;
        fluidnc.probe_away = true;
        fluidnc.coolant_control = true;
        fluidnc.mist_control = true;
        fluidnc.homing_cycle = true;
        fluidnc.soft_homing = true;
        fluidnc.hard_homing = true;
        fluidnc.coordinate_systems = 9;
        fluidnc.local_offsets = true;
        fluidnc.cutter_radius_compensation = true;
        fluidnc.macro_support = true;
        fluidnc.conditional_blocks = true;
        fluidnc.variable_support = true;
        fluidnc.status_reports = true;
        fluidnc.realtime_commands = true;
        fluidnc.flow_control = true;
        fluidnc.soft_limits = true;
        fluidnc.hard_limits = true;
        fluidnc.alarm_conditions = true;
        fluidnc.door_interlock = true;
        self.database
            .insert((FirmwareType::FluidNC, "3.0".to_string()), fluidnc);
    }

    /// Get capabilities for a specific firmware type and version
    pub fn get_capabilities(
        &self,
        firmware_type: FirmwareType,
        version: &SemanticVersion,
    ) -> Option<FirmwareCapabilities> {
        // Try exact version first
        let version_key = format!("{}.{}", version.major, version.minor);
        if let Some(caps) = self.database.get(&(firmware_type, version_key)) {
            return Some(caps.clone());
        }

        // Try major.minor matching (ignore patch)
        for (key, caps) in &self.database {
            if key.0 == firmware_type
                && caps.version.major == version.major
                && caps.version.minor == version.minor
            {
                return Some(caps.clone());
            }
        }

        // Return None if not found
        None
    }

    /// Check if a specific firmware supports a capability
    pub fn supports_capability(
        &self,
        firmware_type: FirmwareType,
        version: &SemanticVersion,
        capability: &str,
    ) -> bool {
        if let Some(caps) = self.get_capabilities(firmware_type, version) {
            caps.supports(capability)
        } else {
            false
        }
    }

    /// Get list of all supported firmware types
    pub fn supported_firmware_types(&self) -> Vec<FirmwareType> {
        let mut types: Vec<_> = self.database.keys().map(|(t, _)| *t).collect();
        types.sort_by_key(|t| format!("{}", t));
        types.dedup();
        types
    }

    /// Register custom capabilities for a firmware version
    pub fn register_custom(
        &mut self,
        mut caps: FirmwareCapabilities,
        custom_caps: HashMap<String, bool>,
    ) {
        caps.custom = custom_caps;
        let version_key = format!("{}.{}", caps.version.major, caps.version.minor);
        self.database
            .insert((caps.firmware_type, version_key), caps);
    }
}

impl Default for CapabilitiesDatabase {
    fn default() -> Self {
        Self::new()
    }
}

