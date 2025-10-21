//! Core controller management and state machine
//!
//! This module provides:
//! - Abstract controller trait for all firmware types
//! - State machine for tracking controller state
//! - Event system for communication between components
//! - Command queuing and execution

use parking_lot::RwLock;
use std::sync::Arc;

/// Represents the operational state of a CNC controller
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerState {
    /// Disconnected from the machine
    Disconnected,
    /// Connected but idle
    Idle,
    /// Actively executing a G-Code command
    Executing,
    /// Paused mid-execution
    Paused,
    /// Alarm condition active
    Alarm,
    /// Machine is homing
    Homing,
    /// Machine is probing
    Probing,
}

/// Main controller interface
///
/// This is the primary interface for interacting with a CNC machine.
/// Different controller implementations (GRBL, TinyG, etc.) implement this trait.
pub struct Controller {
    state: Arc<RwLock<ControllerState>>,
}

impl Controller {
    /// Create a new controller instance
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ControllerState::Disconnected)),
        }
    }

    /// Get the current controller state
    pub fn get_state(&self) -> ControllerState {
        *self.state.read()
    }

    /// Set the controller state
    pub fn set_state(&self, new_state: ControllerState) {
        *self.state.write() = new_state;
        tracing::debug!("Controller state changed to: {:?}", new_state);
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}
