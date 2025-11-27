# GCodeKit4

A modern, cross-platform G-Code sender and CNC machine controller written in Rust with Slint UI framework.

[![Build Status](https://github.com/thawkins/gcodekit4/workflows/CI/badge.svg)](https://github.com/thawkins/gcodekit4/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-0.45.0--alpha-brightgreen.svg)](CHANGELOG.md)

## Overview

GCodeKit4 is a Rust-based CNC machine controller providing a modern alternative to Universal G-Code Sender. It supports multiple controller firmware types including GRBL, grblHAL, TinyG, g2core, Smoothieware, and FluidNC through a unified, intuitive interface built with the Slint UI framework.

## Architecture

GCodeKit4 is organized as a Cargo workspace with 7 crates for modular compilation and better code organization:

- **gcodekit4-core** - Core types, traits, state management, events, and data models
- **gcodekit4-camtools** - CAM tools and special G-code processing operations
- **gcodekit4-designer** - Visual design tools and toolpath generation
- **gcodekit4-gcodeeditor** - ✨ NEW - G-Code text editor and buffer management
- **gcodekit4-communication** - Serial, TCP, WebSocket protocols and firmware implementations (5 firmware types)
- **gcodekit4-ui** - Slint-based UI components, visualizer, settings, and editor integration
- **gcodekit4-devicedb** - Device profile management
- **gcodekit4-settings** - Application settings management
- **gcodekit4-visualizer** - 2D/3D visualization engine
- **gcodekit4** - Main binary that integrates all crates

This modular structure enables:
- Faster incremental builds (only recompile changed crates)
- Better separation of concerns with focused crate responsibilities
- Easier testing of individual components
- Potential for code reuse across different applications
- Clean architecture suitable for future plugin systems

## Features

### 🎯 Multi-Axis CNC Control
- **6-Axis Support**: Complete control of X, Y, Z linear axes and A, B, C rotary axes
- **Real-time DRO**: Digital readout displays all axis positions with 0.001mm precision
- **Live Status Bar**: Machine state, feed rate, spindle speed, and position display
- **Raw Status View**: Debug view showing undecoded GRBL status responses

### 🕹️ Machine Control
- **Incremental Jogging**: Configurable step sizes (0.1, 1, 10, 100mm for linear, degrees for rotary)
  - Linear axes (X, Y, Z) in millimeters
  - Rotary axes (A, B) in degrees
  - G91 relative positioning for precise incremental moves
- **Work Coordinate System Management**:
  - Zero X/Y/Z buttons send G92 commands to set work origins
  - G54-G59 buttons switch between work coordinate systems
  - Quick access to all 6 standard WCS positions
- **Home Command**: Automated homing cycle ($H)
- **Unlock Function**: Clear ALARM state with single click (🔒 icon, $X command)
- **Emergency Stop**: Immediate halt of all operations
- **Real-time Overrides**: Feed rate, rapid rate, and spindle speed adjustments

### 🔌 Device Management
- **Auto-Detect Serial Ports**: Automatic discovery of USB CNC controllers
- **Real-time Status Polling**: 200ms updates of position and machine state
- **Device Information Tab**: Firmware version, build info, and capabilities
- **Configuration Editor**: Edit and save all GRBL settings ($0-$130)
- **Firmware Capabilities Display**:
  - Arc support (G2/G3 circular interpolation)
  - Variable spindle control (PWM)
  - Homing cycle support
  - Probe functionality
  - Laser mode
  - Multi-axis support (4-6 axes)
  - Safety door feature
  - Coolant control
  - Tool change support

### 📝 G-Code Editor & Streaming
- **Text Editor (Phase 2 - COMPLETE)**:
  - ✅ Full keyboard input support (characters, arrows, backspace, delete)
  - ✅ Text insertion and deletion with proper cursor tracking at correct position
  - ✅ Cursor movement keys (arrows, Home, End, PageUp/PageDown) with instant feedback
  - ✅ Correct cursor position display in status bar (Line X:Y)
  - ✅ Undo/Redo via Ctrl+Z/Ctrl+Y (Cmd on Mac) with cursor position preservation
  - ✅ Tab key for indentation (4 spaces)
  - ✅ Virtual scrolling with line numbers
  - ✅ Arrow key navigation (up, down, left, right) with immediate visual response
  - ✅ Home/End key for jumping to line boundaries
  - ✅ PageUp/PageDown for viewport navigation (10 lines per page)
  - ✅ Real-time text updates as you type
  - ✅ Proper 0-based/1-based indexing conversion (backend/UI boundary)
  - ✅ **NEW: Mouse click to position cursor** - Click anywhere to place cursor at that location
  - ✅ **NEW: Complete focus infrastructure** - Automatic focus cascading through UI hierarchy
- **Syntax Highlighting**: Color-coded commands, coordinates, and comments
- **Line Numbers**: Easy navigation and reference
- **File Operations**: Open, edit, and save G-code files
- **Professional G-Code Streaming**:
  - GRBL Character-Counting Protocol for reliable transmission
  - Automatic buffer management (127-byte GRBL RX buffer)
  - Sends up to 5 lines per cycle with "ok" acknowledgment tracking
  - Real-time progress updates (lines sent/total)
  - **Progress Bar**: Visual progress indicator in status bar
  - **Stop Button**: Terminate transmission immediately
  - **Pause Button**: Feed hold (GRBL ! command)
  - **Resume Button**: Cycle start (GRBL ~ command)
  - Error detection and reporting
  - Comments and empty lines filtered automatically
  - Concurrent status polling via real-time `?` command
- **Real-time Validation**: Syntax checking while editing

### 🎨 2D CAD/CAM Designer
- **Vector Drawing Tools**:
  - Geometric shapes: rectangles, circles, ellipses
  - Lines, polygons, Bezier curves, and arcs
  - Round rectangles with adjustable corner radius
- **File Import**: Import SVG and DXF vector files
- **SVG to G-Code Conversion**:
  - Full support for SVG group transforms (matrix transformations)
  - Handles complex curved paths with multi-segment cubic/quadratic Bezier curves
  - Adaptive curve approximation for smooth engraving output
  - Proper handling of multi-part SVG paths (z/m command sequences)
  - Automatically detects path discontinuities and uses rapid moves for disconnected segments
  - Example: 37-path tiger head design converts to 26,000+ precise movement commands with optimal path breaks
- **Interactive Editing**:
  - Zoom, pan, and fit-to-view controls
  - **NEW**: Dynamic grid and origin indicator
  - **NEW**: View controls (Zoom In/Out, Fit, Reset)
  - Precise positioning (X, Y, Width, Height inputs)
  - Properties dialog for detailed shape adjustments
  - Dual-grid system (10mm major + 1mm minor)
- **SVG Canvas Rendering**: High-quality vector-based canvas
- **Context Menu**: Right-click for Delete, Properties, and the new multi-level Align menu
  - Align horizontally (Left/Center/Right) or vertically (Top/Center/Bottom) across multi-selection groups
  - Selecting "Properties" with multiple shapes opens a "Multiple Shapes" dialog that applies pocket/text/toolpath settings to every selected object while keeping individual positions intact
- **Toolpath Generation**: Convert designs to executable G-code

### 👁️ 2D Visualizer
- **Real-time Rendering**: Instant visualization of G-code toolpaths
- **Adaptive Grid System**:
  - Dynamic grid spacing (e.g., 10mm, 100mm) based on zoom level
  - Infinite grid coverage across the entire viewport
  - Grid size indicator in status bar
- **Dynamic Canvas Sizing**: Automatically adjusts to window resize events
- **Interactive Controls**: Zoom, pan, and fit-to-view with mouse interaction
- **Color-Coded Paths**: Distinct colors for Rapid (G0) and Feed (G1/G2/G3) moves
- **Performance**: Optimized rendering for large files
- **Shared Viewport Engine**: A centralized `ViewportTransform` keeps zoom/pan math consistent across toolpaths, grids, and origin markers.
- **Toolpath Cache**: Parsing + SVG generation flow through a single cache so repeated renders skip redundant work.
- **Unified Path Segments**: A single `PathSegment` enum (with shared `MovementMeta`, streaming visitors, lazy arc iterators, and cached arc geometry) powers both line and arc moves so stats/iteration stay fast and feed rates stay consistent.
- **Analytical Bounds**: Bounding boxes are computed from segment metadata (including arcs), so zoom-to-fit and layout decisions never need to re-discretize toolpaths.

### 💬 Smart Device Console
- **Command History**: Scrollable record of all device communications
- **Color-Coded Messages**:
  - Commands (blue/info)
  - Responses (white/output)
  - Errors (red)
  - Success (green)
  - Verbose/Debug (gray)
- **Intelligent Filtering**: Automatically suppresses status polling spam
  - No more "? " query logging
  - No more `<Idle|MPos:...>` status spam
  - No more "X bytes" messages
- **Optional Timestamps**: Toggle timestamp display
- **Clean Interface**: Shows only meaningful commands and responses

### ⚙️ Configuration Management
- **GRBL Settings Editor**: Complete access to all $0-$130 parameters
- **Inline Editing**: Click any setting value to edit
- **Descriptions**: Tooltips explain each parameter's purpose
- **Value Validation**: Ensures valid ranges and data types
- **Save/Restore**: Persist settings to controller EEPROM
- **Import/Export**: Backup and restore configurations

### 🔧 Tool Management
- **CNC Tools Manager**: Comprehensive tool library management
  - **Full CRUD Operations**: Create, read, update, and delete tools
  - **GTC Import**: Import Generic Tool Catalog packages from suppliers (.zip and .json)
  - **Search & Filter**: Find tools by name or filter by type
  - **Tool Properties**: 
    - Basic info (number, name, type, material, coating)
    - Geometry (diameter, length, flute length, shaft diameter, flutes)
    - Manufacturer details (maker, part number, description)
    - Custom notes
  - **Persistent Storage**: Auto-saves custom tools to disk
  - **Standard Library**: Includes 5 common tools (end mills, drills, v-bits)
  - **Scrollable Interface**: Handle unlimited tools with smooth scrolling
  
- **Materials Database Manager**: Material properties and settings
  - **Full CRUD Operations**: Create, read, update, and delete materials
  - **Material Categories**: Metals, plastics, wood, composites, and more
  - **CNC Parameters**: Feed rates, spindle speeds, plunge rates, depth of cut
  - **Surface Finish Control**: Roughing and finishing pass configurations
  - **Search & Filter**: Find materials by name or filter by category
  - **Persistent Storage**: Auto-saves custom materials to disk
  - **Standard Library**: Includes common materials with tested parameters

### 🔨 CAM Tools
- **Tabbed Box Maker**: Generate laser/CNC cut boxes with finger joints
  - Inside/outside dimension modes
  - Configurable tab width and kerf compensation
  - Multiple box types (full, no top, no front, etc.)
  - Layout styles: diagrammatic, three-piece, inline-compact
  - Dividers support for internal compartments
  - Laser settings: multi-pass, power control, feed rate
  - Based on [TabbedBoxMaker algorithm](https://github.com/paulh-rnd/TabbedBoxMaker)

- **Jigsaw Puzzle Maker**: Generate laser cut jigsaw puzzles
  - **Draradech Algorithm**: Advanced cubic Bézier curves for organic pieces
  - **Configurable Dimensions**: Width × Height in millimeters
  - **Variable Piece Count**: 2-20 pieces in each direction (min 15mm per piece)
  - **Seed-Based Generation**: Reproducible random patterns
  - **Tab Size Control**: 10-30% adjustment for difficulty
  - **Jitter Control**: 0-13% randomness for organic positioning
  - **Rounded Corners**: 0-10mm corner radius for professional finish
  - **Laser Parameters**: Multi-pass support, power control, feed rate
  - **Enhanced Features**: Based on [Draradech's jigsaw generator](https://github.com/Draradech/jigsaw)
  - **Smart Initialization**: Automatic homing and work coordinate setup

- **Laser Image Engraver**: Convert bitmap images to G-code for laser engraving
  - **Image Formats**: PNG, JPG, JPEG, BMP, GIF, TIFF
  - **Grayscale Power Control**: Variable laser power based on image brightness
  - **Bidirectional Scanning**: Optimize engraving time with bidirectional passes
  - **Scan Direction**: Horizontal or vertical raster patterns
  - **Image Preview**: Real-time preview of processed grayscale image
  - **Configurable Parameters**:
    - Output size (width in mm, height auto-calculated)
    - Resolution (pixels per mm)
    - Feed rate and travel rate
    - Laser power range (0-100%)
    - Power scale (GRBL compatibility 0-1000)
    - Line spacing for speed/quality balance
    - Image inversion for negative images
  - **Time Estimation**: Calculate engraving time before generating
  - **Background Processing**: Non-blocking G-code generation
  - **Smart Initialization**: Proper homing and coordinate system setup

## Supported Controllers

| Controller | Versions | Protocol | Features |
|-----------|----------|----------|----------|
| **GRBL** | v0.9, v1.0, v1.1 | Text-based | Character counting, real-time commands, status reports |
| **grblHAL** | Latest | Enhanced GRBL | Extended features, faster execution |
| **TinyG** | v0.97+ | JSON | 6-axis support, macros, tool tables |
| **g2core** | Latest | JSON | Advanced planning, file system, networking |
| **Smoothieware** | Latest | RepRap dialect | Extensive M-codes, network support |
| **FluidNC** | Latest | JSON + WebSocket | WiFi, web interface, SD card |

## Installation

### Prerequisites
- **Rust** 1.70 or later
- **Operating System**: Linux, macOS, or Windows
- **Memory**: 512MB minimum (2GB recommended)
- **Display**: 1024x768 minimum resolution

### Build from Source
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

### Development Build
```bash
# Build debug version (faster compilation, includes debug symbols)
cargo build

# Run with debug logging
RUST_LOG=debug cargo run
```

## Quick Start Guide

### 1. Connect to Your CNC Controller
1. Launch GCodeKit4
2. Click **"Refresh Ports"** to detect available serial devices
3. Select your controller's port from the dropdown
4. Choose baud rate (typically **115200** for GRBL)
5. Click **"Connect"**
6. Wait for "Device connected" message in console

### 2. Initialize Machine
1. Click the **Home** button (⌂) to run homing cycle
2. Machine will move to home position
3. DRO will display current position
4. Status bar shows machine state (Idle/Run/Hold/Alarm)

### 3. Jog Machine
1. Select step size (0.1, 1, 10, or 100mm)
2. Click directional buttons to move:
   - **X+/X-**: Move left/right
   - **Y+/Y-**: Move forward/backward
   - **Z+/Z-**: Move up/down
   - **A+/A-**: Rotate A axis (if equipped)
   - **B+/B-**: Rotate B axis (if equipped)

### 4. Load and Run G-Code
1. Click **File → Open** or drag-and-drop a .nc/.gcode file
2. Review code in editor tab
3. Click **"Send to Device"** to execute
4. Monitor progress in status bar and console

### 5. Configure Settings
1. Navigate to **Config Settings** tab
2. View current GRBL settings
3. Click any value to edit
4. Press Enter to save to controller
5. Changes are immediately applied

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **Ctrl+O** | Open G-code file |
| **Ctrl+S** | Save current file |
| **Ctrl+Q** | Quit application |
| **F11** | Toggle fullscreen |
| **Arrow Keys** | Jog X/Y axes (when enabled) |
| **Page Up/Down** | Jog Z axis (when enabled) |

## Architecture

### Module Structure
```
src/
├── main.rs                    # Application entry point, event handlers
├── ui.slint                   # Main window layout and structure
│
├── communication/             # Device communication layer
│   ├── mod.rs                # Serial, TCP, WebSocket traits
│   ├── serial.rs             # Serial port implementation
│   ├── tcp.rs                # TCP/IP networking
│   └── buffered.rs           # Command buffering and flow control
│
├── firmware/                  # Controller-specific implementations
│   ├── grbl/                 # GRBL protocol support
│   │   ├── communicator.rs   # GRBL command sender
│   │   ├── status_parser.rs  # Real-time status parsing
│   │   └── settings.rs       # Settings management
│   ├── tinyg/                # TinyG JSON protocol
│   ├── g2core/               # g2core advanced features
│   └── smoothieware/         # Smoothieware support
│
├── gcode/                     # G-code parsing and generation
│   ├── parser.rs             # G-code tokenizer and parser
│   ├── generator.rs          # G-code generation from toolpaths
│   └── validator.rs          # Syntax validation
│
├── processing/                # Toolpath processing
│   ├── arc_expansion.rs      # Convert arcs to line segments
│   ├── transforms.rs         # Coordinate transformations
│   └── mesh_leveling.rs      # Auto-leveling compensation
│
├── ui/                        # UI state and logic
│   ├── console_panel.rs      # Console data structures
│   ├── device_console_manager.rs  # Console event handling
│   └── gcode_editor.rs       # Editor state management
│
├── ui_panels/                 # Tab panel components
│   ├── machine_control.slint # Machine control interface
│   ├── gcode_editor.slint    # G-code editor UI
│   ├── designer.slint        # CAD/CAM designer
│   ├── config_settings.slint # Settings editor
│   └── device_info.slint     # Device information display
│
├── utils/                     # Utilities and helpers
│   ├── config.rs             # Configuration file management
│   └── logger.rs             # Logging setup
│
└── visualizer/                # 2D/3D rendering
    ├── renderer.rs           # Graphics pipeline
    └── toolpath.rs           # Toolpath visualization
```

### Technology Stack
- **Rust**: System programming language for memory safety and performance
- **Slint**: Modern declarative UI framework (native cross-platform)
- **Tokio**: Async runtime for non-blocking I/O
- **Serialport-rs**: Cross-platform serial communication
- **Tracing**: Structured logging and diagnostics
- **Serde**: Serialization/deserialization
- **Anyhow**: Error handling with context

## Configuration

Settings files are stored in platform-specific locations:

- **Linux**: `~/.config/gcodekit4/config.json`
- **macOS**: `~/Library/Application Support/gcodekit4/config.json`
- **Windows**: `%APPDATA%\gcodekit4\config.json`

## Development

### Building
```bash
# Debug build (fast compilation, includes debug info)
cargo build

# Release build (optimized, no debug info)
cargo build --release

# Check code without building
cargo check
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_grbl_status_parser

# Run tests with output visible
cargo test -- --nocapture

# Run only library tests (skip integration tests)
cargo test --lib

# Run tests with 10-minute timeout
timeout 600 cargo test
```

### Code Quality
```bash
# Format code (Rust standard style)
cargo fmt

# Check formatting without changing files
cargo fmt --check

# Run Clippy linter
cargo clippy

# Run Clippy with warnings as errors
cargo clippy -- -D warnings
```

### Logging
```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable trace logging (very verbose)
RUST_LOG=trace cargo run

# Enable logging for specific module
RUST_LOG=gcodekit4::communication=debug cargo run
```

## Contributing

Contributions are welcome! Please follow these guidelines:

### Code Standards
- Follow Rust standard naming conventions (snake_case, PascalCase)
- Use 4 spaces for indentation
- Maximum line width: 100 characters
- Add DOCBLOCK comments to all public functions and modules
- Include unit tests for new features
- Run `cargo fmt` and `cargo clippy` before committing

### Process
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes following code standards
4. Add tests for new functionality
5. Update documentation (README, CHANGELOG, inline docs)
6. Commit changes (`git commit -m 'Add amazing feature'`)
7. Push to branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request with clear description

### Areas for Contribution
- 🐛 **Bug Fixes**: Fix existing issues
- ✨ **Features**: Implement new functionality
- 📝 **Documentation**: Improve guides and API docs
- 🧪 **Tests**: Increase test coverage
- 🎨 **UI/UX**: Enhance interface and user experience
- 🌐 **Translations**: Add multi-language support
- 📦 **Packaging**: Create installers for platforms

## Roadmap

### v0.26 (Next Release)
- [ ] TCP/IP and WebSocket communication support
- [ ] Work coordinate system (WCS) management (G54-G59)
- [ ] Tool length offset (TLO) support
- [ ] User-definable macro system
- [ ] Enhanced 3D visualization with toolpath preview

### v0.27
- [ ] Multi-language support (i18n)
- [ ] Custom keyboard shortcuts configuration
- [ ] Theme system (light/dark modes, custom colors)
- [ ] Plugin architecture for extensibility

### v1.0 (Stable Release)
- [ ] 100% test coverage for core modules
- [ ] Complete user documentation
- [ ] Performance optimization and profiling
- [ ] Native installers for Windows, macOS, Linux
- [ ] Production-ready stability

## Known Issues

See [CHANGELOG.md](CHANGELOG.md) for detailed version history and [GitHub Issues](https://github.com/thawkins/gcodekit4/issues) for current bug tracker.

## Documentation

- **[docs/USER.md](docs/USER.md)** - **Comprehensive User Manual** (start here!)
- **[SPEC.md](SPEC.md)** - Complete technical specification
- **[PLAN.md](PLAN.md)** - Implementation roadmap
- **[AGENTS.md](AGENTS.md)** - Development guidelines
- **[CHANGELOG.md](CHANGELOG.md)** - Version history
- **[STATS.md](STATS.md)** - Project statistics
- **[docs/](docs/)** - Additional documentation and guides

## License

This project is dual-licensed under:

- **MIT License** - See [LICENSE-MIT](LICENSE) for details
- **Apache License 2.0** - See [LICENSE-APACHE](LICENSE-APACHE) for details

You may choose either license for your use of this software.

## Acknowledgments

- **Inspiration**: Universal G-Code Sender (UGS) project
- **Frameworks**: Slint UI team for excellent cross-platform toolkit
- **Firmware**: GRBL, TinyG, g2core, and other open-source CNC firmware projects
- **Community**: Rust community for excellent tooling and support

## Support

- **Bug Reports**: [GitHub Issues](https://github.com/thawkins/gcodekit4/issues)
- **Feature Requests**: [GitHub Discussions](https://github.com/thawkins/gcodekit4/discussions)
- **Email**: tim.thawkins@gmail.com
- **Documentation**: See docs/ folder and SPEC.md

## Project Status

**Current Version**: 0.44.0-alpha
**Status**: Active Development
**Stability**: Alpha (breaking changes may occur)

### Recent Updates (v0.44.0)
- **UI Aesthetics Overhaul**:
  - **Complete Redesign**: Visualizer, G-Code Editor, Machine Control, Device Console, Device Info, Device Manager, Device Config, and CAM Tools updated to a unified dark theme.
  - **Improved Layouts**: Left sidebar navigation, grid layouts for tools, and responsive design.
  - **Enhanced Components**: Custom checkboxes, tabs, and tool buttons for better visibility and usability.
  - **CAM Tools**: Restored all 7 tools with dynamic icon generation and improved grid layout.

### Recent Updates (v0.43.0)
- **Visualizer UI Overhaul**:
  - **Dark Theme**: Full adoption of Designer's dark theme for consistent aesthetics.
  - **Modern Layout**: Left sidebar controls, floating status/zoom overlays, and edge-to-edge canvas.
  - **Components**: New icon-based tool buttons and styled checkboxes.

### Recent Updates (v0.40.0)
- **Designer Enhancements**:
  - **Shape Naming**: User-editable names for all shapes, displayed in Layers list.
  - **Rectangle Features**: Rounded corners and Slot mode (auto-radius) for rectangles.
  - **Layers Tab**: Improved layout with headers and keyboard navigation (Up/Down arrows).

### Recent Updates (v0.37.14)
- **Designer Copy/Paste**: Copy and paste shapes (single, multiple, groups) with context menu support.
- **Designer Undo/Redo**: Full history stack for all canvas operations with toolbar buttons and keyboard shortcuts.

### Recent Updates (v0.37.13)
- **Designer Grouping**: Group/Ungroup shapes, unified selection handles, and group bounding box.
- **Spatial Index Fix**: Increased bounds to support large coordinate ranges (+/- 1,000,000mm).

### Recent Updates (v0.37.1)
- **Visualizer Performance**: Optimized grid rendering and G-code parsing for smoother zoom/pan operations.

### Recent Updates (v0.37)
- **Pocketing Strategies**: Added Raster (Zig-Zag), Contour-Parallel (Offset), and Adaptive strategies.
- **Designer UI**: Added controls for pocket strategy, raster angle, and bidirectional milling.
- **Implementation**: Integrated `cavalier_contours` for robust polygon offsetting.

### Recent Updates (v0.36)
- **Spoilboard Surfacing**: New CAM tool for flattening CNC beds.
- **Device Manager**: Improved UI with explicit Min/Max labels and auto-correction.
- **CNC Tools**: Added "Specialty" tool category and dynamic tool type handling.

### Recent Updates (v0.35)
- **Speeds & Feeds Calculator**: New CAM tool for calculating optimal cutting parameters based on material and tool data.
- **Designer Improvements**: Renamed Polygon to Polyline, added rendering support for Polyline/Path, and improved Shape Properties dialog.
- **Test Reorganization**: Massive cleanup and migration of tests to standard `tests/` directory structure across all crates.
- **SVG Import Fixes**: Corrected mirroring behavior for SVG imports in Designer and Vector Engraver.

This project is in active development. New features are being added regularly, and breaking changes may occur between versions. While the core functionality is stable, use in production environments is at your own risk.

---

**Built with ❤️ using Rust and Slint**
