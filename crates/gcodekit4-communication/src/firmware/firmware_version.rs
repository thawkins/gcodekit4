//! Firmware version detection and tracking
//!
//! Detects and manages firmware version information for connected CNC controllers.
//! Supports GRBL, TinyG, g2core, Smoothieware, and FluidNC.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Firmware type identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FirmwareType {
    /// GRBL - Open source CNC control software
    Grbl,
    /// TinyG - CNC controller for 3D printers
    TinyG,
    /// g2core - Next generation of TinyG
    G2Core,
    /// Smoothieware - CNC motion controller
    Smoothieware,
    /// FluidNC - Modern open-source CNC controller
    FluidNC,
    /// Unknown firmware type
    Unknown,
}

impl FirmwareType {
    /// Parse firmware type from string
    pub fn from_string(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("grbl") {
            Self::Grbl
        } else if lower.contains("tinyg") {
            Self::TinyG
        } else if lower.contains("g2core") {
            Self::G2Core
        } else if lower.contains("smoothie") {
            Self::Smoothieware
        } else if lower.contains("fluidnc") {
            Self::FluidNC
        } else {
            Self::Unknown
        }
    }
}

impl std::fmt::Display for FirmwareType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Grbl => write!(f, "GRBL"),
            Self::TinyG => write!(f, "TinyG"),
            Self::G2Core => write!(f, "g2core"),
            Self::Smoothieware => write!(f, "Smoothieware"),
            Self::FluidNC => write!(f, "FluidNC"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Semantic version (major.minor.patch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemanticVersion {
    /// Create a new semantic version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse from string (e.g., "1.1f", "3.0.0", "1.0.0-alpha")
    pub fn parse(s: &str) -> Option<Self> {
        // Handle versions like "1.1f" (GRBL), "1.0.0-alpha", etc.
        let version_part = s.split('-').next().unwrap_or(s);
        let parts: Vec<&str> = version_part.split('.').collect();

        if parts.is_empty() {
            return None;
        }

        let major = parts[0]
            .trim_end_matches(|c: char| !c.is_numeric())
            .parse::<u32>()
            .ok()?;
        let minor = if parts.len() > 1 {
            parts[1]
                .trim_end_matches(|c: char| !c.is_numeric())
                .parse::<u32>()
                .ok()
                .unwrap_or(0)
        } else {
            0
        };
        let patch = if parts.len() > 2 {
            parts[2]
                .trim_end_matches(|c: char| !c.is_numeric())
                .parse::<u32>()
                .ok()
                .unwrap_or(0)
        } else {
            0
        };

        Some(Self {
            major,
            minor,
            patch,
        })
    }

    /// Check if this version is at least the given version
    pub fn is_at_least(&self, other: &Self) -> bool {
        if self.major != other.major {
            return self.major > other.major;
        }
        if self.minor != other.minor {
            return self.minor > other.minor;
        }
        self.patch >= other.patch
    }

    /// Check if this version is compatible (same major version)
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.patch.cmp(&other.patch),
                other_ord => other_ord,
            },
            other_ord => other_ord,
        }
    }
}

/// Complete firmware version information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirmwareVersion {
    /// Firmware type
    pub firmware_type: FirmwareType,
    /// Version number
    pub version: SemanticVersion,
    /// Build date (if available)
    pub build_date: Option<String>,
    /// Additional version string (e.g., "1.1f")
    pub version_string: String,
    /// Custom label or variant
    pub variant: Option<String>,
}

impl FirmwareVersion {
    /// Create a new firmware version
    pub fn new(
        firmware_type: FirmwareType,
        version: SemanticVersion,
        version_string: String,
    ) -> Self {
        Self {
            firmware_type,
            version,
            build_date: None,
            version_string,
            variant: None,
        }
    }

    /// Parse from GRBL startup message
    /// Format: Grbl 1.1f ['$' for help]
    pub fn parse_grbl(message: &str) -> Option<Self> {
        if !message.contains("Grbl") && !message.contains("grbl") {
            return None;
        }

        let parts: Vec<&str> = message.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let version_str = parts[1];
        let version = SemanticVersion::parse(version_str)?;

        Some(Self::new(
            FirmwareType::Grbl,
            version,
            version_str.to_string(),
        ))
    }

    /// Parse from TinyG/g2core startup message
    /// Format: {"fv":1.9842,"fb":109.18,"hv":3,"id":"..."}
    pub fn parse_tinygxyz(message: &str, is_g2core: bool) -> Option<Self> {
        // This would parse JSON response - simplified for now
        if message.contains("fv") {
            let firmware_type = if is_g2core {
                FirmwareType::G2Core
            } else {
                FirmwareType::TinyG
            };
            // In real implementation, parse JSON and extract version
            Some(Self::new(
                firmware_type,
                SemanticVersion::new(1, 0, 0),
                message.to_string(),
            ))
        } else {
            None
        }
    }

    /// Get human-readable version string
    pub fn display_string(&self) -> String {
        format!(
            "{} {}{}",
            self.firmware_type,
            self.version,
            self.variant
                .as_ref()
                .map(|v| format!(" ({})", v))
                .unwrap_or_default()
        )
    }
}

impl std::fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_string())
    }
}

