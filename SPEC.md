# GCodeKit4 - Specification Document

**Version**: 0.25.7-alpha  
**Last Updated**: 2025-11-15  
**Status**: SVG to G-Code Vector Engraver Complete - Editor Phase 1 Complete

### Latest Release (v0.25.7-alpha) - SVG to G-Code Vector Engraver
- ✅ **Custom G-Code Text Editor - Phase 1 (COMPLETE)**
  - Full keyboard input support with proper event handling
  - Text insertion with automatic cursor advancement
  - Text deletion (backspace/delete) with cursor adjustment
  - Arrow key navigation (left, right, up, down with boundary checking)
  - Home/End keys for line start/end navigation
  - PageUp/PageDown for viewport scrolling
  - Undo/Redo support (Ctrl+Z/Ctrl+Y, Cmd on Mac)
  - Tab key inserts 4 spaces for proper indentation
  - Enter/Return for newline insertion
  - Virtual scrolling with line numbers (100+ line performance)
  - Real-time cursor position updates
  - Text saved to buffer on every keystroke
  - Complete callback chain: CustomTextEdit → GcodeEditorPanel → MainWindow → Rust backend
- ✅ **Previously Completed Features** (v0.25.5 and earlier)
  - Enhanced Error Reporting
  - G-Code Streaming (GRBL Character-Counting Protocol)
  - CNC Tools Manager with GTC import
  - Materials Database Manager
  - CAM Tools (Tabbed Box Maker, Jigsaw Puzzle Maker)
  - Firmware Detection
  - Console Improvements
  - BrokenPipe Handling

## 1. Executive Summary

GCodeKit4 is a cross-platform, Rust-based G-Code sender and CNC machine controller application. It provides a modern alternative to Universal G-Code Sender (UGS), supporting multiple CNC controller firmware types including GRBL, TinyG, g2core, Smoothieware, and FluidNC. The application enables users to load G-Code files, visualize toolpaths, control CNC machines in real-time, and manage machine operations through an intuitive graphical interface.

## 2. Product Overview

### 2.1 Purpose
GCodeKit4 enables makers, engineers, and manufacturers to:
- Connect to and control CNC machines via serial/network protocols
- Load and process G-Code files
- Visualize machine toolpaths in 2D/3D
- Execute G-Code programs with real-time monitoring
- Monitor machine position and state (DRO - Digital Readout)
- Adjust machine parameters on-the-fly (overrides)
- Perform machine calibration and probing operations
- Implement custom workflows through macros and scripts

### 2.2 Key Features
- **Multi-controller support**: GRBL, TinyG, g2core, Smoothieware, FluidNC
- **Real-time 3D visualization**: Toolpath preview and execution tracking
- **Advanced G-Code processing**: Arc expansion, mesh leveling, transformations
- **Flexible connectivity**: Serial port, TCP/IP, WebSocket
- **Comprehensive machine control**: Jog, home, probe, override, feed hold
- **File management**: Open, process, validate, export G-Code files
- **Customization**: Macros, custom buttons, keyboard shortcuts, themes
- **Cross-platform**: Linux, Windows, macOS support
- **Extensible architecture**: Plugin system for third-party extensions

### 2.3 Target Users
- CNC machine operators and hobbyists
- Makers and DIY enthusiasts
- Professional machinists and manufacturers
- CAM software users requiring post-processing
- Educational institutions teaching CNC programming

## 3. Technical Architecture

### 3.1 System Components

#### 3.1.1 Core Module (`core/`)
**Purpose**: Foundation layer managing controller communication and state
- **Controller Interface**: Abstract controller trait with standard operations
- **State Management**: Tracks controller state, positions, and operational modes
- **Event System**: Event dispatcher for UI updates and listener notifications
- **Message Service**: Centralized message handling for logging and UI display

**Key Structs**:
- `ControllerState`: Enum representing machine state (IDLE, RUN, HOLD, ALARM, etc.)
- `ControllerStatus`: Complete snapshot of controller state including positions
- `GcodeCommand`: Individual G-Code command with tracking metadata
- `Event`: Domain events for controller operations

**Responsibilities**:
- Coordinate all controller operations
- Maintain consistent state across the application
- Dispatch events to UI and other components
- Manage listener registration and notifications

#### 3.1.2 Communication Module (`communication/`)
**Purpose**: Handle serial, TCP, and WebSocket communication with controllers

**Components**:
- **Communicator Trait**: Abstract interface for all communication types
- **Serial Communicator**: serialport-rs based serial communication with GRBL Character-Counting Protocol
  - Integrated G-code streaming with buffer management
  - Single polling thread handles: receive, send, and status queries
  - Mutex held only during actual I/O operations (~1-2ms per cycle)
  - Real-time command support: `?` status query as single byte
  - Buffer tracking: 127-byte limit with "ok" acknowledgment counting
- **TCP Communicator**: Tokio-based TCP networking
- **WebSocket Communicator**: Tokio-Tungstenite WebSocket support
- **Buffered Communicator**: Character counting and command buffering

**Features**:
- Automatic port enumeration
- Configurable baud rates and timeouts
- Command queuing and flow control
- Reconnection handling
- Error recovery and retry logic

#### 3.1.3 G-Code Processing Module (`gcode/`)
**Purpose**: Parse, validate, and process G-Code files

**Sub-components**:
- **Parser**: Command-by-command G-Code parsing
- **State Machine**: Modal state tracking (motion, plane, distance, units, WCS)
- **Preprocessors**: Pluggable command processors
  - Comment removal
  - Whitespace cleanup
  - Arc expansion
  - Line splitting
  - Mesh leveling
  - Transformations (translation, rotation, mirror)
  - Feed rate overrides
  - Statistics collection

**Data Structures**:
- `GcodeState`: Holds modal state and parser parameters
- `GcodeCommand`: Individual command with position/feed info
- `CommandProcessor`: Trait for command processors
- `PointSegment`: Toolpath segment representation

#### 3.1.4 Firmware Support Module (`firmware/`)
**Purpose**: Controller-specific implementations and protocols

**Supported Firmwares**:
1. **GRBL** (v0.9, v1.0, v1.1, v1.2)
   - Character counting protocol
   - Real-time commands
   - Status reports
   - Alarms and errors
   - Settings system

2. **TinyG**
   - JSON protocol
   - Status reporting
   - Queue management

3. **g2core**
   - Advanced JSON protocol
   - Extended capabilities
   - File system support

4. **Smoothieware**
   - Protocol support
   - State machine handling

5. **FluidNC**
   - Extended protocol
   - WiFi support
   - File system operations

**Per-Controller Components**:
- `Controller` implementation
- `CommandCreator` for firmware-specific commands
- `ResponseParser` for protocol messages
- `FirmwareSettings` manager
- `OverrideManager` for real-time overrides
- `Capabilities` definition

#### 3.1.5 Data Models Module (`models/`)
**Purpose**: Core data structures and types

**Key Entities**:
- **Position**: 6-axis position (X, Y, Z, A, B, C) with units
- **PartialPosition**: Subset of axes for targeted updates
- **UnitUtils**: Unit conversion (MM ↔ INCH)
- **UnitDefaults**: The default dimensional unit is mm
- **ControllerStatus**: Complete machine state snapshot
- **Capabilities**: Feature flags for controller capabilities
- **WorkCoordinateSystem**: G54-G59 work coordinate storage
- **Alarm**: Machine alarm codes and descriptions
- **Error**: Error codes and messages

#### 3.1.6 UI Module (`ui/`)
**Purpose**: Slint-based graphical user interface

**Main Components**:
- **Main Window**: Application frame, menu bar, toolbar, status bar
- **Connection Panel**: Port selection, baud rate, connect/disconnect
- **DRO Panel**: Machine/work position display, state indicators
- **Jog Panel**: Jogging controls with incremental steps
- **File Operations**: File browser, open/save dialogs
- **G-Code Editor**: Syntax highlighting, line numbers, preview
- **Console**: Message output, command history, filtering
- **Control Panel**: Start/Pause/Stop, Home, Reset, Unlock
- **Overrides Panel**: Feed/Rapid/Spindle override controls
- **Coordinate Systems**: WCS selection and offset management
- **Macros Panel**: Macro buttons and editor
- **Firmware Settings**: Parameter display and editing
- **3D Visualizer**: Real-time toolpath visualization
- **Settings Dialog**: Preferences, keyboard shortcuts, profiles

**Interaction Model**:
- Event-driven UI updates from controller state changes
- Real-time responsiveness with async operations
- Keyboard shortcuts for common operations
- Drag-and-drop file support
- Context menus and tooltips

#### 3.1.7 Visualizer Module (`visualizer/`)
**Purpose**: 3D rendering of toolpath and machine state

**Features**:
- Real-time G-Code toolpath visualization
- Motion type differentiation (rapid vs. feed)
- Arc rendering
- Current position tracking
- Coordinate system display
- Machine limit visualization
- Grid overlay
- Multiple view presets (top, front, side, isometric)
- Interactive camera controls (rotate, zoom, pan)
- Bounding box calculation

**Technology**: wgpu backend with three-d rendering library

#### 3.1.8 Utilities Module (`utils/`)
**Purpose**: Common helper functions and utilities

**Features**:
- Mathematical operations (transformations, distance calculations)
- File I/O and parsing
- String processing and formatting
- Time/duration calculations
- Configuration management
- Logging setup

### 3.2 Data Flow Architecture

```
┌─────────────┐
│  G-Code     │
│  File       │
└──────┬──────┘
       │
       ▼
┌──────────────────┐
│ File Reader      │
│ (UTF-8/ASCII)    │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ G-Code Parser    │
└──────┬───────────┘
       │
       ▼
┌──────────────────────────┐
│ Preprocessor Pipeline    │
│ ┌────────────────────┐   │
│ │ Comment Processor  │   │
│ │ Arc Expander       │   │
│ │ Transformations    │   │
│ │ Stats Calculator   │   │
│ └────────────────────┘   │
└──────┬───────────────────┘
       │
       ▼
┌──────────────────────────┐
│ Processed G-Code Stream  │
└──────┬───────────────────┘
       │
       ├─────────────┬────────────────┬──────────────┐
       │             │                │              │
       ▼             ▼                ▼              ▼
   ┌────┐      ┌────────┐      ┌──────────┐   ┌────────┐
   │UI  │      │Streamer│      │Visualizer│   │Logger  │
   └────┘      └────────┘      └──────────┘   └────────┘
       │             │                │              │
       │             ▼                │              │
       │       ┌──────────────┐       │              │
       │       │Communicator  │       │              │
       │       └──────┬───────┘       │              │
       │              │               │              │
       ▼              ▼               ▼              ▼
┌────────────────────────────────────────────────────┐
│         Controller (GRBL/TinyG/g2core/etc)         │
└────────────────────────────────────────────────────┘
```

### 3.3 State Machine

#### Controller State Transitions
```
DISCONNECTED
    │
    ├─ open_connection() ──→ CONNECTING
    │
    └─ [Cannot perform operations]

CONNECTING
    │
    ├─ initialization_complete() ──→ IDLE
    │
    ├─ connection_failed() ──→ DISCONNECTED
    │
    └─ [Cannot perform operations]

IDLE
    │
    ├─ begin_streaming() ──→ RUN
    ├─ jog() ──→ JOG
    ├─ home() ──→ HOME
    ├─ [Alarm received] ──→ ALARM
    │
    └─ [Ready for commands]

RUN
    │
    ├─ pause() ──→ HOLD
    ├─ cancel() ──→ IDLE
    ├─ [Alarm received] ──→ ALARM
    ├─ [Program complete] ──→ IDLE
    │
    └─ [Streaming commands]

HOLD
    │
    ├─ resume() ──→ RUN
    ├─ cancel() ──→ IDLE
    ├─ [Alarm received] ──→ ALARM
    │
    └─ [Awaiting user action]

JOG
    │
    ├─ cancel_jog() ──→ IDLE
    ├─ [Jog complete] ──→ IDLE
    ├─ [Alarm received] ──→ ALARM
    │
    └─ [Manual machine movement]

HOME
    │
    ├─ [Homing complete] ──→ IDLE
    ├─ [Alarm received] ──→ ALARM
    │
    └─ [Homing in progress]

ALARM
    │
    ├─ reset() or kill_alarm_lock() ──→ IDLE
    ├─ [No automatic recovery]
    │
    └─ [Machine locked, requires action]

CHECK
    │
    ├─ exit_check_mode() ──→ previous_state
    ├─ [Dry-run mode]
    │
    └─ [Commands processed, not executed]

DOOR
    │
    ├─ [Door closed] ──→ RUN/HOLD
    ├─ [Door open] ──→ HOLD
    │
    └─ [Safety interlock triggered]

SLEEP
    │
    └─ [Low-power idle state]
```

## 4. Core Functionality Specifications

### 4.1 Connection Management

**Supported Connection Types**:
1. **Serial/USB**
   - Baud rates: 9600, 19200, 38400, 57600, 115200 (configurable)
   - Auto-detection of available ports
   - Connection timeout: 5 seconds (configurable)
   - Auto-reconnection on failure (optional)

2. **TCP/IP**
   - Hostname and port configuration
   - Connection timeout: 10 seconds (configurable)
   - Keepalive messages every 30 seconds

3. **WebSocket**
   - Full WebSocket protocol support
   - Automatic reconnection with exponential backoff
   - Message queuing during disconnection

**Connection Workflow**:
1. User selects connection parameters
2. Application establishes connection
3. Soft reset sent to controller
4. Firmware version queried
5. Settings and capabilities retrieved
6. Parser state queried
7. Status polling started
8. UI updates to IDLE state
9. Ready for operations

### 4.2 G-Code File Operations

#### 4.2.1 File Loading
**Supported Formats**:
- `.gcode` - Standard G-Code
- `.gc`, `.ngc` - Common variations
- `.tap` - CNC tap files
- `.iso` - ISO 6983 standard G-Code
- Plain text files with G-Code content

**Validation**:
- Character encoding verification (UTF-8, ASCII)
- Line syntax validation
- Maximum line length check
- Comment format validation
- Modal group consistency checking
- Coordinate range checking against machine limits

**File Statistics**:
- Total line count
- Command count by type (G-codes, M-codes, T-codes)
- Estimated execution time
- Bounding box (min/max coordinates)
- Total travel distance

#### 4.2.2 File Processing Pipeline
1. **Load**: Read file from disk
2. **Parse**: Break into individual commands
3. **Validate**: Check for errors and warnings
4. **Process**: Apply preprocessors
   - Remove comments
   - Normalize whitespace
   - Expand arcs
   - Apply transformations
   - Collect statistics
5. **Cache**: Store processed output
6. **Display**: Show in UI

#### 4.2.3 Preprocessing Operations

**Available Processors** (selectively applicable):
- **Comment Removal**: Strip all comments (/* */ and ())
- **Whitespace Cleanup**: Remove unnecessary spaces/tabs
- **Arc Expansion**: Convert G2/G3 to G1 line segments
  - Segment length: configurable (0.1-2.0 mm)
  - Supports XY, XZ, YZ planes
- **Line Splitting**: Break long commands into segments
  - Max characters per line: configurable
- **Feed Rate Override**: Multiply feed rates by factor
  - Factor: 0.1 to 2.0 (10% to 200%)
  - Preserves rapid movements (G0)
- **Coordinate Translation**: Shift origin
  - X, Y, Z offsets in current units
- **Coordinate Rotation**: Rotate around axis
  - Angle in degrees
  - Support for Z, X, Y axes
- **Coordinate Mirror**: Mirror across plane
  - Planes: XY, XZ, YZ
  - Reverses arc directions (G2 ↔ G3)
- **Mesh Leveling**: Apply height map
  - Generated from probe points
  - Bilinear interpolation
  - Z-axis correction

### 4.3 Streaming and Execution

#### 4.3.1 Streaming Protocol (GRBL)
**Character Counting Protocol**:
- Each command counted by byte size
- Send buffer: typically 128 bytes
- Controller reports available buffer space
- Application maintains command queue
- Automatic rate limiting based on buffer availability

**Implementation**:
```
Send command (N bytes)
Rx buffer credit: available - N
│
├─ When available ≥ next_command_size: send next
│
└─ On "ok" response: credit += command_size
   On "error" response: handle error, optionally retry
```

#### 4.3.2 Execution Modes
1. **Normal**: Standard streaming to controller
2. **Single-Step**: Pause after each command
3. **Simulation/Check Mode**: Execute without moving
4. **Step-Through**: Manual step forward/backward
5. **Dry Run**: Validate file without machine connection

#### 4.3.3 Streaming Controls
- **Start**: Begin execution from current position
- **Pause**: Hold machine, maintain position
- **Resume**: Continue from pause point
- **Cancel**: Stop immediately, clear queue
- **Skip Lines**: Resume from specified line number
  - Recalculates modal state
  - Handles coordinate systems

#### 4.3.4 Streaming Monitoring
**Status Updates** (per 250ms typically):
- Current line number / total lines
- Bytes sent / total bytes
- Time elapsed / estimated time remaining
- Current machine position
- Buffer status
- Feed rate actual vs. commanded
- Spindle speed actual vs. commanded

### 4.4 Machine Control Operations

#### 4.4.1 Real-Time Commands
Available as real-time commands (executed immediately):
- **0x18** - Soft Reset
- **0x85** - Feed Hold (pause)
- **0x9E** - Cycle Start (resume)
- **0x81-0x84** - Feed Rate Override (90%, 95%, 105%, 110%)
- **0x86-0x89** - Rapid Traverse Override (25%, 50%, 100%, 125%)
- **0x8A-0x8D** - Spindle Override (90%, 95%, 105%, 110%)
- **?** - Status Report Query

#### 4.4.2 System Commands (GRBL)
- **$?** - Query parser state
- **$#** - Query parameters (G54-G59, offsets)
- **$H** - Run homing cycle
- **$X** - Kill alarm lock
- **$C** - Check mode toggle
- **$Homing Cycle** - Configurable homing
- **$Setting=Value** - Modify settings

#### 4.4.3 Machine Operations

**Homing**:
- Automatic or manual per axis
- Configurable homing sequence
- Sets machine position to zero (or offset)
- Sets work position to offset

**Jogging**:
- Continuous jogging (pressed button held)
- Incremental jogging (fixed distance per button click)
- Selectable increments: 0.1mm, 1mm, 10mm, 100mm (configurable)
- Selectable feed rate: user-defined
- Keyboard jog support: WASD/arrow keys for XYZ
- Jog cancel on any input or timeout

**Probing**:
- Single point: probe along axis to find surface
- Multi-point: probe grid for auto-leveling
- Result handling: set work offset or store in mesh
- Edge finding: four-directional probing around edges
- Tool length offset probing on known surface

**Tool Change**:
- Manual pause on M6 code
- Automatic tool change support (if hardware present)
- Tool length offset application
- Work offset restoration after change

**Coordinate System Management**:
- G54-G59: Six work coordinate systems
- G59.1-G59.3: Extended WCS (if supported)
- Active WCS display and switching
- Offset storage and recall
- Zero axis buttons (X, Y, Z, All)
- Preset coordinate values

#### 4.4.4 Overrides

**Feed Rate Override**:
- Range: 0-200% (configurable limits)
- 10% increments via buttons
- Continuous via slider
- Applied to non-rapid feed movements
- Real-time without stopping

**Rapid Traverse Override**:
- Preset buttons: 25%, 50%, 100% (125% optional)
- Affects G0 movements
- Real-time application
- 100% is no override

**Spindle Override**:
- Range: 0-200% (configurable limits)
- 10% increments via buttons
- Continuous via slider
- Real-time PWM/relay adjustment
- May not work with all controller firmware

### 4.5 Firmware-Specific Features

#### 4.5.1 GRBL (v0.9, v1.0, v1.1, v1.2)
**Capabilities**:
- Character counting protocol
- Real-time commands
- Status reports (< version)
- Alarms (11 types)
- Error codes (30+ types)
- Settings system ($0-$32)
- Work coordinate systems (G54-G59)
- Homing cycles
- Soft limits
- Probing (G38.2, G38.5)
- Parking
- Override commands
- Hold/Resume
- Safety door interrupt
- M-code support (limited)

**Specific Implementation**:
- Version detection via $I command
- Capability flags based on version
- Settings parsing and validation
- Status report parsing with optional fields
- Alarm/error code lookups

#### 4.5.2 TinyG
**Capabilities**:
- JSON protocol
- Queue-based processing
- Extended G-code support
- 6-axis motion
- Feed rate planning
- Dynamic acceleration
- Status via JSON
- Settings via JSON
- Macros
- Tool table

**Specific Implementation**:
- JSON serialization/deserialization
- Queue management
- Command groups
- Status parsing from JSON

#### 4.5.3 g2core
**Capabilities**:
- Advanced JSON protocol
- Extended motion planning
- Multiple tool heads
- Networked I/O
- File system support (SD card)
- Persistence
- Advanced settings

#### 4.5.4 Smoothieware
**Capabilities**:
- Smoothieware protocol
- RepRap G-code dialect
- Extensive M-code support
- Module-based architecture
- Network connectivity

#### 4.5.5 FluidNC
**Capabilities**:
- JSON protocol (similar to g2core)
- WiFi connectivity
- File system support
- Web-based interface
- Advanced kinematics
- Dynamic axis configuration

### 4.6 Settings and Configuration

#### 4.6.1 Application Settings
Stored in `~/.config/gcodekit4/` or platform equivalent:

**Connection Settings**:
- Last used connection type (Serial/TCP/WebSocket)
- Last used port/hostname/port
- Baud rate
- Default connection timeout
- Auto-reconnect enabled

**UI Preferences**:
- Window size and position
- Panel visibility and layout
- Theme (light/dark)
- Font size
- Language
- Keyboard shortcut customization

**File Processing**:
- Default preprocessors enabled
- Arc segment length
- Max line length
- Comment handling
- Default output directory
- Recent files list

**Machine Preferences**:
- Default jog increment
- Default jog feed rate
- Homing direction per axis
- Machine limits (soft limits)
- Units preference (mm/inch)
- Spindle type (PWM, relay, etc.)

**Advanced**:
- Status poll rate (Hz)
- Buffer timeout (ms)
- Command timeout (ms)
- Debug logging enabled
- Performance monitoring

#### 4.6.2 Firmware Settings
Controller-specific parameters:

**GRBL Settings** ($0-$32):
- Step pulse time
- Step idle delay
- Step port invert mask
- Direction port invert mask
- Stepper enable invert
- Limit pins invert
- Probe pin invert
- Status report options
- Junction deviation
- Arc tolerance
- Homing cycle
- Homing feed rate
- Homing seek rate
- Homing debounce
- Soft limit enable
- Hard limit enable
- Homing direction
- Invert spindle
- Invert coolant
- Spindle PWM frequency
- Spindle off value
- Spindle min value
- Spindle max value
- Modes (parking, safety door, etc.)

**Other Firmware**:
Similar but firmware-specific parameter sets

### 4.7 Error Handling and Recovery

#### 4.7.1 Error Categories

**Connection Errors**:
- Port not found
- Port in use
- Connection timeout
- Read/write timeout
- Connection lost during operation

**Protocol Errors**:
- Malformed response
- Unexpected response
- Protocol version mismatch
- Unsupported command

**G-Code Errors**:
- Syntax error
- Invalid coordinates
- Invalid feed rate
- Tool selection error
- Probing without probe
- Jog outside limits

**Machine Errors**:
- Hard limit hit
- Soft limit violation
- Homing cycle failure
- Spindle error
- Temperature alarm
- Motor overcurrent

#### 4.7.2 Error Recovery Strategies

**Connection Recovery**:
- Automatic reconnection (configurable)
- Manual reconnect button
- Connection status display
- Error log with timestamp

**Protocol Recovery**:
- Command retry (up to 3 attempts)
- Clear buffer on persistent error
- User notification with specific error
- Suggestion for resolution

**G-Code Recovery**:
- Skip problematic command (user choice)
- Pause and display error
- Highlight error line in editor
- Display error description and suggestion

**Machine Recovery**:
- Pause execution on error
- Display alarm code and description
- Provide recovery options:
  - Kill alarm lock
  - Manual intervention required
  - Soft reset and retry

### 4.8 Macro and Script System

#### 4.8.1 Macro Definition
Macros are sequences of G-Code commands:
```
[Macro: Clear XY]
G53 G0 Z-10        ; Move Z to safe height in machine coords
G53 G0 X0 Y0       ; Return to home
G54                ; Back to work coordinates
```

#### 4.8.2 Macro Features
- Variable substitution: `{VAR_NAME}`
- Conditional execution: if/else blocks (if hardware supported)
- Loops: repeat N times
- Comments: documented steps
- Manual pause: `{PAUSE "message"}`
- Input prompts: `{INPUT "parameter name"}`

#### 4.8.3 Built-in Macros
- Home all axes
- Go to XY zero
- Go to Z zero
- Clear offsets
- Probe Z surface
- Spindle on/off
- Coolant on/off
- Tool change prompt

#### 4.8.4 Macro Storage
- User-defined macros in `~/.config/gcodekit4/macros/`
- Macro library with descriptions
- Import/export functionality
- Execution history logging

## 5. User Interface Specification

### 5.1 Main Window Layout

**Top to Bottom**:
1. **Menu Bar**
   - File: New, Open, Save, Save As, Recent, Exit
   - Edit: Undo, Cut, Copy, Paste, Select All
   - View: Toggle Panels, Zoom, Theme, Language
   - Machine: Home, Reset, Unlock, Check Mode
   - Tools: Settings, Macro Editor, Calibration Wizard
   - Help: Documentation, Keyboard Shortcuts, About

2. **Toolbar**
   - Quick connect/disconnect
   - File open
   - Start/Pause/Stop buttons
   - Home button
   - Reset button

3. **Main Content Area** (configurable layout):
   - Left Panel: Connection, Jog, Coordinates (collapsible)
   - Center Panel: G-Code Editor (tabbed with Visualizer)
   - Right Panel: DRO, Overrides, Macros (collapsible)
   - Bottom Panel: Console, Status (collapsible)

4. **Status Bar**
   - Connection status
   - Controller state
   - Current file name
   - Line count / bytes
   - Progress percentage
   - Current time

### 5.2 Panel Specifications

#### 5.2.1 Connection Panel
- **Port Selector**: Dropdown with auto-detected ports
- **Baud Rate**: Dropdown (9600, 19200, 38400, 57600, 115200)
- **Connect/Disconnect**: Toggle button (disabled when connected)
- **Status Indicator**: Color-coded (Red: disconnected, Yellow: connecting, Green: connected)
- **Connection Info**: Shows active port and firmware version
- **Auto-reconnect**: Checkbox to enable auto-reconnection
- **Connection Log**: Recent connection attempts and errors

#### 5.2.2 DRO (Digital Readout) Panel
**Machine Coordinates** (read-only):
- MachX: Display with unit (mm/inch)
- MachY: Display with unit
- MachZ: Display with unit
- MachA, MachB, MachC: (if supported)

**Work Coordinates** (editable):
- WorkX: Display, editable to set offset
- WorkY: Display, editable to set offset
- WorkZ: Display, editable to set offset
- WorkA, WorkB, WorkC: (if supported)

**Status Display**:
- State: Large colored text (IDLE/RUN/HOLD/ALARM/etc.)
- Feed Rate: Current actual feed rate
- Spindle Speed: Current actual spindle speed
- Buffer Status: X/128 bytes

**Quick Buttons**:
- X0, Y0, Z0: Zero single axes
- All Zero: Zero all axes at once
- GoTo: Move to entered coordinates

#### 5.2.3 Jog Panel
**Increment Selection**:
- Radio buttons: 0.1mm, 1mm, 10mm, 100mm (or mm/inch)
- Custom increment: Text entry box

**Feed Rate**:
- Slider: 10-500 mm/min (or current machine units)
- Text input: Direct entry

**Jog Controls**:
```
        [+Z]
         |
  [-X] [0] [+X]
         |
        [-Z]
        
[+Y]    [0]    [-Y]
```
- Buttons for each direction
- Keyboard shortcuts: W/A/S/D or arrow keys
- Key repeat support for continuous jog

**Alternative**: Pendant/gamepad support for jogging

#### 5.2.4 File Operations Panel
- **File Browser**: Tree view of files and directories
- **File Info**: Name, size, line count, estimated time
- **Open Button**: Open selected file (also: drag-and-drop)
- **Recent Files**: Quick access to last 10 files
- **Statistics**:
  - Total lines
  - Total distance
  - Estimated time
  - Min/Max coordinates

#### 5.2.5 G-Code Editor Panel
**Features**:
- **Syntax Highlighting**: G/M/T codes, parameters, comments
- **Line Numbers**: Clickable for navigation
- **Current Line Indicator**: Highlight during execution
- **Breakpoints**: Set/clear at line numbers
- **Search/Replace**: Find and replace G-Code patterns
- **Copy/Paste**: Full editing capability
- **Undo/Redo**: Edit history

**Display**:
- Tab 1: Original file
- Tab 2: Processed file (if different)
- Diff mode: Highlight changes between original/processed

#### 5.2.6 Console Panel
**Display**:
- Scrollable text area
- Auto-scroll to latest
- Color-coded messages:
  - Black: Normal messages
  - Blue: Informational
  - Orange: Warnings
  - Red: Errors
  - Green: Success

**Controls**:
- Clear button
- Copy all button
- Filter options:
  - Show/hide INFO
  - Show/hide VERBOSE
  - Show/hide WARNINGS
  - Show/hide ERRORS

**Content**:
- Sent commands (with [SEND])
- Controller responses (with [RX])
- Status updates
- Error messages
- Connection events

#### 5.2.7 Control Panel
**Main Controls**:
- **Start**: Begin streaming file (large button, green)
- **Pause**: Pause execution (large button, yellow)
- **Stop**: Cancel execution (large button, red)
- **Home**: Home all axes (button)
- **Reset**: Soft reset (button)
- **Unlock**: Kill alarm lock (button)

**Status Display**:
- Lines sent / total lines
- Time elapsed / estimated remaining
- Progress bar
- Current line number

#### 5.2.8 Overrides Panel
**Feed Rate Override**:
- Slider: 0-200%
- Increment buttons: -10%, -5%, +5%, +10%
- Reset to 100% button
- Current percentage display

**Rapid Traverse Override**:
- Preset buttons: 25%, 50%, 100% (125% optional)
- Active indicator (button highlight)

**Spindle Override**:
- Slider: 0-200%
- Increment buttons: -10%, -5%, +5%, +10%
- Reset to 100% button
- Current percentage display

#### 5.2.9 Coordinate System Panel
**Work Coordinate System**:
- Selector: Dropdown G54, G55, G56, G57, G58, G59
- Current WCS highlight
- WCS Offset display (X, Y, Z offsets)

**Quick Set Buttons**:
- Set All: Set all axes to zero
- Set X: Set X to zero
- Set Y: Set Y to zero
- Set Z: Set Z to zero

**Offset Display**:
- Table showing G54-G59 offsets
- Editable cells
- Save/Cancel buttons

#### 5.2.10 Macros Panel
**Macro Buttons**:
- Grid of 12-16 macro buttons
- Button text: Macro name
- Double-click to edit
- Right-click for delete/edit options

**Macro Editor**:
- Modal dialog
- Name field
- Description field
- G-Code editor
- Save/Cancel buttons
- Test button (execute without machine)

#### 5.2.11 3D Visualizer Panel
**Display**:
- 3D viewport showing toolpath
- Color coding: 
  - Red: Rapid movements (G0)
  - Green: Feed movements (G1)
  - Blue: Arc movements (G2/G3)
  - Yellow: Current position
- Coordinate axes display (XYZ tri-axial)
- Grid overlay (configurable)

**Controls**:
- Mouse drag: Rotate view
- Mouse wheel: Zoom in/out
- Right-click drag: Pan
- View presets: Top, Front, Side, Isometric, Fit
- Reset view button
- Toggle grid
- Toggle axes

### 5.3 Dialogs and Modals

#### 5.3.1 Settings Dialog
**Tabs**:
1. **Connection**: Port, baud rate, timeouts, auto-reconnect
2. **UI**: Theme, font size, language, panel layout
3. **Machine**: Limits, default jog increment/feed rate, units
4. **File Processing**: Preprocessor defaults, output directory
5. **Keyboard**: Custom keyboard shortcuts
6. **Advanced**: Polling rate, buffer timeout, debug logging

#### 5.3.2 Firmware Settings Dialog
- Displays firmware-specific settings ($0-$32 for GRBL)
- Tree view: Settings grouped by category
- Per-setting display:
  - Setting number/name
  - Current value
  - Valid range
  - Description
  - Edit field
  - Revert button (if changed)
- Save All button
- Cancel button
- Backup/Restore buttons

#### 5.3.3 File Validation Dialog
- Shows validation results
- Errors: Listed with line numbers and descriptions
- Warnings: Non-blocking issues
- Suggestions: Recommended fixes
- Option to proceed or fix issues
- Details expandable per item

#### 5.3.4 Probing Wizard
1. **Setup**: Confirm probe type, confirm probe on tool
2. **Positioning**: Move to start location
3. **Parameters**: Probe distance, feed rate
4. **Execute**: Run probing cycle, display result
5. **Apply**: Set work offset or store in mesh

#### 5.3.5 About Dialog
- Application name and version
- Supported firmware types
- Build date/git commit
- License information
- Links to documentation/source

### 5.4 Keyboard Shortcuts (Configurable)

**Global**:
- Ctrl+O: Open file
- Ctrl+S: Save file
- Ctrl+Q: Quit application
- Ctrl+H: Home all axes
- Ctrl+L: Kill alarm lock
- Ctrl+R: Soft reset

**During Streaming**:
- Space: Pause/Resume
- Escape: Cancel
- P: Pause toggle
- S: Stop/Cancel

**Jogging**:
- W/Up: +Y
- S/Down: -Y
- A/Left: -X
- D/Right: +X
- Q: +Z
- Z: -Z

**Editing**:
- Ctrl+F: Find
- Ctrl+H: Find/Replace
- Ctrl+A: Select all
- Ctrl+C: Copy
- Ctrl+V: Paste
- Ctrl+Z: Undo
- Ctrl+Y: Redo

### 5.5 Themes and Styling

**Built-in Themes**:
1. **Light**: Light background, dark text
2. **Dark**: Dark background, light text
3. **High Contrast**: Maximum contrast for accessibility

**Customizable Elements**:
- Primary colors (for buttons, highlights)
- Text colors and sizes
- Panel borders and spacing
- Button sizes and corners
- Font selection (monospace for code)

## 6. Non-Functional Requirements

### 6.1 Performance

**File Loading**:
- Files up to 1MB: Load in <2 seconds
- Files up to 100K lines: Process in <5 seconds
- Parsing: >10,000 lines per second

**Streaming**:
- Command rate: >100 commands/second (varies by protocol)
- Buffer management: <1ms latency
- UI responsiveness: <100ms update latency

**2D Visualization**:
- Render time: <100ms for typical G-code files
- Supported formats: PNG (800x600)
- Color coding: Blue (cutting), Gray (rapid), Red (arcs)
- Auto-scaling: Maintains aspect ratio
- Concurrent rendering: Background thread processing

**3D Visualization**:
- Render frame rate: >30 FPS
- Toolpath vertex generation: <100ms for 100K lines
- Interactive camera: <16ms frame time

**Memory Usage**:
- Idle state: <50MB
- With 100K line file loaded: <150MB
- 2D visualization: <10MB per render
- 3D visualization: <100MB additional

### 6.2 Reliability

**Availability**:
- Target uptime: 99.5% (operational hours)
- Graceful degradation: Disconnection doesn't crash
- Auto-recovery: Attempt reconnection within 5 seconds

**Robustness**:
- Timeout all network operations (10-30 seconds)
- Validate all user input
- Handle malformed responses
- Recover from disconnections mid-stream

### 6.3 Usability

**Learnability**:
- First-time users can connect and jog within 2 minutes
- In-app help for all features
- Tooltips for all controls
- Built-in tutorial for common tasks

**Accessibility**:
- WCAG 2.1 AA compliance (where applicable)
- Keyboard navigation for all functions
- High contrast mode
- Adjustable font sizes
- Screen reader compatibility

### 6.4 Maintainability

**Code Quality**:
- Target: >80% code coverage with tests
- Lint warnings: None
- Documentation: All public APIs documented
- Code organization: Modular and extensible

**Build and Deployment**:
- Single-command build: `cargo build`
- CI/CD pipeline with automated tests
- Cross-platform builds (Linux, Windows, macOS)
- Self-contained binaries
- Automatic update mechanism (planned)

### 6.5 Security

**Data Protection**:
- No sensitive data storage (passwords, etc.)
- Settings file permissions: User-readable only
- G-Code file validation: No code injection
- Network communication: SSL/TLS for network connections

**System Integration**:
- No elevated privileges required
- Limited file system access (working directory)
- Serial port access only to selected port
- Network access only to specified hosts

## 7. Supported G-Code Commands

### 7.1 Motion Commands (G-Codes)

| Code | Function | Support |
|------|----------|---------|
| G0 | Rapid positioning | All |
| G1 | Linear interpolation | All |
| G2 | CW arc interpolation | All |
| G3 | CCW arc interpolation | All |
| G4 | Dwell | All |
| G10 | Set position | GRBL, FluidNC |
| G17 | XY plane selection | All |
| G18 | XZ plane selection | Most |
| G19 | YZ plane selection | Most |
| G20 | Programming in inches | All |
| G21 | Programming in mm | All |
| G28 | Home | Most |
| G30 | Home alternate | Most |
| G38.2 | Probe toward, signal | Most |
| G38.3 | Probe away, signal | Most |
| G38.4 | Probe toward, no signal | GRBL 1.1+ |
| G38.5 | Probe away, no signal | GRBL 1.1+ |
| G53 | Machine coordinates | GRBL, TinyG |
| G54-G59 | Work coordinate systems | All |
| G80 | Cancel canned cycle | GRBL, TinyG |
| G90 | Absolute positioning | All |
| G91 | Relative positioning | All |
| G92 | Set position (alias for G10) | TinyG |
| G93 | Feedrate override | TinyG |
| G94 | Feedrate per minute | All |
| G95 | Feedrate per revolution | Some |

### 7.2 Machine Commands (M-Codes)

| Code | Function | Support |
|------|----------|---------|
| M0 | Program stop | Most |
| M1 | Optional stop | Most |
| M2 | Program end | All |
| M3 | Spindle on, CW | All |
| M4 | Spindle on, CCW | Most |
| M5 | Spindle stop | All |
| M6 | Tool change | GRBL, TinyG |
| M7 | Coolant on (mist) | Most |
| M8 | Coolant on (flood) | Most |
| M9 | Coolant off | Most |
| M30 | Program end, rewind | GRBL 1.1+, TinyG |
| M92 | Program suspend | TinyG |

### 7.3 Tool Commands (T-Codes)

| Code | Function | Support |
|------|----------|---------|
| T[0-99] | Tool selection | GRBL, TinyG |

## 8. Firmware Capabilities Database

GcodeKit4 includes a comprehensive **Firmware Capabilities Database** that tracks which features are supported by each firmware version. This enables:

- Version-aware UI feature enabling/disabling
- G-code validation against controller capabilities
- Automatic warnings for unsupported operations
- Firmware-specific G-code generation variants

**Database Features:**
- 10+ capability categories tracked (motion, spindle, probing, offsets, safety, etc.)
- Support for GRBL (v0.9, v1.0, v1.1), TinyG (v2.0), g2core (v3.0), Smoothieware (v1.0), and FluidNC (v3.0)
- Version matching with major.minor fallback strategy
- Custom capability registration for new firmware

See [FIRMWARE_CAPABILITIES_DATABASE.md](docs/FIRMWARE_CAPABILITIES_DATABASE.md) for complete details.

### Firmware Capabilities Matrix

| Feature | GRBL | TinyG | g2core | Smoothie | FluidNC |
|---------|------|-------|--------|----------|---------|
| **Protocol** | Text | JSON | JSON | Text | JSON/WebSocket |
| **Axes** | 3-6 | 6 | 6+ | 3 | 3-6 |
| **Real-time Control** | ✓ | ✓ | ✓ | ✗ | ✓ |
| **Jogging** | ✓ | ✓ | ✓ | ✗ | ✓ |
| **Probing** | ✓ | ✓ | ✓ | ✗ | ✓ |
| **Work Coords** | ✓ | ✓ | ✓ | ✗ | ✓ |
| **Settings** | ✓ | ✓ | ✓ | Limited | ✓ |
| **Macros** | ✗ | ✓ | Limited | ✗ | Limited |
| **Tool Change** | ✓ | ✓ | ✓ | ✗ | ✓ |
| **File System** | ✗ | ✗ | ✓ | ✗ | ✓ |
| **Networking** | ✗ | Ethernet | Ethernet | Network | WiFi |

## 9. Constraints and Limitations

### 9.1 Hardware Constraints
- Minimum RAM: 512MB (2GB recommended)
- Minimum storage: 100MB (1GB recommended)
- Serial port: Standard RS-232 or USB serial adapter
- Network: Ethernet or WiFi for network controllers
- Display: Minimum 1024x768 resolution (1920x1080 recommended)

### 9.2 Software Constraints
- Rust version: 1.70.0 or newer
- Platform: Linux, Windows (7+), macOS (10.13+)
- No root/admin privileges required
- Single application instance (planned: multi-instance support)

### 9.3 Operational Constraints
- File size: Up to 100MB (practical limit ~1MB)
- Command queue: Limited by controller (typically 128 bytes)
- Streaming speed: Limited by controller communication
- Positioning accuracy: Limited by controller firmware
- Number of macros: Unlimited (limited by storage)
- Simultaneous connections: 1 (per instance)

## 10. Glossary

| Term | Definition |
|------|-----------|
| **CNC** | Computer Numerical Control - machine tool controlled by computer |
| **DRO** | Digital Readout - display of current machine position |
| **G-Code** | Numerical code used to program CNC machines |
| **WCS** | Work Coordinate System - user-defined coordinate origin (G54-G59) |
| **MCS** | Machine Coordinate System - machine's native coordinate system (G53) |
| **Jog** | Manual movement of machine axes |
| **Probe** | Tool for detecting part surfaces |
| **Override** | Real-time adjustment of feed rate, rapid rate, or spindle speed |
| **Buffer** | Queue of commands awaiting transmission to controller |
| **Modal** | State that persists until explicitly changed |
| **Preprocessor** | Software that modifies G-Code before transmission |
| **Toolpath** | Path that cutting tool follows |
| **Rapid** | Fast positioning movement (G0) |
| **Feed** | Controlled cutting movement (G1) |
| **Spindle** | Rotating tool holder on CNC machine |
| **Coolant** | Liquid spray for cooling/lubricating cutting |
| **M-Code** | Miscellaneous code for machine functions (vs. G-Code motion) |
| **T-Code** | Tool selection code |

## 11. Future Enhancements (Post-MVP)

### 11.1 Phase 2 Features
- Plugin system for third-party extensions
- Remote access via REST API and WebSocket
- Advanced collision detection
- Tool library management
- Automatic tool length offset probing
- Auto-leveling mesh generation and application
- Network file browsing (SMB/NFS)
- Streaming to cloud for backup

### 11.2 Phase 3 Features
- Kinematics support (non-Cartesian machines)
- Multi-head support
- Automated testing framework
- Performance profiling tools
- Advanced debugging UI
- Machine health monitoring
- Predictive maintenance alerts
- Export to multiple G-Code dialects

### 11.3 Phase 4 Features
- Mobile app (iOS/Android) via REST API
- Augmented reality visualization
- Machine learning-based optimization
- Blockchain-based job tracking
- Enterprise integration (MES systems)
- 3D CAM integration
- STL/OBJ file to G-Code conversion

## 12. Revision History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2024-10-21 | Initial specification document |

---

**Document Status**: Draft for Review
**Last Updated**: 2024-10-21
**Next Review**: Upon completion of Phase 1 implementation

## Designer UI Updates (November 2025)

### SVG Canvas Rendering
- Converted designer from image-based rendering to SVG Path elements
- Separate layers for crosshair, shapes, selected shapes, and selection handles
- Better performance and scalability at any zoom level

### Coordinate System
- Implemented CAD-standard coordinate system: (0,0) at bottom-left, +Y up, +X right
- 20px margin for origin positioning
- Fixed Y-axis flipping for all transformations (viewport, drag, resize)

### Shape Interaction
- Fixed shape movement Y-direction to match coordinate system
- Fixed circle resize to use incremental deltas (not absolute positioning)
- Improved selection handle rendering (8x8px, symmetric positioning)
- Fixed handle detection for Y-axis flip

### Context Menu & Properties
- Right-click context menu on selected shapes (Delete, Properties)
- Properties dialog for shape editing
- Corner radius editor for RoundRectangle shapes (0.1mm increments, 0-100mm range)
- Modal dialog with save/cancel functionality

### Coordinate System Fixes
- Fixed 740mm vertical offset by matching viewport size to canvas size
- Viewport updates on every render to maintain consistency
- Crosshair visibility improved with 10px buffer outside canvas bounds

### UI Cleanup
- Removed canvas status text (object count and mode display)
- Removed all eprintln! debug statements
- Cleaner console output

