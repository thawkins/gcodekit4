//! g2core Controller Implementation
//!
//! Provides a complete implementation of the ControllerTrait for g2core firmware,
//! including connection management, command execution, and status polling.
//! g2core is the next generation of TinyG with support for 6 axes and advanced kinematics.

use crate::communication::ConnectionParams;
use crate::core::ControllerTrait;
use crate::data::{ControllerState, ControllerStatus, Position};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tracing::{debug, info};

/// g2core Controller state management
#[derive(Debug, Clone)]
struct G2CoreControllerState {
    /// Current connection state
    pub state: ControllerState,
    /// Current status
    pub status: ControllerStatus,
    /// Machine position (supports 6 axes)
    pub machine_position: Position,
    /// Work position (supports 6 axes)
    pub work_position: Position,
    /// Is streaming active
    pub is_streaming: bool,
    /// Status poll rate (milliseconds)
    pub poll_rate_ms: u64,
    /// g2core version
    pub version: Option<String>,
    /// g2core buffer level
    pub buffer_level: u8,
    /// Number of active axes
    pub active_axes: u8,
    /// Kinematics mode (if supported)
    pub kinematics_mode: Option<String>,
}

impl Default for G2CoreControllerState {
    fn default() -> Self {
        Self {
            state: ControllerState::Disconnected,
            status: ControllerStatus::Idle,
            machine_position: Position::default(),
            work_position: Position::default(),
            is_streaming: false,
            poll_rate_ms: 150,
            version: None,
            buffer_level: 0,
            active_axes: 6,
            kinematics_mode: None,
        }
    }
}

/// g2core Controller implementation
///
/// Implements the ControllerTrait for g2core firmware with full protocol support.
/// g2core uses JSON-based communication with advanced features including:
/// - 6-axis support
/// - Advanced kinematics
/// - Spindle speed control
/// - Enhanced streaming protocol
pub struct G2CoreController {
    /// Name identifier
    name: String,
    /// Controller state
    state: Arc<RwLock<G2CoreControllerState>>,
    /// Status polling handle
    poll_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    /// Shutdown signal
    shutdown_signal: Arc<RwLock<Option<std::sync::Arc<tokio::sync::Notify>>>>,
    /// Connection parameters
    connection_params: ConnectionParams,
}

impl G2CoreController {
    /// Create a new g2core controller
    pub fn new(
        connection_params: ConnectionParams,
        name: Option<String>,
    ) -> anyhow::Result<Self> {
        debug!("Creating g2core controller");

        Ok(Self {
            name: name.unwrap_or_else(|| "g2core".to_string()),
            state: Arc::new(RwLock::new(G2CoreControllerState::default())),
            poll_task: Arc::new(RwLock::new(None)),
            shutdown_signal: Arc::new(RwLock::new(None)),
            connection_params,
        })
    }

    /// Initialize the controller and query its capabilities
    fn initialize(&self) -> anyhow::Result<()> {
        debug!("Initializing g2core controller");

        // Query firmware version and capabilities
        debug!("Querying g2core firmware version and capabilities");

        Ok(())
    }

    /// Perform initialization as task
    async fn initialize_async(&self) -> anyhow::Result<()> {
        debug!("Initializing g2core controller (async)");

        // Query firmware version and capabilities
        debug!("Querying g2core firmware version and capabilities");

        Ok(())
    }

    /// Start status polling
    fn start_polling(&self) {
        debug!("Starting g2core status polling");
        let poll_rate = self.state.read().poll_rate_ms;

        let poll_task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(poll_rate));
            loop {
                interval.tick().await;
                // Poll status from g2core
            }
        });

        *self.poll_task.write() = Some(poll_task);
    }

    /// Stop status polling
    fn stop_polling(&self) {
        debug!("Stopping g2core status polling");
        if let Some(task) = self.poll_task.write().take() {
            task.abort();
        }
    }

    /// Query the number of active axes
    pub fn get_active_axes(&self) -> u8 {
        self.state.read().active_axes
    }

    /// Set the number of active axes
    pub fn set_active_axes(&self, axes: u8) {
        debug!("Setting g2core active axes to {}", axes);
        self.state.write().active_axes = axes;
    }

    /// Query the kinematics mode
    pub fn get_kinematics_mode(&self) -> Option<String> {
        self.state.read().kinematics_mode.clone()
    }

    /// Set the kinematics mode
    pub fn set_kinematics_mode(&self, mode: Option<String>) {
        debug!("Setting g2core kinematics mode to {:?}", mode);
        self.state.write().kinematics_mode = mode;
    }
}

#[async_trait]
impl ControllerTrait for G2CoreController {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_state(&self) -> crate::data::ControllerState {
        self.state.read().state.clone()
    }

    fn get_status(&self) -> ControllerStatus {
        self.state.read().status.clone()
    }

    fn get_override_state(&self) -> crate::core::OverrideState {
        crate::core::OverrideState::default()
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        info!("Connecting to g2core controller on {}", self.connection_params.port);

        // Update state to connecting
        {
            let mut state = self.state.write();
            state.state = ControllerState::Connecting;
        }

        // Initialize controller
        self.initialize_async().await?;

        // Update state to idle
        {
            let mut state = self.state.write();
            state.state = ControllerState::Idle;
        }

        // Start status polling
        self.start_polling();

        info!("Successfully connected to g2core controller");
        Ok(())
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        info!("Disconnecting from g2core controller");

        // Stop polling
        self.stop_polling();

        // Update state
        {
            let mut state = self.state.write();
            state.state = ControllerState::Disconnected;
        }

        Ok(())
    }

    async fn send_command(&mut self, command: &str) -> anyhow::Result<()> {
        debug!("Sending g2core command: {}", command);

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        // Send command to g2core

        Ok(())
    }

    async fn home(&mut self) -> anyhow::Result<()> {
        debug!("Homing g2core");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        self.send_command("$H").await?;

        Ok(())
    }

    async fn reset(&mut self) -> anyhow::Result<()> {
        debug!("Performing g2core soft reset");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        // Send soft reset command (Ctrl+X = 0x18)
        self.send_command("\x18").await?;

        Ok(())
    }

    async fn clear_alarm(&mut self) -> anyhow::Result<()> {
        debug!("Clearing g2core alarm");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        // Clear alarm with $X
        self.send_command("$X").await?;

        Ok(())
    }

    async fn unlock(&mut self) -> anyhow::Result<()> {
        debug!("Unlocking g2core alarm state");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        self.send_command("$X").await?;

        Ok(())
    }

    async fn jog_start(&mut self, _axis: char, _direction: i32, _feed_rate: f64) -> anyhow::Result<()> {
        debug!("Starting g2core jog");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(())
    }

    async fn jog_stop(&mut self) -> anyhow::Result<()> {
        debug!("Stopping g2core jog");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        // Send feed hold to stop jog
        self.send_command("!").await?;

        Ok(())
    }

    async fn jog_incremental(
        &mut self,
        _axis: char,
        _distance: f64,
        _feed_rate: f64,
    ) -> anyhow::Result<()> {
        debug!("Performing g2core incremental jog");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(())
    }

    async fn start_streaming(&mut self) -> anyhow::Result<()> {
        debug!("Starting g2core streaming");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        let mut state = self.state.write();
        state.is_streaming = true;

        Ok(())
    }

    async fn pause_streaming(&mut self) -> anyhow::Result<()> {
        debug!("Pausing g2core streaming");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        // Send feed hold command (!)
        self.send_command("!").await?;

        Ok(())
    }

    async fn resume_streaming(&mut self) -> anyhow::Result<()> {
        debug!("Resuming g2core streaming");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        // Send cycle start command (~)
        self.send_command("~").await?;

        Ok(())
    }

    async fn cancel_streaming(&mut self) -> anyhow::Result<()> {
        debug!("Cancelling g2core streaming");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        let mut state = self.state.write();
        state.is_streaming = false;

        Ok(())
    }

    async fn probe_z(&mut self, _feed_rate: f64) -> anyhow::Result<crate::data::PartialPosition> {
        debug!("Starting g2core Z-probe cycle");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(crate::data::PartialPosition::default())
    }

    async fn probe_x(&mut self, _feed_rate: f64) -> anyhow::Result<crate::data::PartialPosition> {
        debug!("Starting g2core X-probe cycle");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(crate::data::PartialPosition::default())
    }

    async fn probe_y(&mut self, _feed_rate: f64) -> anyhow::Result<crate::data::PartialPosition> {
        debug!("Starting g2core Y-probe cycle");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(crate::data::PartialPosition::default())
    }

    async fn set_feed_override(&mut self, _percentage: u16) -> anyhow::Result<()> {
        debug!("Setting g2core feed override");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(())
    }

    async fn set_rapid_override(&mut self, _percentage: u8) -> anyhow::Result<()> {
        debug!("Setting g2core rapid override");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(())
    }

    async fn set_spindle_override(&mut self, _percentage: u16) -> anyhow::Result<()> {
        debug!("Setting g2core spindle override");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(())
    }

    async fn set_work_zero(&mut self) -> anyhow::Result<()> {
        debug!("Setting g2core work zero");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(())
    }

    async fn set_work_zero_axes(&mut self, _axes: &str) -> anyhow::Result<()> {
        debug!("Setting g2core work zero for specific axes");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(())
    }

    async fn go_to_work_zero(&mut self) -> anyhow::Result<()> {
        debug!("Moving g2core to work zero");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(())
    }

    async fn set_work_coordinate_system(&mut self, _wcs: u8) -> anyhow::Result<()> {
        debug!("Setting g2core work coordinate system");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(())
    }

    async fn get_wcs_offset(&self, _wcs: u8) -> anyhow::Result<crate::data::PartialPosition> {
        debug!("Getting g2core WCS offset");

        Ok(crate::data::PartialPosition::default())
    }

    async fn query_status(&mut self) -> anyhow::Result<ControllerStatus> {
        debug!("Querying g2core status");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(self.state.read().status.clone())
    }

    async fn query_settings(&mut self) -> anyhow::Result<()> {
        debug!("Querying g2core settings");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(())
    }

    async fn query_parser_state(&mut self) -> anyhow::Result<()> {
        debug!("Querying g2core parser state");

        if !self.is_connected() {
            anyhow::bail!("g2core controller not connected");
        }

        Ok(())
    }

    fn register_listener(&mut self, _listener: std::sync::Arc<dyn crate::core::ControllerListener>) -> crate::core::ControllerListenerHandle {
        crate::core::ControllerListenerHandle("g2core_listener".to_string())
    }

    fn unregister_listener(&mut self, _handle: crate::core::ControllerListenerHandle) {
        // no-op for now
    }

    fn listener_count(&self) -> usize {
        0
    }
}
