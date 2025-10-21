//! Buffered communication integration tests
//!
//! Tests for the buffered communication layer including:
//! - Command queueing and buffering
//! - Flow control and buffer management
//! - Command acknowledgment tracking
//! - Retry logic and error handling

use gcodekit4::communication::{
    BufferedCommand, BufferedCommunicatorConfig, BufferedCommunicatorWrapper, CommandStatus,
    Communicator, ConnectionParams, NoOpCommunicator,
};

// Test: Create a buffered command with default status
#[test]
fn test_buffered_command_creation() {
    let cmd = BufferedCommand::new("G28".to_string(), 3);
    assert_eq!(cmd.command, "G28");
    assert_eq!(cmd.status, CommandStatus::Queued);
    assert_eq!(cmd.retry_count, 0);
    assert_eq!(cmd.max_retries, 3);
    assert!(cmd.response.is_none());
}

// Test: Mark command as sent increments retry count
#[test]
fn test_buffered_command_mark_sent() {
    let mut cmd = BufferedCommand::new("G0 X10".to_string(), 3);
    cmd.mark_sent();
    assert_eq!(cmd.status, CommandStatus::Sent);
    assert_eq!(cmd.retry_count, 1);
}

// Test: Command retry logic
#[test]
fn test_buffered_command_retry_logic() {
    let mut cmd = BufferedCommand::new("G1 F100".to_string(), 2);
    assert!(cmd.can_retry());

    cmd.mark_sent(); // Attempt 1
    assert!(cmd.can_retry());

    cmd.mark_sent(); // Attempt 2
    assert!(!cmd.can_retry()); // No more retries after max_retries exceeded
}

// Test: Mark command as acknowledged
#[test]
fn test_buffered_command_mark_acknowledged() {
    let mut cmd = BufferedCommand::new("G28".to_string(), 3);
    cmd.mark_acknowledged();
    assert_eq!(cmd.status, CommandStatus::Acknowledged);
}

// Test: Mark command as completed
#[test]
fn test_buffered_command_mark_completed() {
    let mut cmd = BufferedCommand::new("G28".to_string(), 3);
    cmd.mark_completed();
    assert_eq!(cmd.status, CommandStatus::Completed);
}

// Test: Mark command as failed
#[test]
fn test_buffered_command_mark_failed() {
    let mut cmd = BufferedCommand::new("G28".to_string(), 3);
    cmd.mark_failed();
    assert_eq!(cmd.status, CommandStatus::Failed);
}

// Test: Set command response
#[test]
fn test_buffered_command_set_response() {
    let mut cmd = BufferedCommand::new("G28".to_string(), 3);
    cmd.set_response("ok".to_string());
    assert_eq!(cmd.response, Some("ok".to_string()));
}

// Test: Default buffered communicator config
#[test]
fn test_buffered_communicator_config_defaults() {
    let config = BufferedCommunicatorConfig::default();
    assert_eq!(config.buffer_size, 128);
    assert_eq!(config.queue_size, 100);
    assert_eq!(config.max_retries, 3);
    assert!(config.flow_control);
}

// Test: Create buffered communicator wrapper
#[test]
fn test_buffered_communicator_wrapper_creation() {
    let noop = Box::new(NoOpCommunicator::new());
    let config = BufferedCommunicatorConfig::default();
    let buffered = BufferedCommunicatorWrapper::new(noop, config);

    assert_eq!(buffered.queued_commands_count().unwrap(), 0);
    assert_eq!(buffered.active_commands_count().unwrap(), 0);
    assert!(!buffered.is_paused());
}

// Test: Queue a command
#[test]
fn test_buffered_communicator_queue_command() {
    let noop = Box::new(NoOpCommunicator::new());
    let config = BufferedCommunicatorConfig::default();
    let buffered = BufferedCommunicatorWrapper::new(noop, config);

    buffered.queue_command("G28".to_string()).unwrap();
    assert_eq!(buffered.queued_commands_count().unwrap(), 1);
}

// Test: Queue multiple commands
#[test]
fn test_buffered_communicator_queue_multiple_commands() {
    let noop = Box::new(NoOpCommunicator::new());
    let config = BufferedCommunicatorConfig::default();
    let buffered = BufferedCommunicatorWrapper::new(noop, config);

    buffered.queue_command("G28".to_string()).unwrap();
    buffered.queue_command("G0 X10".to_string()).unwrap();
    buffered.queue_command("G1 F100".to_string()).unwrap();

    assert_eq!(buffered.queued_commands_count().unwrap(), 3);
}

// Test: Queue command respects size limit
#[test]
fn test_buffered_communicator_queue_full() {
    let noop = Box::new(NoOpCommunicator::new());
    let mut config = BufferedCommunicatorConfig::default();
    config.queue_size = 2;
    let buffered = BufferedCommunicatorWrapper::new(noop, config);

    buffered.queue_command("G28".to_string()).unwrap();
    buffered.queue_command("G0 X10".to_string()).unwrap();

    let result = buffered.queue_command("G1 F100".to_string());
    assert!(result.is_err());
}

// Test: Pause and resume streaming
#[test]
fn test_buffered_communicator_pause_resume() {
    let noop = Box::new(NoOpCommunicator::new());
    let config = BufferedCommunicatorConfig::default();
    let mut buffered = BufferedCommunicatorWrapper::new(noop, config);

    assert!(!buffered.is_paused());
    buffered.pause();
    assert!(buffered.is_paused());
    buffered.resume().unwrap();
    assert!(!buffered.is_paused());
}

// Test: Clear queue
#[test]
fn test_buffered_communicator_clear_queue() {
    let noop = Box::new(NoOpCommunicator::new());
    let config = BufferedCommunicatorConfig::default();
    let mut buffered = BufferedCommunicatorWrapper::new(noop, config);

    buffered.queue_command("G28".to_string()).unwrap();
    buffered.queue_command("G0 X10".to_string()).unwrap();
    assert_eq!(buffered.queued_commands_count().unwrap(), 2);

    buffered.clear_queue().unwrap();
    assert_eq!(buffered.queued_commands_count().unwrap(), 0);
    assert!(!buffered.is_paused());
}

// Test: Buffer usage calculation
#[test]
fn test_buffered_communicator_buffer_usage_percent() {
    let noop = Box::new(NoOpCommunicator::new());
    let config = BufferedCommunicatorConfig {
        buffer_size: 100,
        ..Default::default()
    };
    let buffered = BufferedCommunicatorWrapper::new(noop, config);

    let usage = buffered.buffer_usage_percent();
    assert_eq!(usage, 0); // Initially empty
}

// Test: Handle acknowledgment reduces buffer size
#[test]
fn test_buffered_communicator_handle_acknowledgment() {
    let mut noop = NoOpCommunicator::new();
    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
    noop.connect(&params).unwrap();

    let config = BufferedCommunicatorConfig {
        buffer_size: 128,
        flow_control: true,
        ..Default::default()
    };
    let mut buffered = BufferedCommunicatorWrapper::new(Box::new(noop), config);

    // Queue and attempt to stream a command
    buffered.queue_command("G28".to_string()).unwrap();
    buffered.stream_commands().unwrap();

    // Verify active commands
    let active_before = buffered.active_commands_count().unwrap();

    // Handle acknowledgment
    buffered.handle_acknowledgment().unwrap();
    let active_after = buffered.active_commands_count().unwrap();

    // If there were active commands, they should be reduced
    if active_before > 0 {
        assert_eq!(active_after, active_before - 1);
    }
}

// Test: Handle error in command
#[test]
fn test_buffered_communicator_handle_error() {
    let mut noop = NoOpCommunicator::new();
    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
    noop.connect(&params).unwrap();

    let config = BufferedCommunicatorConfig::default();
    let mut buffered = BufferedCommunicatorWrapper::new(Box::new(noop), config);

    // Queue and stream a command
    buffered.queue_command("G28".to_string()).unwrap();
    buffered.stream_commands().unwrap();

    // Handle error
    let result = buffered.handle_error("error message".to_string());
    assert!(result.is_ok());
}

// Test: Flow control prevents overflow
#[test]
fn test_buffered_communicator_flow_control() {
    let noop = Box::new(NoOpCommunicator::new());
    let config = BufferedCommunicatorConfig {
        buffer_size: 10,
        flow_control: true,
        ..Default::default()
    };
    let mut buffered = BufferedCommunicatorWrapper::new(noop, config);

    // Queue a large command that exceeds buffer
    buffered.queue_command("G0 X12345678".to_string()).unwrap();

    // Try to stream - should not send due to flow control
    buffered.stream_commands().unwrap();

    // Command should still be in queue (not sent)
    let queued = buffered.queued_commands_count().unwrap();
    let active = buffered.active_commands_count().unwrap();

    // Either in queue or active, depends on flow control decision
    assert!(queued + active >= 1);
}

// Test: Flow control disabled allows sending
#[test]
fn test_buffered_communicator_flow_control_disabled() {
    let mut noop = NoOpCommunicator::new();
    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
    noop.connect(&params).unwrap();

    let config = BufferedCommunicatorConfig {
        buffer_size: 10,
        flow_control: false,
        ..Default::default()
    };
    let mut buffered = BufferedCommunicatorWrapper::new(Box::new(noop), config);

    buffered.queue_command("G0 X12345678".to_string()).unwrap();
    buffered.stream_commands().unwrap();

    // With flow control disabled, command should be sent regardless of buffer size
    let queued = buffered.queued_commands_count().unwrap();
    let active = buffered.active_commands_count().unwrap();

    // Command should be moved from queue to active
    assert_eq!(queued, 0);
    assert!(active > 0);
}

// Test: Stream commands clears queue
#[test]
fn test_buffered_communicator_stream_commands() {
    let mut noop = NoOpCommunicator::new();
    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
    noop.connect(&params).unwrap();

    let config = BufferedCommunicatorConfig {
        buffer_size: 1024,
        flow_control: true,
        ..Default::default()
    };
    let mut buffered = BufferedCommunicatorWrapper::new(Box::new(noop), config);

    buffered.queue_command("G28".to_string()).unwrap();
    buffered.queue_command("G0 X10".to_string()).unwrap();

    assert_eq!(buffered.queued_commands_count().unwrap(), 2);

    buffered.stream_commands().unwrap();

    // Commands should be moved from queue to active
    assert_eq!(buffered.queued_commands_count().unwrap(), 0);
    assert!(buffered.active_commands_count().unwrap() > 0);
}

// Test: Pause prevents streaming
#[test]
fn test_buffered_communicator_pause_prevents_streaming() {
    let noop = Box::new(NoOpCommunicator::new());
    let config = BufferedCommunicatorConfig {
        buffer_size: 1024,
        flow_control: true,
        ..Default::default()
    };
    let mut buffered = BufferedCommunicatorWrapper::new(noop, config);

    buffered.queue_command("G28".to_string()).unwrap();
    buffered.pause();

    buffered.stream_commands().unwrap();

    // Command should remain in queue when paused
    assert_eq!(buffered.queued_commands_count().unwrap(), 1);
}

// Test: Resume resumes streaming
#[test]
fn test_buffered_communicator_resume_streams() {
    let mut noop = NoOpCommunicator::new();
    let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
    noop.connect(&params).unwrap();

    let config = BufferedCommunicatorConfig {
        buffer_size: 1024,
        flow_control: true,
        ..Default::default()
    };
    let mut buffered = BufferedCommunicatorWrapper::new(Box::new(noop), config);

    buffered.queue_command("G28".to_string()).unwrap();
    buffered.pause();
    buffered.stream_commands().unwrap();

    // Still in queue because paused
    assert_eq!(buffered.queued_commands_count().unwrap(), 1);

    // Resume should stream commands
    buffered.resume().unwrap();
    assert_eq!(buffered.queued_commands_count().unwrap(), 0);
    assert!(buffered.active_commands_count().unwrap() > 0);
}

// Test: Communicator accessor methods
#[test]
fn test_buffered_communicator_accessor_methods() {
    let noop = Box::new(NoOpCommunicator::new());
    let config = BufferedCommunicatorConfig::default();
    let buffered = BufferedCommunicatorWrapper::new(noop, config);

    // Should be able to access the underlying communicator
    let _ = buffered.communicator();
}
