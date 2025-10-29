//! g2core Capabilities and Version Detection
//!
//! This module provides g2core version detection, feature set determination,
//! and capability querying based on firmware version.
//! g2core supports more advanced features than TinyG, including 6-axis support.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// g2core version information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct G2CoreVersion {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Build or revision number
    pub build: Option<u32>,
}

impl G2CoreVersion {
    /// Create a new g2core version
    pub fn new(major: u32, minor: u32) -> Self {
        Self {
            major,
            minor,
            build: None,
        }
    }

    /// Create a new g2core version with build number
    pub fn with_build(major: u32, minor: u32, build: u32) -> Self {
        Self {
            major,
            minor,
            build: Some(build),
        }
    }

    /// Parse g2core version string from JSON response (e.g., "100.00")
    pub fn parse(version_str: &str) -> Option<Self> {
        let version_str = version_str.trim();
        let parts: Vec<&str> = version_str.split('.').collect();

        if parts.len() < 2 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;

        let build = if parts.len() > 2 {
            parts[2].parse().ok()
        } else {
            None
        };

        Some(Self {
            major,
            minor,
            build,
        })
    }
}

impl Ord for G2CoreVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => self.build.cmp(&other.build),
                order => order,
            },
            order => order,
        }
    }
}

impl PartialOrd for G2CoreVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for G2CoreVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)?;
        if let Some(build) = self.build {
            write!(f, ".{}", build)?;
        }
        Ok(())
    }
}

/// g2core feature set and capabilities
#[derive(Debug, Clone)]
pub struct G2CoreCapabilities {
    /// Version information
    pub version: G2CoreVersion,
    /// Number of axes supported (4-6)
    pub num_axes: u8,
    /// Buffer size for command queuing
    pub buffer_size: usize,
    /// Supports JSON responses
    pub supports_json: bool,
    /// Supports tool change
    pub supports_tool_change: bool,
    /// Supports probing
    pub supports_probing: bool,
    /// Supports spindle control
    pub supports_spindle: bool,
    /// Supports multi-axis homing
    pub supports_auto_home: bool,
    /// Supports soft limits
    pub supports_soft_limits: bool,
    /// Supports rotational axes (A, B, C)
    pub supports_rotational_axes: bool,
    /// Supports kinematics models
    pub supports_kinematics: bool,
    /// Supports advanced motion modes
    pub supports_advanced_motion: bool,
}

impl G2CoreCapabilities {
    /// Create default g2core capabilities for a given version
    pub fn for_version(version: G2CoreVersion) -> Self {
        let supports_advanced = version.major >= 100 && version.minor >= 10;

        Self {
            version,
            num_axes: 6,
            buffer_size: 256,
            supports_json: true,
            supports_tool_change: true,
            supports_probing: true,
            supports_spindle: true,
            supports_auto_home: true,
            supports_soft_limits: true,
            supports_rotational_axes: true,
            supports_kinematics: supports_advanced,
            supports_advanced_motion: supports_advanced,
        }
    }

    /// Check if this version is at least the minimum required version
    pub fn is_minimum_version(&self) -> bool {
        self.version >= G2CoreVersion::new(100, 0)
    }

    /// Check if version is newer than another version
    pub fn is_newer_than(&self, other: &G2CoreVersion) -> bool {
        self.version > *other
    }

    /// Check if this supports advanced features (newer versions)
    pub fn supports_advanced_features(&self) -> bool {
        self.version >= G2CoreVersion::new(100, 10)
    }

    /// Get the maximum number of axes this version supports
    pub fn max_axes(&self) -> u8 {
        if self.version >= G2CoreVersion::new(100, 5) {
            6
        } else {
            4
        }
    }
}

/// Version comparison result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionComparison {
    /// Version is older than requirement
    TooOld,
    /// Version matches requirement
    Compatible,
    /// Version is newer than requirement
    Newer,
}

impl VersionComparison {
    /// Check if version is acceptable (compatible or newer)
    pub fn is_acceptable(&self) -> bool {
        matches!(self, Self::Compatible | Self::Newer)
    }
}
