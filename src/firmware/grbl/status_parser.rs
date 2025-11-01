//! GRBL Status Report Parsing
//!
//! This module provides advanced status parsing for GRBL status reports,
//! including machine position, work position, coordinates offsets, buffer state,
//! and spindle/feed rate state extraction.

use crate::data::CNCPoint;
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
    pub fn to_cncpoint(&self, unit: crate::data::Units) -> CNCPoint {
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
    /// Parse machine position from status report
    pub fn parse_mpos(status_line: &str) -> Option<MachinePosition> {
        Self::extract_field(status_line, "MPos:")
            .and_then(MachinePosition::parse)
    }

    /// Parse work position from status report
    pub fn parse_wpos(status_line: &str) -> Option<WorkPosition> {
        Self::extract_field(status_line, "WPos:").and_then(WorkPosition::parse)
    }

    /// Parse work coordinate offset from status report
    pub fn parse_wco(status_line: &str) -> Option<WorkCoordinateOffset> {
        Self::extract_field(status_line, "WCO:")
            .and_then(WorkCoordinateOffset::parse)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_machine_position() {
        let mpos = MachinePosition::parse("10.000,20.000,30.000").unwrap();
        assert_eq!(mpos.x, 10.0);
        assert_eq!(mpos.y, 20.0);
        assert_eq!(mpos.z, 30.0);
        assert_eq!(mpos.a, None);
    }

    #[test]
    fn test_parse_machine_position_multiaxis() {
        let mpos = MachinePosition::parse("10.000,20.000,30.000,5.000,2.000").unwrap();
        assert_eq!(mpos.x, 10.0);
        assert_eq!(mpos.a, Some(5.0));
        assert_eq!(mpos.b, Some(2.0));
    }

    #[test]
    fn test_parse_work_position() {
        let wpos = WorkPosition::parse("0.000,0.000,0.000").unwrap();
        assert_eq!(wpos.x, 0.0);
        assert_eq!(wpos.y, 0.0);
        assert_eq!(wpos.z, 0.0);
    }

    #[test]
    fn test_parse_work_coordinate_offset() {
        let wco = WorkCoordinateOffset::parse("10.000,0.000,5.000").unwrap();
        assert_eq!(wco.x, 10.0);
        assert_eq!(wco.y, 0.0);
        assert_eq!(wco.z, 5.0);
    }

    #[test]
    fn test_parse_feed_spindle_state() {
        let state = FeedSpindleState::parse("1500.0", "1200").unwrap();
        assert_eq!(state.feed_rate, 1500.0);
        assert_eq!(state.spindle_speed, 1200);
    }

    #[test]
    fn test_parse_buffer_rx_state() {
        let buf = BufferRxState::parse("15:128").unwrap();
        assert_eq!(buf.plan, 15);
        assert_eq!(buf.rx, 128);
    }

    #[test]
    fn test_status_parser_extract_mpos() {
        let status = "<Idle|MPos:10.000,20.000,30.000|WPos:0.000,0.000,0.000>";
        let mpos = StatusParser::parse_mpos(status).unwrap();
        assert_eq!(mpos.x, 10.0);
        assert_eq!(mpos.y, 20.0);
        assert_eq!(mpos.z, 30.0);
    }

    #[test]
    fn test_status_parser_extract_wpos() {
        let status = "<Idle|MPos:10.000,20.000,30.000|WPos:5.000,8.000,2.000>";
        let wpos = StatusParser::parse_wpos(status).unwrap();
        assert_eq!(wpos.x, 5.0);
        assert_eq!(wpos.y, 8.0);
        assert_eq!(wpos.z, 2.0);
    }

    #[test]
    fn test_status_parser_extract_buffer() {
        let status = "<Run|MPos:0,0,0|WPos:0,0,0|Buf:12:96>";
        let buffer = StatusParser::parse_buffer(status).unwrap();
        assert_eq!(buffer.plan, 12);
        assert_eq!(buffer.rx, 96);
    }

    #[test]
    fn test_status_parser_extract_feed_rate() {
        let status = "<Run|MPos:0,0,0|WPos:0,0,0|F:2000.5>";
        let rate = StatusParser::parse_feed_rate(status).unwrap();
        assert_eq!(rate, 2000.5);
    }

    #[test]
    fn test_status_parser_extract_spindle_speed() {
        let status = "<Run|MPos:0,0,0|WPos:0,0,0|S:5000>";
        let speed = StatusParser::parse_spindle_speed(status).unwrap();
        assert_eq!(speed, 5000);
    }

    #[test]
    fn test_status_parser_feed_spindle_state() {
        let status = "<Run|MPos:0,0,0|WPos:0,0,0|F:1500.0|S:1200>";
        let state = StatusParser::parse_feed_spindle(status).unwrap();
        assert_eq!(state.feed_rate, 1500.0);
        assert_eq!(state.spindle_speed, 1200);
    }

    #[test]
    fn test_status_parser_full_parse() {
        let status = "<Run|MPos:10,20,30|WPos:5,8,2|Buf:10:100|F:1500|S:1000>";
        let full = StatusParser::parse_full(status);

        assert!(full.mpos.is_some());
        assert!(full.wpos.is_some());
        assert!(full.buffer.is_some());
        assert_eq!(full.feed_rate, Some(1500.0));
        assert_eq!(full.spindle_speed, Some(1000));
    }

    #[test]
    fn test_work_coordinate_offset_to_cncpoint() {
        let wco = WorkCoordinateOffset::parse("10.000,20.000,30.000").unwrap();
        let point = wco.to_cncpoint(crate::data::Units::MM);

        assert_eq!(point.x, 10.0);
        assert_eq!(point.y, 20.0);
        assert_eq!(point.z, 30.0);
        assert_eq!(point.unit, crate::data::Units::MM);
    }
}
