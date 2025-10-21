//! Error handling for GCodeKit4
//!
//! Provides comprehensive error types for all layers of the application:
//! - Controller errors (device/firmware related)
//! - G-Code errors (parsing/validation)
//! - Connection errors (communication)
//! - Firmware errors (firmware-specific)
//!
//! All error types use `thiserror` for ergonomic error handling.

use thiserror::Error;

/// Controller error type
///
/// Represents errors related to CNC controller operation,
/// including state machine violations, command failures, and device issues.
#[derive(Error, Debug, Clone)]
pub enum ControllerError {
    /// Controller is not connected
    #[error("Controller not connected")]
    NotConnected,

    /// Controller is already connected
    #[error("Controller already connected")]
    AlreadyConnected,

    /// Controller operation timed out
    #[error("Controller operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Invalid state transition
    #[error("Invalid state transition from {current:?} to {requested:?}")]
    InvalidStateTransition { current: String, requested: String },

    /// Command was rejected by controller
    #[error("Command rejected: {reason}")]
    CommandRejected { reason: String },

    /// Buffer overflow - too many commands queued
    #[error("Buffer overflow: {message}")]
    BufferOverflow { message: String },

    /// Alarm condition detected
    #[error("Alarm: {code} - {message}")]
    Alarm { code: u32, message: String },

    /// Machine hard limit triggered
    #[error("Hard limit triggered on {axis}")]
    HardLimit { axis: String },

    /// Machine soft limit exceeded
    #[error("Soft limit exceeded on {axis}")]
    SoftLimit { axis: String },

    /// Probe operation failed
    #[error("Probe failed: {reason}")]
    ProbeFailed { reason: String },

    /// Homing cycle failed
    #[error("Homing failed: {reason}")]
    HomingFailed { reason: String },

    /// Unknown controller state
    #[error("Unknown controller state: {state}")]
    UnknownState { state: String },

    /// Generic controller error
    #[error("Controller error: {message}")]
    Other { message: String },
}

/// G-Code error type
///
/// Represents errors related to G-Code parsing, validation, and processing.
#[derive(Error, Debug, Clone)]
pub enum GcodeError {
    /// Invalid G-Code syntax
    #[error("Invalid syntax at line {line_number}: {reason}")]
    InvalidSyntax { line_number: u32, reason: String },

    /// Unknown G-Code command
    #[error("Unknown G-Code at line {line_number}: {code}")]
    UnknownCode { line_number: u32, code: String },

    /// Invalid parameter value
    #[error("Invalid parameter '{param}' at line {line_number}: {reason}")]
    InvalidParameter {
        line_number: u32,
        param: String,
        reason: String,
    },

    /// Missing required parameter
    #[error("Missing required parameter '{param}' at line {line_number}")]
    MissingParameter { line_number: u32, param: String },

    /// Coordinate out of machine limits
    #[error("Coordinate {coordinate} out of limits at line {line_number}: {bounds}")]
    CoordinateOutOfBounds {
        line_number: u32,
        coordinate: String,
        bounds: String,
    },

    /// Invalid modal state
    #[error("Invalid modal state: {reason}")]
    InvalidModalState { reason: String },

    /// Tool not found
    #[error("Tool {tool_number} not found")]
    ToolNotFound { tool_number: u32 },

    /// Probe not present when required
    #[error("Probe required but not available")]
    ProbeNotAvailable,

    /// Spindle error
    #[error("Spindle error: {reason}")]
    SpindleError { reason: String },

    /// Coolant system error
    #[error("Coolant error: {reason}")]
    CoolantError { reason: String },

    /// File parsing error
    #[error("File error: {reason}")]
    FileError { reason: String },

    /// Generic G-Code error
    #[error("G-Code error: {message}")]
    Other { message: String },
}

/// Connection error type
///
/// Represents errors related to communication with CNC controllers,
/// including serial port, TCP, and WebSocket connection issues.
#[derive(Error, Debug, Clone)]
pub enum ConnectionError {
    /// Port not found
    #[error("Port not found: {port}")]
    PortNotFound { port: String },

    /// Port is already in use
    #[error("Port already in use: {port}")]
    PortInUse { port: String },

    /// Failed to open port
    #[error("Failed to open port {port}: {reason}")]
    FailedToOpen { port: String, reason: String },

    /// Connection timeout
    #[error("Connection timeout after {timeout_ms}ms")]
    ConnectionTimeout { timeout_ms: u64 },

    /// Connection lost
    #[error("Connection lost: {reason}")]
    ConnectionLost { reason: String },

    /// Invalid hostname/IP
    #[error("Invalid hostname: {hostname}")]
    InvalidHostname { hostname: String },

    /// Failed to resolve hostname
    #[error("Failed to resolve hostname {hostname}")]
    HostnameResolution { hostname: String },

    /// TCP connection error
    #[error("TCP connection error: {reason}")]
    TcpError { reason: String },

    /// WebSocket error
    #[error("WebSocket error: {reason}")]
    WebSocketError { reason: String },

    /// Serial port error
    #[error("Serial port error: {reason}")]
    SerialError { reason: String },

    /// Baud rate not supported
    #[error("Baud rate {baud} not supported")]
    UnsupportedBaudRate { baud: u32 },

    /// I/O error
    #[error("I/O error: {reason}")]
    IoError { reason: String },

    /// Invalid connection parameters
    #[error("Invalid connection parameters: {reason}")]
    InvalidParameters { reason: String },

    /// Generic connection error
    #[error("Connection error: {message}")]
    Other { message: String },
}

/// Firmware error type
///
/// Represents errors specific to firmware implementations and protocols.
#[derive(Error, Debug, Clone)]
pub enum FirmwareError {
    /// Unknown firmware type
    #[error("Unknown firmware type: {firmware_type}")]
    UnknownFirmware { firmware_type: String },

    /// Firmware version not supported
    #[error("Firmware version {version} not supported")]
    UnsupportedVersion { version: String },

    /// Protocol mismatch
    #[error("Protocol mismatch: expected {expected}, got {actual}")]
    ProtocolMismatch { expected: String, actual: String },

    /// Unsupported feature
    #[error("Feature not supported by {firmware}: {feature}")]
    UnsupportedFeature { firmware: String, feature: String },

    /// Settings not available
    #[error("Setting {setting} not available")]
    SettingNotAvailable { setting: String },

    /// Invalid setting value
    #[error("Invalid setting value for {setting}: {reason}")]
    InvalidSettingValue { setting: String, reason: String },

    /// Capability not available
    #[error("Capability not available: {capability}")]
    CapabilityNotAvailable { capability: String },

    /// Response parsing error
    #[error("Failed to parse firmware response: {reason}")]
    ResponseParseError { reason: String },

    /// Command not supported by firmware
    #[error("Command not supported by {firmware}")]
    CommandNotSupported { firmware: String },

    /// Configuration error
    #[error("Firmware configuration error: {reason}")]
    ConfigurationError { reason: String },

    /// Generic firmware error
    #[error("Firmware error: {message}")]
    Other { message: String },
}

/// Main error type for GCodeKit4
///
/// A unified error type that can represent any error from all layers.
/// This is the primary error type used in public APIs.
#[derive(Error, Debug)]
pub enum Error {
    /// Controller error
    #[error(transparent)]
    Controller(#[from] ControllerError),

    /// G-Code error
    #[error(transparent)]
    Gcode(#[from] GcodeError),

    /// Connection error
    #[error(transparent)]
    Connection(#[from] ConnectionError),

    /// Firmware error
    #[error(transparent)]
    Firmware(#[from] FirmwareError),

    /// Standard I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create an error from a string message
    pub fn other(msg: impl Into<String>) -> Self {
        Error::Other(msg.into())
    }

    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(
            self,
            Error::Controller(ControllerError::Timeout { .. })
                | Error::Connection(ConnectionError::ConnectionTimeout { .. })
        )
    }

    /// Check if this is a connection error
    pub fn is_connection_error(&self) -> bool {
        matches!(self, Error::Connection(_))
    }

    /// Check if this is a G-Code error
    pub fn is_gcode_error(&self) -> bool {
        matches!(self, Error::Gcode(_))
    }

    /// Check if this is a controller error
    pub fn is_controller_error(&self) -> bool {
        matches!(self, Error::Controller(_))
    }

    /// Check if this is a firmware error
    pub fn is_firmware_error(&self) -> bool {
        matches!(self, Error::Firmware(_))
    }
}

/// Result type using Error
pub type Result<T> = std::result::Result<T, Error>;

// Conversions between error types are automatic via `from` implementations
