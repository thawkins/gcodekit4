//! Data models for positions, status, and machine information
//!
//! This module provides:
//! - Position tracking (machine and work coordinates)
//! - Controller status representation
//! - Machine capabilities
//! - Command structures

use serde::{Deserialize, Serialize};

/// Position in 3D space with optional fourth axis
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
    /// Create a new position
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z, a: None }
    }

    /// Create a position with four axes
    pub fn with_a(x: f64, y: f64, z: f64, a: f64) -> Self {
        Self {
            x,
            y,
            z,
            a: Some(a),
        }
    }

    /// Calculate distance to another position (XYZ only)
    pub fn distance_to(&self, other: &Position) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
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
    fn test_machine_status_default() {
        let status = MachineStatus::default();
        assert_eq!(status.status, ControllerStatus::Idle);
        assert_eq!(status.spindle_speed, 0.0);
    }
}
