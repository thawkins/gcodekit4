//! Connection Panel - Task 68
//!
//! UI for managing serial/network connections to CNC controllers

use std::collections::HashMap;

/// Connection port information
#[derive(Debug, Clone)]
pub struct PortInfo {
    /// Port name/path
    pub name: String,
    /// Port description
    pub description: String,
    /// Is available
    pub available: bool,
}

impl PortInfo {
    /// Create new port info
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            available: true,
        }
    }
}

/// Common baud rates for serial communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BaudRate {
    /// 9600 baud
    BR9600 = 9600,
    /// 19200 baud
    BR19200 = 19200,
    /// 38400 baud
    BR38400 = 38400,
    /// 57600 baud
    BR57600 = 57600,
    /// 115200 baud
    BR115200 = 115200,
    /// 230400 baud
    BR230400 = 230400,
}

impl BaudRate {
    /// Get all available baud rates
    pub fn all() -> Vec<Self> {
        vec![
            BaudRate::BR9600,
            BaudRate::BR19200,
            BaudRate::BR38400,
            BaudRate::BR57600,
            BaudRate::BR115200,
            BaudRate::BR230400,
        ]
    }

    /// Get baud rate value as u32
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "9600" => Some(BaudRate::BR9600),
            "19200" => Some(BaudRate::BR19200),
            "38400" => Some(BaudRate::BR38400),
            "57600" => Some(BaudRate::BR57600),
            "115200" => Some(BaudRate::BR115200),
            "230400" => Some(BaudRate::BR230400),
            _ => None,
        }
    }
}

impl std::fmt::Display for BaudRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}

/// Connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    /// Serial/USB connection
    Serial,
    /// TCP/IP connection
    TCP,
    /// WebSocket connection
    WebSocket,
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serial => write!(f, "Serial"),
            Self::TCP => write!(f, "TCP/IP"),
            Self::WebSocket => write!(f, "WebSocket"),
        }
    }
}

/// Connection settings
#[derive(Debug, Clone)]
pub struct ConnectionSettings {
    /// Connection type
    pub connection_type: ConnectionType,
    /// Selected port
    pub port: String,
    /// Baud rate (for serial)
    pub baud_rate: BaudRate,
    /// Host (for TCP)
    pub host: String,
    /// Port number (for TCP/WebSocket)
    pub port_number: u16,
    /// Auto-reconnect enabled
    pub auto_reconnect: bool,
    /// Connection timeout (seconds)
    pub timeout: u32,
}

impl ConnectionSettings {
    /// Create new connection settings
    pub fn new() -> Self {
        Self {
            connection_type: ConnectionType::Serial,
            port: String::new(),
            baud_rate: BaudRate::BR115200,
            host: String::new(),
            port_number: 8080,
            auto_reconnect: true,
            timeout: 10,
        }
    }

    /// Set serial connection
    pub fn set_serial(mut self, port: impl Into<String>, baud_rate: BaudRate) -> Self {
        self.connection_type = ConnectionType::Serial;
        self.port = port.into();
        self.baud_rate = baud_rate;
        self
    }

    /// Set TCP connection
    pub fn set_tcp(mut self, host: impl Into<String>, port: u16) -> Self {
        self.connection_type = ConnectionType::TCP;
        self.host = host.into();
        self.port_number = port;
        self
    }

    /// Set WebSocket connection
    pub fn set_websocket(mut self, host: impl Into<String>, port: u16) -> Self {
        self.connection_type = ConnectionType::WebSocket;
        self.host = host.into();
        self.port_number = port;
        self
    }
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Disconnected
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Connection error
    Error,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting..."),
            Self::Connected => write!(f, "Connected"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Connection panel UI component
#[derive(Debug)]
pub struct ConnectionPanel {
    /// Available ports
    pub ports: Vec<PortInfo>,
    /// Current settings
    pub settings: ConnectionSettings,
    /// Current status
    pub status: ConnectionStatus,
    /// Status message
    pub status_message: String,
    /// Recent connections (port -> settings)
    pub recent_connections: HashMap<String, BaudRate>,
}

impl ConnectionPanel {
    /// Create new connection panel
    pub fn new() -> Self {
        Self {
            ports: Vec::new(),
            settings: ConnectionSettings::new(),
            status: ConnectionStatus::Disconnected,
            status_message: "Ready to connect".to_string(),
            recent_connections: HashMap::new(),
        }
    }

    /// Update available ports
    pub fn update_ports(&mut self, ports: Vec<PortInfo>) {
        self.ports = ports;
    }

    /// Select port
    pub fn select_port(&mut self, port: impl Into<String>) {
        self.settings.port = port.into();
    }

    /// Set baud rate
    pub fn set_baud_rate(&mut self, baud_rate: BaudRate) {
        self.settings.baud_rate = baud_rate;
    }

    /// Set connection type
    pub fn set_connection_type(&mut self, conn_type: ConnectionType) {
        self.settings.connection_type = conn_type;
    }

    /// Update connection status
    pub fn update_status(&mut self, status: ConnectionStatus, message: impl Into<String>) {
        self.status = status;
        self.status_message = message.into();
    }

    /// Add to recent connections
    pub fn add_recent_connection(&mut self, port: String, baud_rate: BaudRate) {
        self.recent_connections.insert(port, baud_rate);
    }

    /// Get recent connections list
    pub fn recent_connections_list(&self) -> Vec<String> {
        self.recent_connections.keys().cloned().collect()
    }

    /// Can connect
    pub fn can_connect(&self) -> bool {
        match self.settings.connection_type {
            ConnectionType::Serial => !self.settings.port.is_empty(),
            ConnectionType::TCP | ConnectionType::WebSocket => !self.settings.host.is_empty(),
        }
    }

    /// Get connection summary
    pub fn connection_summary(&self) -> String {
        match self.settings.connection_type {
            ConnectionType::Serial => {
                format!(
                    "{} @ {} baud",
                    self.settings.port,
                    self.settings.baud_rate.as_u32()
                )
            }
            ConnectionType::TCP => {
                format!("{}:{}", self.settings.host, self.settings.port_number)
            }
            ConnectionType::WebSocket => {
                format!("ws://{}:{}", self.settings.host, self.settings.port_number)
            }
        }
    }
}

impl Default for ConnectionPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_info() {
        let port = PortInfo::new("COM1", "USB Serial");
        assert_eq!(port.name, "COM1");
        assert_eq!(port.description, "USB Serial");
        assert!(port.available);
    }

    #[test]
    fn test_baud_rates() {
        let rates = BaudRate::all();
        assert_eq!(rates.len(), 6);
        assert_eq!(rates[4], BaudRate::BR115200);
    }

    #[test]
    fn test_baud_rate_parsing() {
        assert_eq!(BaudRate::from_str("115200"), Some(BaudRate::BR115200));
        assert_eq!(BaudRate::from_str("9999"), None);
    }

    #[test]
    fn test_connection_settings_serial() {
        let settings = ConnectionSettings::new().set_serial("COM1", BaudRate::BR115200);
        assert_eq!(settings.connection_type, ConnectionType::Serial);
        assert_eq!(settings.port, "COM1");
    }

    #[test]
    fn test_connection_panel() {
        let mut panel = ConnectionPanel::new();
        assert_eq!(panel.status, ConnectionStatus::Disconnected);

        panel.select_port("COM1");
        assert!(panel.can_connect());
    }

    #[test]
    fn test_recent_connections() {
        let mut panel = ConnectionPanel::new();
        panel.add_recent_connection("COM1".to_string(), BaudRate::BR115200);
        assert_eq!(panel.recent_connections_list().len(), 1);
    }

    #[test]
    fn test_connection_summary() {
        let mut panel = ConnectionPanel::new();
        panel.select_port("COM1");
        let summary = panel.connection_summary();
        assert!(summary.contains("COM1"));
        assert!(summary.contains("115200"));
    }
}
