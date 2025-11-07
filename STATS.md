# Project Statistics

## Overview
- **Version:** 0.25.5-alpha
- **Last Updated:** 2025-11-07

## Code Metrics
- **Rust Code:** ~56,400 lines
- **Slint UI:** ~5,210 lines
- **Total:** ~61,610 lines
- **Test Coverage:** 694 comprehensive tests
- **Test Files:** 91 test modules

## Recent Changes (v0.25.5)
- **Firmware Detection**: Reliable GRBL version detection using $I command
- **Device Info Panel**: Now displays actual detected firmware version and capabilities
- **Laser Mode Support**: Added to GRBL 1.1, 1.2, and 1.3 capability profiles
- **Console Improvements**: Command input, cleaned output (no status spam)
- **Tool Panels**: New Laser Tools and CNC Tools panels with scrollable card grids
- **Work Coordinate System**: Zero X/Y/Z buttons send G92 commands
- **WCS Switching**: G54-G59 buttons switch work coordinate systems
- **Code Cleanup**: Removed ~1,600 lines of unused GTools code

## Module Breakdown
- **visualizer:** 2297 lines
- **utils:** 6006 lines
- **ui/ui_components:** 840 lines
- **ui_panels:** 3261 lines (includes laser_tools.slint, cnc_tools.slint)
- **ui:** 12984 lines
- **testing:** 501 lines
- **processing:** 1801 lines
- **gcode:** 2134 lines
- **firmware/tinyg:** 1510 lines
- **firmware/smoothieware:** 519 lines
- **firmware/grbl:** 3512 lines
- **firmware/g2core:** 1499 lines
- **firmware/fluidnc:** 552 lines
- **firmware:** 3120 lines
- **designer:** 10475 lines
- **data:** 1875 lines
- **core:** 784 lines
- **communication:** 1502 lines
