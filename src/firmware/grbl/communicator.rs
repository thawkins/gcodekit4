//! GRBL-specific communicator
//!
//! Handles GRBL protocol specifics including character counting and streaming protocols.
//! GRBL uses a real-time character counting protocol to manage command flow without
//! needing traditional handshaking.

use crate::communication::{Communicator, ConnectionParams};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::trace;

/// GRBL communicator configuration
#[derive(Debug, Clone)]
pub struct GrblCommunicatorConfig {
    /// RX buffer size for character counting (typical 128 bytes for GRBL)
    pub rx_buffer_size: usize,
    /// TX buffer size for command queueing (typical 128 bytes for GRBL)
    pub tx_buffer_size: usize,
}

impl Default for GrblCommunicatorConfig {
    fn default() -> Self {
        Self {
            rx_buffer_size: 128,
            tx_buffer_size: 128,
        }
    }
}

/// Manages character counting state for GRBL streaming protocol
#[derive(Debug, Clone, Copy)]
struct CharacterCountingState {
    /// Number of characters sent but not yet acknowledged
    pub pending_chars: usize,
    /// Total characters acknowledged by GRBL
    pub acked_chars: usize,
}

impl Default for CharacterCountingState {
    fn default() -> Self {
        Self {
            pending_chars: 0,
            acked_chars: 0,
        }
    }
}

/// GRBL-specific communicator
///
/// Implements character counting protocol for GRBL firmware, which uses
/// real-time character counting instead of traditional flow control.
pub struct GrblCommunicator {
    /// Underlying communicator
    communicator: Arc<RwLock<Box<dyn Communicator>>>,
    /// Character counting state
    char_counting: Arc<RwLock<CharacterCountingState>>,
    /// Configuration
    config: Arc<GrblCommunicatorConfig>,
    /// Whether communicator is running
    running: Arc<RwLock<bool>>,
}

impl GrblCommunicator {
    /// Create a new GRBL communicator from an existing communicator
    pub fn new(communicator: Box<dyn Communicator>, config: GrblCommunicatorConfig) -> Self {

        Self {
            communicator: Arc::new(RwLock::new(communicator)),
            char_counting: Arc::new(RwLock::new(CharacterCountingState::default())),
            config: Arc::new(config),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Connect to GRBL device
    pub fn connect(&self, params: &ConnectionParams) -> anyhow::Result<()> {
        let mut comm = self.communicator.write();
        comm.connect(params)
            .map_err(|e| anyhow::anyhow!("Connection failed: {}", e))?;
        *self.running.write() = true;
        Ok(())
    }

    /// Disconnect from GRBL device
    pub fn disconnect(&self) -> anyhow::Result<()> {
        *self.running.write() = false;
        let mut comm = self.communicator.write();
        comm.disconnect()
            .map_err(|e| anyhow::anyhow!("Disconnection failed: {}", e))?;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.communicator.read().is_connected()
    }

    /// Send raw bytes to GRBL device
    pub fn send_bytes(&self, data: &[u8]) -> anyhow::Result<()> {
        trace!("Sending {} bytes to GRBL", data.len());
        let mut comm = self.communicator.write();
        comm.send(data)
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))?;

        // Update character counting
        let mut counting = self.char_counting.write();
        counting.pending_chars += data.len();

        Ok(())
    }

    /// Send a command to GRBL device
    ///
    /// This sends a command with proper CRLF termination.
    pub fn send_command(&self, command: &str) -> anyhow::Result<()> {
        let formatted = if command.ends_with('\n') {
            command.to_string()
        } else {
            format!("{}\n", command)
        };

        self.send_bytes(formatted.as_bytes())?;
        Ok(())
    }

    /// Read response from GRBL device
    pub fn read_response(&self) -> anyhow::Result<Vec<u8>> {
        let mut comm = self.communicator.write();
        let response = comm
            .receive()
            .map_err(|e| anyhow::anyhow!("Receive failed: {}", e))?;
        trace!("Received {} bytes from GRBL", response.len());
        Ok(response)
    }

    /// Read a line from GRBL (terminated by newline)
    pub fn read_line(&self) -> anyhow::Result<String> {
        let response = self.read_response()?;
        let line = String::from_utf8_lossy(&response).to_string();
        Ok(line)
    }

    /// Acknowledge received characters (update character counting)
    ///
    /// This should be called when GRBL acknowledges receipt of characters.
    pub fn acknowledge_chars(&self, count: usize) {
        let mut counting = self.char_counting.write();
        counting.acked_chars = counting.acked_chars.saturating_add(count);
        counting.pending_chars = counting.pending_chars.saturating_sub(count);
    }

    /// Get available buffer space (for character counting protocol)
    pub fn get_available_buffer(&self) -> usize {
        let counting = self.char_counting.read();
        let available = self
            .config
            .rx_buffer_size
            .saturating_sub(counting.pending_chars);
        available
    }

    /// Get pending character count
    pub fn get_pending_chars(&self) -> usize {
        self.char_counting.read().pending_chars
    }

    /// Check if ready to send next command (character counting)
    pub fn is_ready_to_send(&self, command_size: usize) -> bool {
        let available = self.get_available_buffer();
        available >= command_size
    }

    /// Clear all pending data (reset character counting)
    pub fn clear(&self) -> anyhow::Result<()> {
        *self.char_counting.write() = CharacterCountingState::default();
        Ok(())
    }

    /// Get whether communicator is running
    pub fn is_running(&self) -> bool {
        *self.running.read()
    }

    /// Send a real-time command (single byte)
    ///
    /// Real-time commands are sent immediately and don't follow the character counting protocol.
    pub fn send_realtime_byte(&self, byte: u8) -> anyhow::Result<()> {
        self.send_bytes(&[byte])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_counting_state_default() {
        let state = CharacterCountingState::default();
        assert_eq!(state.pending_chars, 0);
        assert_eq!(state.acked_chars, 0);
    }

    #[test]
    fn test_grbl_config_default() {
        let config = GrblCommunicatorConfig::default();
        assert_eq!(config.rx_buffer_size, 128);
        assert_eq!(config.tx_buffer_size, 128);
    }
}
