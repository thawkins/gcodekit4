# Project Statistics

## Overview
- **Version:** 0.25.5-alpha
- **Last Updated:** 2025-11-08

## Code Metrics
- **Rust Code:** ~58,000 lines
- **Slint UI:** ~6,400 lines
- **Total:** ~64,400 lines
- **Test Coverage:** 694 comprehensive tests
- **Test Files:** 91 test modules

## Recent Changes (v0.25.5)
- **G-Code Streaming Fixed**: Complete rewrite of send functionality
  - GRBL Character-Counting Protocol implementation
  - Single-threaded design: polling thread handles all serial I/O
  - Proper buffer management with "ok" acknowledgment tracking
  - Real-time `?` command sent as single byte (doesn't use buffer)
  - Minimal mutex locking (~1-2ms per cycle vs 40-45ms before)
  - Jog commands now respond immediately (was 10-15 second delay)
  - Sends up to 100 lines/second with full buffer respect
- **CNC Tools Manager**: Full CRUD interface with GTC import capability
  - Import tool catalogs from suppliers (ZIP and JSON formats)
  - Search, filter, create, edit, and delete tools
  - Persistent storage with auto-save
  - Standard library with 5 common tools
- **Materials Database Manager**: Complete material properties management
  - CRUD operations for custom materials
  - CNC parameters (feeds, speeds, DOC)
  - Category-based organization
  - Persistent storage with auto-save
- **Firmware Detection**: Reliable GRBL version detection using $I command
- **Device Info Panel**: Displays actual detected firmware version and capabilities
- **Console Improvements**: Command input, cleaned output (no status spam)
- **Work Coordinate System**: Zero X/Y/Z buttons send G92 commands
- **WCS Switching**: G54-G59 buttons switch work coordinate systems
- **UI Cleanup**: Removed LaserTools panel, improved scrolling
- **Code Cleanup**: Removed ~1,600 lines of unused GTools code

## Module Breakdown
- **visualizer:** 2297 lines
- **utils:** 6006 lines
- **ui/ui_components:** 840 lines
- **ui_panels:** 3261 lines
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
