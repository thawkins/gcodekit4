# Project Statistics

## Overview
- **Version:** 0.25.6-alpha
- **Last Updated:** 2025-11-10

## Code Metrics
- **Rust Code:** ~73,800 lines (194 files)
- **Slint UI:** ~14,200 lines (36 files)
- **Test Code:** ~17,200 lines (91 files)
- **Total:** ~105,200 lines
- **Test Coverage:** 712+ comprehensive tests
- **Architecture:** Workspace with 4 crates + main binary

## Recent Changes (v0.25.6)
- **Enhanced Error Reporting**: User-friendly error dialogs
  - Connection failures with port/baud details
  - Disconnect errors with error descriptions
  - GRBL errors during G-code sending
  - Send failures with line numbers
  - "Device not connected" and "No G-code content" warnings
  - Modal dialogs require user acknowledgment
  - Supplements console logging for critical errors
- **Workspace Architecture**: Divided into 4 modular crates
  - `gcodekit4-core`: Core types and state management
  - `gcodekit4-parser`: G-code parsing and CAM tools
  - `gcodekit4-communication`: Serial/TCP protocols
  - `gcodekit4-ui`: Slint UI components
  - Faster incremental builds and better code organization
- **Laser Image Engraver**: Bitmap to G-code conversion
  - Supports PNG, JPG, BMP, GIF, TIFF formats
  - Grayscale power modulation for quality engraving
  - Background processing for large images
  - Real-time preview and time estimation
- **Progress Bar**: Visual indicator in status bar
  - Shows G-code transmission progress
  - 100px bar with percentage display
  - Auto-hides when idle
- **G-Code Editor Controls**: Stop, Pause, Resume buttons
  - Emergency stop during transmission
  - GRBL feed hold/cycle start support
- **Project Cleanup**: Removed 95% of duplicate files

## Previous Changes (v0.25.5)
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

## Recent Changes (v0.25.6-alpha)

### New Features
- Jigsaw Puzzle Maker with Draradech algorithm
- Enhanced Tabbed Box Maker with feed rate control
- BrokenPipe error handling improvements
- Feed rate on all G1 commands for GRBL compatibility

### Code Quality
- All tests passing (709 total)
- Comprehensive error handling
- Debug logging improvements
- Clean separation of concerns

## Component Sizes

  530 src/processing/jigsaw_puzzle.rs
  555 src/processing/tabbed_box.rs
  725 src/communication/mod.rs

## Build Information
- **Rust Edition**: 2021
- **Minimum Rust Version**: 1.70
- **Build Time (Release)**: ~2-3 minutes
- **Binary Size (Release)**: ~15-20 MB

## Testing
- Unit tests: 709+ tests
- Integration tests: Communication, UI, Processing
- All tests passing
