# GCodeKit4 User Manual

**Version:** 0.51.0-alpha  
**Last Updated:** November 28, 2025  
**Application:** GCodeKit4 CNC Controller & G-Code Sender

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Getting Started](#2-getting-started)
3. [User Interface Overview](#3-user-interface-overview)
4. [Machine Control](#4-machine-control)
5. [G-Code Editor](#5-g-code-editor)
6. [2D CAD/CAM Designer](#6-2d-cadcam-designer)
7. [G-Code Visualizer](#7-g-code-visualizer)
8. [CAM Tools](#8-cam-tools)
9. [Device Console](#9-device-console)
10. [Configuration & Settings](#10-configuration--settings)
11. [Advanced Features](#11-advanced-features)
12. [Troubleshooting](#12-troubleshooting)
13. [Keyboard Shortcuts](#13-keyboard-shortcuts)
14. [Supported Controllers](#14-supported-controllers)
15. [Appendix](#15-appendix)

---

## 1. Introduction

### 1.1 What is GCodeKit4?

GCodeKit4 is a modern, cross-platform CNC machine controller application written in Rust with a Slint UI framework. It provides a comprehensive solution for controlling CNC machines, editing G-Code, designing toolpaths, and monitoring machine operations in real-time.

### 1.2 Key Features

- **Multi-Controller Support**: Works with GRBL, (grblHAL, TinyG, g2core, Smoothieware, DSP, and FluidNC) are under development
- **Real-Time Control**: Live position tracking, status monitoring, and machine state display
- **6-Axis Support**: Control X, Y, Z linear axes plus A, B, C rotary axes if device supports them
- **G-Code Editor**: large file support (1M line Gcode) line numbers, and file operations
- **2D CAD/CAM Designer**: Create vector drawings and generate G-Code toolpaths, supports pocketing and profile cuts
- **Advanced CAM Tools**: Laser engraving, box generation, surfacing, and more
- **Visualizer**: 2D preview with intensity heatmaps for laser operations
- **Smart Console**: Filtered command history
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

#### Downloading Release Binaries (Recommended)

Pre-compiled binaries are available for Linux, Windows, and macOS.

1.  Go to the [GCodeKit4 Releases Page](https://github.com/thawkins/gcodekit4/releases).
2.  Find the latest release (e.g., `v0.51.0-alpha`).
3.  Download the archive for your operating system:
    *   **Linux**: `gcodekit4-linux-x86_64.flatpak`
    *   **Windows**: `gcodekit4-windows-x86_64.msi`
    *   **macOS**: `gcodekit4-macos-x86_64.dmg`
4.  Extract the archive to a folder of your choice.
5.  Run the `gcodekit4` executable (or `gcodekit4.exe` on Windows).

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

### 2.2 First Launch

When you first launch GCodeKit4, you'll see:

1. **Main Window**: Tabbed interface with Machine Control, G-Code Editor, Designer, and other panels
2. **Menu Bar**: File, Machine, Tools, and Help menus
3. **Connection Status**: Disconnected indicator in the status bar
4. **Empty ConsolGcode-editor**: Ready to enter gcode to send to your device.

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

#### Step 3: Connect

1. Click the **"Connect"** button
2. Wait for "Device connected" message in the console
3. The status indicator will turn green
4. Device information tab will populate (firmware version, build info)

### 2.4 Initial Machine Setup

#### Homing the Machine

**Important**: Always home your machine after connecting to establish a known position reference.

1. Ensure the machine has clear travel to home position
2. Click the **Home** button (⌂ icon) in the Machine Control panel
3. The machine will execute the homing cycle ($H command)
4. Wait for the cycle to complete
5. The DRO (Digital Readout) will display the home position

---

## 3. User Interface Overview

### 3.1 Main Window Layout

The GCodeKit4 interface features a modern dark theme for reduced eye strain and better visibility of toolpaths.

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
- **New**: Create new design or clear editor
- **Open**: Load G-Code or Design file
- **Save/Save As**: Save current work
- **Export**: Export G-Code (Designer)
- **Load/Add**: Import DXF/SVG files
- **Exit**: Close application

#### Machine Menu
- **Connect/Disconnect**: Toggle connection
- **Home**: Execute homing cycle ($H)
- **Unlock**: Clear alarm state ($X)
- **Reset**: Soft reset controller
- **Emergency Stop**: Immediate halt

#### View Menu
- **Zoom In/Out/Fit**: Canvas controls
- **Reset View**: Restore default view
- **Toggle Panels**: Show/hide specific tabs

### 3.3 Tabs Overview

1. **G-Code Editor**: Edit and preview G-Code files
2. **Machine Control**: Connection, jogging, position display, status
3. **Device Console**: Command history and communication log
4. **Device Info**: Firmware version, build info, capabilities
5. **Device Manager**: Manage multiple machine profiles
6. **Device Config**: Edit controller settings ($0-$130)
7. **Visualizer**: 2D/3D preview of toolpaths with intensity mapping
8. **Designer**: 2D CAD/CAM vector drawing and toolpath generation
9. **CAM Tools**: Specialized generators (Box, Puzzle, Surfacing, etc.)
10. **Materials**: Material database management
11. **CNC Tools**: Tool library management

---

## 4. Machine Control

### 4.1 Digital Readout (DRO)

The DRO displays real-time position information with 0.001mm precision:

#### Machine Position (MPos)
- **MachX, MachY, MachZ**: Position in machine coordinate system
- **MachA, MachB, MachC**: Rotary axes (if equipped)

#### Work Position (WPos)
- **WorkX, WorkY, WorkZ**: Position in work coordinate system
- **WorkA, WorkB, WorkC**: Rotary axes in work coordinates

#### Status Indicators
- **State**: Large colored text indicating machine state (Idle, Run, Hold, Alarm)
- **Feed Rate**: Current feed rate
- **Spindle Speed**: Current RPM

### 4.2 Jogging Controls

Jogging allows manual movement of machine axes.

#### Step Sizes
Select from predefined increments: 0.1mm, 1mm, 10mm, 100mm.

#### Jogging Buttons
- **X+/X-**: Move right/left
- **Y+/Y-**: Move forward/backward
- **Z+/Z-**: Move up/down
- **A/B**: Rotary axis control

### 4.3 Machine Operations

- **Home ($H)**: Establishes machine zero.
- **Unlock ($X)**: Clears alarm state.
- **Reset**: Soft resets the controller.
- **Zero Axis**: Sets current work coordinate to 0 for specific axis.

### 4.4 Overrides

Real-time adjustments to machine operation:
- **Feed Rate**: 10% to 200%
- **Rapid Rate**: 25%, 50%, 100%
- **Spindle Speed**: 10% to 200%

---

## 5. G-Code Editor

### 5.1 Features

- **Syntax Highlighting**: Color-coded G-Code (Blue), M-Code (Purple), Coordinates (Green), Comments (Gray).
- **Line Numbers**: Track execution progress.
- **Editing**: Cut, Copy, Paste, Undo, Redo, Find/Replace.
- **Cursor Tracking**: Real-time line/column display.
- **Blinking Cursor**: Visual feedback for insertion point.

### 5.2 File Operations

- **Open**: Load .gcode, .nc, .ngc, .tap files.
- **Save**: Save ck execution progress.
- **Editing**: Cut, Copy, Paste, Undo, Redo, Find/Replace.
- **Cursor Tracking**: Real-time line/column display.
- **Blinking Cursor**: Visual feedback fchanges to disk.
- **Send to Device**: Stream current file to connected controller.

### 5.3 Execution Control

- **Pause**: Feed hold.
- **Resume**: Cycle start.
- **Stop**: Cancel execution.

---

## 6. 2D CAD/CAM Designer

### 6.1 Overview

The Designer tab provides vector drawing tools and toolpath generation for creating G-Code programs directly within GCodeKit4.

### 6.2 Drawing Tools

- **Rectangle**: Draw rectangles (supports rounded corners and slot mode).
- **Circle**: Draw circles by center and radius.
- **Ellipse**: Draw ellipses.
- **Line**: Draw simple lines.
- **Polyline/Path**: Draw complex multi-segment paths.
- **Text**: Add text shapes (Fira Mono font).

### 6.3 Editing Tools

- **Select**: Click to select, Shift+Click for multiple, Drag for rubber band selection.
- **Move**: Drag shapes to reposition.
- **Resize**: Use handles to scale shapes or groups.
- **Group/Ungroup**: Combine shapes into a single unit.
- **Copy/Paste**: Duplicate shapes.
- **Undo/Redo**: Full history stack for all operations.
- **Align**: Align shapes horizontally or vertically.

### 6.4 Shape Properties

Right-click or use the properties panel to edit:
- **Geometry**: X, Y, Width, Height, Radius.
- **CAM Settings**:
  - **Pocket Strategy**: Raster (Zig-Zag), Contour (Offset), Adaptive.
  - **Cut Depth**: Total depth.
  - **Step Down**: Depth per pass.
  - **Step In**: Horizontal stepover.
  - **Raster Angle**: Angle for raster pocketing.
  - **Bidirectional**: Cut in both directions.

### 6.5 Toolpath Generation

1. Select shapes.
2. Configure tool parameters (Diameter, Feed Rate, Spindle Speed).
3. Click **Generate Toolpath**.
4. G-Code is generated and loaded into the Editor.

---

## 7. G-Code Visualizer

### 7.1 Overview

The Visualizer provides a 2D/3D preview of the G-Code toolpath, allowing you to verify the program before cutting.

### 7.2 Visualization Modes

- **Standard**: Shows toolpaths colored by operation type (G1=Yellow, G2=Green, G3=Red, G0=Dashed).
- **Intensity (Heatmap)**: Visualizes laser power/spindle speed ('S' value).
  - **Show Intensity**: Toggle heatmap mode.
  - **Max S**: Set the maximum S value (e.g., 1000) to scale the heatmap.
  - **Opacity**: Higher power is rendered darker/more opaque.
  - **White Background**: Automatically enabled in intensity mode for better contrast.

### 7.3 Controls

- **Zoom/Pan**: Mouse wheel to zoom, drag to pan.
- **Fit to View**: Center the toolpath in the window.
- **Show/Hide**: Toggle Grid, Rapid Moves, Cutting Moves.

---

## 8. CAM Tools

GCodeKit4 includes specialized generators for common tasks:

### 8.1 Tabbed Box Generator
Create laser-cut boxes with finger joints.
- **Parameters**: Dimensions, material thickness, kerf compensation.
- **Features**: Dividers (X/Y), Dogbone fillets, Open/Closed box types.

### 8.2 Jigsaw Puzzle Generator
Create parametric jigsaw puzzle patterns.
- **Parameters**: Rows, Columns, Tab size, Jitter.

### 8.3 Spoilboard Surfacing
Generate G-Code to flatten your CNC wasteboard.
- **Parameters**: Bed dimensions, Tool diameter, Stepover, Feed rate.

### 8.4 Speeds and Feeds Calculator
Calculate optimal cutting parameters.
- **Inputs**: Material, Tool type, Machine limits.
- **Outputs**: RPM, Feed Rate, Surface Speed, Chip Load.

### 8.5 Laser Engraver (Image)
Convert bitmap images to G-Code for laser engraving.
- **Algorithms**:
  - **Threshold**: Black/White based on cutoff.
  - **Bayer 4x4**: Ordered dithering.
  - **Floyd-Steinberg**: Error diffusion (high quality).
  - **Atkinson**: Error diffusion (high contrast).
- **Settings**: DPI, Size, Invert, Contrast/Brightness.

### 8.6 Vector Engraver
Convert SVG/DXF files to G-Code for plotting or laser cutting.
- **Features**: Hatching (fill), Multi-pass, Scaling.

---

## 9. Device Console

The console displays raw communication with the controller.

- **Color Coded**: Blue (Send), Green (Success), Red (Error), Gray (Debug).
- **Filtering**: Automatically hides status poll spam.
- **History**: Scroll back to see previous commands.
- **Input**: Manually send G-Code commands.

---

## 10. Configuration & Settings

### 10.1 Application Settings
- **Connection**: Default port, baud rate, timeout.
- **UI**: Theme (Dark/Light), Font size.
- **Machine**: Default units, jog increments.

### 10.2 Controller Settings ($)
View and edit GRBL firmware settings ($0-$130).
- **Export**: Backup settings to JSON.
- **Import**: Restore settings from JSON.

---

## 11. Advanced Features

### 11.1 Probing
Support for G38.2 probing cycles for Z-zero and edge finding.

### 11.2 Tool Changes
Support for manual tool changes (M6). The machine pauses and waits for user confirmation.

### 11.3 Keyboard Jogging
Enable in settings to control the machine with arrow keys and PageUp/PageDown.

---

## 12. Troubleshooting

### 12.1 Connection Issues
- Check USB cable and power.
- Verify correct port and baud rate (usually 115200).
- Close other applications using the serial port.

### 12.2 ALARM State
- Machine is locked due to error or limit switch.
- Click **Unlock ($X)** to clear.
- Home the machine if position is lost.

### 12.3 G-Code Errors
- Check console for specific error messages.
- Verify G-Code syntax and supported commands.

---

## 13. Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| **Ctrl+O** | Open File |
| **Ctrl+S** | Save File |
| **Ctrl+Q** | Exit |
| **Ctrl+H** | Home Machine |
| **Ctrl+R** | Reset Controller |
| **Ctrl+Z** | Undo |
| **Ctrl+Y** | Redo |
| **Ctrl+A** | Select All |
| **Space** | Pause/Resume |
| **Esc** | Stop/Cancel |
| **F11** | Toggle Fullscreen |

---

## 14. Supported Controllers

- **GRBL** (v0.9, v1.1)
- **grblHAL**
- **TinyG**
- **g2core**
- **Smoothieware**
- **FluidNC**

---

## 15. Appendix

### 15.1 Configuration Locations
- **Linux**: `~/.config/gcodekit4/config.json`
- **macOS**: `~/Library/Application Support/gcodekit4/config.json`
- **Windows**: `%APPDATA%\gcodekit4\config.json`

### 15.2 License
GCodeKit4 is dual-licensed under MIT and Apache 2.0.

---

**End of User Manual**
