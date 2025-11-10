//! GRBL Capabilities and Version Detection
//!
//! This module provides GRBL version detection, feature set determination,
//! and capability querying based on firmware version.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// GRBL version information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GrblVersion {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Patch version number
    pub patch: u32,
    /// Optional build string
    pub build: Option<String>,
}

impl GrblVersion {
    /// Create a new GRBL version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            build: None,
        }
    }

    /// Create a new GRBL version with build string
    pub fn with_build(major: u32, minor: u32, patch: u32, build: String) -> Self {
        Self {
            major,
            minor,
            patch,
            build: Some(build),
        }
    }

    /// Parse GRBL version string (e.g., "Grbl 1.1h ['$' for help]")
    pub fn parse(version_str: &str) -> Option<Self> {
        let version_str = version_str.trim();

        if !version_str.starts_with("Grbl ") {
            return None;
        }

        let parts: Vec<&str> = version_str.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let version_part = parts[1];
        let mut chars = version_part.chars().peekable();

        let major = parse_version_number(&mut chars)?;

        if chars.next() != Some('.') {
            return None;
        }

        let minor = parse_version_number(&mut chars)?;

        let mut patch = 0;
        let mut build = None;

        if chars.peek() == Some(&'.') {
            chars.next();
            patch = parse_version_number(&mut chars)?;
        }

        if let Some(build_char) = chars.next() {
            build = Some(build_char.to_string());
        }

        Some(Self {
            major,
            minor,
            patch,
            build,
        })
    }

    /// Check if this version meets a minimum requirement
    pub fn meets_minimum(&self, minimum: &GrblVersion) -> bool {
        self >= minimum
    }

    /// Check if this version is 1.1 or later
    pub fn is_1_1_or_later(&self) -> bool {
        self >= &GrblVersion::new(1, 1, 0)
    }

    /// Check if this version is 1.2 or later
    pub fn is_1_2_or_later(&self) -> bool {
        self >= &GrblVersion::new(1, 2, 0)
    }

    /// Check if this version is 0.9 or later
    pub fn is_0_9_or_later(&self) -> bool {
        self >= &GrblVersion::new(0, 9, 0)
    }

    /// Get a summary string representation
    pub fn to_summary(&self) -> String {
        let base = format!("{}.{}", self.major, self.minor);
        match &self.build {
            Some(build) => format!("{}.{}{}", self.major, self.minor, build),
            None => base,
        }
    }
}

impl Ord for GrblVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                other => other,
            },
            other => other,
        }
    }
}

impl PartialOrd for GrblVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for GrblVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(build) = &self.build {
            write!(f, "Grbl {}.{}{}", self.major, self.minor, build)
        } else {
            write!(f, "Grbl {}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// GRBL feature set based on version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrblFeatureSet {
    /// Supports status reports
    pub status_reports: bool,
    /// Supports real-time commands
    pub real_time_commands: bool,
    /// Supports g-code comments
    pub comments: bool,
    /// Supports coordinate systems (G54-G59)
    pub coordinate_systems: bool,
    /// Supports probing
    pub probing: bool,
    /// Supports spindle control
    pub spindle_control: bool,
    /// Supports coolant control
    pub coolant_control: bool,
    /// Supports safety door
    pub safety_door: bool,
    /// Supports homing
    pub homing: bool,
    /// Supports soft limits
    pub soft_limits: bool,
    /// Supports jog command
    pub jog_command: bool,
    /// Supports character counting
    pub character_counting: bool,
    /// Supports build info command
    pub build_info: bool,
}

impl GrblFeatureSet {
    /// Create feature set for GRBL 0.9
    pub fn grbl_0_9() -> Self {
        Self {
            status_reports: true,
            real_time_commands: true,
            comments: true,
            coordinate_systems: true,
            probing: true,
            spindle_control: true,
            coolant_control: false,
            safety_door: false,
            homing: true,
            soft_limits: true,
            jog_command: false,
            character_counting: false,
            build_info: false,
        }
    }

    /// Create feature set for GRBL 1.1
    pub fn grbl_1_1() -> Self {
        Self {
            status_reports: true,
            real_time_commands: true,
            comments: true,
            coordinate_systems: true,
            probing: true,
            spindle_control: true,
            coolant_control: true,
            safety_door: true,
            homing: true,
            soft_limits: true,
            jog_command: true,
            character_counting: true,
            build_info: true,
        }
    }

    /// Create feature set for GRBL 1.2 (same capabilities as 1.1)
    pub fn grbl_1_2() -> Self {
        Self::grbl_1_1()
    }

    /// Create feature set based on version
    pub fn for_version(version: &GrblVersion) -> Self {
        if version.major > 1 || (version.major == 1 && version.minor >= 2) {
            Self::grbl_1_2()
        } else if version.is_1_1_or_later() {
            Self::grbl_1_1()
        } else if version.is_0_9_or_later() {
            Self::grbl_0_9()
        } else {
            // Default/minimal feature set for unknown versions
            Self::grbl_0_9()
        }
    }
}

impl Default for GrblFeatureSet {
    fn default() -> Self {
        Self::grbl_1_1()
    }
}

/// GRBL capabilities information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrblCapabilities {
    /// GRBL version
    pub version: GrblVersion,
    /// Feature set
    pub features: GrblFeatureSet,
    /// Maximum number of axes
    pub max_axes: u8,
    /// Buffer size for commands
    pub buffer_size: usize,
    /// Maximum block size
    pub max_block_size: usize,
    /// Supported baud rates
    pub baud_rates: Vec<u32>,
    /// Hardware version string
    pub hardware_version: Option<String>,
    /// Build information
    pub build_info: Option<String>,
}

impl GrblCapabilities {
    /// Create capabilities for a specific version
    pub fn for_version(version: GrblVersion) -> Self {
        let features = GrblFeatureSet::for_version(&version);

        Self {
            version,
            features,
            max_axes: 6,
            buffer_size: 128,
            max_block_size: 256,
            baud_rates: vec![9600, 19200, 38400, 57600, 115200],
            hardware_version: None,
            build_info: None,
        }
    }

    /// Parse GRBL capabilities from startup string
    pub fn from_startup_string(startup: &str) -> Option<Self> {
        let version = GrblVersion::parse(startup)?;
        Some(Self::for_version(version))
    }

    /// Get maximum feed rate for this GRBL version
    pub fn max_feed_rate(&self) -> f64 {
        24000.0 // Typical for GRBL
    }

    /// Get maximum rapid rate for this GRBL version
    pub fn max_rapid_rate(&self) -> f64 {
        1000.0 // Typical for GRBL
    }

    /// Get maximum spindle speed for this GRBL version
    pub fn max_spindle_speed(&self) -> u32 {
        if self.version.is_1_1_or_later() {
            10000
        } else {
            255
        }
    }

    /// Check if a feature is supported
    pub fn supports(&self, feature: GrblFeature) -> bool {
        match feature {
            GrblFeature::StatusReports => self.features.status_reports,
            GrblFeature::RealTimeCommands => self.features.real_time_commands,
            GrblFeature::Comments => self.features.comments,
            GrblFeature::CoordinateSystems => self.features.coordinate_systems,
            GrblFeature::Probing => self.features.probing,
            GrblFeature::SpindleControl => self.features.spindle_control,
            GrblFeature::CoolantControl => self.features.coolant_control,
            GrblFeature::SafetyDoor => self.features.safety_door,
            GrblFeature::Homing => self.features.homing,
            GrblFeature::SoftLimits => self.features.soft_limits,
            GrblFeature::JogCommand => self.features.jog_command,
            GrblFeature::CharacterCounting => self.features.character_counting,
            GrblFeature::BuildInfo => self.features.build_info,
        }
    }
}

impl Default for GrblCapabilities {
    fn default() -> Self {
        Self::for_version(GrblVersion::new(1, 1, 0))
    }
}

/// Enumeration of GRBL features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrblFeature {
    /// Status reports ($10)
    StatusReports,
    /// Real-time commands (?, !, ~, ctrl+x)
    RealTimeCommands,
    /// G-code comments (parentheses and semicolon)
    Comments,
    /// Coordinate systems (G54-G59, G59.1-G59.3)
    CoordinateSystems,
    /// Probing support (G38 commands)
    Probing,
    /// Spindle control (M3, M4, M5)
    SpindleControl,
    /// Coolant control (M7, M8, M9)
    CoolantControl,
    /// Safety door support
    SafetyDoor,
    /// Homing support
    Homing,
    /// Soft limits
    SoftLimits,
    /// Jog command (G91 G0 with $J=...)
    JogCommand,
    /// Character counting
    CharacterCounting,
    /// Build info command ($I)
    BuildInfo,
}

impl std::fmt::Display for GrblFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StatusReports => write!(f, "Status Reports"),
            Self::RealTimeCommands => write!(f, "Real-time Commands"),
            Self::Comments => write!(f, "Comments"),
            Self::CoordinateSystems => write!(f, "Coordinate Systems"),
            Self::Probing => write!(f, "Probing"),
            Self::SpindleControl => write!(f, "Spindle Control"),
            Self::CoolantControl => write!(f, "Coolant Control"),
            Self::SafetyDoor => write!(f, "Safety Door"),
            Self::Homing => write!(f, "Homing"),
            Self::SoftLimits => write!(f, "Soft Limits"),
            Self::JogCommand => write!(f, "Jog Command"),
            Self::CharacterCounting => write!(f, "Character Counting"),
            Self::BuildInfo => write!(f, "Build Info"),
        }
    }
}

/// Version comparison result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    /// Version is earlier than minimum
    TooOld,
    /// Version meets or exceeds minimum
    Compatible,
    /// Version is newer than maximum
    TooNew,
}

impl VersionComparison {
    /// Check version compatibility
    pub fn check(
        version: &GrblVersion,
        minimum: &GrblVersion,
        maximum: Option<&GrblVersion>,
    ) -> Self {
        if version < minimum {
            return Self::TooOld;
        }

        if let Some(max) = maximum {
            if version > max {
                return Self::TooNew;
            }
        }

        Self::Compatible
    }
}

/// Helper function to parse version number components
fn parse_version_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<u32> {
    let mut num_str = String::new();

    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    if num_str.is_empty() {
        None
    } else {
        num_str.parse::<u32>().ok()
    }
}
