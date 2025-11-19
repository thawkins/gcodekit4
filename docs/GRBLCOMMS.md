# GRBL Communication Protocol Review

**Date**: 2025-11-19
**Version**: 0.33.0-alpha
**Status**: Analysis of Stalling Issues

## Executive Summary

The GRBL communication implementation suffers from a critical race condition between the status polling loop and the command streaming loop. Both loops attempt to read from the single serial port input stream concurrently without synchronization or demultiplexing. This results in "stolen" responses, where the status poller consumes command acknowledgments ("ok"), causing the command streamer to block until timeout (appearing as a stall). Additionally, the streaming logic is implemented as a synchronous "stop-and-wait" protocol rather than a true streaming protocol, and buffer management checks are ignored.

## Identified Issues

### 1. Race Condition in Response Reading (Critical)

**Location**: `crates/gcodekit4-communication/src/firmware/grbl/controller.rs`

The `GrblController` spawns a background `poll_task` (lines 113-135) that periodically sends `?` and reads from the communicator:

```rust
// In poll_task
communicator.send_realtime_byte(b'?')?;
tokio::time::sleep(Duration::from_millis(10)).await;
communicator.read_response()?; // <--- Reads from serial port
```

Simultaneously, the main execution flow calls `send_command` (lines 200-214), which also reads from the communicator:

```rust
// In send_command
self.communicator.send_command(command)?;
let response = self.communicator.read_line()?; // <--- Reads from serial port
```

**Impact**:
- If `poll_task` reads the `ok` response intended for `send_command`, the `send_command` function will block (wait) until the read times out (default 5s) or another message arrives. This manifests as the application "stalling" or becoming unresponsive during streaming.
- Conversely, if `send_command` reads a status report `<Idle|...>`, it ignores it (due to Issue #4), and the UI status fails to update for that cycle.

### 2. Synchronous "Stop-and-Wait" Implementation

**Location**: `crates/gcodekit4-communication/src/firmware/grbl/controller.rs`

The `send_command` method waits for a response immediately after sending each command:

```rust
self.communicator.send_command(command)?;
let response = self.communicator.read_line()?;
```

**Impact**:
- This defeats the purpose of the GRBL character-counting streaming protocol.
- It forces a "ping-pong" communication style, significantly reducing throughput.
- It makes the application highly sensitive to latency and the race condition described above.

### 3. Ignored Buffer Flow Control

**Location**: `crates/gcodekit4-communication/src/firmware/grbl/controller.rs`

The buffer check in `send_command` is effectively a no-op:

```rust
if !self.communicator.is_ready_to_send(command_size) {
    // Empty block!
}
```

**Impact**:
- The controller continues to send commands even if the virtual buffer is full.
- While `send_command` waits for "ok" (throttling it naturally to 1 command in flight), if the "ok" logic were fixed to be asynchronous without fixing this, it would instantly overflow the controller's hardware buffer, causing data loss and errors.

### 4. Missing Acknowledgment Logic

**Location**: `crates/gcodekit4-communication/src/firmware/grbl/controller.rs`

The `send_command` method reads the response but never calls `communicator.acknowledge_chars()`.

**Impact**:
- The `pending_chars` counter in `GrblCommunicator` strictly increases and never decreases.
- `is_ready_to_send` will eventually always return `false`.
- Since the check is ignored (Issue #3), this doesn't stop transmission, but it renders the flow control logic useless.

### 5. Discarded Response Check

**Location**: `crates/gcodekit4-communication/src/firmware/grbl/controller.rs`

Line 211 contains a logic error where the check is discarded:

```rust
let _ = !response.contains("ok");
```

**Impact**:
- The code does not actually verify if the response was "ok". It accepts any response (including a stolen status report or an error) as success.

## Root Cause of "Stall" and "Resume" Fix

The "stall" is the `send_command` function blocking on `read_line()` because its expected `ok` response was consumed by the `poll_task`. It waits until the serial read times out (typically 5 seconds).

Clicking "Resume" sends a `~` (Cycle Start) command. This is a real-time command. If the controller responds (or if the act of sending breaks a deadlock in the serial driver or triggers a status update that unblocks the reader), the loop advances one step.

## Recommendations

### 1. Implement Centralized Response Dispatcher
Create a single background task that owns the `read` end of the serial port. This task should:
- Continuously read lines/data from the port.
- Parse the incoming data.
- If it is a status report (`<...>`): Update the controller state/status.
- If it is a command response (`ok`, `error:...`): Dispatch it to the streaming logic (e.g., via a channel or condition variable) and call `acknowledge_chars`.

### 2. Refactor to True Streaming
- **Producer**: A task that pushes commands to the `GrblCommunicator` as long as `is_ready_to_send` is true.
- **Consumer**: The Centralized Response Dispatcher (above) that receives "ok" and updates the buffer count.
- Remove the synchronous `read_line` from `send_command`.

### 3. Fix Buffer Logic
- Implement proper waiting in `send_command` (or the streaming loop) when `is_ready_to_send` is false.
- Ensure `acknowledge_chars` is called for every `ok` received.

### 4. Separate Command Queue
- Maintain a queue of sent commands to match "ok" responses to specific commands (for error handling).

## Proposed Architecture

```
[UI/App] -> [Command Queue] -> [Sender Task] -> (Serial Port TX)
                                     ^
                                     | (Check Buffer)
                                     |
(Serial Port RX) -> [Reader/Dispatcher Task]
                          |          |
                          v          v
                    [Status State]  [Ack Handler] -> (Update Buffer Count)
```
