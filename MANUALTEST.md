# GCodeKit4 Manual Test Plan

**Version**: 0.8.2-alpha  
**Last Updated**: 2025-10-21  
**Document Purpose**: Comprehensive manual testing procedures for GCodeKit4 CNC sender application

---

## Table of Contents

1. [Introduction](#introduction)
2. [Test Scope & Coverage](#test-scope--coverage)
3. [Pre-Test Setup](#pre-test-setup)
4. [Functional Test Cases](#functional-test-cases)
5. [UI/UX Test Cases](#uiux-test-cases)
6. [Integration Test Cases](#integration-test-cases)
7. [Performance Test Cases](#performance-test-cases)
8. [Error Handling Test Cases](#error-handling-test-cases)
9. [Security Test Cases](#security-test-cases)
10. [Cross-Platform Test Cases](#cross-platform-test-cases)
11. [Test Reporting](#test-reporting)

---

## Introduction

This document provides comprehensive manual testing procedures for GCodeKit4, a Rust-based G-Code sender for CNC machines. The test plan covers functional features, UI/UX, integration, performance, error handling, and cross-platform compatibility.

### Test Execution Guidelines

- **Test Environment**: Linux, Windows, macOS (as applicable)
- **Hardware**: Standard development machine (2GB+ RAM, USB serial port)
- **Build Type**: Debug and Release builds
- **Duration**: Estimated 8-16 hours for complete test cycle
- **Frequency**: After major feature completion, before each release

### Terminology

- **PASS**: Test executed successfully, expected behavior observed
- **FAIL**: Test did not execute as expected
- **SKIP**: Test not applicable to current configuration
- **BLOCK**: Prerequisite not met, cannot execute

---

## Test Scope & Coverage

### In Scope

✓ Connection management (Serial, TCP, WebSocket)  
✓ File operations (Open, validate, process)  
✓ G-Code parsing and preprocessing  
✓ Streaming and execution control  
✓ Machine control (Jog, Home, Reset)  
✓ Real-time overrides (Feed, Rapid, Spindle)  
✓ 3D visualization  
✓ UI/UX functionality  
✓ Keyboard shortcuts  
✓ Error handling and recovery  
✓ Settings and configuration  
✓ Cross-platform compatibility  

### Out of Scope

✗ Hardware-specific CNC machine behavior  
✗ Firmware bugs in GRBL/TinyG/g2core  
✗ Performance profiling beyond target thresholds  
✗ Plugin system (post-MVP feature)  
✗ Remote API (planned feature)

---

## Pre-Test Setup

### System Requirements

- **OS**: Linux (Ubuntu 20.04+), Windows 10+, macOS 10.13+
- **RAM**: 2GB minimum (4GB recommended)
- **Storage**: 500MB free space
- **Display**: 1024x768 minimum resolution

### Build Preparation

```bash
cd /home/thawkins/Projects/gcodekit4
cargo build                 # Debug build
cargo build --release       # Release build
cargo fmt                   # Format code
cargo clippy               # Run linter
```

### Test File Preparation

Create test G-Code files in `target/temp/` directory:

**simple_square.gcode**:
```gcode
G21 G90
G0 X0 Y0 Z0
G0 Z5
G0 X100 Y0
G0 X100 Y100
G0 X0 Y100
G0 X0 Y0
G0 Z0
M2
```

**arc_pattern.gcode**:
```gcode
G21 G90
G0 X0 Y0 Z0
G0 Z5
G1 X50 Y0 F100
G2 X100 Y50 I50 J0 F100
G2 X50 Y100 I0 J50 F100
G2 X0 Y50 I-50 J0 F100
G2 X50 Y0 I0 J-50 F100
M2
```

### Serial Device Setup

If hardware available:
- Connect USB serial adapter or CNC controller
- Note port: `/dev/ttyUSB0` (Linux), `COM3` (Windows)
- Typical baud rate: 115200 for GRBL

---

## Functional Test Cases

### TC-F001: Application Launch

**Objective**: Verify application starts correctly

**Steps**:
1. Execute `./target/debug/gcodekit4`
2. Wait for UI window to appear (max 5 seconds)
3. Observe all panels visible

**Expected Results**:
- ✓ Application window opens without errors
- ✓ Connection panel visible (left)
- ✓ Main editor area (center)
- ✓ DRO panel (right)
- ✓ Console (bottom)
- ✓ All controls responsive

**Pass Criteria**: All panels display correctly, no console errors

---

### TC-F002: Serial Port Detection

**Objective**: Verify automatic port enumeration

**Steps**:
1. Launch application
2. Check Connection Panel dropdown
3. Observe available ports
4. If hardware: Connect USB device, check if added

**Expected Results**:
- ✓ Available serial ports listed
- ✓ List updates dynamically
- ✓ Baud rates selectable (9600-115200)
- ✓ Default sensible (115200 for GRBL)

**Pass Criteria**: Port detection works and updates

---

### TC-F003: Serial Connection

**Objective**: Verify successful connection establishment

**Prerequisites**: Hardware connected and powered

**Steps**:
1. Select port from dropdown
2. Select baud rate (115200)
3. Click "Connect" button
4. Wait 3-5 seconds

**Expected Results**:
- ✓ Connect button becomes disabled
- ✓ Status indicator changes to green
- ✓ Console shows connection messages
- ✓ DRO displays current position
- ✓ Jog controls become enabled

**Pass Criteria**: Connection established, status green

---

### TC-F005: File Operations - Open

**Objective**: Verify file opening and loading

**Steps**:
1. Click "File" menu → "Open"
2. Navigate to test file directory
3. Select "simple_square.gcode"
4. Wait for parsing

**Expected Results**:
- ✓ File dialog opens
- ✓ File name appears in title bar
- ✓ G-Code content displayed in editor
- ✓ Line count shown
- ✓ Statistics displayed (lines, time, bounds)
- ✓ No parse errors

**Pass Criteria**: File loads without errors

---

### TC-F010: Start Streaming

**Objective**: Verify command streaming begins

**Prerequisites**: Connected, file loaded

**Steps**:
1. Load simple_square.gcode
2. Click "Start" button
3. Observe execution for 3-5 seconds
4. Monitor console

**Expected Results**:
- ✓ Start button changes to Pause
- ✓ Commands sent to controller
- ✓ Line counter incrementing
- ✓ Time elapsed running
- ✓ Buffer status updating

**Pass Criteria**: Streaming starts, commands sent

---

### TC-F011: Pause/Resume

**Objective**: Verify pause and resume

**Prerequisites**: Streaming in progress

**Steps**:
1. Click "Pause" during execution
2. Wait 2 seconds
3. Click "Resume"
4. Observe continuation

**Expected Results**:
- ✓ Execution pauses immediately
- ✓ Pause button becomes Resume
- ✓ Line counter stops
- ✓ Resume continues from pause point
- ✓ No commands lost

**Pass Criteria**: Pause/resume works cleanly

---

### TC-F012: Stop/Cancel

**Objective**: Verify stop command

**Prerequisites**: Streaming in progress

**Steps**:
1. Click "Stop" during execution
2. Wait for execution to halt
3. Observe final state

**Expected Results**:
- ✓ Execution stops immediately
- ✓ Command queue cleared
- ✓ State returns to IDLE
- ✓ Start button re-enabled

**Pass Criteria**: Stop works cleanly

---

### TC-F013: Jog Control

**Objective**: Verify jogging with buttons and keyboard

**Prerequisites**: Connected

**Steps**:
1. Set increment to 1mm
2. Set jog feed rate to 50mm/min
3. Click [+X] button
4. Observe movement
5. Repeat for Y, Z axes
6. Try arrow keys

**Expected Results**:
- ✓ Buttons send jog commands
- ✓ Machine moves in correct direction
- ✓ Keyboard shortcuts work (arrows, WASD)
- ✓ Movement distance equals increment
- ✓ DRO updates

**Pass Criteria**: Jogging works

---

### TC-F015: Home All Axes

**Objective**: Verify homing operation

**Prerequisites**: Hardware supports homing

**Steps**:
1. Click "Home" button in Control Panel
2. Observe machine movement
3. Wait for completion

**Expected Results**:
- ✓ Machine moves to home position
- ✓ DRO shows ~0.0 (or configured offset)
- ✓ State changes to HOME then IDLE
- ✓ Console shows progress
- ✓ Controls remain responsive

**Pass Criteria**: Machine homes successfully

---

### TC-F016: Soft Reset

**Objective**: Verify soft reset

**Prerequisites**: Connected

**Steps**:
1. Send any command (jog, stream)
2. Click "Reset" button
3. Observe state change

**Expected Results**:
- ✓ Reset command sent
- ✓ Machine stops
- ✓ State resets to IDLE
- ✓ Buffer cleared
- ✓ New operations possible

**Pass Criteria**: Reset works cleanly

---

### TC-F017: Feed Rate Override

**Objective**: Verify feed rate override

**Prerequisites**: Streaming

**Steps**:
1. Start streaming
2. Click [-10%] button
3. Click [+10%] button
4. Drag slider to 50%
5. Drag slider to 150%

**Expected Results**:
- ✓ Speed changes in real-time
- ✓ Buttons adjust by 10%
- ✓ Slider provides continuous adjustment
- ✓ Percentage displayed accurately
- ✓ Changes in DRO feed rate

**Pass Criteria**: Feed override works real-time

---

### TC-F019: DRO Position Display

**Objective**: Verify Digital Readout accuracy

**Prerequisites**: Connected

**Steps**:
1. Check Machine Coordinates display
2. Check Work Coordinates display
3. Jog to known position
4. Use "X0", "Y0", "Z0" buttons

**Expected Results**:
- ✓ Machine coordinates update in real-time
- ✓ Work coordinates reflect offset
- ✓ Accuracy 0.01mm
- ✓ Zero buttons set coordinates to 0

**Pass Criteria**: DRO reads correctly

---

### TC-F021: 3D Visualizer

**Objective**: Verify 3D visualization

**Prerequisites**: File loaded

**Steps**:
1. Observe toolpath in visualizer
2. Verify geometry matches G-Code
3. Rotate view with mouse
4. Zoom in/out
5. Pan with right-click

**Expected Results**:
- ✓ Toolpath displayed in 3D
- ✓ Geometry matches file
- ✓ View rotatable
- ✓ View zoomable
- ✓ View pannable
- ✓ Colors: red=rapid, green=feed

**Pass Criteria**: Visualization works

---

### TC-F023: Console Output

**Objective**: Verify console functionality

**Prerequisites**: Streaming or connected

**Steps**:
1. Start streaming
2. Observe messages in console
3. Scroll through console
4. Verify message types

**Expected Results**:
- ✓ Sent commands display [SEND]
- ✓ Responses display [RX]
- ✓ Color-coded messages
- ✓ Console scrolls to latest
- ✓ Clear button works

**Pass Criteria**: Console captures activity

---

### TC-F025: Settings Persistence

**Objective**: Verify settings save/restore

**Prerequisites**: Application launched

**Steps**:
1. Open Settings dialog
2. Change connection settings
3. Close application
4. Restart application
5. Verify settings preserved

**Expected Results**:
- ✓ Settings dialog accessible
- ✓ Options changeable
- ✓ Settings persist across restart
- ✓ Sensible defaults
- ✓ Config file readable

**Pass Criteria**: Settings save/restore correctly

---

## UI/UX Test Cases

### TC-U001: Window Layout

**Objective**: Verify layout and component visibility

**Steps**:
1. Launch maximized
2. Check all components present
3. Resize to 1024x768 minimum
4. Verify all accessible
5. Resize to large screen

**Expected Results**:
- ✓ All components visible at min size
- ✓ No overlapping elements
- ✓ Scales appropriately
- ✓ Controls functional at all sizes

**Pass Criteria**: Layout responsive

---

### TC-U002: Menu Bar

**Objective**: Verify all menus functional

**Steps**:
1. Click File, Edit, View, Machine, Tools, Help
2. Check submenus
3. Verify disabled items grayed
4. Check keyboard shortcuts

**Expected Results**:
- ✓ All menus present
- ✓ Submenus expand
- ✓ Disabled items grayed appropriately
- ✓ Shortcuts displayed

**Pass Criteria**: Menus accessible

---

### TC-U003: Keyboard Shortcuts

**Objective**: Verify shortcuts work

**Steps**:
1. Test: Ctrl+O (Open), Ctrl+H (Home), Ctrl+R (Reset)
2. Test: Arrow keys, W/A/S/D (Jog)
3. Test: Space (Pause), Esc (Stop)

**Expected Results**:
- ✓ All shortcuts functional
- ✓ Work in different controls
- ✓ No conflicts
- ✓ Clear feedback

**Pass Criteria**: Shortcuts work

---

### TC-U008: Color Coding

**Objective**: Verify visual indicators

**Steps**:
1. Check connection status: Red/Yellow/Green
2. Check state colors
3. Check override colors
4. Check console message colors
5. Check toolpath colors

**Expected Results**:
- ✓ Colors consistent
- ✓ Meanings obvious
- ✓ Colorblind-friendly
- ✓ Sufficient contrast

**Pass Criteria**: Color coding clear

---

## Integration Test Cases

### TC-I001: File to Visualization

**Objective**: Verify file load → parse → visualize

**Steps**:
1. Open complex_toolpath.gcode
2. Wait for parsing
3. Observe 3D visualization
4. Open different file
5. Check visualization updates

**Expected Results**:
- ✓ Loads <2s for 1MB
- ✓ Parsing with progress
- ✓ Statistics calculated
- ✓ Visualization correct
- ✓ Switching files updates viz

**Pass Criteria**: Pipeline functional

---

### TC-I002: Connection to Streaming

**Objective**: Verify connection → file → stream

**Prerequisites**: Hardware available

**Steps**:
1. Connect to device
2. Load file
3. Verify preview
4. Click Start
5. Monitor execution

**Expected Results**:
- ✓ Smooth transitions
- ✓ No lost commands
- ✓ Machine moves correctly
- ✓ Positions update real-time
- ✓ Completion detected

**Pass Criteria**: Pipeline seamless

---

## Performance Test Cases

### TC-P001: File Loading Speed

**Objective**: Verify load time targets

**Steps**:
1. Time: simple_square.gcode (small)
2. Time: complex_toolpath.gcode (1000+ lines)
3. Time: large file if available (100K+ lines)
4. Check memory

**Expected Results**:
- ✓ Small (<100KB): <1s
- ✓ Medium (100KB-1MB): <2s
- ✓ Large (1MB+): <5s
- ✓ Memory <150MB for 100K lines

**Pass Criteria**: Load times within targets

---

### TC-P002: Parsing Speed

**Objective**: Verify parsing performance

**Prerequisites**: Large file (10K+ lines)

**Steps**:
1. Load large file
2. Note parsing time
3. Calculate lines/second

**Expected Results**:
- ✓ Rate >10,000 lines/second
- ✓ No UI lag
- ✓ Progress shown
- ✓ Memory released

**Pass Criteria**: Parsing fast

---

### TC-P005: Memory Usage

**Objective**: Verify memory limits

**Steps**:
1. Launch application
2. Note idle memory
3. Load large file
4. Note loaded memory
5. Monitor during streaming

**Expected Results**:
- ✓ Idle: <50MB
- ✓ 100K file: <150MB
- ✓ Streaming: stable, no leaks
- ✓ After stop: memory released

**Pass Criteria**: Memory within targets

---

## Error Handling Test Cases

### TC-E001: Invalid Port

**Objective**: Handle non-existent port gracefully

**Steps**:
1. Select non-existent port
2. Click Connect
3. Observe error handling

**Expected Results**:
- ✓ Fails gracefully (no crash)
- ✓ Error message displayed
- ✓ Application usable
- ✓ Can retry

**Pass Criteria**: Invalid port handled

---

### TC-E003: Disconnection During Stream

**Objective**: Handle unexpected disconnection

**Prerequisites**: Connected and streaming

**Steps**:
1. Start streaming
2. Physically disconnect
3. Observe behavior

**Expected Results**:
- ✓ Detected within 5s
- ✓ Streaming paused
- ✓ Error shown
- ✓ Application stable
- ✓ Can reconnect

**Pass Criteria**: Disconnection handled

---

## Security Test Cases

### TC-S001: File Path Validation

**Objective**: Verify file path security

**Steps**:
1. Try absolute path: `/etc/passwd`
2. Try relative: `../../etc/passwd`
3. Try symbolic link

**Expected Results**:
- ✓ Only allowed dirs accessible
- ✓ Absolute paths rejected
- ✓ Path traversal blocked
- ✓ Error if restricted

**Pass Criteria**: File access secured

---

## Cross-Platform Test Cases

### TC-X001: Linux Build

**Objective**: Verify Linux build/run

**Prerequisites**: Linux (Ubuntu 20.04+)

**Steps**:
1. Build debug: `cargo build`
2. Build release: `cargo build --release`
3. Run debug binary
4. Run release binary
5. Verify features work

**Expected Results**:
- ✓ Build succeeds
- ✓ Binary runs without errors
- ✓ UI renders correctly
- ✓ File dialogs work
- ✓ Serial ports detected

**Pass Criteria**: Linux build successful

---

### TC-X002: Windows Build

**Objective**: Verify Windows build/run

**Prerequisites**: Windows 10+

**Steps**:
1. Build debug: `cargo build`
2. Build release: `cargo build --release`
3. Run executable
4. Verify features work

**Expected Results**:
- ✓ Build succeeds
- ✓ Executable runs
- ✓ UI renders correctly
- ✓ File dialogs work
- ✓ COM ports detected

**Pass Criteria**: Windows build successful

---

### TC-X003: macOS Build

**Objective**: Verify macOS build/run

**Prerequisites**: macOS 10.13+

**Steps**:
1. Build debug: `cargo build`
2. Build release: `cargo build --release`
3. Run executable
4. Verify features work

**Expected Results**:
- ✓ Build succeeds
- ✓ Application runs
- ✓ UI renders correctly
- ✓ File dialogs work
- ✓ Serial ports detected

**Pass Criteria**: macOS build successful

---

## Test Reporting

### Test Execution Checklist

Essential tests before each release:

```
[ ] TC-F001: Application Launch
[ ] TC-F002: Serial Port Detection
[ ] TC-F003: Serial Connection
[ ] TC-F005: File Operations - Open
[ ] TC-F010: Start Streaming
[ ] TC-F011: Pause/Resume
[ ] TC-F012: Stop/Cancel
[ ] TC-F013: Jog Control
[ ] TC-F015: Home All Axes
[ ] TC-F017: Feed Rate Override
[ ] TC-F019: DRO Position Display
[ ] TC-F021: 3D Visualizer
[ ] TC-F023: Console Output
[ ] TC-U001: Window Layout
[ ] TC-U003: Keyboard Shortcuts
[ ] TC-I002: Connection to Streaming
[ ] TC-P001: File Loading Speed
[ ] TC-E001: Invalid Port
[ ] TC-X001: Linux Build
```

### Report Template

```
# Test Execution Report

**Date**: [YYYY-MM-DD]
**Tester**: [Name]
**Build**: [Version]
**Platform**: [OS]
**Duration**: [Hours]

## Summary
- Total: [N]
- Passed: [N] (%)
- Failed: [N] (%)
- Skipped: [N]

## Categories
- Functional: [X/Y]
- UI/UX: [X/Y]
- Integration: [X/Y]
- Performance: [X/Y]
- Error Handling: [X/Y]
- Security: [X/Y]
- Cross-Platform: [X/Y]

## Failed Tests
1. TC-XXXX: [Description]
   - Expected: [What should happen]
   - Actual: [What happened]
   - Severity: [Critical/High/Medium/Low]
```

---

**Document Version**: 1.0  
**Last Modified**: 2025-10-21  
**Status**: Complete
