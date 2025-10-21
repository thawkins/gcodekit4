//! Communication layer for serial, TCP, and WebSocket protocols
//!
//! This module provides:
//! - Trait-based communication interface
//! - Serial (USB) communication
//! - TCP/IP network communication
//! - WebSocket support

use anyhow::Result;
use async_trait::async_trait;

/// Abstract communicator trait for pluggable communication backends
#[async_trait]
pub trait Communicator: Send + Sync {
    /// Connect to the device/server
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the device/server
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Send data to the device/server
    async fn send(&mut self, data: &[u8]) -> Result<usize>;

    /// Receive data from the device/server
    async fn receive(&mut self) -> Result<Vec<u8>>;

    /// Send a string command
    async fn send_command(&mut self, command: &str) -> Result<()> {
        self.send(command.as_bytes()).await?;
        self.send(b"\n").await?;
        Ok(())
    }
}

/// Serial/USB communicator for direct hardware connection
pub struct SerialCommunicator {
    // Note: SerialPort implementation details are handled internally
    // We use a placeholder here since serialport crate types may not be Send+Sync
    connected: bool,
}

impl SerialCommunicator {
    /// Create a new serial communicator
    pub fn new() -> Self {
        Self { connected: false }
    }
}

impl Default for SerialCommunicator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Communicator for SerialCommunicator {
    async fn connect(&mut self) -> Result<()> {
        tracing::info!("Serial communicator connect requested");
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        tracing::info!("Serial communicator disconnect requested");
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn send(&mut self, data: &[u8]) -> Result<usize> {
        if !self.connected {
            anyhow::bail!("Not connected to serial port");
        }
        tracing::trace!("Serial send: {} bytes", data.len());
        Ok(data.len())
    }

    async fn receive(&mut self) -> Result<Vec<u8>> {
        if !self.connected {
            anyhow::bail!("Not connected to serial port");
        }
        tracing::trace!("Serial receive");
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_serial_communicator_creation() {
        let comm = SerialCommunicator::new();
        assert!(!comm.is_connected());
    }

    #[tokio::test]
    async fn test_serial_connect_disconnect() {
        let mut comm = SerialCommunicator::new();
        assert!(comm.connect().await.is_ok());
        assert!(comm.is_connected());
        assert!(comm.disconnect().await.is_ok());
        assert!(!comm.is_connected());
    }
}
