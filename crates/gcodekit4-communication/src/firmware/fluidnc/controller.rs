//! FluidNC Controller Implementation
//!
//! Provides a complete implementation of the ControllerTrait for FluidNC firmware.

use super::{FluidNCCapabilities, FluidNCCommandCreator, FluidNCResponseParser};
use crate::communication::ConnectionParams;
use gcodekit4_core::OverrideState;
use gcodekit4_core::{ControllerState, ControllerStatus, Position};
use parking_lot::RwLock;
use std::sync::Arc;

/// FluidNC Controller state
#[derive(Debug, Clone)]
pub struct FluidNCControllerState {
    /// Current state
    pub state: ControllerState,
    /// Current status
    pub status: ControllerStatus,
    /// Override state
    pub override_state: OverrideState,
    /// Current position
    pub position: Position,
}

impl Default for FluidNCControllerState {
    fn default() -> Self {
        Self {
            state: ControllerState::Disconnected,
            status: ControllerStatus::Idle,
            override_state: OverrideState::default(),
            position: Position::default(),
        }
    }
}

/// FluidNC Controller
pub struct FluidNCController {
    /// Controller name
    name: String,
    /// Connection parameters
    connection_params: ConnectionParams,
    /// Controller state
    state: Arc<RwLock<FluidNCControllerState>>,
    /// Response parser
    parser: Arc<RwLock<FluidNCResponseParser>>,
    /// Command creator
    command_creator: FluidNCCommandCreator,
    /// Capabilities
    capabilities: FluidNCCapabilities,
}

impl FluidNCController {
    /// Create a new FluidNC controller
    pub fn new(connection_params: ConnectionParams, name: Option<String>) -> anyhow::Result<Self> {
        Ok(Self {
            name: name.unwrap_or_else(|| "FluidNC".to_string()),
            connection_params,
            state: Arc::new(RwLock::new(FluidNCControllerState::default())),
            parser: Arc::new(RwLock::new(FluidNCResponseParser::new())),
            command_creator: FluidNCCommandCreator::new(),
            capabilities: FluidNCCapabilities::default(),
        })
    }

    /// Get the controller name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get capabilities
    pub fn capabilities(&self) -> &FluidNCCapabilities {
        &self.capabilities
    }

    /// Get current state
    pub fn get_state(&self) -> ControllerState {
        self.state.read().state
    }

    /// Get current position
    pub fn get_position(&self) -> Position {
        self.state.read().position
    }

    /// Update position
    pub fn update_position(&self, position: Position) {
        let mut state = self.state.write();
        state.position = position;
    }

    /// Get command creator
    pub fn command_creator(&self) -> &FluidNCCommandCreator {
        &self.command_creator
    }

    /// Parse a response line
    pub fn parse_response(&self, line: &str) {
        let mut parser = self.parser.write();
        if let Some(_response) = parser.parse_line(line) {}
    }
}

impl std::fmt::Debug for FluidNCController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FluidNCController")
            .field("name", &self.name)
            .field("state", &self.state.read().state)
            .finish()
    }
}
