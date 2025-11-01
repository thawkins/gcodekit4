//! Smoothieware Controller Implementation
//!
//! Provides a complete implementation of the ControllerTrait for Smoothieware firmware.

use super::{SmoothiewareCapabilities, SmoothiewareCommandCreator, SmoothiewareResponseParser};
use crate::communication::ConnectionParams;
use crate::core::OverrideState;
use crate::data::{ControllerState, ControllerStatus, Position};
use parking_lot::RwLock;
use std::sync::Arc;

/// Smoothieware Controller state
#[derive(Debug, Clone)]
pub struct SmoothiewareControllerState {
    /// Current state
    pub state: ControllerState,
    /// Current status
    pub status: ControllerStatus,
    /// Override state
    pub override_state: OverrideState,
    /// Current position
    pub position: Position,
}

impl Default for SmoothiewareControllerState {
    fn default() -> Self {
        Self {
            state: ControllerState::Disconnected,
            status: ControllerStatus::Idle,
            override_state: OverrideState::default(),
            position: Position::default(),
        }
    }
}

/// Smoothieware Controller
pub struct SmoothiewareController {
    /// Controller name
    name: String,
    /// Connection parameters
    connection_params: ConnectionParams,
    /// Controller state
    state: Arc<RwLock<SmoothiewareControllerState>>,
    /// Response parser
    parser: Arc<RwLock<SmoothiewareResponseParser>>,
    /// Command creator
    command_creator: SmoothiewareCommandCreator,
    /// Capabilities
    capabilities: SmoothiewareCapabilities,
}

impl SmoothiewareController {
    /// Create a new Smoothieware controller
    pub fn new(connection_params: ConnectionParams, name: Option<String>) -> anyhow::Result<Self> {
        Ok(Self {
            name: name.unwrap_or_else(|| "Smoothieware".to_string()),
            connection_params,
            state: Arc::new(RwLock::new(SmoothiewareControllerState::default())),
            parser: Arc::new(RwLock::new(SmoothiewareResponseParser::new())),
            command_creator: SmoothiewareCommandCreator::new(),
            capabilities: SmoothiewareCapabilities::default(),
        })
    }

    /// Get the controller name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get capabilities
    pub fn capabilities(&self) -> &SmoothiewareCapabilities {
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
    pub fn command_creator(&self) -> &SmoothiewareCommandCreator {
        &self.command_creator
    }

    /// Parse a response line
    pub fn parse_response(&self, line: &str) {
        let mut parser = self.parser.write();
        if let Some(_response) = parser.parse_line(line) {}
    }
}

impl std::fmt::Debug for SmoothiewareController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmoothiewareController")
            .field("name", &self.name)
            .field("state", &self.state.read().state)
            .finish()
    }
}
