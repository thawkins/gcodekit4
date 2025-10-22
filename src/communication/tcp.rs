//! TCP/Network communication implementation
//!
//! Provides TCP/IP network communication for remote CNC controller connections.
//!
//! Supports:
//! - TCP client connections
//! - Hostname/IP and port configuration
//! - Connection timeout
//! - Read/write operations
//! - Connection pooling preparation
//! - Error handling and recovery

use crate::{ConnectionDriver, ConnectionParams, Error, Result};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// TCP connection information
#[derive(Debug, Clone)]
pub struct TcpConnectionInfo {
    /// Remote host address (hostname or IP)
    pub host: String,

    /// Remote port number
    pub port: u16,

    /// Local bind address (optional)
    pub local_addr: Option<String>,

    /// Connection timeout
    pub timeout: Duration,
}

impl TcpConnectionInfo {
    /// Create new TCP connection info
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            local_addr: None,
            timeout: Duration::from_secs(5),
        }
    }

    /// Set the connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the local bind address
    pub fn with_local_addr(mut self, addr: &str) -> Self {
        self.local_addr = Some(addr.to_string());
        self
    }

    /// Get connection address string (host:port)
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// TCP network interface trait
pub trait TcpPort: Send + Sync {
    /// Write data to the connection
    fn write(&mut self, data: &[u8]) -> io::Result<usize>;

    /// Read data from the connection
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;

    /// Get the peer address string
    fn peer_addr(&self) -> io::Result<String>;

    /// Close the connection
    fn close(&mut self) -> io::Result<()>;
}

/// Real TCP connection using std::net::TcpStream
pub struct RealTcpPort {
    stream: TcpStream,
}

impl RealTcpPort {
    /// Open a TCP connection with the given parameters
    pub fn open(params: &ConnectionParams) -> Result<Self> {
        if params.driver != ConnectionDriver::Tcp {
            return Err(Error::other("RealTcpPort requires Tcp driver type"));
        }

        let address = format!("{}:{}", params.port, params.network_port);
        tracing::info!("Connecting to TCP server at {}", address);

        match TcpStream::connect_timeout(
            &address
                .parse()
                .map_err(|e| Error::other(format!("Invalid address '{}': {}", address, e)))?,
            Duration::from_millis(params.timeout_ms),
        ) {
            Ok(stream) => {
                // Set read timeout
                stream
                    .set_read_timeout(Some(Duration::from_millis(params.timeout_ms)))
                    .map_err(|e| Error::other(format!("Failed to set read timeout: {}", e)))?;

                // Set write timeout
                stream
                    .set_write_timeout(Some(Duration::from_millis(params.timeout_ms)))
                    .map_err(|e| Error::other(format!("Failed to set write timeout: {}", e)))?;

                tracing::info!("Successfully connected to TCP server {}", address);
                Ok(RealTcpPort { stream })
            }
            Err(e) => {
                tracing::error!("Failed to connect to TCP server {}: {}", address, e);
                Err(Error::other(format!(
                    "Failed to connect to {}: {}",
                    address, e
                )))
            }
        }
    }

    /// Reconnect the TCP connection
    pub fn reconnect(&mut self, params: &ConnectionParams) -> Result<()> {
        self.close().ok();
        let new_port = RealTcpPort::open(params)?;
        self.stream = new_port.stream;
        Ok(())
    }
}

impl TcpPort for RealTcpPort {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.stream.write(data)
    }

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }

    fn peer_addr(&self) -> io::Result<String> {
        Ok(self.stream.peer_addr()?.to_string())
    }

    fn close(&mut self) -> io::Result<()> {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
        Ok(())
    }
}


