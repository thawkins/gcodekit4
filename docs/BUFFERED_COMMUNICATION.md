# Buffered Communication

## Overview

The buffered communication module implements the GRBL streaming protocol with advanced buffer management, flow control, and command acknowledgment tracking. This ensures reliable communication with CNC controllers while preventing buffer overflow and maintaining smooth operation.

## Architecture

### Components

#### BufferedCommand
Represents a single command in the buffer with metadata:
- `command`: The G-code command string to send
- `status`: Current status (Queued, Sent, Acknowledged, Completed, Failed)
- `retry_count`: Number of retry attempts made
- `max_retries`: Maximum allowed retry attempts
- `response`: Response received from the device

#### CommandStatus
Enum representing the lifecycle of a command:
- `Queued`: Command is queued and waiting to be sent
- `Sent`: Command has been sent to the device
- `Acknowledged`: Command has been acknowledged by the device
- `Completed`: Command execution is complete
- `Failed`: Command failed

#### BufferedCommunicatorConfig
Configuration for the buffered communicator:
- `buffer_size`: Maximum size of the controller's buffer in bytes (default: 128)
- `queue_size`: Maximum number of commands in the queue (default: 100)
- `max_retries`: Maximum retries per command (default: 3)
- `flow_control`: Enable/disable flow control (default: true)

#### BufferedCommunicatorWrapper
Main wrapper class that adds buffering capabilities to any communicator:
- Manages command queues and active commands
- Implements flow control
- Handles command streaming and acknowledgments
- Supports pause/resume functionality

## Features

### 1. Command Queue Management
Commands are queued before being sent to the device. The queue respects the configured size limit.

```rust
let buffered = BufferedCommunicatorWrapper::new(communicator, config);
buffered.queue_command("G28".to_string())?;
buffered.queue_command("G0 X10".to_string())?;
```

### 2. Flow Control
Prevents buffer overflow by tracking the amount of data sent to the controller and ensuring new commands don't exceed the controller's buffer capacity.

```rust
let config = BufferedCommunicatorConfig {
    buffer_size: 128,  // Controller's buffer size
    flow_control: true, // Enable flow control
    ..Default::default()
};
```

### 3. Command Acknowledgment Tracking
Tracks active commands and their responses, updating buffer usage as commands are acknowledged.

```rust
buffered.handle_acknowledgment()?;
```

### 4. Retry Logic
Failed commands are automatically retried up to a configured maximum number of attempts.

```rust
let config = BufferedCommunicatorConfig {
    max_retries: 3,  // Retry up to 3 times
    ..Default::default()
};

// Handle error with automatic retry
buffered.handle_error("error message".to_string())?;
```

### 5. Pause/Resume
Streaming can be paused and resumed at any time.

```rust
buffered.pause();
// ... perform some operation ...
buffered.resume()?;
```

### 6. Buffer Usage Monitoring
Monitor the controller's buffer usage as a percentage.

```rust
let usage_percent = buffered.buffer_usage_percent();
```

## Usage Example

```rust
use gcodekit4::communication::{
    BufferedCommunicatorConfig, BufferedCommunicatorWrapper,
    NoOpCommunicator, Communicator, ConnectionParams,
};

// Create a communicator (e.g., Serial)
let mut communicator = NoOpCommunicator::new();
let params = ConnectionParams::serial("/dev/ttyUSB0", 115200);
communicator.connect(&params)?;

// Wrap with buffering
let config = BufferedCommunicatorConfig {
    buffer_size: 128,
    queue_size: 100,
    max_retries: 3,
    flow_control: true,
};
let mut buffered = BufferedCommunicatorWrapper::new(
    Box::new(communicator),
    config
);

// Queue commands
buffered.queue_command("G28".to_string())?;
buffered.queue_command("G0 X10 Y10".to_string())?;

// Stream commands to device
buffered.stream_commands()?;

// Handle responses
buffered.handle_acknowledgment()?;
```

## Integration with GRBL Protocol

The module is designed to work with GRBL's streaming protocol:

1. **Buffer Size Management**: GRBL's typical RX buffer is 128 bytes. Each command + newline is tracked.
2. **Acknowledgment Handling**: When GRBL sends "ok", it indicates buffer space is available.
3. **Error Handling**: When GRBL sends "error", the command is retried or failed based on retry count.
4. **Flow Control**: Prevents sending when buffer space is insufficient.

## Performance Considerations

- **Buffer Overflow Prevention**: Flow control prevents flooding the device with too much data
- **Efficient Retry**: Failed commands are retried intelligently without re-queuing from disk
- **Lock-Free Operations**: Uses Arc<Mutex> for thread-safe queue access
- **Buffer Tracking**: Precise tracking of sent bytes allows accurate flow control

## Testing

Comprehensive tests are provided in `tests/buffered_communication.rs`:
- Command lifecycle (creation, status transitions)
- Queue operations (add, full, clear)
- Buffering mechanics (flow control, buffer usage)
- Streaming and acknowledgment handling
- Error handling and retry logic
- Pause/resume functionality

Run tests with:
```bash
cargo test --test buffered_communication
```

## Future Enhancements

1. **Statistics**: Track command execution times, error rates, and throughput
2. **Adaptive Flow Control**: Dynamically adjust buffer sizes based on device response
3. **Priority Queues**: Support for priority-based command streaming
4. **Command Grouping**: Batch related commands for improved efficiency
5. **Metrics Export**: Prometheus-style metrics for monitoring
