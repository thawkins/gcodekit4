//! FluidNC command creator
//!
//! Generates G-code and M-code commands specific to FluidNC.

use crate::data::Position;

/// FluidNC command creator
#[derive(Debug, Clone)]
pub struct FluidNCCommandCreator;

impl FluidNCCommandCreator {
    /// Create a new command creator
    pub fn new() -> Self {
        Self
    }

    /// Create a jog command for FluidNC
    pub fn jog_command(&self, axis: char, distance: f64, feed_rate: f64) -> String {
        format!(
            "$J=G91G21{}{}F{}",
            axis.to_ascii_uppercase(),
            distance,
            feed_rate
        )
    }

    /// Create a move command
    pub fn move_command(&self, position: &Position, feed_rate: f64) -> String {
        let mut cmd = String::from("G1");
        cmd.push_str(&format!(" X{:.4}", position.x));
        cmd.push_str(&format!(" Y{:.4}", position.y));
        cmd.push_str(&format!(" Z{:.4}", position.z));
        if let Some(a) = position.a {
            cmd.push_str(&format!(" A{:.4}", a));
        }
        cmd.push_str(&format!(" F{:.0}", feed_rate));
        cmd
    }

    /// Create a rapid move command
    pub fn rapid_move_command(&self, position: &Position) -> String {
        let mut cmd = String::from("G0");
        cmd.push_str(&format!(" X{:.4}", position.x));
        cmd.push_str(&format!(" Y{:.4}", position.y));
        cmd.push_str(&format!(" Z{:.4}", position.z));
        if let Some(a) = position.a {
            cmd.push_str(&format!(" A{:.4}", a));
        }
        cmd
    }

    /// Create a home command
    pub fn home_command(&self, axes: Option<&str>) -> String {
        match axes {
            Some(a) => format!("$H {}", a),
            None => "$H".to_string(),
        }
    }

    /// Create a spindle on command (clockwise)
    pub fn spindle_on_cw(&self, speed: u16) -> String {
        format!("M3 S{}", speed)
    }

    /// Create a spindle on command (counter-clockwise)
    pub fn spindle_on_ccw(&self, speed: u16) -> String {
        format!("M4 S{}", speed)
    }

    /// Create a spindle off command
    pub fn spindle_off(&self) -> String {
        "M5".to_string()
    }

    /// Create a dwell command
    pub fn dwell(&self, seconds: f64) -> String {
        format!("G4 P{:.3}", seconds)
    }

    /// Create a status request command
    pub fn status_request(&self) -> String {
        "?".to_string()
    }

    /// Create a reset command
    pub fn reset(&self) -> String {
        "\x18".to_string() // Ctrl-X
    }

    /// Create a probe command
    pub fn probe_command(
        &self,
        axis: char,
        distance: f64,
        feed_rate: f64,
        pull_off: f64,
    ) -> String {
        format!(
            "G38.2 {}:{:.4} F{:.0}; G1 {}:{:.4}",
            axis.to_ascii_uppercase(),
            distance,
            feed_rate,
            axis.to_ascii_uppercase(),
            pull_off
        )
    }

    /// Create an arc command (clockwise)
    pub fn arc_cw(&self, end: &Position, center_x: f64, center_y: f64, feed_rate: f64) -> String {
        let mut cmd = String::from("G2");
        cmd.push_str(&format!(" X{:.4}", end.x));
        cmd.push_str(&format!(" Y{:.4}", end.y));
        cmd.push_str(&format!(" I{:.4} J{:.4}", center_x, center_y));
        cmd.push_str(&format!(" F{:.0}", feed_rate));
        cmd
    }

    /// Create an arc command (counter-clockwise)
    pub fn arc_ccw(&self, end: &Position, center_x: f64, center_y: f64, feed_rate: f64) -> String {
        let mut cmd = String::from("G3");
        cmd.push_str(&format!(" X{:.4}", end.x));
        cmd.push_str(&format!(" Y{:.4}", end.y));
        cmd.push_str(&format!(" I{:.4} J{:.4}", center_x, center_y));
        cmd.push_str(&format!(" F{:.0}", feed_rate));
        cmd
    }

    /// Create file list request
    pub fn list_files(&self, path: Option<&str>) -> String {
        match path {
            Some(p) => format!("$SD/List {}", p),
            None => "$SD/List /".to_string(),
        }
    }

    /// Create file upload request
    pub fn upload_file(&self, filename: &str, size: usize) -> String {
        format!("$SD/Upload {} {}", filename, size)
    }
}

impl Default for FluidNCCommandCreator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jog_command() {
        let creator = FluidNCCommandCreator::new();
        let cmd = creator.jog_command('X', 10.0, 1000.0);
        assert!(cmd.contains("X10"));
        assert!(cmd.contains("1000"));
    }

    #[test]
    fn test_spindle_commands() {
        let creator = FluidNCCommandCreator::new();
        assert_eq!(creator.spindle_on_cw(1000), "M3 S1000");
        assert_eq!(creator.spindle_on_ccw(500), "M4 S500");
        assert_eq!(creator.spindle_off(), "M5");
    }

    #[test]
    fn test_reset_command() {
        let creator = FluidNCCommandCreator::new();
        assert_eq!(creator.reset(), "\x18");
    }
}
