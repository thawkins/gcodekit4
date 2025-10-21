//! Data models for positions, status, and machine information
//!
//! This module provides:
//! - Position tracking with full 6-axis support (X, Y, Z, A, B, C)
//! - Partial position updates for selective axis changes
//! - Controller status representation
//! - Machine capabilities
//! - Command structures
//! - Unit management (MM, INCH)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Machine coordinate units (millimeters or inches)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    /// Millimeters (metric)
    MM,
    /// Inches (imperial)
    INCH,
    /// Unknown or uninitialized
    Unknown,
}

impl Units {
    /// Convert a value from one unit to another
    ///
    /// # Arguments
    /// * `value` - The value to convert
    /// * `from` - The unit of the input value
    /// * `to` - The target unit
    ///
    /// # Returns
    /// The converted value, or the original value if units are the same or unknown
    pub fn convert(value: f64, from: Units, to: Units) -> f64 {
        if from == to {
            return value;
        }

        match (from, to) {
            (Units::MM, Units::INCH) => value / 25.4,
            (Units::INCH, Units::MM) => value * 25.4,
            _ => value,
        }
    }
}

impl fmt::Display for Units {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Units::MM => write!(f, "mm"),
            Units::INCH => write!(f, "in"),
            Units::Unknown => write!(f, "unknown"),
        }
    }
}

/// Base CNC point structure representing a 6-axis coordinate
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CNCPoint {
    /// X-axis position
    pub x: f64,
    /// Y-axis position
    pub y: f64,
    /// Z-axis position
    pub z: f64,
    /// A-axis (4th axis) position
    pub a: f64,
    /// B-axis (5th axis) position
    pub b: f64,
    /// C-axis (6th axis) position
    pub c: f64,
    /// Coordinate unit
    pub unit: Units,
}

impl CNCPoint {
    /// Create a new 6-axis CNC point with all axes at zero
    pub fn new(unit: Units) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            a: 0.0,
            b: 0.0,
            c: 0.0,
            unit,
        }
    }

    /// Create a CNC point with specified 6-axis coordinates
    pub fn with_axes(
        x: f64,
        y: f64,
        z: f64,
        a: f64,
        b: f64,
        c: f64,
        unit: Units,
    ) -> Self {
        Self { x, y, z, a, b, c, unit }
    }

    /// Get all axes as a tuple
    pub fn get_axes(&self) -> (f64, f64, f64, f64, f64, f64) {
        (self.x, self.y, self.z, self.a, self.b, self.c)
    }

    /// Set all axes from a tuple
    pub fn set_axes(&mut self, x: f64, y: f64, z: f64, a: f64, b: f64, c: f64) {
        self.x = x;
        self.y = y;
        self.z = z;
        self.a = a;
        self.b = b;
        self.c = c;
    }

    /// Convert this point to a different unit
    pub fn convert_to(&self, target_unit: Units) -> Self {
        let scale = match (self.unit, target_unit) {
            (Units::MM, Units::INCH) => 1.0 / 25.4,
            (Units::INCH, Units::MM) => 25.4,
            _ => 1.0,
        };

        Self {
            x: self.x * scale,
            y: self.y * scale,
            z: self.z * scale,
            a: self.a * scale,
            b: self.b * scale,
            c: self.c * scale,
            unit: target_unit,
        }
    }
}

impl Default for CNCPoint {
    fn default() -> Self {
        Self::new(Units::MM)
    }
}

impl fmt::Display for CNCPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "X:{:.3} Y:{:.3} Z:{:.3} A:{:.3} B:{:.3} C:{:.3} ({})",
            self.x, self.y, self.z, self.a, self.b, self.c, self.unit
        )
    }
}

/// Position in 3D space with optional fourth axis (simplified for backward compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// X-axis position
    pub x: f64,
    /// Y-axis position
    pub y: f64,
    /// Z-axis position
    pub z: f64,
    /// Fourth axis (A/U) if present
    pub a: Option<f64>,
}

impl Position {
    /// Create a new position with X, Y, Z coordinates
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z, a: None }
    }

    /// Create a position with four axes including the A axis
    pub fn with_a(x: f64, y: f64, z: f64, a: f64) -> Self {
        Self {
            x,
            y,
            z,
            a: Some(a),
        }
    }

    /// Convert from a CNCPoint to Position (takes only X, Y, Z, A)
    pub fn from_cnc_point(point: &CNCPoint) -> Self {
        Self {
            x: point.x,
            y: point.y,
            z: point.z,
            a: Some(point.a),
        }
    }

    /// Convert this Position to a CNCPoint
    pub fn to_cnc_point(&self, unit: Units) -> CNCPoint {
        CNCPoint::with_axes(
            self.x,
            self.y,
            self.z,
            self.a.unwrap_or(0.0),
            0.0,
            0.0,
            unit,
        )
    }

    /// Calculate distance to another position (XYZ only)
    pub fn distance_to(&self, other: &Position) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Get absolute value of all coordinates
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
            a: self.a.map(|v| v.abs()),
        }
    }

    /// Add another position (component-wise)
    pub fn add(&self, other: &Position) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            a: match (self.a, other.a) {
                (Some(a1), Some(a2)) => Some(a1 + a2),
                (Some(a), None) | (None, Some(a)) => Some(a),
                _ => None,
            },
        }
    }

    /// Subtract another position (component-wise)
    pub fn subtract(&self, other: &Position) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            a: match (self.a, other.a) {
                (Some(a1), Some(a2)) => Some(a1 - a2),
                (Some(a), None) | (None, Some(a)) => Some(a),
                _ => None,
            },
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.a {
            Some(a) => write!(
                f,
                "X:{:.3} Y:{:.3} Z:{:.3} A:{:.3}",
                self.x, self.y, self.z, a
            ),
            None => write!(f, "X:{:.3} Y:{:.3} Z:{:.3}", self.x, self.y, self.z),
        }
    }
}

/// Partial position for updating only specific axes
///
/// Used when only some axes need to be updated. Each axis is represented as an `Option`
/// where `None` means "don't change this axis" and `Some(value)` means "set to value".
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct PartialPosition {
    /// X-axis position (if Some, update this axis)
    pub x: Option<f64>,
    /// Y-axis position (if Some, update this axis)
    pub y: Option<f64>,
    /// Z-axis position (if Some, update this axis)
    pub z: Option<f64>,
    /// A-axis position (if Some, update this axis)
    pub a: Option<f64>,
    /// B-axis position (if Some, update this axis)
    pub b: Option<f64>,
    /// C-axis position (if Some, update this axis)
    pub c: Option<f64>,
}

impl PartialPosition {
    /// Create a new empty partial position (all axes None)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a partial position with only X axis set
    pub fn x_only(x: f64) -> Self {
        Self {
            x: Some(x),
            ..Default::default()
        }
    }

    /// Create a partial position with only Y axis set
    pub fn y_only(y: f64) -> Self {
        Self {
            y: Some(y),
            ..Default::default()
        }
    }

    /// Create a partial position with only Z axis set
    pub fn z_only(z: f64) -> Self {
        Self {
            z: Some(z),
            ..Default::default()
        }
    }

    /// Create a partial position with XY axes set
    pub fn xy(x: f64, y: f64) -> Self {
        Self {
            x: Some(x),
            y: Some(y),
            ..Default::default()
        }
    }

    /// Create a partial position with XYZ axes set
    pub fn xyz(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: Some(x),
            y: Some(y),
            z: Some(z),
            ..Default::default()
        }
    }

    /// Apply this partial position to an existing position, updating only specified axes
    pub fn apply_to(&self, pos: &Position) -> Position {
        Position {
            x: self.x.unwrap_or(pos.x),
            y: self.y.unwrap_or(pos.y),
            z: self.z.unwrap_or(pos.z),
            a: self.a.or(pos.a),
        }
    }

    /// Apply this partial position to a CNC point, updating only specified axes
    pub fn apply_to_cnc_point(&self, point: &CNCPoint) -> CNCPoint {
        CNCPoint {
            x: self.x.unwrap_or(point.x),
            y: self.y.unwrap_or(point.y),
            z: self.z.unwrap_or(point.z),
            a: self.a.unwrap_or(point.a),
            b: self.b.unwrap_or(point.b),
            c: self.c.unwrap_or(point.c),
            unit: point.unit,
        }
    }

    /// Count how many axes are set in this partial position
    pub fn axis_count(&self) -> usize {
        [self.x, self.y, self.z, self.a, self.b, self.c]
            .iter()
            .filter(|opt| opt.is_some())
            .count()
    }

    /// Check if this partial position is empty (no axes set)
    pub fn is_empty(&self) -> bool {
        self.x.is_none()
            && self.y.is_none()
            && self.z.is_none()
            && self.a.is_none()
            && self.b.is_none()
            && self.c.is_none()
    }
}

/// Current status of the controller
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerStatus {
    /// Idle and ready for commands
    Idle,
    /// Processing a command
    Run,
    /// Paused during execution
    Hold,
    /// Alarm condition
    Alarm,
    /// Error state
    Error,
}

impl std::fmt::Display for ControllerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Run => write!(f, "Run"),
            Self::Hold => write!(f, "Hold"),
            Self::Alarm => write!(f, "Alarm"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Real-time status from the controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineStatus {
    /// Current position (machine coordinates)
    pub position: Position,
    /// Current position (work coordinates)
    pub work_position: Position,
    /// Current status
    pub status: ControllerStatus,
    /// Current spindle speed (RPM)
    pub spindle_speed: f64,
    /// Current feed rate (units per minute)
    pub feed_rate: f64,
}

impl Default for MachineStatus {
    fn default() -> Self {
        Self {
            position: Position::default(),
            work_position: Position::default(),
            status: ControllerStatus::Idle,
            spindle_speed: 0.0,
            feed_rate: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit conversion tests
    #[test]
    fn test_unit_display() {
        assert_eq!(Units::MM.to_string(), "mm");
        assert_eq!(Units::INCH.to_string(), "in");
        assert_eq!(Units::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_unit_conversion_mm_to_inch() {
        let result = Units::convert(25.4, Units::MM, Units::INCH);
        assert!((result - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_unit_conversion_inch_to_mm() {
        let result = Units::convert(1.0, Units::INCH, Units::MM);
        assert!((result - 25.4).abs() < 0.0001);
    }

    #[test]
    fn test_unit_conversion_same_unit() {
        let result = Units::convert(100.0, Units::MM, Units::MM);
        assert_eq!(result, 100.0);
    }

    // CNCPoint tests
    #[test]
    fn test_cncpoint_new() {
        let point = CNCPoint::new(Units::MM);
        assert_eq!(point.x, 0.0);
        assert_eq!(point.y, 0.0);
        assert_eq!(point.z, 0.0);
        assert_eq!(point.a, 0.0);
        assert_eq!(point.b, 0.0);
        assert_eq!(point.c, 0.0);
        assert_eq!(point.unit, Units::MM);
    }

    #[test]
    fn test_cncpoint_with_axes() {
        let point = CNCPoint::with_axes(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, Units::INCH);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
        assert_eq!(point.z, 3.0);
        assert_eq!(point.a, 4.0);
        assert_eq!(point.b, 5.0);
        assert_eq!(point.c, 6.0);
        assert_eq!(point.unit, Units::INCH);
    }

    #[test]
    fn test_cncpoint_get_axes() {
        let point = CNCPoint::with_axes(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, Units::MM);
        let axes = point.get_axes();
        assert_eq!(axes, (1.0, 2.0, 3.0, 4.0, 5.0, 6.0));
    }

    #[test]
    fn test_cncpoint_set_axes() {
        let mut point = CNCPoint::new(Units::MM);
        point.set_axes(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
        assert_eq!(point.z, 3.0);
        assert_eq!(point.a, 4.0);
        assert_eq!(point.b, 5.0);
        assert_eq!(point.c, 6.0);
    }

    #[test]
    fn test_cncpoint_convert_mm_to_inch() {
        let point_mm = CNCPoint::with_axes(25.4, 50.8, 0.0, 0.0, 0.0, 0.0, Units::MM);
        let point_inch = point_mm.convert_to(Units::INCH);
        assert!((point_inch.x - 1.0).abs() < 0.0001);
        assert!((point_inch.y - 2.0).abs() < 0.0001);
        assert_eq!(point_inch.unit, Units::INCH);
    }

    #[test]
    fn test_cncpoint_convert_inch_to_mm() {
        let point_inch = CNCPoint::with_axes(1.0, 2.0, 3.0, 0.0, 0.0, 0.0, Units::INCH);
        let point_mm = point_inch.convert_to(Units::MM);
        assert!((point_mm.x - 25.4).abs() < 0.0001);
        assert!((point_mm.y - 50.8).abs() < 0.0001);
        assert!((point_mm.z - 76.2).abs() < 0.0001);
        assert_eq!(point_mm.unit, Units::MM);
    }

    // Position tests
    #[test]
    fn test_position_creation() {
        let pos = Position::new(10.5, 20.3, 5.0);
        assert_eq!(pos.x, 10.5);
        assert_eq!(pos.y, 20.3);
        assert_eq!(pos.z, 5.0);
        assert_eq!(pos.a, None);
    }

    #[test]
    fn test_position_with_a() {
        let pos = Position::with_a(10.5, 20.3, 5.0, 45.0);
        assert_eq!(pos.a, Some(45.0));
    }

    #[test]
    fn test_position_distance() {
        let pos1 = Position::new(0.0, 0.0, 0.0);
        let pos2 = Position::new(3.0, 4.0, 0.0);
        let distance = pos1.distance_to(&pos2);
        assert!((distance - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_position_add() {
        let pos1 = Position::new(1.0, 2.0, 3.0);
        let pos2 = Position::new(4.0, 5.0, 6.0);
        let result = pos1.add(&pos2);
        assert_eq!(result.x, 5.0);
        assert_eq!(result.y, 7.0);
        assert_eq!(result.z, 9.0);
    }

    #[test]
    fn test_position_subtract() {
        let pos1 = Position::new(10.0, 20.0, 30.0);
        let pos2 = Position::new(1.0, 2.0, 3.0);
        let result = pos1.subtract(&pos2);
        assert_eq!(result.x, 9.0);
        assert_eq!(result.y, 18.0);
        assert_eq!(result.z, 27.0);
    }

    #[test]
    fn test_position_abs() {
        let pos = Position::new(-10.5, -20.3, 5.0);
        let abs_pos = pos.abs();
        assert_eq!(abs_pos.x, 10.5);
        assert_eq!(abs_pos.y, 20.3);
        assert_eq!(abs_pos.z, 5.0);
    }

    #[test]
    fn test_position_from_cnc_point() {
        let point = CNCPoint::with_axes(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, Units::MM);
        let pos = Position::from_cnc_point(&point);
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 3.0);
        assert_eq!(pos.a, Some(4.0));
    }

    #[test]
    fn test_position_to_cnc_point() {
        let pos = Position::with_a(1.0, 2.0, 3.0, 4.0);
        let point = pos.to_cnc_point(Units::INCH);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
        assert_eq!(point.z, 3.0);
        assert_eq!(point.a, 4.0);
        assert_eq!(point.unit, Units::INCH);
    }

    // PartialPosition tests
    #[test]
    fn test_partial_position_new() {
        let pp = PartialPosition::new();
        assert!(pp.is_empty());
    }

    #[test]
    fn test_partial_position_x_only() {
        let pp = PartialPosition::x_only(5.0);
        assert_eq!(pp.x, Some(5.0));
        assert_eq!(pp.y, None);
        assert_eq!(pp.axis_count(), 1);
    }

    #[test]
    fn test_partial_position_xyz() {
        let pp = PartialPosition::xyz(1.0, 2.0, 3.0);
        assert_eq!(pp.x, Some(1.0));
        assert_eq!(pp.y, Some(2.0));
        assert_eq!(pp.z, Some(3.0));
        assert_eq!(pp.axis_count(), 3);
    }

    #[test]
    fn test_partial_position_apply_to() {
        let pos = Position::new(10.0, 20.0, 30.0);
        let pp = PartialPosition::x_only(5.0);
        let result = pp.apply_to(&pos);
        assert_eq!(result.x, 5.0);
        assert_eq!(result.y, 20.0);
        assert_eq!(result.z, 30.0);
    }

    #[test]
    fn test_partial_position_apply_to_cnc_point() {
        let point = CNCPoint::with_axes(10.0, 20.0, 30.0, 40.0, 50.0, 60.0, Units::MM);
        let pp = PartialPosition {
            x: Some(1.0),
            y: None,
            z: Some(3.0),
            a: None,
            b: Some(5.0),
            c: None,
        };
        let result = pp.apply_to_cnc_point(&point);
        assert_eq!(result.x, 1.0);
        assert_eq!(result.y, 20.0);
        assert_eq!(result.z, 3.0);
        assert_eq!(result.a, 40.0);
        assert_eq!(result.b, 5.0);
        assert_eq!(result.c, 60.0);
    }

    #[test]
    fn test_machine_status_default() {
        let status = MachineStatus::default();
        assert_eq!(status.status, ControllerStatus::Idle);
        assert_eq!(status.spindle_speed, 0.0);
    }
}
