//! Core controller management and state machine
//!
//! This module provides:
//! - Abstract controller trait for all firmware types
//! - State machine for tracking controller state
//! - Event system for communication between components
//! - Command queuing and execution
//! - Listener registration for controller events

pub mod event;
pub mod listener;
pub mod message;

use crate::data::{ControllerState, ControllerStatus, PartialPosition};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::sync::Arc;

pub use listener::{ControllerListener, ControllerListenerHandle};

/// Override state for controller operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverrideState {
    /// Feed rate override percentage (0-200%)
    pub feed_override: u16,
    /// Rapid override percentage (25%, 50%, 100%)
    pub rapid_override: u8,
    /// Spindle override percentage (0-200%)
    pub spindle_override: u16,
}

impl Default for OverrideState {
    fn default() -> Self {
        Self {
            feed_override: 100,
            rapid_override: 100,
            spindle_override: 100,
        }
    }
}

/// Trait for CNC controller implementations
///
/// This trait defines the interface that all controller implementations
/// (GRBL, TinyG, g2core, etc.) must implement.
#[async_trait]
pub trait ControllerTrait: Send + Sync {
    /// Get the controller name/type
    fn name(&self) -> &str;

    /// Get the current controller state
    fn get_state(&self) -> ControllerState;

    /// Get the current controller status
    fn get_status(&self) -> ControllerStatus;

    /// Get the current override state
    fn get_override_state(&self) -> OverrideState;

    /// Connect to the machine
    ///
    /// # Returns
    /// Returns a Result indicating connection success or failure
    async fn connect(&mut self) -> anyhow::Result<()>;

    /// Disconnect from the machine
    async fn disconnect(&mut self) -> anyhow::Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool {
        !matches!(self.get_state(), ControllerState::Disconnected)
    }

    // ===== Action Methods =====

    /// Send a single G-code command
    async fn send_command(&mut self, command: &str) -> anyhow::Result<()>;

    /// Send multiple G-code commands
    async fn send_commands(&mut self, commands: &[&str]) -> anyhow::Result<()> {
        for cmd in commands {
            self.send_command(cmd).await?;
        }
        Ok(())
    }

    /// Home the machine (G28)
    async fn home(&mut self) -> anyhow::Result<()>;

    /// Perform a soft reset
    async fn reset(&mut self) -> anyhow::Result<()>;

    /// Clear all alarms
    async fn clear_alarm(&mut self) -> anyhow::Result<()>;

    /// Unlock the machine (kill alarm lock)
    async fn unlock(&mut self) -> anyhow::Result<()>;

    // ===== Jogging Methods =====

    /// Start continuous jogging in a direction
    ///
    /// # Arguments
    /// * `axis` - Axis to jog (X, Y, Z, A, B, or C)
    /// * `direction` - Direction (+1 or -1)
    /// * `feed_rate` - Feed rate in units/min
    async fn jog_start(&mut self, axis: char, direction: i32, feed_rate: f64)
        -> anyhow::Result<()>;

    /// Stop jogging and cancel any pending jog commands
    async fn jog_stop(&mut self) -> anyhow::Result<()>;

    /// Perform an incremental jog
    ///
    /// # Arguments
    /// * `axis` - Axis to jog
    /// * `distance` - Distance to jog
    /// * `feed_rate` - Feed rate in units/min
    async fn jog_incremental(
        &mut self,
        axis: char,
        distance: f64,
        feed_rate: f64,
    ) -> anyhow::Result<()>;

    // ===== Streaming Methods =====

    /// Start streaming a G-code file
    async fn start_streaming(&mut self) -> anyhow::Result<()>;

    /// Pause streaming
    async fn pause_streaming(&mut self) -> anyhow::Result<()>;

    /// Resume streaming after pause
    async fn resume_streaming(&mut self) -> anyhow::Result<()>;

    /// Cancel streaming
    async fn cancel_streaming(&mut self) -> anyhow::Result<()>;

    // ===== Probing Methods =====

    /// Probe to work surface (Z-axis)
    ///
    /// # Arguments
    /// * `feed_rate` - Probe feed rate in units/min
    async fn probe_z(&mut self, feed_rate: f64) -> anyhow::Result<PartialPosition>;

    /// Probe X axis corner
    ///
    /// # Arguments
    /// * `feed_rate` - Probe feed rate in units/min
    async fn probe_x(&mut self, feed_rate: f64) -> anyhow::Result<PartialPosition>;

    /// Probe Y axis corner
    ///
    /// # Arguments
    /// * `feed_rate` - Probe feed rate in units/min
    async fn probe_y(&mut self, feed_rate: f64) -> anyhow::Result<PartialPosition>;

    // ===== Override Methods =====

    /// Set feed rate override
    ///
    /// # Arguments
    /// * `percentage` - Override percentage (0-200)
    async fn set_feed_override(&mut self, percentage: u16) -> anyhow::Result<()>;

    /// Set rapid override
    ///
    /// # Arguments
    /// * `percentage` - Override percentage (25, 50, or 100)
    async fn set_rapid_override(&mut self, percentage: u8) -> anyhow::Result<()>;

    /// Set spindle override
    ///
    /// # Arguments
    /// * `percentage` - Override percentage (0-200)
    async fn set_spindle_override(&mut self, percentage: u16) -> anyhow::Result<()>;

    // ===== Work Coordinate System Methods =====

    /// Set work zero (set work position to current position)
    async fn set_work_zero(&mut self) -> anyhow::Result<()>;

    /// Set work zero on specific axes
    ///
    /// # Arguments
    /// * `axes` - String containing axes to zero (e.g., "XY", "Z")
    async fn set_work_zero_axes(&mut self, axes: &str) -> anyhow::Result<()>;

    /// Go to work zero
    async fn go_to_work_zero(&mut self) -> anyhow::Result<()>;

    /// Set work coordinate system
    ///
    /// # Arguments
    /// * `wcs` - Work coordinate system number (54-59 for G54-G59)
    async fn set_work_coordinate_system(&mut self, wcs: u8) -> anyhow::Result<()>;

    /// Get work coordinate system offset
    ///
    /// # Arguments
    /// * `wcs` - Work coordinate system number
    async fn get_wcs_offset(&self, wcs: u8) -> anyhow::Result<PartialPosition>;

    // ===== Status Query Methods =====

    /// Request status from controller
    async fn query_status(&mut self) -> anyhow::Result<ControllerStatus>;

    /// Request settings from controller
    async fn query_settings(&mut self) -> anyhow::Result<()>;

    /// Request parser state
    async fn query_parser_state(&mut self) -> anyhow::Result<()>;

    // ===== Listener Management =====

    /// Register a controller listener
    fn register_listener(
        &mut self,
        listener: Arc<dyn ControllerListener>,
    ) -> ControllerListenerHandle;

    /// Unregister a listener
    fn unregister_listener(&mut self, handle: ControllerListenerHandle);

    /// Get number of registered listeners
    fn listener_count(&self) -> usize;
}

/// Simple controller implementation for testing
pub struct SimpleController {
    state: Arc<RwLock<ControllerState>>,
    status: Arc<RwLock<ControllerStatus>>,
    name: String,
}

impl SimpleController {
    /// Create a new simple controller
    pub fn new(name: &str) -> Self {
        Self {
            state: Arc::new(RwLock::new(ControllerState::Disconnected)),
            status: Arc::new(RwLock::new(ControllerStatus::Idle)),
            name: name.to_string(),
        }
    }

    /// Set the controller state
    pub fn set_state(&self, new_state: ControllerState) {
        *self.state.write() = new_state;
    }
}

#[async_trait]
impl ControllerTrait for SimpleController {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_state(&self) -> ControllerState {
        *self.state.read()
    }

    fn get_status(&self) -> ControllerStatus {
        *self.status.read()
    }

    fn get_override_state(&self) -> OverrideState {
        OverrideState::default()
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        self.set_state(ControllerState::Idle);
        Ok(())
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        self.set_state(ControllerState::Disconnected);
        Ok(())
    }

    async fn send_command(&mut self, _command: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn home(&mut self) -> anyhow::Result<()> {
        self.set_state(ControllerState::Home);
        Ok(())
    }

    async fn reset(&mut self) -> anyhow::Result<()> {
        self.set_state(ControllerState::Disconnected);
        Ok(())
    }

    async fn clear_alarm(&mut self) -> anyhow::Result<()> {
        self.set_state(ControllerState::Idle);
        Ok(())
    }

    async fn unlock(&mut self) -> anyhow::Result<()> {
        self.set_state(ControllerState::Idle);
        Ok(())
    }

    async fn jog_start(
        &mut self,
        _axis: char,
        _direction: i32,
        _feed_rate: f64,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn jog_stop(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn jog_incremental(
        &mut self,
        _axis: char,
        _distance: f64,
        _feed_rate: f64,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn start_streaming(&mut self) -> anyhow::Result<()> {
        self.set_state(ControllerState::Run);
        Ok(())
    }

    async fn pause_streaming(&mut self) -> anyhow::Result<()> {
        self.set_state(ControllerState::Hold);
        Ok(())
    }

    async fn resume_streaming(&mut self) -> anyhow::Result<()> {
        self.set_state(ControllerState::Run);
        Ok(())
    }

    async fn cancel_streaming(&mut self) -> anyhow::Result<()> {
        self.set_state(ControllerState::Idle);
        Ok(())
    }

    async fn probe_z(&mut self, _feed_rate: f64) -> anyhow::Result<PartialPosition> {
        Ok(PartialPosition::default())
    }

    async fn probe_x(&mut self, _feed_rate: f64) -> anyhow::Result<PartialPosition> {
        Ok(PartialPosition::default())
    }

    async fn probe_y(&mut self, _feed_rate: f64) -> anyhow::Result<PartialPosition> {
        Ok(PartialPosition::default())
    }

    async fn set_feed_override(&mut self, _percentage: u16) -> anyhow::Result<()> {
        Ok(())
    }

    async fn set_rapid_override(&mut self, _percentage: u8) -> anyhow::Result<()> {
        Ok(())
    }

    async fn set_spindle_override(&mut self, _percentage: u16) -> anyhow::Result<()> {
        Ok(())
    }

    async fn set_work_zero(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn set_work_zero_axes(&mut self, _axes: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn go_to_work_zero(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn set_work_coordinate_system(&mut self, _wcs: u8) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_wcs_offset(&self, _wcs: u8) -> anyhow::Result<PartialPosition> {
        Ok(PartialPosition::default())
    }

    async fn query_status(&mut self) -> anyhow::Result<ControllerStatus> {
        Ok(self.get_status())
    }

    async fn query_settings(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn query_parser_state(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn register_listener(
        &mut self,
        _listener: Arc<dyn ControllerListener>,
    ) -> ControllerListenerHandle {
        ControllerListenerHandle(uuid::Uuid::new_v4().to_string())
    }

    fn unregister_listener(&mut self, _handle: ControllerListenerHandle) {}

    fn listener_count(&self) -> usize {
        0
    }
}

impl Default for SimpleController {
    fn default() -> Self {
        Self::new("default")
    }
}
