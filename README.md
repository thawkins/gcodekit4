# GCodeKit4

A modern, cross-platform G-Code sender and CNC machine controller written in Rust with Slint UI framework.

[![Build Status](https://github.com/thawkins/gcodekit4/workflows/CI/badge.svg)](https://github.com/thawkins/gcodekit4/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-0.25.4--alpha-brightgreen.svg)](CHANGELOG.md)

## Overview

GCodeKit4 is a Rust-based CNC machine controller providing a modern alternative to Universal G-Code Sender. It supports multiple controller firmware types including GRBL, grblHAL, TinyG, g2core, Smoothieware, and FluidNC through a unified, intuitive interface built with the Slint UI framework.

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

### 📝 G-Code Editor
- **Syntax Highlighting**: Color-coded commands, coordinates, and comments
- **Line Numbers**: Easy navigation and reference
- **File Operations**: Open, edit, and save G-code files
- **Send to Device**: Stream G-code directly to connected controller
- **Real-time Validation**: Syntax checking while editing

### 🎨 2D CAD/CAM Designer
- **Vector Drawing Tools**:
  - Geometric shapes: rectangles, circles, ellipses
  - Lines, polygons, Bezier curves, and arcs
  - Round rectangles with adjustable corner radius
- **File Import**: Import SVG and DXF vector files
- **Interactive Editing**:
  - Zoom, pan, and fit-to-view controls
  - Precise positioning (X, Y, Width, Height inputs)
  - Properties dialog for detailed shape adjustments
  - Dual-grid system (10mm major + 1mm minor)
- **SVG Canvas Rendering**: High-quality vector-based canvas
- **Context Menu**: Right-click for Delete and Properties
- **Toolpath Generation**: Convert designs to executable G-code

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
- **Email**: toby@hawkins.com
- **Documentation**: See docs/ folder and SPEC.md

## Project Status

**Current Version**: 0.25.4-alpha  
**Status**: Active Development  
**Stability**: Alpha (breaking changes may occur)

This project is in active development. New features are being added regularly, and breaking changes may occur between versions. While the core functionality is stable, use in production environments is at your own risk.

---

**Built with ❤️ using Rust and Slint**
