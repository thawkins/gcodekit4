//! TinyG Command Creator
//!
//! This module provides utilities for creating TinyG JSON commands and queries.

use serde_json::{json, Value};

/// Real-time commands for TinyG
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealTimeCommand {
    /// Request status report
    StatusRequest,
    /// Feed hold (pause)
    FeedHold,
    /// Cycle start/resume
    CycleStart,
    /// Soft reset
    SoftReset,
    /// System reset
    SystemReset,
}

impl RealTimeCommand {
    /// Get the byte representation for real-time commands
    pub fn as_byte(&self) -> u8 {
        match self {
            Self::StatusRequest => b'?',
            Self::FeedHold => b'!',
            Self::CycleStart => b'~',
            Self::SoftReset => 0x18,
            Self::SystemReset => 0x1B,
        }
    }
}

/// Motion types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionType {
    /// Rapid movement
    Rapid,
    /// Linear interpolated movement
    Linear,
    /// Arc (CW)
    ArcCw,
    /// Arc (CCW)
    ArcCcw,
}

/// TinyG command creator
#[derive(Debug)]
pub struct CommandCreator {
    /// Line number counter
    line_number: u32,
}

impl CommandCreator {
    /// Create a new command creator
    pub fn new() -> Self {
        Self { line_number: 0 }
    }

    /// Create a G-code command string
    pub fn create_gcode_command(&mut self, gcode: &str) -> String {
        self.line_number += 1;
        format!("N{} {}\n", self.line_number, gcode)
    }

    /// Create a JSON query command
    pub fn create_query(&self, key: &str) -> String {
        format!(r#"{{"{}":null}}"#, key)
    }

    /// Create a status query
    pub fn create_status_query(&self) -> String {
        r#"{"sr":null}"#.to_string()
    }

    /// Create a system query
    pub fn create_system_query(&self) -> String {
        r#"{"sys":null}"#.to_string()
    }

    /// Create a settings query
    pub fn create_settings_query(&self, setting_name: &str) -> String {
        format!(r#"{{"{}":{}}}"#, setting_name, "null")
    }

    /// Create a motion command (G0 or G1)
    pub fn create_motion_command(
        &mut self,
        motion: MotionType,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        a: Option<f64>,
        feed_rate: Option<f64>,
    ) -> String {
        let mut cmd = match motion {
            MotionType::Rapid => "G0".to_string(),
            MotionType::Linear => "G1".to_string(),
            MotionType::ArcCw => "G2".to_string(),
            MotionType::ArcCcw => "G3".to_string(),
        };

        if let Some(x_val) = x {
            cmd.push_str(&format!(" X{}", x_val));
        }
        if let Some(y_val) = y {
            cmd.push_str(&format!(" Y{}", y_val));
        }
        if let Some(z_val) = z {
            cmd.push_str(&format!(" Z{}", z_val));
        }
        if let Some(a_val) = a {
            cmd.push_str(&format!(" A{}", a_val));
        }
        if let Some(f) = feed_rate {
            cmd.push_str(&format!(" F{}", f));
        }

        self.create_gcode_command(&cmd)
    }

    /// Create a jog command
    pub fn create_jog_command(
        &mut self,
        x: Option<f64>,
        y: Option<f64>,
        z: Option<f64>,
        feed_rate: f64,
    ) -> String {
        // TinyG uses G91 (incremental) for jog
        let mut cmd = "G91".to_string();
        if let Some(x_val) = x {
            cmd.push_str(&format!(" X{}", x_val));
        }
        if let Some(y_val) = y {
            cmd.push_str(&format!(" Y{}", y_val));
        }
        if let Some(z_val) = z {
            cmd.push_str(&format!(" Z{}", z_val));
        }
        cmd.push_str(&format!(" F{}", feed_rate));

        self.create_gcode_command(&cmd)
    }

    /// Create a spindle control command
    pub fn create_spindle_command(&mut self, on: bool, speed: Option<u16>) -> String {
        if on {
            match speed {
                Some(s) => self.create_gcode_command(&format!("M3 S{}", s)),
                None => self.create_gcode_command("M3"),
            }
        } else {
            self.create_gcode_command("M5")
        }
    }

    /// Create a coolant control command
    pub fn create_coolant_command(&mut self, flood: bool, mist: bool) -> String {
        match (flood, mist) {
            (true, false) => self.create_gcode_command("M8"),
            (false, true) => self.create_gcode_command("M7"),
            (true, true) => self.create_gcode_command("M8"),
            (false, false) => self.create_gcode_command("M9"),
        }
    }

    /// Create a home command
    pub fn create_home_command(&mut self, home_all: bool) -> String {
        if home_all {
            self.create_gcode_command("G28.2 X0 Y0 Z0 A0")
        } else {
            self.create_gcode_command("G28.2 X0 Y0 Z0")
        }
    }

    /// Create a set position command
    pub fn create_set_position(&self, x: Option<f64>, y: Option<f64>, z: Option<f64>) -> String {
        let mut json_obj = serde_json::Map::new();
        let mut pos_obj = serde_json::Map::new();

        if let Some(x_val) = x {
            pos_obj.insert("x".to_string(), json!(x_val));
        }
        if let Some(y_val) = y {
            pos_obj.insert("y".to_string(), json!(y_val));
        }
        if let Some(z_val) = z {
            pos_obj.insert("z".to_string(), json!(z_val));
        }

        json_obj.insert("xpo".to_string(), Value::Object(pos_obj));
        let json_value = Value::Object(json_obj);
        json_value.to_string()
    }

    /// Create a probe command
    pub fn create_probe_command(&mut self, toward_positive: bool, feed_rate: f64) -> String {
        let direction = if toward_positive { "1" } else { "-1" };
        self.create_gcode_command(&format!("G38.2 Z{} F{}", direction, feed_rate))
    }

    /// Create a tool length offset command
    pub fn create_tool_length_offset(&mut self, offset: f64) -> String {
        self.create_gcode_command(&format!("G43 H{}", offset))
    }

    /// Create a work offset command (G54-G59)
    pub fn create_work_offset(&mut self, offset_num: u8) -> String {
        let g_code = match offset_num {
            1 => "G54",
            2 => "G55",
            3 => "G56",
            4 => "G57",
            5 => "G58",
            6 => "G59",
            _ => "G54", // Default to G54
        };
        self.create_gcode_command(g_code)
    }

    /// Create a settings change command
    pub fn create_settings_change(&self, setting: &str, value: &str) -> String {
        format!(r#"{{"{}":{}}}"#, setting, value)
    }

    /// Reset the line number counter
    pub fn reset_line_number(&mut self) {
        self.line_number = 0;
    }

    /// Get the current line number
    pub fn get_line_number(&self) -> u32 {
        self.line_number
    }
}

impl Default for CommandCreator {
    fn default() -> Self {
        Self::new()
    }
}
