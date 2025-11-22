//! Firmware detection and information querying
//!
//! Automatically detects firmware type and version by sending query commands
//! and parsing responses.

use super::firmware_version::{FirmwareType, FirmwareVersion, SemanticVersion};
use anyhow::{Context, Result};

/// Firmware detection result
#[derive(Debug, Clone)]
pub struct FirmwareDetectionResult {
    /// Detected firmware type
    pub firmware_type: FirmwareType,
    /// Parsed version
    pub version: SemanticVersion,
    /// Full version string
    pub version_string: String,
    /// Build date if available
    pub build_date: Option<String>,
    /// Build info if available
    pub build_info: Option<String>,
    /// Protocol version
    pub protocol_version: Option<String>,
}

impl FirmwareDetectionResult {
    /// Create a new detection result
    pub fn new(
        firmware_type: FirmwareType,
        version: SemanticVersion,
        version_string: String,
    ) -> Self {
        Self {
            firmware_type,
            version,
            version_string,
            build_date: None,
            build_info: None,
            protocol_version: None,
        }
    }

    /// Convert to FirmwareVersion
    pub fn to_firmware_version(&self) -> FirmwareVersion {
        let mut fw = FirmwareVersion::new(
            self.firmware_type,
            self.version,
            self.version_string.clone(),
        );
        fw.build_date = self.build_date.clone();
        fw
    }
}

/// Firmware detector
pub struct FirmwareDetector;

impl FirmwareDetector {
    /// Parse GRBL version info response
    ///
    /// Expected formats:
    /// - `[VER:1.1h.20190825:]` - GRBL 1.1h built on 2019-08-25
    /// - `[OPT:V,15,128]` - Options: Variable spindle, axes mask 15, buffer size 128
    ///
    /// # Arguments
    /// * `response` - Full response from $I command
    ///
    /// # Returns
    /// Parsed firmware detection result
    pub fn parse_grbl_version_info(response: &str) -> Result<FirmwareDetectionResult> {

        let mut version_string = String::new();
        let mut build_date = None;
        let mut build_info = None;

        // Parse [VER:1.1h.20190825:] line
        for line in response.lines() {
            let line = line.trim();

            if line.starts_with("[VER:") && line.ends_with(']') {
                // Extract version: [VER:1.1h.20190825:]
                let ver_part = line
                    .trim_start_matches("[VER:")
                    .trim_end_matches(']')
                    .trim_end_matches(':');

                // Split by '.' to get version and build date
                let parts: Vec<&str> = ver_part.split('.').collect();

                if parts.len() >= 2 {
                    // First two parts are version (e.g., "1.1h")
                    version_string = format!("{}.{}", parts[0], parts[1]);

                    // Third part (if exists) is build date (e.g., "20190825")
                    // It may have a colon suffix like "20190825:Some string"
                    if parts.len() >= 3 {
                        // Take only the date part before any colon
                        let date_part = parts[2].split(':').next().unwrap_or(parts[2]);
                        build_date = Some(date_part.to_string());
                    }
                }
            } else if line.starts_with("[OPT:") && line.ends_with(']') {
                // Parse options: [OPT:V,15,128]
                let opt_part = line
                    .trim_start_matches("[OPT:")
                    .trim_end_matches(']')
                    .trim_end_matches(':');
                build_info = Some(opt_part.to_string());
            }
        }

        // Parse semantic version
        let version = SemanticVersion::parse(&version_string)
            .context("Failed to parse GRBL version number")?;

        let mut result = FirmwareDetectionResult::new(FirmwareType::Grbl, version, version_string);
        result.build_date = build_date;
        result.build_info = build_info;
        result.protocol_version = Some("1.1".to_string());

        Ok(result)
    }

    /// Parse GRBL startup message
    ///
    /// Expected format: `Grbl 1.1h ['$' for help]`
    ///
    /// # Arguments
    /// * `message` - Startup message from GRBL
    ///
    /// # Returns
    /// Parsed firmware detection result
    pub fn parse_grbl_startup(message: &str) -> Result<FirmwareDetectionResult> {

        if !message.contains("Grbl") && !message.contains("grbl") {
            anyhow::bail!("Not a GRBL startup message");
        }

        let parts: Vec<&str> = message.split_whitespace().collect();
        if parts.len() < 2 {
            anyhow::bail!("Invalid GRBL startup message format");
        }

        let version_str = parts[1];
        let version = SemanticVersion::parse(version_str)
            .context("Failed to parse GRBL version from startup")?;

        Ok(FirmwareDetectionResult::new(
            FirmwareType::Grbl,
            version,
            version_str.to_string(),
        ))
    }

    /// Parse Marlin version info response (M115)
    ///
    /// Expected format:
    /// ```text
    /// FIRMWARE_NAME:Marlin 2.0.9.3
    /// PROTOCOL_VERSION:1.0
    /// MACHINE_TYPE:3D Printer
    /// EXTRUDER_COUNT:1
    /// ```
    pub fn parse_marlin_version_info(response: &str) -> Result<FirmwareDetectionResult> {

        let mut version_string = String::new();

        for line in response.lines() {
            if line.starts_with("FIRMWARE_NAME:") {
                // Extract version from "FIRMWARE_NAME:Marlin 2.0.9.3"
                let name_part = line.trim_start_matches("FIRMWARE_NAME:");
                if let Some(version_part) = name_part.split_whitespace().nth(1) {
                    version_string = version_part.to_string();
                }
            }
        }

        if version_string.is_empty() {
            anyhow::bail!("Could not extract Marlin version");
        }

        let version = SemanticVersion::parse(&version_string)
            .context("Failed to parse Marlin version number")?;

        Ok(FirmwareDetectionResult::new(
            FirmwareType::Unknown,
            version,
            version_string,
        ))
    }

    /// Get query command for firmware type
    ///
    /// Returns the command to send to query firmware information.
    pub fn get_query_command(firmware_type: FirmwareType) -> &'static str {
        match firmware_type {
            FirmwareType::Grbl => "$I",
            FirmwareType::Unknown => "$I", // Try GRBL first
            _ => "$I",                     // Default to GRBL command
        }
    }

    /// Parse generic firmware response
    ///
    /// Tries to detect firmware type from response and parse accordingly.
    pub fn parse_response(response: &str) -> Result<FirmwareDetectionResult> {
        // Try GRBL version info format first
        if response.contains("[VER:") {
            return Self::parse_grbl_version_info(response);
        }

        // Try GRBL startup format
        if response.contains("Grbl") || response.contains("grbl") {
            return Self::parse_grbl_startup(response);
        }

        // Try Marlin format
        if response.contains("FIRMWARE_NAME") {
            return Self::parse_marlin_version_info(response);
        }

        anyhow::bail!("Unknown firmware response format")
    }
}

