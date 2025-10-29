# GCodeKit4 Enhancement Checklist vs UGS

## Critical Priority (Complete First)

### Data Models Enhancement
- [ ] Add `Alarm` enum with GRBL alarm codes (1-13, etc.)
- [ ] Add `TinyGAlarm` enum with TinyG-specific codes
- [ ] Add `Axis` enum (X, Y, Z, A, B, C)
- [ ] Add `AxisGroup` for multi-axis operations
- [ ] Add `WorkCoordinateSystem` struct (G54-G59)
- [ ] Add `Overrides` struct (feed, rapid, spindle percentages)
- [ ] Add `PointSegment` for visualization (line segments)
- [ ] Add `AlarmSeverity` enum (Warning, Error, Fatal)
- [ ] Enhance `GcodeState` with full modal tracking

### Listener System
- [ ] Create `ControllerListener` trait with methods:
  - [ ] on_connection_opened()
  - [ ] on_connection_closed()
  - [ ] on_connection_error()
  - [ ] on_status_changed()
  - [ ] on_alarm()
  - [ ] on_command_sent()
  - [ ] on_command_completed()
  - [ ] on_state_changed()
  - [ ] on_override_changed()
- [ ] Create `CommunicatorListener` trait:
  - [ ] on_message_received()
  - [ ] on_message_sent()
  - [ ] on_communication_error()
  - [ ] on_buffer_state_changed()

### Controller Interface
- [ ] Expand `Controller` trait with methods:
  - [ ] open_connection()
  - [ ] close_connection()
  - [ ] is_connected()
  - [ ] jog() / jog_to()
  - [ ] probe()
  - [ ] home() / home_axis()
  - [ ] reset_coordinates()
  - [ ] set_work_position()
  - [ ] request_status_report()
  - [ ] get_machine_status()
  - [ ] get_controller_state()
  - [ ] set_override() / get_override()
  - [ ] send_command()
  - [ ] pause() / resume()
  - [ ] add/remove_listener()

---

## High Priority (Weeks 2-3)

### G-Code Processors Framework
- [ ] Create `CommandProcessor` base trait
- [ ] Create `CommandProcessorList` for chaining
- [ ] Implement `CommentProcessor`
- [ ] Implement `WhitespaceProcessor`
- [ ] Implement `DecimalProcessor`
- [ ] Implement `CommandLengthProcessor`
- [ ] Implement `ArcExpander`
- [ ] Implement `LineSplitter`
- [ ] Implement `M30Processor`
- [ ] Implement `MeshLeveler`
- [ ] Implement `FeedOverrideProcessor`
- [ ] Implement `RunFromProcessor`
- [ ] Implement `SpindleOnDweller`
- [ ] Implement `TranslateProcessor`
- [ ] Implement `RotateProcessor`
- [ ] Implement `MirrorProcessor`
- [ ] Implement `PatternRemover`
- [ ] Implement `GcodeStats` processor

### Status Management
- [ ] Create `StatusPollTimer` for periodic updates
- [ ] Create `ConnectionWatchTimer` for health checks
- [ ] Implement automatic status polling
- [ ] Add configurable poll rate
- [ ] State synchronization mechanism
- [ ] Alarm detection and notification

---

## High Impact (Weeks 4-8)

### Firmware: GRBL Support
- [ ] Create `GrblController` implementation
- [ ] Implement `GrblCommunicator` with character counting
- [ ] Implement GRBL response parser
- [ ] Map GRBL alarm codes
- [ ] Implement GRBL-specific commands
- [ ] GRBL settings parser
- [ ] Version detection (0.9, 1.1, 1.2)
- [ ] Serial protocol implementation

### Firmware: TinyG Support
- [ ] Create `TinyGController` implementation
- [ ] Implement `TinyGCommunicator` with JSON protocol
- [ ] Implement TinyG response parser
- [ ] Map TinyG alarm codes
- [ ] TinyG settings management
- [ ] JSON command generation

### Firmware: g2core Support
- [ ] Create `G2CoreController` implementation
- [ ] Implement `G2CoreCommunicator` with JSON
- [ ] g2core response parser
- [ ] g2core settings management

### Firmware: Other Platforms
- [ ] Create `SmoothieController`
- [ ] Create `FluidNCController`
- [ ] Create `SmoothieCommunicator`
- [ ] Create `FluidNCCommunicator`

---

## Essential (Concurrent with Firmware)

### Communication Layer
- [ ] Create `BufferedCommunicator` base
- [ ] GRBL character-counting implementation
- [ ] TinyG JSON protocol handler
- [ ] g2core JSON protocol handler
- [ ] Response buffering and queuing
- [ ] Flow control implementation
- [ ] Error recovery mechanisms

### Connection Management
- [ ] Connection pooling for multiple devices
- [ ] Auto-reconnect on disconnect
- [ ] Connection timeout handling
- [ ] Port enumeration
- [ ] Baud rate validation
- [ ] Stop bits, parity configuration

---

## Important (Weeks 9-12)

### File Service
- [ ] Create `FileService` trait
- [ ] File loading and parsing
- [ ] G-Code validation
- [ ] Statistics gathering (line count, time estimate)
- [ ] File streaming to controller
- [ ] Command buffer management
- [ ] Resume from line capability
- [ ] Progress tracking

### Override Management
- [ ] Create `OverrideManager` trait
- [ ] Feed rate override commands
- [ ] Rapid override commands
- [ ] Spindle override commands
- [ ] Real-time override messaging
- [ ] Override state tracking
- [ ] Firmware-specific override formats

### Firmware Settings
- [ ] Create per-firmware `FirmwareSetting` model
- [ ] Settings metadata (min/max/type/units)
- [ ] Settings validation and constraints
- [ ] Settings persistence
- [ ] Settings UI binding support
- [ ] GRBL settings ($0-$132)
- [ ] TinyG settings
- [ ] g2core settings

---

## Nice-to-Have (Future)

### Advanced Features
- [ ] Macro system implementation
- [ ] Custom preprocessor support
- [ ] Simulation/dry-run mode
- [ ] Tool change handling
- [ ] Mesh probing
- [ ] 3D visualization support
- [ ] Real-time DRO display
- [ ] Feed/speed graphs
- [ ] Performance monitoring

### Testing Infrastructure
- [ ] Mock controller for testing
- [ ] Firmware simulator
- [ ] Property-based testing for state machine
- [ ] Integration tests with real protocols
- [ ] Serialization round-trip tests
- [ ] Performance benchmarks

---

## Testing Checklist

For Each New Component:
- [ ] Unit tests with 100% code path coverage
- [ ] Integration tests with mock dependencies
- [ ] Error case handling tests
- [ ] Serialization tests (if applicable)
- [ ] Performance/stress tests
- [ ] Documentation and examples

---

## Documentation Checklist

For Each New Component:
- [ ] Doc comments on all public items
- [ ] Usage examples
- [ ] Error handling guide
- [ ] Protocol/firmware notes
- [ ] Configuration guide
- [ ] Troubleshooting guide

---

## Performance Targets

Based on UGS:
- [ ] Status updates: ≤100ms latency
- [ ] Command buffering: Support 100k+ commands
- [ ] Jog responsiveness: <50ms
- [ ] File streaming: >1000 lines/sec
- [ ] Memory footprint: <500MB for 100k commands
- [ ] Connection establish: <2s

---

## Priority Order for Tasks 7-20

1. **Task 7** - Alarm Model
2. **Task 8** - Axis Enum
3. **Task 9** - ControllerListener
4. **Task 10** - Expanded Controller Trait
5. **Task 11** - Processor Framework
6. **Task 12** - CommentProcessor
7. **Task 13** - StatusPollTimer
8. **Task 14** - GrblController
9. **Task 15** - GrblCommunicator
10. **Task 16** - TinyGController
11. **Task 17** - G2CoreController
12. **Task 18** - BufferedCommunicator
13. **Task 19** - FileService
14. **Task 20** - OverrideManager

This prioritization focuses on foundational components first, then firmware support, then advanced features.
