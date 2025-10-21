//! G-Code parser and state machine
//!
//! This module provides:
//! - G-Code command parsing
//! - Modal state tracking
//! - Command validation
//! - Preprocessor framework

use lazy_static::lazy_static;
use regex::Regex;

/// Represents a parsed G-Code command
#[derive(Debug, Clone, PartialEq)]
pub struct GcodeCommand {
    /// G-Code line (e.g., "G00 X10.5 Y20.3 Z0.0")
    pub line: String,
    /// Line number if present
    pub line_number: Option<u32>,
    /// Raw command text
    pub command: String,
}

impl GcodeCommand {
    /// Create a new G-Code command
    pub fn new(line: impl Into<String>) -> Self {
        let line = line.into();
        Self {
            command: line.clone(),
            line,
            line_number: None,
        }
    }
}

/// G-Code parser with modal state tracking
pub struct GcodeParser {
    current_mode: ModalState,
}

/// Modal state for G-Code execution
#[derive(Debug, Clone, Copy)]
pub struct ModalState {
    /// Motion mode (G00=rapid, G01=linear, G02=arc_cw, G03=arc_ccw)
    pub motion_mode: u8,
    /// Plane selection (G17=XY, G18=XZ, G19=YZ)
    pub plane: u8,
    /// Distance mode (G90=absolute, G91=incremental)
    pub distance_mode: u8,
    /// Feed rate mode (G93=inverse_time, G94=units_per_minute, G95=units_per_revolution)
    pub feed_rate_mode: u8,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            motion_mode: 0,     // G00
            plane: 17,          // G17 (XY plane)
            distance_mode: 90,  // G90 (absolute)
            feed_rate_mode: 94, // G94 (units per minute)
        }
    }
}

impl Default for GcodeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl GcodeParser {
    /// Create a new G-Code parser
    pub fn new() -> Self {
        Self {
            current_mode: ModalState::default(),
        }
    }

    /// Parse a G-Code line
    pub fn parse(&mut self, line: &str) -> Result<GcodeCommand, String> {
        // Remove comments
        let cleaned = self.remove_comments(line);

        if cleaned.trim().is_empty() {
            return Err("Empty command".to_string());
        }

        tracing::debug!("Parsing G-Code: {}", cleaned);
        Ok(GcodeCommand::new(cleaned))
    }

    /// Remove comments from a G-Code line
    fn remove_comments(&self, line: &str) -> String {
        lazy_static! {
            static ref COMMENT_REGEX: Regex = Regex::new(r"[;(].*").unwrap();
        }
        COMMENT_REGEX.replace(line, "").to_string()
    }

    /// Get current modal state
    pub fn get_modal_state(&self) -> ModalState {
        self.current_mode
    }

    /// Update modal state based on parsed command
    pub fn update_modal_state(&mut self, command: &GcodeCommand) {
        tracing::trace!("Updating modal state for: {}", command.command);
        // Modal state updates would be implemented here
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcode_command_creation() {
        let cmd = GcodeCommand::new("G00 X10.5 Y20.3");
        assert_eq!(cmd.line, "G00 X10.5 Y20.3");
    }

    #[test]
    fn test_parser_creation() {
        let parser = GcodeParser::new();
        assert_eq!(parser.get_modal_state().distance_mode, 90); // G90
    }

    #[test]
    fn test_comment_removal() {
        let parser = GcodeParser::new();
        let line_with_comment = "G00 X10.5 ; Move to X=10.5";
        let cleaned = parser.remove_comments(line_with_comment);
        assert_eq!(cleaned.trim(), "G00 X10.5");
    }

    #[test]
    fn test_parse_command() {
        let mut parser = GcodeParser::new();
        let result = parser.parse("G00 X10.5 Y20.3");
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(cmd.command.contains("X10.5"));
    }
}
