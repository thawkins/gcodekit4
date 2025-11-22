//! GRBL Status Report Parsing
//!
//! This module provides advanced status parsing for GRBL status reports,
//! including machine position, work position, coordinates offsets, buffer state,
//! and spindle/feed rate state extraction.

use gcodekit4_core::CNCPoint;
use serde::{Deserialize, Serialize};

/// Parsed machine position components
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MachinePosition {
    /// X position in machine coordinates
    pub x: f64,
    /// Y position in machine coordinates
    pub y: f64,
    /// Z position in machine coordinates
    pub z: f64,
    /// A axis (4th axis) position
    pub a: Option<f64>,
    /// B axis (5th axis) position
    pub b: Option<f64>,
    /// C axis (6th axis) position
    pub c: Option<f64>,
}

impl MachinePosition {
    /// Parse machine position from string
    pub fn parse(pos_str: &str) -> Option<Self> {
        let coords: Vec<f64> = pos_str
            .split(',')
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect();

        if coords.is_empty() || coords.len() < 3 {
            return None;
        }

        Some(Self {
            x: coords[0],
            y: coords[1],
            z: coords[2],
            a: coords.get(3).copied(),
            b: coords.get(4).copied(),
            c: coords.get(5).copied(),
        })
    }
}

/// Parsed work position components
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorkPosition {
    /// X position in work coordinates
    pub x: f64,
    /// Y position in work coordinates
    pub y: f64,
    /// Z position in work coordinates
    pub z: f64,
    /// A axis (4th axis) position
    pub a: Option<f64>,
    /// B axis (5th axis) position
    pub b: Option<f64>,
    /// C axis (6th axis) position
    pub c: Option<f64>,
}

impl WorkPosition {
    /// Parse work position from string
    pub fn parse(pos_str: &str) -> Option<Self> {
        let coords: Vec<f64> = pos_str
            .split(',')
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect();

        if coords.is_empty() || coords.len() < 3 {
            return None;
        }

        Some(Self {
            x: coords[0],
            y: coords[1],
            z: coords[2],
            a: coords.get(3).copied(),
            b: coords.get(4).copied(),
            c: coords.get(5).copied(),
        })
    }
}

/// Work coordinate offset
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorkCoordinateOffset {
    /// X offset
    pub x: f64,
    /// Y offset
    pub y: f64,
    /// Z offset
    pub z: f64,
    /// A axis offset
    pub a: Option<f64>,
    /// B axis offset
    pub b: Option<f64>,
    /// C axis offset
    pub c: Option<f64>,
}

impl WorkCoordinateOffset {
    /// Parse work coordinate offset from string
    pub fn parse(offset_str: &str) -> Option<Self> {
        let coords: Vec<f64> = offset_str
            .split(',')
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect();

        if coords.is_empty() || coords.len() < 3 {
            return None;
        }

        Some(Self {
            x: coords[0],
            y: coords[1],
            z: coords[2],
            a: coords.get(3).copied(),
            b: coords.get(4).copied(),
            c: coords.get(5).copied(),
        })
    }

    /// Convert to CNCPoint
    pub fn to_cncpoint(&self, unit: gcodekit4_core::Units) -> CNCPoint {
        CNCPoint::with_axes(
            self.x,
            self.y,
            self.z,
            self.a.unwrap_or(0.0),
            self.b.unwrap_or(0.0),
            self.c.unwrap_or(0.0),
            unit,
        )
    }
}

/// Feed and spindle state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FeedSpindleState {
    /// Current feed rate (units/min)
    pub feed_rate: f64,
    /// Current spindle speed (RPM)
    pub spindle_speed: u32,
}

impl FeedSpindleState {
    /// Parse feed and spindle state
    pub fn parse(feed_str: &str, spindle_str: &str) -> Option<Self> {
        let feed_rate = feed_str.trim().parse::<f64>().ok()?;
        let spindle_speed = spindle_str.trim().parse::<u32>().ok()?;

        Some(Self {
            feed_rate,
            spindle_speed,
        })
    }
}

/// Buffer state with RX counter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BufferRxState {
    /// Plan buffer blocks
    pub plan: u8,
    /// RX buffer bytes
    pub rx: u8,
}

impl BufferRxState {
    /// Parse buffer state from string (format: "plan:rx")
    pub fn parse(buf_str: &str) -> Option<Self> {
        let parts: Vec<&str> = buf_str.split(':').collect();

        if parts.len() < 2 {
            return None;
        }

        let plan = parts[0].trim().parse::<u8>().ok()?;
        let rx = parts[1].trim().parse::<u8>().ok()?;

        Some(Self { plan, rx })
    }
}

/// Comprehensive status parsing
pub struct StatusParser;

impl StatusParser {
    /// Parse machine state from status report
    /// Extracts state from format: <Idle|...> or <Run|...>
    pub fn parse_machine_state(status_line: &str) -> Option<String> {
        if let Some(start) = status_line.find('<') {
            if let Some(end) = status_line[start..].find('|') {
                let state = &status_line[start + 1..start + end];
                return Some(state.to_string());
            }
        }
        None
    }

    /// Parse machine position from status report
    pub fn parse_mpos(status_line: &str) -> Option<MachinePosition> {
        Self::extract_field(status_line, "MPos:").and_then(MachinePosition::parse)
    }

    /// Parse work position from status report
    pub fn parse_wpos(status_line: &str) -> Option<WorkPosition> {
        Self::extract_field(status_line, "WPos:").and_then(WorkPosition::parse)
    }

    /// Parse work coordinate offset from status report
    pub fn parse_wco(status_line: &str) -> Option<WorkCoordinateOffset> {
        Self::extract_field(status_line, "WCO:").and_then(WorkCoordinateOffset::parse)
    }

    /// Parse buffer state from status report
    pub fn parse_buffer(status_line: &str) -> Option<BufferRxState> {
        Self::extract_field(status_line, "Buf:").and_then(BufferRxState::parse)
    }

    /// Parse feed rate from status report
    pub fn parse_feed_rate(status_line: &str) -> Option<f64> {
        Self::extract_field(status_line, "F:")
            .and_then(|rate_str| rate_str.trim().parse::<f64>().ok())
    }

    /// Parse spindle speed from status report
    pub fn parse_spindle_speed(status_line: &str) -> Option<u32> {
        Self::extract_field(status_line, "S:")
            .and_then(|speed_str| speed_str.trim().parse::<u32>().ok())
    }

    /// Parse feed and spindle state together
    pub fn parse_feed_spindle(status_line: &str) -> Option<FeedSpindleState> {
        let feed_rate = Self::parse_feed_rate(status_line)?;
        let spindle_speed = Self::parse_spindle_speed(status_line)?;

        Some(FeedSpindleState {
            feed_rate,
            spindle_speed,
        })
    }

    /// Extract field value from status report
    fn extract_field<'a>(status_line: &'a str, field_prefix: &str) -> Option<&'a str> {
        // Strip angle brackets if present
        let search_line = if status_line.starts_with('<') && status_line.ends_with('>') {
            &status_line[1..status_line.len() - 1]
        } else {
            status_line
        };

        let start = search_line.find(field_prefix)? + field_prefix.len();
        let rest = &search_line[start..];

        // Find the end of this field (next pipe or end of string)
        let end = rest.find('|').unwrap_or(rest.len());
        Some(&rest[..end])
    }

    /// Parse complete status line into all components
    pub fn parse_full(status_line: &str) -> FullStatus {
        FullStatus {
            machine_state: Self::parse_machine_state(status_line),
            mpos: Self::parse_mpos(status_line),
            wpos: Self::parse_wpos(status_line),
            wco: Self::parse_wco(status_line),
            buffer: Self::parse_buffer(status_line),
            feed_rate: Self::parse_feed_rate(status_line),
            spindle_speed: Self::parse_spindle_speed(status_line),
        }
    }
}

/// Complete parsed status report
#[derive(Debug, Clone, PartialEq)]
pub struct FullStatus {
    /// Machine state (Idle, Run, Hold, Alarm, Door, Check, Home, Jog, etc.)
    pub machine_state: Option<String>,
    /// Machine position
    pub mpos: Option<MachinePosition>,
    /// Work position
    pub wpos: Option<WorkPosition>,
    /// Work coordinate offset
    pub wco: Option<WorkCoordinateOffset>,
    /// Buffer state
    pub buffer: Option<BufferRxState>,
    /// Feed rate
    pub feed_rate: Option<f64>,
    /// Spindle speed
    pub spindle_speed: Option<u32>,
}


