//! Communication module integration tests
//!
//! Tests for serial, TCP, and communicator interface implementations.

pub mod buffered;

use super::common;

use gcodekit4::communication::{
    serial::{MockSerialPort, SerialPort, SerialPortError, SerialPortInfo},
    tcp::{MockTcpPort, TcpConnectionInfo, TcpPort},
    Communicator, CommunicatorListenerHandle,
    ConnectionDriver, ConnectionParams, NoOpCommunicator, SerialCommunicator, TcpCommunicator,
};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// Serial Port Tests
// ============================================================================

#[test]
fn test_serial_port_error_creation() {
    let err = SerialPortError::new("Test error");
    assert_eq!(err.to_string(), "Serial port error: Test error");
}

#[test]
fn test_serial_port_info_creation() {
    let info = SerialPortInfo::new("/dev/ttyUSB0", "USB Serial Port");
    assert_eq!(info.port_name, "/dev/ttyUSB0");
    assert_eq!(info.description, "USB Serial Port");
}

#[test]
fn test_serial_port_info_builder() {
    let info = SerialPortInfo::new("/dev/ttyUSB0", "USB Serial Port")
        .with_manufacturer("Arduino")
        .with_serial_number("12345678")
        .with_usb_ids(0x2341, 0x0043);

    assert_eq!(info.manufacturer, Some("Arduino".to_string()));
    assert_eq!(info.serial_number, Some("12345678".to_string()));
    assert_eq!(info.vid, Some(0x2341));
}

#[test]
fn test_mock_serial_port_write_read() {
    let mut port = MockSerialPort::new("/dev/ttyUSB0");
    port.write(b"G28").unwrap();
    assert_eq!(port.get_written(), b"G28");

    port.clear_written();
    port.set_read_data(b"ok\n".to_vec());

    let mut buf = [0u8; 10];
    let n = port.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"ok\n");
}

// ============================================================================
// TCP Network Tests
// ============================================================================

#[test]
fn test_tcp_connection_info_creation() {
    let info = TcpConnectionInfo::new("192.168.1.100", 8888);
    assert_eq!(info.address(), "192.168.1.100:8888");
}

#[test]
fn test_tcp_connection_info_builder() {
    let info = TcpConnectionInfo::new("grbl.local", 8888)
        .with_timeout(Duration::from_secs(10))
        .with_local_addr("192.168.1.50");

    assert_eq!(info.timeout, Duration::from_secs(10));
    assert_eq!(info.local_addr, Some("192.168.1.50".to_string()));
}

#[test]
fn test_mock_tcp_port_write_read() {
    let mut port = MockTcpPort::new("192.168.1.100:8888");
    port.write(b"G28\n").unwrap();
    assert_eq!(port.get_written(), b"G28\n");

    port.clear_written();
    port.set_read_data(b"[ok]\n".to_vec());

    let mut buf = [0u8; 100];
    let n = port.read(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"[ok]\n");
}

// ============================================================================
// Connection Parameters Tests
// ============================================================================

#[test]
fn test_connection_params_serial() {
    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
    assert_eq!(params.driver, ConnectionDriver::Serial);
    assert_eq!(params.port, "/dev/ttyUSB0");
    assert_eq!(params.baud_rate, 115200);
}

#[test]
fn test_connection_params_tcp() {
    let params = ConnectionParams::tcp("192.168.1.100", 8888);
    assert_eq!(params.driver, ConnectionDriver::Tcp);
    assert_eq!(params.port, "192.168.1.100");
    assert_eq!(params.network_port, 8888);
}

#[test]
fn test_connection_params_validate() {
    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
    assert!(params.validate().is_ok());

    let mut bad_params = params.clone();
    bad_params.port = "".to_string();
    assert!(bad_params.validate().is_err());
}

// ============================================================================
// NoOpCommunicator Tests
// ============================================================================

#[test]
fn test_noop_communicator_basic() {
    let mut comm = NoOpCommunicator::new();
    assert!(!comm.is_connected());

    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
    assert!(comm.connect(&params).is_ok());
    assert!(comm.is_connected());

    assert!(comm.send(b"G28").is_ok());
    assert!(comm.receive().is_ok());

    assert!(comm.disconnect().is_ok());
    assert!(!comm.is_connected());
}

// ============================================================================
// SerialCommunicator Tests
// ============================================================================

#[test]
fn test_serial_communicator_basic() {
    let mut comm = SerialCommunicator::with_mock_port("/dev/ttyUSB0");
    assert!(comm.is_connected());

    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
    assert!(comm.connect(&params).is_ok());

    let result = comm.send(b"G28");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);
}

#[test]
fn test_serial_communicator_connect_wrong_driver() {
    let mut comm = SerialCommunicator::new();
    let params = ConnectionParams::tcp("192.168.1.100", 8888);
    assert!(comm.connect(&params).is_err());
}

#[test]
fn test_serial_communicator_params() {
    let mut comm = SerialCommunicator::new();
    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);

    assert!(comm.set_connection_params(params.clone()).is_ok());
    assert_eq!(comm.connection_params(), Some(&params));
    assert_eq!(comm.driver_type(), ConnectionDriver::Serial);
    assert_eq!(comm.port_name(), "/dev/ttyUSB0");
}

// ============================================================================
// TcpCommunicator Tests
// ============================================================================

#[test]
fn test_tcp_communicator_basic() {
    let mut comm = TcpCommunicator::with_mock_port("192.168.1.100:8888");
    assert!(comm.is_connected());

    let params = ConnectionParams::tcp("192.168.1.100", 8888);
    assert!(comm.connect(&params).is_ok());

    let result = comm.send(b"G0 X10 Y20");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 10);
}

#[test]
fn test_tcp_communicator_connect_wrong_driver() {
    let mut comm = TcpCommunicator::new();
    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
    assert!(comm.connect(&params).is_err());
}

#[test]
fn test_tcp_communicator_params() {
    let mut comm = TcpCommunicator::new();
    let params = ConnectionParams::tcp("192.168.1.100", 8888);

    assert!(comm.set_connection_params(params.clone()).is_ok());
    assert_eq!(comm.connection_params(), Some(&params));
    assert_eq!(comm.driver_type(), ConnectionDriver::Tcp);
    assert_eq!(comm.port_name(), "192.168.1.100");
}

// ============================================================================
// Event Listener Tests
// ============================================================================

#[test]
fn test_serial_communicator_listeners() {
    let mut comm = SerialCommunicator::new();
    let listener: CommunicatorListenerHandle = Arc::new(common::TestListener::new());

    comm.add_listener(listener.clone());
    comm.add_listener(listener.clone());

    comm.remove_listener(&listener);
}

#[test]
fn test_tcp_communicator_listeners() {
    let mut comm = TcpCommunicator::new();
    let listener: CommunicatorListenerHandle = Arc::new(common::TestListener::new());

    comm.add_listener(listener.clone());
    comm.remove_listener(&listener);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_serial_send_receive_sequence() {
    let mut comm = SerialCommunicator::with_mock_port("/dev/ttyUSB0");
    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
    comm.set_connection_params(params).unwrap();

    // Send command
    let send_result = comm.send(b"G28\n");
    assert!(send_result.is_ok());
    assert_eq!(send_result.unwrap(), 4);

    // Receive response
    let receive_result = comm.receive();
    assert!(receive_result.is_ok());
}

#[test]
fn test_tcp_send_receive_sequence() {
    let mut comm = TcpCommunicator::with_mock_port("192.168.1.100:8888");
    let params = ConnectionParams::tcp("192.168.1.100", 8888);
    comm.set_connection_params(params).unwrap();

    // Send command
    let send_result = comm.send(b"G0 X100\n");
    assert!(send_result.is_ok());
    assert_eq!(send_result.unwrap(), 8);

    // Receive response
    let receive_result = comm.receive();
    assert!(receive_result.is_ok());
}
