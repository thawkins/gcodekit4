//! Serial port communication implementation
//!
//! Provides low-level serial port operations for direct hardware connection
//! to CNC controllers via USB or RS-232.
//!
//! Supports:
//! - Port enumeration and discovery
//! - Baud rate configuration
//! - Flow control settings
//! - Parity and stop bit configuration
//! - Blocking read/write operations

use crate::{ConnectionDriver, ConnectionParams, Error, Result, SerialParity};
use std::io::{self, Read, Write};
use std::time::Duration;

/// Result type for serial operations
pub type SerialResult<T> = std::result::Result<T, SerialPortError>;

/// Serial port specific errors
#[derive(Debug, Clone)]
pub struct SerialPortError {
    message: String,
}

impl std::fmt::Display for SerialPortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Serial port error: {}", self.message)
    }
}

impl std::error::Error for SerialPortError {}

impl SerialPortError {
    /// Create a new serial port error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Information about an available serial port
#[derive(Debug, Clone)]
pub struct SerialPortInfo {
    /// Port name (e.g., "/dev/ttyUSB0", "COM3")
    pub port_name: String,

    /// Port description (e.g., "USB Serial Port")
    pub description: String,

    /// Manufacturer name if available
    pub manufacturer: Option<String>,

    /// Serial number if available
    pub serial_number: Option<String>,

    /// USB vendor ID if applicable
    pub vid: Option<u16>,

    /// USB product ID if applicable
    pub pid: Option<u16>,
}

impl SerialPortInfo {
    /// Create a new port info
    pub fn new(port_name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            port_name: port_name.into(),
            description: description.into(),
            manufacturer: None,
            serial_number: None,
            vid: None,
            pid: None,
        }
    }

    /// Set manufacturer
    pub fn with_manufacturer(mut self, manufacturer: impl Into<String>) -> Self {
        self.manufacturer = Some(manufacturer.into());
        self
    }

    /// Set serial number
    pub fn with_serial_number(mut self, serial_number: impl Into<String>) -> Self {
        self.serial_number = Some(serial_number.into());
        self
    }

    /// Set USB IDs
    pub fn with_usb_ids(mut self, vid: u16, pid: u16) -> Self {
        self.vid = Some(vid);
        self.pid = Some(pid);
        self
    }
}

/// List available serial ports on the system
///
/// Returns a list of available COM ports with information about each port.
/// On Linux, this lists /dev/tty* devices. On Windows, it lists COM ports.
/// On macOS, it lists /dev/tty.* and /dev/cu.* devices.
pub fn list_ports() -> Result<Vec<SerialPortInfo>> {
    tracing::debug!("Listing available serial ports");

    match serialport::available_ports() {
        Ok(ports) => {
            let port_infos: Vec<SerialPortInfo> = ports
                .iter()
                .map(|port| {
                    let info = SerialPortInfo::new(&port.port_name, get_port_description(&port));

                    let info = match &port.port_type {
                        serialport::SerialPortType::UsbPort(usb_info) => {
                            let mut info = info.with_usb_ids(usb_info.vid, usb_info.pid);
                            if let Some(ref mfg) = usb_info.manufacturer {
                                info = info.with_manufacturer(mfg);
                            }
                            if let Some(ref serial) = usb_info.serial_number {
                                info = info.with_serial_number(serial);
                            }
                            info
                        }
                        _ => info,
                    };

                    info
                })
                .collect();

            tracing::info!("Found {} serial ports", port_infos.len());
            Ok(port_infos)
        }
        Err(e) => {
            tracing::error!("Failed to enumerate serial ports: {}", e);
            Err(Error::other(format!("Failed to enumerate ports: {}", e)))
        }
    }
}

/// Get a user-friendly description for a port
fn get_port_description(port: &serialport::SerialPortInfo) -> String {
    match &port.port_type {
        serialport::SerialPortType::UsbPort(usb_info) => {
            format!(
                "USB {} {}",
                usb_info.manufacturer.as_deref().unwrap_or("Device"),
                usb_info.product.as_deref().unwrap_or("Serial Port")
            )
        }
        serialport::SerialPortType::BluetoothPort => "Bluetooth Serial".to_string(),
        serialport::SerialPortType::PciPort => "PCI Serial".to_string(),
        _ => "Serial Port".to_string(),
    }
}

/// Convert a parity setting to serialport format
fn to_serialport_parity(parity: SerialParity) -> serialport::Parity {
    match parity {
        SerialParity::None => serialport::Parity::None,
        SerialParity::Even => serialport::Parity::Even,
        SerialParity::Odd => serialport::Parity::Odd,
    }
}

/// Low-level serial port interface
pub trait SerialPort: Send + Sync {
    /// Write data to the port
    fn write(&mut self, data: &[u8]) -> io::Result<usize>;

    /// Read data from the port
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;

    /// Get the port name
    fn name(&self) -> String;

    /// Close the port
    fn close(&mut self) -> io::Result<()>;
}

/// Trait for serial port I/O operations
pub trait ReadWrite: std::io::Read + std::io::Write + Send + Sync {}
impl<T: std::io::Read + std::io::Write + Send + Sync> ReadWrite for T {}

/// Real serial port implementation using serialport crate
pub struct RealSerialPort {
    port: Box<dyn ReadWrite>,
}

impl RealSerialPort {
    /// Open a serial port with the given parameters
    pub fn open(params: &ConnectionParams) -> Result<Self> {
        if params.driver != ConnectionDriver::Serial {
            return Err(Error::other("RealSerialPort requires Serial driver type"));
        }

        let builder = serialport::new(&params.port, params.baud_rate)
            .timeout(Duration::from_millis(params.timeout_ms))
            .data_bits(match params.data_bits {
                5 => serialport::DataBits::Five,
                6 => serialport::DataBits::Six,
                7 => serialport::DataBits::Seven,
                8 => serialport::DataBits::Eight,
                _ => {
                    return Err(Error::other(format!(
                        "Invalid data bits: {}",
                        params.data_bits
                    )))
                }
            })
            .stop_bits(match params.stop_bits {
                1 => serialport::StopBits::One,
                2 => serialport::StopBits::Two,
                _ => {
                    return Err(Error::other(format!(
                        "Invalid stop bits: {}",
                        params.stop_bits
                    )))
                }
            })
            .parity(to_serialport_parity(params.parity))
            .flow_control(if params.flow_control {
                serialport::FlowControl::Hardware
            } else {
                serialport::FlowControl::None
            });

        tracing::info!(
            "Opening serial port {} at {} baud",
            params.port,
            params.baud_rate
        );

        match builder.open_native() {
            Ok(port) => {
                tracing::info!("Successfully opened serial port {}", params.port);
                Ok(RealSerialPort {
                    port: Box::new(port),
                })
            }
            Err(e) => {
                tracing::error!("Failed to open serial port {}: {}", params.port, e);
                Err(Error::other(format!(
                    "Failed to open port {}: {}",
                    params.port, e
                )))
            }
        }
    }
}

impl SerialPort for RealSerialPort {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.port.write(data)
    }

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.port.read(buf)
    }

    fn name(&self) -> String {
        "serial_port".to_string()
    }

    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Mock serial port for testing
pub struct MockSerialPort {
    name: String,
    data: Vec<u8>,
    pos: usize,
    written: Vec<u8>,
}

impl MockSerialPort {
    /// Create a new mock port
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            data: Vec::new(),
            pos: 0,
            written: Vec::new(),
        }
    }

    /// Set data to be read
    pub fn set_read_data(&mut self, data: Vec<u8>) {
        self.data = data;
        self.pos = 0;
    }

    /// Get data that was written
    pub fn get_written(&self) -> &[u8] {
        &self.written
    }

    /// Clear written data
    pub fn clear_written(&mut self) {
        self.written.clear();
    }
}

impl SerialPort for MockSerialPort {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.written.extend_from_slice(data);
        Ok(data.len())
    }

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let available = self.data.len() - self.pos;
        let to_read = buf.len().min(available);

        if to_read > 0 {
            buf[..to_read].copy_from_slice(&self.data[self.pos..self.pos + to_read]);
            self.pos += to_read;
        }

        Ok(to_read)
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}
