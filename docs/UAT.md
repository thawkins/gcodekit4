# User Acceptance Tests (UAT) for GCodeKit4

**Date**: 2025-12-04
**Version**: 0.67.0-alpha.0
**Environment**:
- **Device 1**: 130x130mm Laser Engraver (GRBL 1.1)
- **Device 2**: 300x180mm CNC Router (GRBL 1.1)
- **OS**: Linux / Windows / macOS (Cross-platform)

## Test Protocol
- All tests should be performed on both devices unless specified otherwise.
- Ensure safety glasses are worn when operating the laser or CNC.
- Keep the emergency stop button accessible at all times.

---

## 1. Connection & Device Management

| ID | Test Case | Steps | Expected Result | Pass/Fail |
|----|-----------|-------|-----------------|-----------|
| 1.1 | Port Detection | 1. Connect device via USB.<br>2. Launch GCodeKit4.<br>3. Open "Connection" panel.<br>4. Click "Refresh Ports". | The device's serial port (e.g., `/dev/ttyUSB0` or `COM3`) appears in the dropdown list.<br>If no port was selected, the first available port is automatically selected. | |
| 1.2 | Connection (Laser) | 1. Select Laser port.<br>2. Set Baud Rate to 115200.<br>3. Click "Connect". | Console shows "Grbl 1.1..." welcome message.<br>Status bar changes to "Idle".<br>DRO updates with positions.<br>Console log shows "Connecting..." then "Connected". | |
| 1.3 | Connection (CNC) | 1. Select CNC port.<br>2. Set Baud Rate to 115200.<br>3. Click "Connect". | Console shows "Grbl 1.1..." welcome message.<br>Status bar changes to "Idle".<br>DRO updates with positions.<br>Console log shows "Connecting..." then "Connected". | |
| 1.4 | Disconnect | 1. While connected, click "Disconnect". | Status bar changes to "Disconnected".<br>Port becomes available for other apps. | |
| 1.5 | Auto-Reconnect | 1. Enable "Auto-reconnect" in Settings.<br>2. Unplug USB cable.<br>3. Plug USB cable back in. | Application detects disconnection and automatically re-establishes connection within 5 seconds. | |

## 2. Machine Control (Jogging & Homing)

| ID | Test Case | Steps | Expected Result | Pass/Fail |
|----|-----------|-------|-----------------|-----------|
| 2.1 | Homing Cycle ($H) | 1. Ensure path is clear.<br>2. Click "Home" (⌂) button. | Machine moves to home switches (typically top-right or top-left).<br>DRO resets to Machine Zero (or defined pull-off).<br>Status shows "Idle" after completion. | |
| 2.2 | Continuous Jog X/Y | 1. Select "Continuous" mode (if avail) or hold jog button.<br>2. Press and hold X+ arrow.<br>3. Release arrow. | Machine moves right while button is held.<br>Stops immediately upon release. | |
| 2.3 | Incremental Jog | 1. Select "10mm" step size.<br>2. Click Y+ button once. | Machine moves exactly 10mm in Y+ direction.<br>DRO updates by 10.000. | |
| 2.4 | Jog Z Axis (CNC) | 1. Select "1mm" step size.<br>2. Click Z+ (Up) button.<br>3. Click Z- (Down) button. | Spindle moves up 1mm, then down 1mm.<br>**Safety**: Ensure tool doesn't crash into bed. | |
| 2.5 | Soft Reset | 1. Click "Reset" button. | Controller resets.<br>Alarm lock is cleared (if applicable).<br>Console shows reset message. | |
| 2.6 | Unlock ($X) | 1. Trigger a soft limit or alarm state.<br>2. Click "Unlock" (Lock icon). | Alarm state is cleared.<br>Machine returns to "Idle". | |

## 3. Coordinate Systems & Zeroing

| ID | Test Case | Steps | Expected Result | Pass/Fail |
|----|-----------|-------|-----------------|-----------|
| 3.1 | Zero All Axes | 1. Jog to a random position.<br>2. Click "Zero All Axes" button in Machine Control. | Work Coordinates (WPos) in DRO reset to 0.000 for all axes.<br>Machine Coordinates (MPos) remain unchanged.<br>Console logs "Zeroing all axes...". | |
| 3.2 | WCS Selection | 1. Switch from G54 to G55 using buttons.<br>2. Zero X axis.<br>3. Switch back to G54. | G55 X is 0.000.<br>G54 X retains its previous value (independent offsets).<br>Active WCS button is highlighted. | |
| 3.3 | Return to Zero | 1. Jog away from Work Zero.<br>2. Click "Go to Zero" (or type `G0 X0 Y0`). | Machine rapids back to the Work Zero position. | |
| 3.4 | World Coordinates | 1. Observe "World Coordinates (G53)" section in Machine Control. | Displays Machine Coordinates (MPos) distinct from Work Coordinates (WPos).<br>Updates in real-time during jogging. | |

## 4. G-Code Loading & Visualization

| ID | Test Case | Steps | Expected Result | Pass/Fail |
|----|-----------|-------|-----------------|-----------|
| 4.1 | Load File | 1. Click "Open File".<br>2. Select a valid `.gcode` or `.nc` file. | File loads.<br>G-Code appears in Editor.<br>Line count and dimensions appear in Status/Info panel. | |
| 4.2 | 2D Visualization | 1. Load a file with arcs and lines.<br>2. Open "Visualizer" tab. | Toolpath is rendered correctly.<br>Rapid moves (G0) are distinct from Feed moves (G1).<br>Bounding box matches file dimensions. | |
| 4.3 | 3D Visualization | 1. Switch to 3D view.<br>2. Rotate and Zoom using mouse. | 3D toolpath is visible.<br>Camera orbits around the model smoothly. | |
| 4.4 | Syntax Highlighting | 1. View loaded code in Editor. | G-commands (G0, G1) are colored differently from coordinates (X, Y) and comments. | |

## 5. Job Execution

| ID | Test Case | Steps | Expected Result | Pass/Fail |
|----|-----------|-------|-----------------|-----------|
| 5.1 | Dry Run (Check Mode) | 1. Enable "Check Mode" ($C).<br>2. Click "Start". | G-Code streams to controller.<br>No physical motion occurs.<br>Console reports any errors.<br>Disable Check Mode afterwards. | |
| 5.2 | Start Job | 1. Load a test file (e.g., 20x20mm square).<br>2. Zero WCS.<br>3. Go to Machine Control panel.<br>4. Click "Send". | Machine begins executing moves.<br>Progress bar advances.<br>Console logs commands as "Command => ok".<br>Console auto-scrolls to show latest commands. | |
| 5.3 | Pause/Resume | 1. During job, click "Pause" (!).<br>2. Wait 5 seconds.<br>3. Click "Resume" (~). | Machine stops motion immediately (decelerates).<br>Spindle may stay on (depending on config).<br>Resumes exactly where it left off.<br>Console logs pause/resume actions. | |
| 5.4 | Stop/Abort | 1. During job, click "Stop". | Machine stops immediately.<br>Job is cancelled.<br>Queue is cleared.<br>State returns to Idle/Alarm.<br>Console logs "Transmission stopped". | |
| 5.5 | Job Completion | 1. Allow a short job to finish. | Machine stops at end of file.<br>Progress reaches 100%.<br>Status returns to "Idle". | |

## 6. Overrides (Real-time)

| ID | Test Case | Steps | Expected Result | Pass/Fail |
|----|-----------|-------|-----------------|-----------|
| 6.1 | Feed Rate Override | 1. Start a job with long moves.<br>2. Adjust Feed slider to 50%.<br>3. Adjust to 200%. | Machine visibly slows down to half speed.<br>Machine speeds up to double speed (limited by max rate). | |
| 6.2 | Rapid Override | 1. During G0 moves, set Rapid to 50%. | Rapid travel speed is reduced by half. | |
| 6.3 | Spindle Override | 1. Start job with M3 S1000.<br>2. Adjust Spindle slider to 50%. | Spindle RPM drops (audible change or visible on VFD/Laser intensity). | |

## 7. CAM Tools (Specific)

| ID | Test Case | Steps | Expected Result | Pass/Fail |
|----|-----------|-------|-----------------|-----------|
| 7.1 | Laser Image Engraver | 1. Open "Laser Engraver" tool.<br>2. Load a small JPG image.<br>3. Set Width: 50mm.<br>4. Click "Generate G-Code". | Preview shows grayscale representation.<br>G-Code is generated with M3/M4 S-values varying by pixel density. | |
| 7.2 | Tabbed Box Maker | 1. Open "Tabbed Box" tool.<br>2. Set W:50, D:50, H:30, Thickness:3.<br>3. Click "Generate". | Visualizer shows a flat-pack box layout with finger joints.<br>Generated G-Code includes cutouts for all sides. | |
| 7.3 | Spoilboard Surfacing | 1. Open "Spoilboard" tool.<br>2. Set dimensions (e.g., 280x160 for CNC).<br>3. Generate. | Toolpath shows a zig-zag or spiral pattern covering the specified area. | |
| 7.4 | Vector Engraving | 1. Open "Vector Engraver" tool.<br>2. Load `Tigerhead.svg` (or similar SVG).<br>3. Set Width: 100mm.<br>4. Click "Generate G-Code". | Preview shows vector paths.<br>G-Code is generated with G1 moves following the vector paths. | |
| 7.5 | Jigsaw Puzzle Maker | 1. Open "Jigsaw Puzzle" tool.<br>2. Set Width: 150mm, Height: 100mm.<br>3. Set Pieces: 5 Across, 4 Down.<br>4. Click "Generate". | Visualizer shows puzzle piece pattern.<br>Generated G-Code includes cuts for all puzzle pieces. | |

## 8. Settings & Configuration

| ID | Test Case | Steps | Expected Result | Pass/Fail |
|----|-----------|-------|-----------------|-----------|
| 8.1 | GRBL Settings Read | 1. Open "Config Settings". | Table populates with current values ($0-$132) from the controller. | |
| 8.2 | Modify Setting | 1. Change a non-critical setting (e.g., $10 Status Report Mask).<br>2. Save.<br>3. Restart controller. | New value persists after restart. | |
| 8.3 | Units (Metric/Imperial) | 1. Go to App Settings.<br>2. Change Units to "Inches". | DRO displays values in inches.<br>Grid in Visualizer updates to inch scale. | |

## 9. Stress & Reliability

| ID | Test Case | Steps | Expected Result | Pass/Fail |
|----|-----------|-------|-----------------|-----------|
| 9.1 | Long Job Simulation | 1. Load a large file (>50k lines).<br>2. Run in Check Mode or Air Cut. | UI remains responsive.<br>Memory usage is stable.<br>No buffer underflow errors. | |
| 9.2 | Rapid Jogging | 1. Rapidly click jog buttons in random directions. | Machine responds without locking up.<br>Command queue handles input correctly. | |
| 9.3 | Console Performance | 1. Run a job with verbose output.<br>2. Observe Device Console. | Console updates smoothly.<br>Auto-scroll keeps latest line visible.<br>No repeated "ok" messages or blank lines.<br>Commands are logged as "Command => Result". | |

---

**Tester Name**: ____________________
**Signature**: ____________________
