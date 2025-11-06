# GCodeKit4 User Manual

**Version:** 0.25.4-alpha  
**Last Updated:** November 6, 2025  
**Application:** GCodeKit4 CNC Controller & G-Code Sender

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Getting Started](#2-getting-started)
3. [User Interface Overview](#3-user-interface-overview)
4. [Machine Control](#4-machine-control)
5. [G-Code Editor](#5-g-code-editor)
6. [2D CAD/CAM Designer](#6-2d-cadcam-designer)
7. [Device Console](#7-device-console)
8. [Configuration & Settings](#8-configuration--settings)
9. [Advanced Features](#9-advanced-features)
10. [Troubleshooting](#10-troubleshooting)
11. [Keyboard Shortcuts](#11-keyboard-shortcuts)
12. [Supported Controllers](#12-supported-controllers)
13. [Appendix](#13-appendix)

---

## 1. Introduction

### 1.1 What is GCodeKit4?

GCodeKit4 is a modern, cross-platform CNC machine controller application written in Rust with a Slint UI framework. It provides a comprehensive solution for controlling CNC machines, editing G-Code, designing toolpaths, and monitoring machine operations in real-time.

### 1.2 Key Features

- **Multi-Controller Support**: Works with GRBL, grblHAL, TinyG, g2core, Smoothieware, and FluidNC
- **Real-Time Control**: Live position tracking, status monitoring, and machine state display
- **6-Axis Support**: Control X, Y, Z linear axes plus A, B, C rotary axes
- **G-Code Editor**: Syntax highlighting, line numbers, and file operations
- **2D CAD/CAM Designer**: Create vector drawings and generate G-Code toolpaths
- **Smart Console**: Filtered command history with color-coded messages
- **Configuration Management**: Edit and save controller settings
- **Cross-Platform**: Runs on Linux, macOS, and Windows

### 1.3 System Requirements

**Minimum Requirements:**
- **RAM**: 512MB (2GB recommended)
- **Storage**: 100MB free space
- **Display**: 1024x768 resolution (1920x1080 recommended)
- **OS**: Linux, macOS 10.13+, or Windows 7+

**Required Hardware:**
- USB serial port or USB-to-serial adapter
- CNC controller (GRBL, TinyG, g2core, etc.)

---

## 2. Getting Started

### 2.1 Installation

#### Building from Source

```bash
# Clone the repository
git clone https://github.com/thawkins/gcodekit4.git
cd gcodekit4

# Build release version (optimized)
cargo build --release

# Run the application
cargo run --release
```

The compiled binary will be located at `target/release/gcodekit4`.

#### Development Build

```bash
# Build debug version (faster compilation)
cargo build

# Run with debug logging
RUST_LOG=debug cargo run
```

### 2.2 First Launch

When you first launch GCodeKit4, you'll see:

1. **Main Window**: Tabbed interface with Machine Control, G-Code Editor, Designer, and other panels
2. **Menu Bar**: File, Machine, Tools, and Help menus
3. **Connection Status**: Disconnected indicator in the status bar
4. **Empty Console**: Ready to display device communication

### 2.3 Connecting to Your CNC Controller

#### Step 1: Refresh Serial Ports

1. Navigate to the **Machine Control** tab
2. Click the **"Refresh Ports"** button
3. The port dropdown will populate with available serial devices

#### Step 2: Select Connection Parameters

1. **Port**: Choose your controller's serial port from the dropdown
   - Linux: `/dev/ttyUSB0`, `/dev/ttyACM0`, etc.
   - macOS: `/dev/cu.usbserial-*` or `/dev/tty.usbserial-*`
   - Windows: `COM3`, `COM4`, etc.

2. **Baud Rate**: Select the appropriate baud rate
   - GRBL: 115200 (most common)
   - TinyG: 115200
   - Other controllers: Check documentation

#### Step 3: Connect

1. Click the **"Connect"** button
2. Wait for "Device connected" message in the console
3. The status indicator will turn green
4. Device information will populate (firmware version, build info)

**Troubleshooting Connection Issues:**
- Ensure the controller is powered on
- Check that the USB cable is properly connected
- Verify the correct port is selected
- Try a different baud rate if connection fails
- Check that another application isn't using the port

### 2.4 Initial Machine Setup

#### Homing the Machine

**Important**: Always home your machine after connecting to establish a known position reference.

1. Ensure the machine has clear travel to home position
2. Click the **Home** button (⌂ icon) in the Machine Control panel
3. The machine will execute the homing cycle ($H command)
4. Wait for the cycle to complete
5. The DRO (Digital Readout) will display the home position

#### Understanding Machine States

After homing, your machine will typically be in one of these states:

- **Idle**: Ready to receive commands
- **Run**: Executing G-Code
- **Hold**: Paused, awaiting resume
- **Alarm**: Locked state requiring attention
- **Jog**: Manual jogging in progress
- **Home**: Homing cycle in progress

---

## 3. User Interface Overview

### 3.1 Main Window Layout

The GCodeKit4 interface is organized into several tabs and panels:

```
┌─────────────────────────────────────────────────┐
│ Menu Bar (File, Machine, Tools, Help)          │
├─────────────────────────────────────────────────┤
│ Tab Bar: [Machine] [Editor] [Designer] [...]   │
├─────────────────────────────────────────────────┤
│                                                 │
│                                                 │
│           Main Content Area                     │
│           (Tab-specific content)                │
│                                                 │
│                                                 │
├─────────────────────────────────────────────────┤
│ Status Bar: Connection | State | Progress      │
└─────────────────────────────────────────────────┘
```

### 3.2 Menu Bar

#### File Menu
- **Open**: Load G-Code file
- **Save**: Save current file
- **Recent Files**: Quick access to recently opened files
- **Exit**: Close application

#### Machine Menu
- **Connect/Disconnect**: Toggle connection to controller
- **Home**: Execute homing cycle ($H)
- **Unlock**: Clear alarm state ($X)
- **Reset**: Soft reset controller
- **Emergency Stop**: Immediate halt

#### Tools Menu
- **Settings**: Open configuration dialog
- **Device Settings**: Edit controller parameters
- **Import SVG**: Load SVG file into designer
- **Import DXF**: Load DXF file into designer

#### Help Menu
- **Documentation**: Open this manual
- **Keyboard Shortcuts**: Display shortcut reference
- **About**: Application version and information

### 3.3 Tabs Overview

1. **Machine Control**: Connection, jogging, position display, status
2. **G-Code Editor**: Edit and preview G-Code files
3. **Designer**: 2D CAD/CAM vector drawing and toolpath generation
4. **G-Code Visualizer**: 2D preview of toolpaths
5. **Device Console**: Command history and communication log
6. **Config Settings**: Edit controller settings ($0-$130)
7. **Device Info**: Firmware version, build info, capabilities
8. **G-Tools**: Utilities and helper tools

### 3.4 Status Bar

The status bar displays:

- **Connection Status**: Green (connected), Red (disconnected), Yellow (connecting)
- **Machine State**: Current controller state (Idle, Run, Hold, Alarm, etc.)
- **Position**: Current X, Y, Z coordinates
- **Feed Rate**: Actual feed rate during operation
- **Spindle Speed**: Current spindle RPM
- **Progress**: Percentage complete during G-Code execution

---

## 4. Machine Control

### 4.1 Digital Readout (DRO)

The DRO displays real-time position information with 0.001mm precision:

#### Machine Position (MPos)
- **MachX, MachY, MachZ**: Position in machine coordinate system
- **MachA, MachB, MachC**: Rotary axes (if equipped)
- Reference: Based on machine home position

#### Work Position (WPos)
- **WorkX, WorkY, WorkZ**: Position in work coordinate system
- **WorkA, WorkB, WorkC**: Rotary axes in work coordinates
- Reference: Based on current work coordinate system (G54-G59)

#### Status Indicators
- **State**: Large colored text indicating machine state
  - Green: Idle (ready)
  - Blue: Run (executing)
  - Yellow: Hold (paused)
  - Red: Alarm (locked)
- **Feed Rate**: Current feed rate in mm/min or in/min
- **Spindle Speed**: Current RPM

### 4.2 Jogging Controls

Jogging allows manual movement of machine axes.

#### Step Sizes

Select from predefined increments:
- **0.1mm**: Fine positioning
- **1mm**: Standard positioning
- **10mm**: Coarse positioning
- **100mm**: Rapid repositioning

For rotary axes (A, B, C), step sizes are in degrees.

#### Jogging Buttons

**Linear Axes:**
- **X+/X-**: Move right/left
- **Y+/Y-**: Move forward/backward
- **Z+/Z-**: Move up/down

**Rotary Axes:**
- **A+/A-**: Rotate A axis clockwise/counterclockwise
- **B+/B-**: Rotate B axis clockwise/counterclockwise

**Important Notes:**
- Jog commands use G91 (relative positioning)
- Commands are sent immediately to controller
- Ensure sufficient clearance before jogging
- Stop jogging by clicking any control or pressing ESC

### 4.3 Machine Operations

#### Homing ($H)

The homing cycle establishes a known position reference:

1. Click the **Home** button
2. Machine moves to find home switches
3. Typical sequence: Z→X→Y (configurable)
4. Position resets to machine home
5. Status changes to Idle when complete

**Safety**: Ensure clear travel path to home position.

#### Unlock ($X)

Clears alarm state and unlocks the controller:

1. Click the **Unlock** button (🔒 icon)
2. Sends $X command to controller
3. Alarm state clears
4. Machine returns to Idle state

**When to Use:**
- After hitting a limit switch
- After emergency stop
- After soft limit violation
- When controller shows ALARM state

#### Reset (Soft Reset)

Performs a soft reset of the controller:

1. Click **Reset** or press Ctrl+R
2. Sends 0x18 character (Ctrl-X)
3. Clears command queue
4. Resets parser state
5. Does not lose position

**Use Cases:**
- Cancel running program
- Clear error state
- Reset parser modals

### 4.4 Work Coordinate Systems

GCodeKit4 supports six work coordinate systems (WCS):

- **G54**: Default work coordinate system
- **G55-G59**: Additional work coordinate systems

#### Setting Work Zero

To set the current position as work zero:

1. Jog machine to desired zero position
2. Click **Zero X**, **Zero Y**, or **Zero Z** button
3. Or click **Zero All** to zero all axes
4. The work position (WPos) will show 0.000

#### Switching Work Coordinate Systems

1. Navigate to **Config Settings** tab
2. Send appropriate command:
   - `G54` for first WCS
   - `G55` for second WCS
   - etc.
3. Current WCS is displayed in status

### 4.5 Feed Rate & Spindle Overrides

Real-time adjustments to machine operation:

#### Feed Rate Override
- **Range**: 10% to 200%
- **Default**: 100%
- **Use**: Slow down or speed up cutting operations
- **Real-time**: Applied without stopping

#### Rapid Override
- **Options**: 25%, 50%, 100%
- **Use**: Reduce rapid traverse speed for safety
- **Real-time**: Applied immediately

#### Spindle Override
- **Range**: 10% to 200%
- **Default**: 100%
- **Use**: Adjust spindle speed during operation
- **Real-time**: Applied via PWM/relay

**Note**: Override availability depends on controller firmware.

---

## 5. G-Code Editor

### 5.1 Opening G-Code Files

#### Methods to Open Files:

1. **Menu**: File → Open
2. **Keyboard**: Ctrl+O
3. **Drag and Drop**: Drag .gcode, .nc, .ngc, or .tap file into window

#### Supported Formats:
- `.gcode` - Standard G-Code
- `.gc`, `.ngc` - Common variations
- `.tap` - CNC tap files
- `.iso` - ISO 6983 standard
- Plain text files with G-Code

### 5.2 Editor Features

#### Syntax Highlighting

The editor provides color-coded syntax highlighting:

- **G-Codes**: Blue (G0, G1, G2, G3, etc.)
- **M-Codes**: Purple (M3, M5, M8, M9, etc.)
- **Coordinates**: Green (X, Y, Z, F, S values)
- **Comments**: Gray (parentheses and semicolon comments)

#### Line Numbers

- Displayed on the left margin
- Click line number to select line
- Useful for tracking execution progress

#### Editing

The editor supports full text editing:
- **Cut/Copy/Paste**: Standard clipboard operations
- **Undo/Redo**: Ctrl+Z / Ctrl+Y
- **Find**: Ctrl+F
- **Select All**: Ctrl+A

### 5.3 File Operations

#### Save File

1. **Menu**: File → Save
2. **Keyboard**: Ctrl+S
3. Saves current editor content to file
4. Creates backup if file exists

#### Save As

1. **Menu**: File → Save As
2. Choose location and filename
3. Saves to new file path

#### Recent Files

Quick access to previously opened files:
1. **Menu**: File → Recent Files
2. Click filename to reopen

### 5.4 Sending G-Code to Controller

#### Execute Current File

1. Ensure controller is connected
2. Click **"Send to Device"** button
3. Monitor progress in status bar
4. Watch execution in console

#### Execution Controls

- **Pause**: Hold execution (feed hold)
- **Resume**: Continue from pause
- **Stop**: Cancel execution, clear queue

#### Progress Monitoring

During execution, the editor displays:
- Current line highlighted
- Progress percentage
- Lines sent / total lines
- Estimated time remaining

### 5.5 File Validation

Before sending, GCodeKit4 validates:
- **Syntax**: Correct command format
- **Coordinates**: Within machine limits (if set)
- **Modal State**: Valid command sequences
- **Feed Rates**: Non-zero where required

Warnings and errors display in the console.

---

## 6. 2D CAD/CAM Designer

### 6.1 Overview

The Designer tab provides vector drawing tools and toolpath generation for creating simple G-Code programs directly within GCodeKit4.

### 6.2 Coordinate System

The designer uses a CAD-standard coordinate system:
- **Origin (0,0)**: Bottom-left corner
- **+X**: Right direction
- **+Y**: Up direction
- **Units**: Millimeters (mm)
- **Grid**: 10mm major grid, 1mm minor grid

### 6.3 Drawing Tools

Access drawing tools from the toolbar:

#### Rectangle
1. Click **Rectangle** tool
2. Click canvas to set first corner
3. Drag to set size
4. Click to complete

#### Circle
1. Click **Circle** tool
2. Click canvas to set center
3. Drag to set radius
4. Click to complete

#### Ellipse
1. Click **Ellipse** tool
2. Click canvas to set center
3. Drag to set width
4. Drag again to set height
5. Click to complete

#### Line
1. Click **Line** tool
2. Click canvas for start point
3. Click for end point
4. Line is created

#### Polygon
1. Click **Polygon** tool
2. Click canvas for first vertex
3. Click for each additional vertex
4. Double-click or press Enter to complete

#### Arc
1. Click **Arc** tool
2. Click for start point
3. Click for end point
4. Click for arc bulge point
5. Arc is created

#### Round Rectangle
1. Click **Round Rectangle** tool
2. Click canvas to set first corner
3. Drag to set size
4. Click to complete
5. Adjust corner radius in properties

### 6.4 Selecting and Editing Shapes

#### Selection

- **Click**: Select single shape
- **Shift+Click**: Add to selection
- **Drag Box**: Select multiple shapes
- **Ctrl+A**: Select all shapes

#### Moving Shapes

1. Select shape(s)
2. Drag to new position
3. Release to place

#### Resizing Shapes

1. Select shape
2. Drag selection handle (small squares)
3. Resize proportionally or stretch
4. Release to apply

#### Selection Handles

Selected shapes show 8 handles:
- **Corners**: Resize diagonally
- **Edges**: Resize along edge
- **Center**: Move shape

### 6.5 Shape Properties

Right-click a selected shape to access:

#### Properties Dialog

- **Position**: X, Y coordinates
- **Size**: Width, Height
- **Stroke Color**: Outline color
- **Fill Color**: Interior color
- **Line Width**: Stroke thickness
- **Corner Radius**: (Round rectangles only)

Edit values and click **Save** to apply changes.

#### Delete Shape

Right-click → **Delete** to remove selected shape(s).

### 6.6 Canvas Controls

#### Zoom

- **Zoom In**: Ctrl+Plus or mouse wheel up
- **Zoom Out**: Ctrl+Minus or mouse wheel down
- **Fit to View**: Click **Fit** button
- **100%**: Click **1:1** button

#### Pan

- **Drag**: Click and drag empty canvas area
- **Arrow Keys**: Pan in direction

#### Grid

- **Toggle Grid**: Click grid icon
- **Major Grid**: 10mm spacing
- **Minor Grid**: 1mm spacing

### 6.7 Importing Files

#### Import SVG

1. Menu: Tools → Import SVG
2. Select SVG file
3. Shapes are imported to canvas
4. Edit as needed

#### Import DXF

1. Menu: Tools → Import DXF
2. Select DXF file
3. Shapes are imported to canvas
4. Edit as needed

**Supported Elements:**
- Lines
- Circles
- Arcs
- Polylines
- Rectangles
- Ellipses

### 6.8 Generating Toolpaths

#### Toolpath Options

1. Select shapes to include
2. Click **Generate Toolpath** button
3. Configure options:
   - **Tool Diameter**: Cutting tool size (mm)
   - **Cut Depth**: Total depth of cut (mm)
   - **Pass Depth**: Depth per pass (mm)
   - **Feed Rate**: Cutting speed (mm/min)
   - **Plunge Rate**: Z-axis feed rate (mm/min)
   - **Spindle Speed**: RPM

#### Toolpath Strategies

- **Pocket**: Fill interior with cuts
- **Contour**: Cut along outline
- **Drill**: Point operations

#### Generate G-Code

1. Configure toolpath options
2. Click **Generate**
3. G-Code is created in editor
4. Save or send to machine

### 6.9 Designer Workflow Example

**Creating a Simple Pocket:**

1. Draw rectangle (50mm × 50mm)
2. Position at X=10, Y=10
3. Select rectangle
4. Click **Generate Toolpath**
5. Set:
   - Tool: 3mm
   - Depth: 5mm
   - Pass: 1mm
   - Feed: 500mm/min
6. Click **Generate**
7. Review G-Code in editor
8. Send to machine

---

## 7. Device Console

### 7.1 Overview

The Device Console displays communication between GCodeKit4 and the CNC controller.

### 7.2 Message Types

Messages are color-coded for clarity:

- **Blue**: Commands sent to controller
- **White**: Standard responses
- **Green**: Success messages (ok)
- **Red**: Error messages
- **Gray**: Verbose/debug information

### 7.3 Intelligent Filtering

The console automatically filters:
- **Status Polls**: No "?" commands logged
- **Status Responses**: No `<Idle|MPos:...>` spam
- **Byte Counts**: Buffer status messages hidden

This keeps the console clean and focused on meaningful commands and responses.

### 7.4 Console Features

#### Scroll

- Auto-scrolls to latest message
- Scroll up to view history
- Manual scroll stops auto-scroll

#### Clear Console

Click **Clear** button to remove all messages.

#### Copy Messages

Select text and press Ctrl+C to copy.

#### Timestamps

Toggle timestamp display:
- Click **Show Timestamps** checkbox
- Format: `[HH:MM:SS] message`

### 7.5 Interpreting Messages

#### Command Echo

```
[SEND] G0 X10 Y20
```
Command sent to controller.

#### Response

```
[RX] ok
```
Controller acknowledged command.

#### Error Example

```
[RX] error: Expected command letter
```
Controller rejected command due to syntax error.

#### Alarm Example

```
[RX] ALARM:1 (Hard limit triggered)
```
Controller entered alarm state.

---

## 8. Configuration & Settings

### 8.1 Application Settings

Access via: **Tools → Settings**

#### Connection Tab

- **Default Port**: Last used serial port
- **Default Baud Rate**: 115200
- **Connection Timeout**: 5 seconds
- **Auto-Reconnect**: Enable/disable

#### UI Preferences

- **Theme**: Light or Dark
- **Font Size**: 10-16pt
- **Language**: English (more coming)
- **Panel Layout**: Customize tab order

#### Machine Preferences

- **Default Jog Increment**: 1mm
- **Default Feed Rate**: 500 mm/min
- **Units**: Metric (mm) or Imperial (inch)
- **Soft Limits**: Enable warnings

#### File Processing

- **Arc Segment Length**: 0.5mm
- **Max Line Length**: 80 characters
- **Comment Handling**: Keep or remove

### 8.2 Controller Settings (GRBL)

Access via: **Config Settings** tab

#### Settings Display

The Config Settings tab shows all GRBL parameters ($0-$130):

- **Setting Number**: $0, $1, $2, etc.
- **Description**: Parameter purpose
- **Current Value**: Value from controller
- **Units**: mm, mm/min, µs, etc.

#### Editing Settings

1. Click on a setting value
2. Enter new value
3. Press Enter to send
4. Controller confirms change
5. Value updates in table

#### Common Settings

| Setting | Parameter | Typical Value |
|---------|-----------|---------------|
| $100 | X steps/mm | 250.000 |
| $101 | Y steps/mm | 250.000 |
| $102 | Z steps/mm | 250.000 |
| $110 | X max rate mm/min | 5000.000 |
| $111 | Y max rate mm/min | 5000.000 |
| $112 | Z max rate mm/min | 2500.000 |
| $120 | X acceleration mm/sec² | 250.000 |
| $121 | Y acceleration mm/sec² | 250.000 |
| $122 | Z acceleration mm/sec² | 250.000 |
| $130 | X max travel mm | 200.000 |
| $131 | Y max travel mm | 200.000 |
| $132 | Z max travel mm | 200.000 |

#### Homing Settings

| Setting | Parameter | Values |
|---------|-----------|--------|
| $20 | Soft limits enable | 0=off, 1=on |
| $21 | Hard limits enable | 0=off, 1=on |
| $22 | Homing cycle enable | 0=off, 1=on |
| $23 | Homing dir invert | Bitmask |
| $24 | Homing feed mm/min | 100.000 |
| $25 | Homing seek mm/min | 2000.000 |
| $27 | Homing pull-off mm | 5.000 |

#### Backup and Restore

**Export Settings:**
1. Click **Export** button
2. Save to .json file
3. Settings preserved for backup

**Import Settings:**
1. Click **Import** button
2. Select .json file
3. Confirm restore
4. Settings written to controller

### 8.3 Device Information

Access via: **Device Info** tab

#### Firmware Information

- **Version**: GRBL v1.1f, TinyG v0.97, etc.
- **Build Date**: Firmware compilation date
- **Build Options**: Enabled features

#### Capabilities

GCodeKit4 detects and displays controller capabilities:

- ✓ **Arc Support**: G2/G3 circular interpolation
- ✓ **Variable Spindle**: PWM spindle control
- ✓ **Homing Cycle**: $H support
- ✓ **Probe**: G38.x probing commands
- ✓ **Laser Mode**: Laser cutting support
- ✓ **Multi-Axis**: 4-6 axis support
- ✓ **Safety Door**: Door interlock
- ✓ **Coolant**: M7/M8/M9 support
- ✓ **Tool Change**: M6 support

#### Statistics

- **Commands Sent**: Total command count
- **Errors**: Error count
- **Alarms**: Alarm count
- **Uptime**: Connection duration

---

## 9. Advanced Features

### 9.1 Probing Operations

**Note**: Requires probe connected to controller.

#### Z-Axis Probing

1. Mount probe or touch plate
2. Position tool above probe
3. Send command: `G38.2 Z-25 F100`
   - Z-25: Probe down 25mm max
   - F100: Feed rate 100mm/min
4. Controller stops at contact
5. Use result to set work Z zero

#### Edge Finding

1. Position tool near edge
2. Send probe command:
   - `G38.2 X10 F50` (probe +X direction)
   - `G38.2 Y-10 F50` (probe -Y direction)
3. Set work zero at detected edge

### 9.2 Tool Changes

#### Manual Tool Change

When G-Code contains `M6 T1`:

1. Machine pauses
2. Change tool manually
3. Resume operation
4. Machine continues

#### Tool Length Offset

1. Probe Z with each tool
2. Record heights
3. Apply offset: `G43.1 Z[offset]`
4. Cancel: `G49`

### 9.3 Auto-Leveling (Mesh Compensation)

**Future Feature**: Generate height map and apply Z corrections.

Workflow:
1. Define probe grid
2. Probe points
3. Generate mesh
4. Apply to G-Code

### 9.4 Custom Macros

**Future Feature**: User-definable G-Code sequences.

Example macro:
```gcode
; Macro: Safe Z Clear
G53 G0 Z-10    ; Machine coords Z safe
G54            ; Back to work coords
```

### 9.5 Keyboard Jogging

Enable keyboard jogging in Settings:

- **W/Up**: +Y
- **S/Down**: -Y
- **A/Left**: -X
- **D/Right**: +X
- **Q/Page Up**: +Z
- **Z/Page Down**: -Z

Hold key for continuous jog, or tap for single increment.

---

## 10. Troubleshooting

### 10.1 Connection Issues

#### Port Not Found

**Symptoms**: No ports in dropdown

**Solutions:**
- Verify USB cable is connected
- Check controller is powered on
- Click **Refresh Ports**
- Check USB cable is data-capable (not charge-only)
- Try different USB port
- Check USB drivers installed (Windows)

#### Connection Timeout

**Symptoms**: "Connection timeout" error

**Solutions:**
- Verify correct baud rate (usually 115200)
- Try different baud rates
- Check for other applications using port
- Restart controller
- Try different USB cable

#### Device Busy

**Symptoms**: "Port in use" or "Device busy"

**Solutions:**
- Close other terminal programs (Arduino IDE, CoolTerm, etc.)
- Disconnect from other GCodeKit4 instances
- Restart computer if issue persists

### 10.2 Controller Errors

#### ALARM State

**Symptoms**: Red "ALARM" status, machine locked

**Common Alarms:**
- **ALARM:1**: Hard limit triggered
- **ALARM:2**: Soft limit violated
- **ALARM:3**: Reset during motion
- **ALARM:4**: Probe fail
- **ALARM:5**: Probe fail (initial state)
- **ALARM:6**: Homing fail (door open)
- **ALARM:7**: Homing fail (limit switch)
- **ALARM:8**: Homing fail (other)
- **ALARM:9**: Limit error during cycle

**Recovery:**
1. Determine cause (check console)
2. Clear physical issue (hit switch, door open, etc.)
3. Click **Unlock** button ($X)
4. Re-home if needed

#### Position Lost

**Symptoms**: Incorrect position after reset

**Solutions:**
- Always home after controller reset
- Check homing switches functional
- Verify $23 homing direction setting
- Recalibrate steps/mm if persistent

### 10.3 G-Code Errors

#### Syntax Error

**Symptoms**: "error: Expected command letter"

**Cause**: Invalid G-Code syntax

**Solutions:**
- Review line in editor
- Check for typos
- Verify modal state
- Validate with online checker

#### Unsupported Command

**Symptoms**: "error: Unsupported or invalid g-code"

**Cause**: Controller doesn't support command

**Solutions:**
- Check Device Info capabilities
- Modify G-Code for controller
- Use alternative command
- Update firmware if available

### 10.4 Performance Issues

#### Slow UI Response

**Solutions:**
- Close unused tabs
- Reduce status poll rate (Settings)
- Clear console history
- Restart application

#### Choppy Streaming

**Solutions:**
- Reduce G-Code complexity
- Increase buffer size (if supported)
- Check USB cable quality
- Reduce real-time overrides

### 10.5 File Issues

#### File Won't Load

**Symptoms**: Error opening G-Code file

**Solutions:**
- Verify file encoding (UTF-8 or ASCII)
- Check file isn't corrupted
- Try opening in text editor first
- Ensure file size < 100MB

#### Visualization Fails

**Symptoms**: Blank visualizer canvas

**Solutions:**
- Check file has valid coordinates
- Verify G0/G1 commands present
- Reduce file size if very large
- Clear cache and reload

---

## 11. Keyboard Shortcuts

### 11.1 Global Shortcuts

| Shortcut | Action |
|----------|--------|
| **Ctrl+O** | Open G-Code file |
| **Ctrl+S** | Save current file |
| **Ctrl+Q** | Quit application |
| **Ctrl+H** | Home all axes |
| **Ctrl+R** | Soft reset controller |
| **F11** | Toggle fullscreen |

### 11.2 Editor Shortcuts

| Shortcut | Action |
|----------|--------|
| **Ctrl+A** | Select all text |
| **Ctrl+C** | Copy selected text |
| **Ctrl+V** | Paste from clipboard |
| **Ctrl+X** | Cut selected text |
| **Ctrl+Z** | Undo last change |
| **Ctrl+Y** | Redo last undo |
| **Ctrl+F** | Find text |

### 11.3 Jogging Shortcuts

*Note: Enable in Settings → Machine → Enable Keyboard Jogging*

| Shortcut | Action |
|----------|--------|
| **W** or **↑** | Jog +Y (forward) |
| **S** or **↓** | Jog -Y (backward) |
| **A** or **←** | Jog -X (left) |
| **D** or **→** | Jog +X (right) |
| **Q** or **PgUp** | Jog +Z (up) |
| **Z** or **PgDn** | Jog -Z (down) |

### 11.4 Streaming Shortcuts

| Shortcut | Action |
|----------|--------|
| **Space** | Pause/Resume |
| **Esc** | Stop/Cancel |

### 11.5 Designer Shortcuts

| Shortcut | Action |
|----------|--------|
| **Ctrl+A** | Select all shapes |
| **Delete** | Delete selected shapes |
| **Ctrl+Plus** | Zoom in |
| **Ctrl+Minus** | Zoom out |
| **Arrow Keys** | Pan canvas |

---

## 12. Supported Controllers

### 12.1 GRBL

**Versions**: v0.9, v1.0, v1.1, v1.2

**Features:**
- Character counting protocol
- Real-time commands (feed hold, resume, reset)
- Status reports (`<Idle|MPos:...>`)
- Settings system ($0-$32)
- Work coordinate systems (G54-G59)
- Homing cycles ($H)
- Soft limits
- Probing (G38.2, G38.5)
- Feed/rapid/spindle overrides

**Typical Baud Rate**: 115200

**Resources:**
- [GRBL GitHub](https://github.com/grbl/grbl)
- [GRBL Wiki](https://github.com/grbl/grbl/wiki)

### 12.2 grblHAL

**Versions**: Latest

**Features:**
- Enhanced GRBL protocol
- Faster execution
- Extended features
- Plugin architecture
- Network support (some boards)

**Typical Baud Rate**: 115200

**Resources:**
- [grblHAL GitHub](https://github.com/grblHAL)

### 12.3 TinyG

**Versions**: v0.97+

**Features:**
- JSON protocol
- 6-axis support
- Advanced motion planning
- Macros
- Tool tables

**Typical Baud Rate**: 115200

**Resources:**
- [TinyG Wiki](https://github.com/synthetos/TinyG/wiki)

### 12.4 g2core

**Versions**: Latest

**Features:**
- Advanced JSON protocol
- File system support
- Networking
- Multi-tool support
- Advanced kinematics

**Typical Baud Rate**: 115200

**Resources:**
- [g2core GitHub](https://github.com/synthetos/g2)

### 12.5 Smoothieware

**Versions**: Latest

**Features:**
- RepRap G-Code dialect
- Extensive M-code support
- Network connectivity
- Module-based architecture

**Typical Baud Rate**: 115200

**Resources:**
- [Smoothieware.org](http://smoothieware.org/)

### 12.6 FluidNC

**Versions**: Latest

**Features:**
- JSON protocol
- WiFi connectivity
- WebSocket support
- SD card file system
- Web interface
- Dynamic axis configuration

**Typical Baud Rate**: 115200

**Resources:**
- [FluidNC GitHub](https://github.com/bdring/FluidNC)

---

## 13. Appendix

### 13.1 G-Code Reference

#### Motion Commands

| Code | Description | Example |
|------|-------------|---------|
| **G0** | Rapid positioning | `G0 X10 Y20` |
| **G1** | Linear feed | `G1 X10 Y20 F500` |
| **G2** | Clockwise arc | `G2 X10 Y10 I5 J0` |
| **G3** | CCW arc | `G3 X10 Y10 I5 J0` |
| **G4** | Dwell | `G4 P1.5` (1.5 sec) |

#### Coordinate Systems

| Code | Description |
|------|-------------|
| **G53** | Machine coordinates |
| **G54** | Work coordinate 1 |
| **G55** | Work coordinate 2 |
| **G56** | Work coordinate 3 |
| **G57** | Work coordinate 4 |
| **G58** | Work coordinate 5 |
| **G59** | Work coordinate 6 |

#### Plane Selection

| Code | Description |
|------|-------------|
| **G17** | XY plane (default) |
| **G18** | XZ plane |
| **G19** | YZ plane |

#### Units

| Code | Description |
|------|-------------|
| **G20** | Inches |
| **G21** | Millimeters |

#### Positioning Mode

| Code | Description |
|------|-------------|
| **G90** | Absolute positioning |
| **G91** | Relative/incremental |

#### Machine Commands

| Code | Description |
|------|-------------|
| **M3** | Spindle on CW |
| **M4** | Spindle on CCW |
| **M5** | Spindle off |
| **M6** | Tool change |
| **M7** | Coolant mist on |
| **M8** | Coolant flood on |
| **M9** | Coolant off |

### 13.2 Units and Precision

#### Internal Representation

- **Dimensions**: Stored in millimeters (mm)
- **Type**: f32 (32-bit floating point)
- **Display Precision**: 2 decimal places (0.01mm)
- **DateTime**: UTC internally, local in UI
- **Text Encoding**: UTF-8

#### Unit Conversion

- **1 inch = 25.4 mm**
- **1 mm = 0.03937 inches**

Toggle units in Settings → Machine → Units.

### 13.3 Configuration File Locations

#### Linux
- Config: `~/.config/gcodekit4/config.json`
- Macros: `~/.config/gcodekit4/macros/`
- Logs: `~/.local/share/gcodekit4/logs/`

#### macOS
- Config: `~/Library/Application Support/gcodekit4/config.json`
- Macros: `~/Library/Application Support/gcodekit4/macros/`
- Logs: `~/Library/Logs/gcodekit4/`

#### Windows
- Config: `%APPDATA%\gcodekit4\config.json`
- Macros: `%APPDATA%\gcodekit4\macros\`
- Logs: `%APPDATA%\gcodekit4\logs\`

### 13.4 File Format Specifications

#### G-Code Files (.gcode, .nc, .ngc, .tap)

**Format**: Plain text, UTF-8 or ASCII encoding

**Structure:**
```gcode
; Comment line (semicolon)
(Comment in parentheses)
G21         ; Set units to mm
G90         ; Absolute positioning
G0 Z5       ; Rapid to Z=5
G0 X0 Y0    ; Rapid to origin
M3 S12000   ; Spindle on at 12000 RPM
G1 Z-1 F100 ; Plunge at 100mm/min
G1 X10 F500 ; Feed to X=10 at 500mm/min
M5          ; Spindle off
M2          ; Program end
```

#### Settings Files (.json)

**Format**: JSON

**Example:**
```json
{
  "connection": {
    "port": "/dev/ttyUSB0",
    "baud_rate": 115200,
    "timeout_ms": 5000
  },
  "machine": {
    "default_jog_increment": 1.0,
    "default_feed_rate": 500.0,
    "units": "mm"
  }
}
```

### 13.5 Error Code Reference

#### GRBL Error Codes

| Code | Description |
|------|-------------|
| error:1 | G-code words consist of a letter and a value |
| error:2 | Numeric value format is not valid |
| error:3 | Grbl '$' system command was not recognized |
| error:4 | Negative value received for expected positive |
| error:5 | Homing cycle not enabled via settings |
| error:6 | Minimum step pulse time must be greater than 3µs |
| error:7 | EEPROM read failed |
| error:8 | Grbl not in IDLE or JOG state |
| error:9 | G-code lock-out during alarm |
| error:10 | Soft limits cannot be enabled without homing |
| error:11 | Max characters per line exceeded |
| error:12 | GRBL $ setting value exceeds maximum |
| error:13 | Safety door opened and door state initiated |
| error:14 | Build info or startup line exceeded EEPROM |
| error:15 | Jog target exceeds machine travel |
| error:16 | Jog command with no '=' or contains prohibited |
| error:17 | Laser mode requires PWM output |
| error:20 | Unsupported or invalid g-code command |
| error:21 | More than one g-code command per modal group |
| error:22 | Feed rate has not yet been set or is undefined |
| error:23 | G-code command requires an integer value |
| error:24 | Two G-code commands that both require XYZ |
| error:25 | Repeated G-code word found in block |
| error:26 | No axis words in command block |
| error:27 | Line number value is invalid |
| error:28 | G-code command is missing a required value |
| error:29 | G59.x work coordinate systems not supported |
| error:30 | G53 only allowed with G0 and G1 motion modes |
| error:31 | Axis words exist in block while no command |
| error:32 | G2/G3 arcs need at least one in-plane axis |
| error:33 | Motion command target is invalid |
| error:34 | Arc radius value is invalid |
| error:35 | G2/G3 arcs need at least one offset in plane |
| error:36 | Unused value words found in block |
| error:37 | G43.1 dynamic tool length offset not assigned |
| error:38 | Tool number greater than max supported value |

### 13.6 Glossary

| Term | Definition |
|------|-------------|
| **CNC** | Computer Numerical Control - automated control of machine tools |
| **DRO** | Digital Readout - display showing current machine position |
| **G-Code** | Programming language for CNC machines |
| **MPos** | Machine Position - position in machine coordinate system |
| **WPos** | Work Position - position in work coordinate system |
| **WCS** | Work Coordinate System - user-defined coordinate origin (G54-G59) |
| **Jog** | Manual incremental movement of machine axes |
| **Home** | Reference position established by homing cycle |
| **Probe** | Tool for detecting surfaces and establishing positions |
| **Override** | Real-time adjustment of feed rate, rapid rate, or spindle speed |
| **Feed Rate** | Speed of cutting movement (mm/min or in/min) |
| **Spindle** | Rotating tool holder |
| **Coolant** | Liquid for cooling and lubricating during cutting |
| **Alarm** | Error condition requiring user intervention |
| **Soft Limit** | Software-enforced travel boundary |
| **Hard Limit** | Physical limit switch |

### 13.7 Additional Resources

#### Documentation
- **SPEC.md**: Complete technical specification
- **README.md**: Quick start and overview
- **CHANGELOG.md**: Version history and updates
- **AGENTS.md**: Development guidelines

#### Online Resources
- **GitHub**: [github.com/thawkins/gcodekit4](https://github.com/thawkins/gcodekit4)
- **Issue Tracker**: Report bugs and request features
- **Discussions**: Community support and questions

#### Support
- **Email**: toby@hawkins.com
- **Documentation**: This manual and project docs/
- **Bug Reports**: GitHub Issues

### 13.8 License

GCodeKit4 is dual-licensed under:
- **MIT License**
- **Apache License 2.0**

You may choose either license for your use.

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-11-06 | Initial comprehensive user manual |

---

**End of User Manual**

*For technical specifications and development information, see SPEC.md and README.md*
