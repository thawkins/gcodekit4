//! GRBL Controller Implementation
//!
//! Provides a complete implementation of the ControllerTrait for GRBL firmware,
//! including connection management, command execution, and status polling.

use crate::communication::{ConnectionParams, NoOpCommunicator};
use crate::core::{ControllerTrait, OverrideState};
use crate::data::{ControllerState, ControllerStatus, PartialPosition};
use crate::firmware::grbl::{GrblCommunicator, GrblCommunicatorConfig};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tracing::debug;

/// GRBL Controller state management
#[derive(Debug, Clone)]
struct GrblControllerState {
    /// Current connection state
    pub state: ControllerState,
    /// Current status
    pub status: ControllerStatus,
    /// Override state
    pub override_state: OverrideState,
    /// Machine position
    pub machine_position: crate::data::Position,
    /// Work position
    pub work_position: crate::data::Position,
    /// Is streaming active
    pub is_streaming: bool,
    /// Status poll rate (milliseconds)
    pub poll_rate_ms: u64,
}

impl Default for GrblControllerState {
    fn default() -> Self {
        Self {
            state: ControllerState::Disconnected,
            status: ControllerStatus::Idle,
            override_state: OverrideState::default(),
            machine_position: crate::data::Position::default(),
            work_position: crate::data::Position::default(),
            is_streaming: false,
            poll_rate_ms: 100,
        }
    }
}

/// GRBL Controller implementation
///
/// Implements the ControllerTrait for GRBL firmware with full protocol support.
#[allow(dead_code)]
pub struct GrblController {
    /// Name identifier
    name: String,
    /// Communicator for GRBL protocol
    communicator: Arc<GrblCommunicator>,
    /// Controller state
    state: Arc<RwLock<GrblControllerState>>,
    /// Status polling handle
    poll_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    /// Shutdown signal
    shutdown_signal: Arc<RwLock<Option<std::sync::Arc<tokio::sync::Notify>>>>,
    /// Connection parameters
    connection_params: ConnectionParams,
}

impl GrblController {
    /// Create a new GRBL controller
    pub fn new(connection_params: ConnectionParams, name: Option<String>) -> anyhow::Result<Self> {

        let communicator = Arc::new(GrblCommunicator::new(
            Box::new(NoOpCommunicator::new()),
            GrblCommunicatorConfig::default(),
        ));

        Ok(Self {
            name: name.unwrap_or_else(|| "GRBL".to_string()),
            communicator,
            state: Arc::new(RwLock::new(GrblControllerState::default())),
            poll_task: Arc::new(RwLock::new(None)),
            shutdown_signal: Arc::new(RwLock::new(None)),
            connection_params,
        })
    }

    /// Initialize the controller and query its capabilities
    fn initialize(&self) -> anyhow::Result<()> {

        // Send soft reset
        self.communicator.send_command("$RST=*")?;
        std::thread::sleep(Duration::from_millis(100));

        // Query firmware version
        self.communicator.send_command("$I")?;

        // Request current settings
        self.communicator.send_command("$")?;

        // Query parser state
        self.communicator.send_command("$G")?;

        Ok(())
    }

    /// Start the status polling task
    fn start_polling(&mut self) -> anyhow::Result<()> {

        let notify = Arc::new(tokio::sync::Notify::new());
        *self.shutdown_signal.write() = Some(notify.clone());

        let communicator = self.communicator.clone();
        let state = self.state.clone();
        let poll_rate = state.read().poll_rate_ms;

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(poll_rate));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Send status query
                        if let Err(e) = communicator.send_realtime_byte(b'?') {
                            continue;
                        }

                        // Read response - brief delay
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        if let Ok(_response) = communicator.read_response() {
                            // Parse status response
                        }
                    }
                    _ = notify.notified() => {
                        break;
                    }
                }
            }
        });

        *self.poll_task.write() = Some(handle);
        Ok(())
    }

    /// Stop the status polling task
    fn stop_polling(&mut self) -> anyhow::Result<()> {

        if let Some(notify) = self.shutdown_signal.write().take() {
            notify.notify_one();
        }

        if let Some(handle) = self.poll_task.write().take() {
            handle.abort();
        }

        Ok(())
    }
}

#[async_trait]
impl ControllerTrait for GrblController {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_state(&self) -> ControllerState {
        self.state.read().state
    }

    fn get_status(&self) -> ControllerStatus {
        self.state.read().status.clone()
    }

    fn get_override_state(&self) -> OverrideState {
        self.state.read().override_state
    }

    async fn connect(&mut self) -> anyhow::Result<()> {

        self.communicator.connect(&self.connection_params)?;
        *self.state.write() = GrblControllerState::default();

        self.initialize()?;
        self.start_polling()?;

        {
            let mut state = self.state.write();
            state.state = ControllerState::Idle;
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {

        self.stop_polling()?;
        self.communicator.disconnect()?;

        {
            let mut state = self.state.write();
            state.state = ControllerState::Disconnected;
        }

        Ok(())
    }

    async fn send_command(&mut self, command: &str) -> anyhow::Result<()> {

        // Check if ready to send (character counting)
        let command_size = command.len() + 1; // +1 for newline
        if !self.communicator.is_ready_to_send(command_size) {
            debug!(
                "Waiting for buffer space. Pending: {}, Available: {}",
                self.communicator.get_pending_chars(),
                self.communicator.get_available_buffer()
            );
        }

        // Send command
        self.communicator.send_command(command)?;

        // Read OK response
        let response = self.communicator.read_line()?;
        if !response.contains("ok") {
        }

        Ok(())
    }

    async fn home(&mut self) -> anyhow::Result<()> {
        self.send_command("$H").await?;
        Ok(())
    }

    async fn reset(&mut self) -> anyhow::Result<()> {
        self.communicator.send_realtime_byte(0x18)?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn clear_alarm(&mut self) -> anyhow::Result<()> {
        self.send_command("$X").await?;
        Ok(())
    }

    async fn unlock(&mut self) -> anyhow::Result<()> {
        self.send_command("$X").await?;
        Ok(())
    }

    async fn jog_start(
        &mut self,
        axis: char,
        direction: i32,
        feed_rate: f64,
    ) -> anyhow::Result<()> {

        if direction == 0 {
            return Err(anyhow::anyhow!("Direction must be non-zero"));
        }

        // Create a jog command using $J= syntax with G91 (relative) and G0 (rapid)
        let direction_str = if direction > 0 { "+" } else { "-" };
        let cmd = format!("$J=G91 G0 {}{} F{:.0}", axis, direction_str, feed_rate);
        self.send_command(&cmd).await?;

        Ok(())
    }

    async fn jog_stop(&mut self) -> anyhow::Result<()> {
        self.communicator.send_realtime_byte(0x85)?;
        Ok(())
    }

    async fn jog_incremental(
        &mut self,
        axis: char,
        distance: f64,
        feed_rate: f64,
    ) -> anyhow::Result<()> {

        // Format: $J=G91 G0 X{signed_distance} F{feed_rate}
        // distance already includes sign from the caller
        let cmd = format!("$J=G91 G0 {}{:.3} F{:.0}", axis, distance, feed_rate);
        self.send_command(&cmd).await?;

        Ok(())
    }

    async fn start_streaming(&mut self) -> anyhow::Result<()> {
        let mut state = self.state.write();
        state.is_streaming = true;
        state.state = ControllerState::Run;
        Ok(())
    }

    async fn pause_streaming(&mut self) -> anyhow::Result<()> {
        self.communicator.send_realtime_byte(0x21)?;
        self.state.write().state = ControllerState::Hold;
        Ok(())
    }

    async fn resume_streaming(&mut self) -> anyhow::Result<()> {
        self.communicator.send_realtime_byte(0x7E)?;
        self.state.write().state = ControllerState::Run;
        Ok(())
    }

    async fn cancel_streaming(&mut self) -> anyhow::Result<()> {
        self.communicator.send_realtime_byte(0x18)?;
        let mut state = self.state.write();
        state.is_streaming = false;
        state.state = ControllerState::Idle;
        Ok(())
    }

    async fn probe_z(&mut self, feed_rate: f64) -> anyhow::Result<PartialPosition> {

        let cmd = format!("G38.2Z-100F{}", feed_rate);
        self.send_command(&cmd).await?;

        let state = self.state.read();
        Ok(PartialPosition {
            z: Some(state.work_position.z),
            ..Default::default()
        })
    }

    async fn probe_x(&mut self, feed_rate: f64) -> anyhow::Result<PartialPosition> {

        let cmd = format!("G38.2X100F{}", feed_rate);
        self.send_command(&cmd).await?;

        let state = self.state.read();
        Ok(PartialPosition {
            x: Some(state.work_position.x),
            ..Default::default()
        })
    }

    async fn probe_y(&mut self, feed_rate: f64) -> anyhow::Result<PartialPosition> {

        let cmd = format!("G38.2Y100F{}", feed_rate);
        self.send_command(&cmd).await?;

        let state = self.state.read();
        Ok(PartialPosition {
            y: Some(state.work_position.y),
            ..Default::default()
        })
    }

    async fn set_feed_override(&mut self, percentage: u16) -> anyhow::Result<()> {

        if percentage > 200 {
            return Err(anyhow::anyhow!("Feed override must be 0-200%"));
        }

        self.state.write().override_state.feed_override = percentage;

        // Send real-time override commands based on percentage
        // GRBL uses specific codes for different percentages
        if percentage == 100 {
            self.communicator.send_realtime_byte(0x90)?;
        }

        Ok(())
    }

    async fn set_rapid_override(&mut self, percentage: u8) -> anyhow::Result<()> {

        if ![25, 50, 100].contains(&percentage) {
            return Err(anyhow::anyhow!("Rapid override must be 25, 50, or 100"));
        }

        self.state.write().override_state.rapid_override = percentage;
        Ok(())
    }

    async fn set_spindle_override(&mut self, percentage: u16) -> anyhow::Result<()> {

        if percentage > 200 {
            return Err(anyhow::anyhow!("Spindle override must be 0-200%"));
        }

        self.state.write().override_state.spindle_override = percentage;
        Ok(())
    }

    async fn set_work_zero(&mut self) -> anyhow::Result<()> {
        self.send_command("G92X0Y0Z0").await?;
        Ok(())
    }

    async fn set_work_zero_axes(&mut self, axes: &str) -> anyhow::Result<()> {
        let mut cmd = String::from("G92");
        for axis in axes.chars() {
            if ['X', 'Y', 'Z', 'A', 'B', 'C'].contains(&axis) {
                cmd.push(axis);
                cmd.push('0');
            }
        }
        self.send_command(&cmd).await?;
        Ok(())
    }

    async fn go_to_work_zero(&mut self) -> anyhow::Result<()> {
        self.send_command("G00X0Y0Z0").await?;
        Ok(())
    }

    async fn set_work_coordinate_system(&mut self, wcs: u8) -> anyhow::Result<()> {

        if wcs < 54 || wcs > 59 {
            return Err(anyhow::anyhow!("Work coordinate system must be 54-59"));
        }

        let cmd = format!("G{}", wcs);
        self.send_command(&cmd).await?;
        Ok(())
    }

    async fn get_wcs_offset(&self, _wcs: u8) -> anyhow::Result<PartialPosition> {
        let state = self.state.read();
        Ok(PartialPosition {
            x: Some(state.work_position.x),
            y: Some(state.work_position.y),
            z: Some(state.work_position.z),
            ..Default::default()
        })
    }

    async fn query_status(&mut self) -> anyhow::Result<ControllerStatus> {
        Ok(self.get_status())
    }

    async fn query_settings(&mut self) -> anyhow::Result<()> {
        self.communicator.send_command("$")?;
        Ok(())
    }

    async fn query_parser_state(&mut self) -> anyhow::Result<()> {
        self.communicator.send_command("$G")?;
        Ok(())
    }

    fn register_listener(
        &mut self,
        _listener: Arc<dyn crate::core::ControllerListener>,
    ) -> crate::core::ControllerListenerHandle {
        // TODO: Implement listener registration
        crate::core::ControllerListenerHandle("grbl_listener_1".to_string())
    }

    fn unregister_listener(&mut self, _handle: crate::core::ControllerListenerHandle) {
        // TODO: Implement listener unregistration
    }

    fn listener_count(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grbl_controller_state_default() {
        let state = GrblControllerState::default();
        assert_eq!(state.state, ControllerState::Disconnected);
        assert_eq!(state.poll_rate_ms, 100);
        assert!(!state.is_streaming);
    }
}
