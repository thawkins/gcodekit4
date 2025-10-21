# GCodeKit4 - Universal G-Code Sender Rust Implementation Plan

**Current Status**: Phase 5 Task 66 COMPLETE - 66/150 Tasks (44%)  
**Version**: 0.7.0-alpha  
**Last Updated**: 2025-10-21

## Completion Summary

| Phase | Tasks | Status | Completion |
|-------|-------|--------|-----------|
| Phase 1: Core Foundation | 1-20 | ✅ COMPLETE | 100% |
| Phase 2: GRBL Controller | 21-35 | ✅ COMPLETE | 100% |
| Phase 3: Additional Firmware | 36-50 | ✅ COMPLETE | 100% |
| Phase 4: G-Code Processing | 51-65 | ✅ COMPLETE | 100% |
| Phase 5: UI Implementation | 66-90 | ⏳ IN PROGRESS | 4% (Task 66) |
| Phase 6: Advanced Features | 91-120 | ⏰ PENDING | 0% |
| Phase 7: Polish & Release | 121-150 | ⏰ PENDING | 0% |

## Project Overview
GCodeKit4 is a Rust-based implementation of Universal G-Code Sender (UGS), a cross-platform G-Code sender compatible with GRBL, TinyG, g2core, Smoothieware, and FluidNC controllers. This plan outlines the complete feature set and implementation tasks based on analysis of the UGS Java codebase.

## Technology Stack
- **Language**: Rust (edition 2021+)
- **UI Framework**: Slint
- **Serial Communication**: serialport-rs
- **Async Runtime**: tokio
- **3D Visualization**: wgpu/three-d
- **Error Handling**: anyhow, thiserror
- **Logging**: tracing

## High-Level Architecture

### 1. Core Module (`src/core/`)
The foundational layer handling controller communication and state management.

### 2. Communication Layer (`src/communication/`)
Serial and network protocol handling for different connection types.

### 3. G-Code Processing (`src/gcode/`)
Parsing, preprocessing, and command generation.

### 4. Firmware Support (`src/firmware/`)
Controller-specific implementations (GRBL, TinyG, g2core, Smoothieware, FluidNC).

### 5. UI Layer (`src/ui/`)
Slint-based graphical interface with multiple panels and windows.

### 6. Utilities (`src/utils/`)
Helper functions, mathematical operations, file I/O.

## Task Breakdown

### Phase 1: Core Foundation (Tasks 1-20)

#### Task 1: Project Initialization
- Initialize Rust project with Cargo
- Set up directory structure following best practices
- Configure Cargo.toml with dependencies
- Set up logging infrastructure (tracing)
- Create initial module structure

#### Task 2: Data Models - Position and Coordinates
- Implement `Position` struct (X, Y, Z, A, B, C coordinates)
- Implement `PartialPosition` for partial axis updates
- Implement `CNCPoint` base structure
- Add unit support (MM, INCH, UNKNOWN)
- Implement coordinate conversion functions

#### Task 3: Data Models - Controller State
- Implement `ControllerState` enum (DISCONNECTED, CONNECTING, IDLE, RUN, HOLD, JOG, ALARM, CHECK, DOOR, HOME, SLEEP)
- Implement `ControllerStatus` struct with machine/work coordinates
- Implement `CommunicatorState` enum
- Add builder pattern for ControllerStatus

#### Task 4: Data Models - G-Code Command
- Implement `GcodeCommand` struct
- Add command state tracking (sent, ok, error, skipped, done)
- Implement command numbering and ID generation
- Add response handling
- Implement command listener trait

#### Task 5: Error Handling
- Create `ControllerError` type with thiserror
- Create `GcodeError` type
- Create `ConnectionError` type
- Create `FirmwareError` type
- Implement From/Into conversions

#### Task 6: Configuration and Settings
- Implement settings file structure (JSON/TOML)
- Add firmware settings trait
- Create settings manager
- Implement default settings for different controllers
- Add settings validation

#### Task 7: Serial Communication - Interface
- Define `Communicator` trait
- Implement connection driver enum (Serial, TCP, WebSocket)
- Create connection parameters struct
- Define communicator events and callbacks

#### Task 8: Serial Communication - Serial Port
- Implement SerialCommunicator using serialport-rs
- Add port enumeration
- Implement connect/disconnect
- Add baud rate configuration
- Implement read/write operations

#### Task 9: Serial Communication - TCP/Network
- Implement TCPCommunicator
- Add hostname/port configuration
- Implement connection management
- Add timeout handling

#### Task 10: Serial Communication - Buffered Communication
- Implement command queue
- Add buffer management
- Implement flow control
- Add command acknowledgment tracking
- Implement retry logic

#### Task 11: G-Code Parser - Core
- Implement GcodeParser struct
- Add G-code command parsing
- Implement modal state tracking
- Parse G/M/T codes
- Handle coordinate parsing

#### Task 12: G-Code Parser - State Machine
- Implement `GcodeState` struct
- Track modal groups (motion, plane, distance, feed rate, units, coordinate system)
- Implement state updates from commands
- Add state validation

#### Task 13: G-Code Preprocessors - Framework
- Create `CommandProcessor` trait
- Implement processor pipeline
- Add processor registration system
- Create processor configuration

#### Task 14: G-Code Preprocessors - Basic
- Implement WhitespaceProcessor
- Implement CommentProcessor
- Implement EmptyLineRemoverProcessor
- Implement CommandLengthProcessor
- Implement DecimalProcessor

#### Task 15: G-Code Preprocessors - Advanced
- Implement ArcExpander
- Implement LineSplitter
- Implement PatternRemover
- Implement M30Processor

#### Task 16: G-Code Stream Management
- Implement stream reader trait
- Create file-based stream reader
- Add string-based stream reader
- Implement stream position tracking
- Add pause/resume capabilities

#### Task 17: Controller Interface - Base Trait
- Define `Controller` trait with all required methods
- Add listener registration
- Define action methods (home, jog, probe, reset, etc.)
- Add streaming control methods
- Define status query methods

#### Task 18: Controller Interface - Abstract Base
- Implement `AbstractController` base struct
- Add common state management
- Implement listener notification
- Add command queue management
- Implement common utilities

#### Task 19: Event System
- Implement event types (ControllerStatusEvent, CommandEvent, AlarmEvent, etc.)
- Create event dispatcher
- Add listener registration/deregistration
- Implement async event handling

#### Task 20: Message Service
- Implement message types (INFO, WARNING, ERROR, VERBOSE)
- Create message dispatcher
- Add console output formatting
- Implement message filtering

### Phase 2: GRBL Controller Implementation (Tasks 21-35)

#### Task 21: GRBL Protocol - Constants and Capabilities
- Define GRBL capabilities constants
- Implement version detection
- Add feature flags
- Define GRBL-specific error codes

#### Task 22: GRBL Protocol - Response Parser
- Implement GRBL response parser
- Parse status reports
- Parse error messages
- Parse alarm messages
- Parse settings responses

#### Task 23: GRBL Protocol - Status Parsing
- Parse machine position (MPos)
- Parse work position (WPos)
- Parse work coordinate offset (WCO)
- Parse buffer state (Buf/RX)
- Parse feed/spindle state

#### Task 24: GRBL Protocol - Utils
- Implement GRBL utilities
- Add response validation
- Create command formatters
- Implement state lookups

#### Task 25: GRBL Command Creator
- Implement GRBL-specific command creation
- Add real-time commands (status query, feed hold, cycle start, reset)
- Create system commands ($H, $X, $C, etc.)
- Implement jog commands
- Add probe commands

#### Task 26: GRBL Communicator
- Implement GRBL-specific communicator
- Handle GRBL protocol specifics
- Implement character counting protocol
- Add streaming protocol support

#### Task 27: GRBL Controller - Initialization
- Implement controller initialization sequence
- Send soft reset
- Query firmware version
- Request settings
- Query parser state

#### Task 28: GRBL Controller - Core Implementation
- Implement `GrblController` struct
- Add connection handling
- Implement command sending
- Handle responses
- Manage controller state

#### Task 29: GRBL Controller - Status Polling
- Implement status poll timer
- Configure polling rate
- Handle status updates
- Update UI state

#### Task 30: GRBL Controller - Streaming
- Implement file streaming
- Add command buffering
- Handle streaming state
- Implement pause/resume
- Add cancel functionality

#### Task 31: GRBL Controller - Jogging
- Implement continuous jogging
- Add incremental jogging
- Support jog cancellation
- Handle jog state transitions

#### Task 32: GRBL Firmware Settings
- Implement settings query
- Add settings parser
- Create settings update
- Implement settings validation
- Add settings backup/restore

#### Task 33: GRBL Override Manager
- Implement feed rate override (0-200%)
- Add rapid override (25%, 50%, 100%)
- Implement spindle override (0-200%)
- Add real-time override commands

#### Task 34: GRBL Alarms and Errors
- Parse alarm codes
- Parse error codes
- Create alarm descriptions
- Implement alarm handling
- Add error recovery

#### Task 35: GRBL Special Features
- Implement homing cycle
- Add probing support
- Implement tool length offset
- Add sleep mode handling
- Implement check mode

### Phase 3: Additional Firmware Support (Tasks 36-50)

#### Task 36: TinyG Protocol Support
- Implement TinyG response parser
- Add JSON response handling
- Create TinyG command creator
- Implement TinyG-specific capabilities

#### Task 37: TinyG Controller
- Implement `TinyGController`
- Add TinyG initialization
- Handle TinyG state machine
- Implement TinyG streaming

#### Task 38: TinyG Utilities
- Implement TinyG utilities
- Add JSON parsing helpers
- Create TinyG formatters

#### Task 39: g2core Protocol Support
- Implement g2core response parser
- Add g2core command creator
- Define g2core capabilities
- Handle g2core JSON protocol

#### Task 40: g2core Controller
- Implement `G2CoreController`
- Add g2core-specific features
- Handle advanced capabilities
- Implement g2core streaming

#### Task 41: Smoothieware Protocol Support
- Implement Smoothieware response parser
- Add Smoothieware command creator
- Define Smoothieware capabilities

#### Task 42: Smoothieware Controller
- Implement `SmoothieController`
- Add Smoothieware-specific features
- Handle Smoothieware state

#### Task 43: FluidNC Protocol Support
- Implement FluidNC response parser
- Add FluidNC command creator
- Define FluidNC capabilities
- Handle FluidNC extensions

#### Task 44: FluidNC Controller
- Implement `FluidNCController`
- Add FluidNC file system support
- Implement FluidNC commands
- Handle FluidNC WiFi features

#### Task 45: Controller Auto-Detection
- Implement firmware detection
- Parse version strings
- Auto-select controller type
- Handle version compatibility

#### Task 46: Firmware Settings Framework
- Create firmware settings trait
- Implement settings storage
- Add settings validation
- Create settings UI binding

#### Task 47: Override Manager Framework
- Create override manager trait
- Implement default override manager
- Add override state tracking
- Handle override commands

#### Task 48: Controller Capabilities System
- Define capability flags
- Implement capability detection
- Add capability queries
- Handle capability-based features

#### Task 49: File Service Interface
- Create file service trait (for controllers with file systems)
- Add file operations (list, upload, download, delete)
- Implement progress tracking

#### Task 50: Connection Watch Timer
- Implement connection monitoring
- Add timeout detection
- Handle reconnection
- Implement heartbeat

### Phase 4: Advanced G-Code Processing (Tasks 51-65)

#### Task 51: Arc Expansion
- Implement arc to line segment conversion
- Add configurable segment length
- Handle G2/G3 commands
- Support multiple planes (XY, XZ, YZ)

#### Task 52: Line Splitting
- Split long lines into segments
- Add maximum line length configuration
- Preserve command semantics

#### Task 53: Mesh Leveling
- Implement surface mesh representation
- Add probe point interpolation
- Apply Z-axis correction
- Support different mesh patterns

#### Task 54: Comment Processing
- Extract comments
- Preserve/remove comments based on settings
- Handle different comment styles

#### Task 55: Feed Override Processor
- Apply feed rate multipliers
- Preserve rapid movements
- Handle feed rate changes

#### Task 56: Pattern Remover
- Remove specific patterns from G-code
- Add regex support
- Configure pattern lists

#### Task 57: Command Transformation - Translation
- Implement XYZ translation
- Add origin offset
- Support coordinate system shifts

#### Task 58: Command Transformation - Rotation
- Implement rotation around axes
- Add angle specification
- Handle coordinate transformation

#### Task 59: Command Transformation - Mirror
- Implement mirroring across planes
- Add axis selection
- Handle arc direction reversal

#### Task 60: Run From Line
- Implement partial file execution
- Calculate modal state at start line
- Handle coordinate system setup

#### Task 61: Spindle On Dweller
- Add dwell after spindle start
- Configure dwell duration
- Handle M3/M4 commands

#### Task 62: Stats Processor
- Calculate total distance
- Estimate execution time
- Track min/max coordinates
- Count command types

#### Task 63: G-Code Optimization
- Remove redundant commands
- Optimize rapid movements
- Merge consecutive moves

#### Task 64: Toolpath Representation
- Create point segment structure
- Build toolpath from G-code
- Track motion type (rapid, linear, arc)
- Store feed rates

#### Task 65: G-Code Validation
- Validate command syntax
- Check coordinate ranges
- Verify modal state consistency
- Report validation errors

### Phase 5: UI Implementation - Slint (Tasks 66-90)

#### Task 66: UI Architecture Setup
- Design Slint component hierarchy
- Create main window layout
- Implement component communication
- Set up UI state management

#### Task 67: Main Window
- Create main window frame
- Add menu bar
- Implement status bar
- Add toolbar
- Configure window properties

#### Task 68: Connection Panel
- Create connection UI
- Add port selection dropdown
- Implement baud rate selection
- Add connect/disconnect buttons
- Show connection status

#### Task 69: Controller State Panel (DRO)
- Display machine position (MPos)
- Display work position (WPos)
- Show controller state
- Add unit display (mm/inch)
- Show feed rate and spindle speed

#### Task 70: Jog Controller Panel
- Create jog button layout
- Implement XYZ jog buttons
- Add step size selection
- Implement feed rate control
- Add keyboard shortcuts

#### Task 71: File Operations Panel
- Add file browser
- Implement file open dialog
- Show file information
- Display estimated run time
- Show file statistics

#### Task 72: G-Code Viewer/Editor
- Implement text editor for G-code
- Add syntax highlighting
- Show line numbers
- Add search/replace
- Implement editing capabilities

#### Task 73: Console/Output Panel
- Display controller responses
- Show command history
- Add message filtering
- Implement auto-scroll
- Add clear button

#### Task 74: Control Buttons
- Add Start/Pause/Stop buttons
- Implement Home button
- Add Reset button
- Create Unlock button
- Add Kill Alarm Lock button

#### Task 75: Overrides Panel
- Create feed rate override slider (0-200%)
- Add rapid override buttons (25%, 50%, 100%)
- Implement spindle override slider (0-200%)
- Show current override values

#### Task 76: Coordinate System Panel
- Display active WCS (G54-G59)
- Add Zero buttons (X, Y, Z, All)
- Implement Go To Zero
- Add Set Work Position
- Show coordinate offsets

#### Task 77: Macros Panel
- Create macro button grid
- Implement macro editor
- Add macro execution
- Support variables in macros
- Add macro import/export

#### Task 78: Settings/Preferences Dialog
- Create settings categories
- Implement controller settings
- Add UI preferences
- Configure file processing
- Add keyboard shortcuts configuration

#### Task 79: Firmware Settings Panel
- Display firmware parameters
- Allow parameter editing
- Add parameter descriptions
- Implement save/restore
- Show parameter validation

#### Task 80: 3D Visualizer - Setup
- Initialize 3D rendering context (wgpu)
- Create camera system
- Implement basic scene
- Add lighting

#### Task 81: 3D Visualizer - Toolpath Rendering
- Render G-code toolpath
- Color code by movement type (rapid/feed)
- Show current position
- Implement arc rendering

#### Task 82: 3D Visualizer - Controls
- Add camera rotation (mouse drag)
- Implement zoom (mouse wheel)
- Add pan (middle mouse)
- Reset view button
- Add view presets (top, front, side, isometric)

#### Task 83: 3D Visualizer - Features
- Show work coordinate system
- Display machine limits
- Add grid display
- Show tool position marker
- Implement bounding box

#### Task 84: Progress Indicators
- Show file send progress bar
- Display time elapsed
- Show time remaining
- Add rows sent/remaining counters
- Implement completion percentage

#### Task 85: Status Notifications
- Create notification system
- Add success notifications
- Show warning notifications
- Display error alerts
- Implement dismissable messages

#### Task 86: Keyboard Shortcuts
- Implement global shortcuts
- Add jog keyboard control
- Create function key actions
- Add space bar pause/resume
- Implement ESC for stop

#### Task 87: Themes and Styling
- Implement light/dark themes
- Create custom color schemes
- Add font size options
- Support high DPI displays

#### Task 88: Multi-language Support (i18n)
- Set up localization framework
- Extract UI strings
- Create translation files
- Implement language selection

#### Task 89: Responsive Layout
- Implement resizable panels
- Add panel show/hide
- Create dockable windows
- Support custom layouts
- Save/restore layout

#### Task 90: Help and Documentation
- Add help menu
- Create keyboard shortcuts reference
- Implement tooltips
- Add about dialog
- Link to documentation

### Phase 6: File Management and Processing (Tasks 91-100)

#### Task 91: File I/O - Reading
- Implement G-code file reader
- Support various encodings (UTF-8, ASCII)
- Handle large files efficiently
- Add file validation

#### Task 92: File I/O - Recent Files
- Track recently opened files
- Add recent files menu
- Implement file history
- Add favorites/bookmarks

#### Task 93: File Processing Pipeline
- Create file processor pipeline
- Apply preprocessors
- Generate processed output
- Cache processed results

#### Task 94: File Statistics
- Calculate file statistics
- Estimate execution time
- Determine bounding box
- Count commands by type
- Calculate total distance

#### Task 95: File Export
- Export processed G-code
- Save modified files
- Add file format options

#### Task 96: Drag and Drop Support
- Implement file drag and drop
- Support multiple file types
- Show drop indicators
- Handle drop events

#### Task 97: File Validation UI
- Show validation results
- Display errors/warnings
- Highlight problematic lines
- Suggest fixes

#### Task 98: File Comparison
- Compare original vs processed
- Show differences
- Highlight changes

#### Task 99: Backup and Recovery
- Auto-save current state
- Implement crash recovery
- Save unsent commands
- Restore session

#### Task 100: File Templates
- Create G-code templates
- Add template variables
- Implement template expansion
- Template library management

### Phase 7: Advanced Features (Tasks 101-125)

#### Task 101: Probing - Basic
- Implement Z-axis probing
- Add probe to work surface
- Handle probe result
- Set work offset from probe

#### Task 102: Probing - Advanced
- Implement multi-point probing
- Add corner finding
- Center finding on circular features
- 3D surface probing

#### Task 103: Probing - Auto-leveling
- Generate probe mesh
- Store height map
- Apply leveling to files
- Visualize mesh in 3D

#### Task 104: Tool Change Management
- Handle M6 tool change
- Pause for manual tool change
- Support automatic tool change
- Track tool library

#### Task 105: Tool Length Offset
- Implement TLO probing
- Store tool offsets
- Apply offsets during execution
- Tool offset management UI

#### Task 106: Work Coordinate Systems
- Support G54-G59 (WCS 1-6)
- Implement G59.1-G59.3 (extended WCS)
- Add WCS selection UI
- Store WCS offsets

#### Task 107: Soft Limits
- Configure machine limits
- Validate movements against limits
- Show limit violations
- Add soft limit override

#### Task 108: Simulation Mode
- Implement dry-run mode
- Execute without sending to controller
- Track simulated position
- Validate file before sending

#### Task 109: Step-Through Execution
- Implement single-step mode
- Add step forward/backward
- Show next command preview
- Pause between commands

#### Task 110: Bookmarks/Breakpoints
- Add line bookmarks
- Implement breakpoints
- Pause at breakpoints
- Manage bookmark list

#### Task 111: Program Restart
- Restart from current line
- Restore modal state
- Handle coordinate systems
- Resume after error

#### Task 112: Performance Monitoring
- Track command throughput
- Monitor buffer usage
- Display performance metrics
- Log performance data

#### Task 113: Command History
- Store sent commands
- Display command history
- Resend from history
- Export history

#### Task 114: Custom Scripts/Macros
- Support custom G-code macros
- Add variable substitution
- Implement loops/conditionals
- Macro programming language

#### Task 115: Pendant Support
- Support USB/Bluetooth pendants
- Map pendant buttons
- Handle pendant jogging
- Configure pendant settings

#### Task 116: Custom Buttons/Actions
- Create custom button actions
- Add user-defined commands
- Implement action sequences
- Save custom actions

#### Task 117: Auto-connect
- Implement automatic connection on startup
- Remember last connection
- Auto-detect controller type
- Handle connection failures

#### Task 118: Network/Remote Access
- Implement WebSocket server
- Add REST API
- Support remote monitoring
- Remote control features

#### Task 119: Data Logging
- Log all communications
- Record position history
- Save error logs
- Export logs

#### Task 120: Alarms and Notifications
- Visual alarm indicators
- Sound notifications
- System tray notifications
- Email/webhook notifications

#### Task 121: Safety Features
- Implement emergency stop
- Add motion interlock
- Soft limits checking
- Feed hold on errors

#### Task 122: Plugin System Architecture
- Design plugin interface
- Implement plugin loading
- Add plugin configuration
- Create plugin API

#### Task 123: Export to Different Formats
- Export to different G-code dialects
- Support post-processor selection
- Add format conversion

#### Task 124: Calibration Wizards
- Step calibration wizard
- Backlash measurement
- Axis squareness check
- Create calibration reports

#### Task 125: Diagnostic Tools
- Communication diagnostics
- Buffer state monitoring
- Performance profiler
- Debug mode with detailed logging

### Phase 8: Testing and Documentation (Tasks 126-150)

#### Task 126: Unit Tests - Data Models
- Test Position and coordinates
- Test ControllerStatus
- Test GcodeCommand
- Test error types

#### Task 127: Unit Tests - G-Code Parser
- Test command parsing
- Test state machine
- Test modal groups
- Test error handling

#### Task 128: Unit Tests - Preprocessors
- Test each processor independently
- Test processor chains
- Test edge cases
- Test error conditions

#### Task 129: Unit Tests - Communication
- Test serial communication (with mocks)
- Test TCP communication
- Test buffer management
- Test timeout handling

#### Task 130: Unit Tests - Controllers
- Test GRBL controller
- Test TinyG controller
- Test g2core controller
- Test state transitions

#### Task 131: Integration Tests - GRBL
- Test GRBL connection flow
- Test streaming operations
- Test jogging
- Test homing

#### Task 132: Integration Tests - File Processing
- Test file loading
- Test preprocessing pipeline
- Test file statistics
- Test file export

#### Task 133: Integration Tests - UI Components
- Test UI initialization
- Test panel interactions
- Test state updates
- Test event handling

#### Task 134: Mock Controller for Testing
- Create virtual controller
- Simulate GRBL responses
- Support different scenarios
- Add configurable behaviors

#### Task 135: Performance Tests
- Benchmark file parsing
- Test streaming performance
- Measure UI responsiveness
- Profile memory usage

#### Task 136: Stress Tests
- Large file handling
- Continuous operation
- Memory leak detection
- Error recovery

#### Task 137: End-to-End Tests
- Complete workflow tests
- Multi-controller tests
- Error recovery scenarios
- State persistence tests

#### Task 138: Documentation - Architecture
- Document system architecture
- Create component diagrams
- Explain design decisions
- Add sequence diagrams

#### Task 139: Documentation - API Reference
- Generate API documentation
- Document public interfaces
- Add usage examples
- Document error codes

#### Task 140: Documentation - User Guide
- Create getting started guide
- Document UI components
- Add feature tutorials
- Include troubleshooting

#### Task 141: Documentation - Developer Guide
- Set up development environment
- Explain code structure
- Document build process
- Add contribution guidelines

#### Task 142: Documentation - Controller Setup
- GRBL setup guide
- TinyG setup guide
- g2core setup guide
- FluidNC setup guide

#### Task 143: Documentation - G-Code Reference
- Document supported G-codes
- Explain G-code processors
- Add examples
- Include best practices

#### Task 144: Code Examples
- Basic connection example
- Streaming example
- Jogging example
- Custom processor example

#### Task 145: CI/CD Pipeline
- Set up GitHub Actions
- Implement automated testing
- Add code coverage
- Configure linting

#### Task 146: Release Process
- Version management
- Changelog automation
- Release notes
- Binary distribution

#### Task 147: Cross-Platform Testing
- Test on Linux
- Test on Windows
- Test on macOS
- Document platform differences

#### Task 148: Accessibility Testing
- Keyboard navigation
- Screen reader support
- High contrast mode
- Font scaling

#### Task 149: Security Audit
- Review serial port access
- Check file handling
- Validate user input
- Review network features

#### Task 150: Performance Optimization
- Optimize hot paths
- Reduce allocations
- Improve rendering performance
- Profile and optimize

## Dependencies (Cargo.toml)

```toml
[package]
name = "gcodekit4"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Serial communication
serialport = "4"

# UI framework
slint = { version = "1", features = ["backend-winit"] }

# 3D rendering
wgpu = "0.18"
three-d = "0.16"

# Error handling
anyhow = "1"
thiserror = "1"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Utilities
regex = "1"
lazy_static = "1"
chrono = "0.4"
uuid = { version = "1", features = ["v4"] }

# Network
tokio-tungstenite = "0.21" # WebSocket

[dev-dependencies]
mockall = "0.12"
proptest = "1"
criterion = "0.5"
```

## File Structure

```
gcodekit4/
├── Cargo.toml
├── AGENTS.md
├── README.md
├── CHANGELOG.md
├── SPEC.md
├── STATS.md
├── docs/
│   ├── PLAN.md (this file)
│   ├── ARCHITECTURE.md
│   ├── USER_GUIDE.md
│   ├── DEVELOPER_GUIDE.md
│   └── API_REFERENCE.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── core/
│   │   ├── mod.rs
│   │   ├── controller.rs
│   │   ├── state.rs
│   │   └── events.rs
│   ├── communication/
│   │   ├── mod.rs
│   │   ├── communicator.rs
│   │   ├── serial.rs
│   │   ├── tcp.rs
│   │   └── buffer.rs
│   ├── gcode/
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   ├── command.rs
│   │   ├── state.rs
│   │   └── processors/
│   │       ├── mod.rs
│   │       ├── arc_expander.rs
│   │       ├── comment.rs
│   │       └── ...
│   ├── firmware/
│   │   ├── mod.rs
│   │   ├── grbl/
│   │   │   ├── mod.rs
│   │   │   ├── controller.rs
│   │   │   ├── commands.rs
│   │   │   └── settings.rs
│   │   ├── tinyg/
│   │   ├── g2core/
│   │   ├── smoothie/
│   │   └── fluidnc/
│   ├── models/
│   │   ├── mod.rs
│   │   ├── position.rs
│   │   ├── status.rs
│   │   └── capabilities.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── main_window.slint
│   │   ├── components/
│   │   └── theme/
│   ├── visualizer/
│   │   ├── mod.rs
│   │   ├── renderer.rs
│   │   └── camera.rs
│   └── utils/
│       ├── mod.rs
│       ├── math.rs
│       └── file.rs
└── tests/
    ├── core/
    ├── communication/
    ├── gcode/
    ├── firmware/
    └── integration/
```

## Priority Order

### Immediate Priority (Milestone 1 - MVP)
- Tasks 1-20: Core Foundation
- Tasks 21-35: GRBL Controller
- Tasks 66-74: Basic UI
- Tasks 91-94: File Management

### High Priority (Milestone 2 - Functional Release)
- Tasks 36-50: Additional Firmware Support
- Tasks 51-65: Advanced G-Code Processing
- Tasks 75-83: Enhanced UI Features
- Tasks 101-107: Basic Advanced Features

### Medium Priority (Milestone 3 - Feature Complete)
- Tasks 84-90: UI Polish
- Tasks 95-100: Advanced File Features
- Tasks 108-121: Advanced Features
- Tasks 126-137: Testing

### Lower Priority (Milestone 4 - Production Ready)
- Tasks 122-125: Extensibility
- Tasks 138-150: Documentation and Polish

## Success Criteria

1. **Functionality**: Successfully connect to and control GRBL controllers
2. **Performance**: Handle files up to 100k lines smoothly
3. **Stability**: Run continuously for hours without crashes
4. **UI/UX**: Intuitive interface comparable to UGS
5. **Cross-platform**: Build and run on Linux, Windows, macOS
6. **Documentation**: Complete user and developer documentation
7. **Tests**: >80% code coverage with comprehensive tests

## Notes

- Each task should be implemented incrementally with tests
- Follow the AGENTS.md guidelines strictly
- Update CHANGELOG.md before each push
- Maintain SPEC.md with current specifications
- Keep STATS.md updated with metrics
- All documentation in docs/ folder except root-level .md files
- Use semantic versioning for releases
