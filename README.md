# GCodeKit4

A modern, cross-platform G-Code sender and CNC machine controller written in Rust.

[![Build Status](https://github.com/thawkins/gcodekit4/workflows/CI/badge.svg)](https://github.com/thawkins/gcodekit4/actions)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/Version-0.12.0-brightgreen.svg)](CHANGELOG.md)

## Overview

GCodeKit4 is a Rust-based implementation of Universal G-Code Sender, providing a modern alternative for controlling CNC machines. It supports multiple controller firmware types including GRBL, TinyG, g2core, Smoothieware, and FluidNC through a unified, intuitive interface.

## Features

### Core Capabilities
- **Multi-Controller Support**: GRBL (v0.9/v1.0/v1.1), TinyG, g2core, Smoothieware, FluidNC
- **Real-time 3D Visualization**: Interactive toolpath preview with live position tracking
- **Advanced G-Code Processing**: Arc expansion, mesh leveling, coordinate transformations
- **Flexible Connectivity**: Serial/USB, TCP/IP, and WebSocket support
- **Comprehensive Machine Control**: Jogging, homing, probing, overrides, tool changes
- **File Management**: Open, validate, process, and export G-Code files

### User Interface
- Connection panel with auto-detect and auto-reconnect
- Digital Readout (DRO) with machine and work coordinates
- Jog controller with keyboard shortcuts
- Real-time 3D visualizer with multiple view modes
- G-Code editor with syntax highlighting
- Console with color-coded messages
- Firmware settings management
- Customizable macros

### Advanced Features
- 14 G-Code preprocessors
- Coordinate system management (G54-G59)
- Real-time overrides (feed rate, rapid, spindle)
- Single-step execution mode
- Dry-run/simulation mode
- Performance monitoring

## Quick Start

### Installation

#### Requirements
- Rust 1.70.0 or later
- Linux, Windows (7+), or macOS (10.13+)
- 512MB RAM minimum (2GB recommended)

#### Build from Source
```bash
git clone https://github.com/your-username/gcodekit4.git
cd gcodekit4
cargo build --release
```

The binary will be available at `target/release/gcodekit4`.

### Running

```bash
# Debug build
cargo run

# Release build
./target/release/gcodekit4
```

### First Connection

1. **Connect to Controller**
   - Select serial port from dropdown
   - Choose appropriate baud rate (typically 115200 for GRBL)
   - Click "Connect"

2. **Home Machine**
   - Wait for connection to establish
   - Click "Home" button to execute homing cycle
   - Machine will return to home position

3. **Load G-Code**
   - File → Open (or drag-and-drop)
   - File will be parsed and validated
   - Preview appears in 3D visualizer

4. **Run Program**
   - Click "Start" to begin streaming
   - Use "Pause" to hold machine
   - Click "Stop" to cancel

## Supported Controllers

### GRBL (v0.9, v1.0, v1.1)
- Text-based protocol with character counting
- Real-time commands (pause, resume, overrides)
- Status reports with position feedback
- 11 alarm types with descriptions
- 30+ error codes
- Settings system ($0-$32)
- Homing, soft limits, probing

### TinyG
- JSON protocol
- 6-axis motion support
- Queue-based command processing
- Macros and tool tables
- Extended G-code support

### g2core
- Advanced JSON protocol
- Extended motion planning
- File system support (SD card)
- Networked I/O capabilities

### Smoothieware
- RepRap G-code dialect
- Extensive M-code support
- Network connectivity

### FluidNC
- JSON protocol with WebSocket support
- WiFi connectivity
- File system operations
- Web-based interface support

## Documentation

- **[SPEC.md](SPEC.md)** - Complete system specification (1,379 lines)
- **[PLAN.md](PLAN.md)** - Implementation roadmap with 150 tasks (1,147 lines)
- **[AGENTS.md](AGENTS.md)** - Development guidelines and code standards
- **[CHANGELOG.md](CHANGELOG.md)** - Version history and changes
- **[docs/MILESTONES.md](docs/MILESTONES.md)** - GitHub milestone definitions (4 milestones)
- **[docs/MILESTONES_SETUP.md](docs/MILESTONES_SETUP.md)** - Milestone setup guide
- **[docs/](docs/)** - Additional documentation (user guides, API reference)

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         UI Layer (Slint)                         │
│  ┌────────┬──────────┬──────────┬──────────┬──────────┬────────┐│
│  │Connect │   DRO    │   Jog    │ G-Code  │ Console  │ Macros ││
│  │ Panel  │  Panel   │  Panel   │ Editor  │  Panel   │ Panel  ││
│  └────────┴──────────┴──────────┴──────────┴──────────┴────────┘│
├─────────────────────────────────────────────────────────────────┤
│                    3D Visualizer (wgpu)                          │
├─────────────────────────────────────────────────────────────────┤
│                     Core Layer (Events)                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┬──────────────┬──────────────┬──────────────┐  │
│  │    GRBL      │    TinyG     │   g2core    │  Smoothie    │  │
│  │  Controller  │  Controller  │  Controller │  Controller  │  │
│  └──────────────┴──────────────┴──────────────┴──────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┬──────────────┬──────────────┐                 │
│  │  G-Code      │  Preprocessor│  Settings    │                 │
│  │  Parser      │  Pipeline    │  Manager     │                 │
│  └──────────────┴──────────────┴──────────────┘                 │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┬──────────────┬──────────────┐                 │
│  │  Serial      │  TCP         │  WebSocket   │                 │
│  │  Communicator│  Communicator│  Communicator│                 │
│  └──────────────┴──────────────┴──────────────┘                 │
├─────────────────────────────────────────────────────────────────┤
│                  CNC Machine Controller                          │
└─────────────────────────────────────────────────────────────────┘
```

## Supported G-Code

- **Motion Commands**: G0, G1, G2, G3, G4 (rapid, linear, arc, dwell)
- **Plane Selection**: G17, G18, G19 (XY, XZ, YZ)
- **Coordinate Systems**: G20/G21 (inch/mm), G53/G54-G59 (MCS/WCS)
- **Special**: G10, G28, G30, G38.2-G38.5 (set position, home, probe)
- **Machine Commands**: M0-M2 (stop), M3-M5 (spindle), M6 (tool change)
- **Coolant**: M7-M9 (mist, flood, off)
- **Tool Selection**: T0-T99

See [SPEC.md](SPEC.md) for complete G-Code matrix.

## Building and Testing

### Build Commands
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Library tests only
cargo test --lib
```

### Code Quality
```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

## Performance Targets

- **File Loading**: <2 seconds for 1MB files
- **G-Code Parsing**: >10,000 lines per second
- **Command Streaming**: >100 commands per second
- **3D Visualization**: >30 FPS
- **Memory Usage**: <150MB for 100K line files

## Development Status

**Current Version**: 0.3.0 (Planning & Setup Phase)

### Implemented
- ✓ Complete system specification (SPEC.md)
- ✓ Implementation roadmap (PLAN.md)
- ✓ Development guidelines (AGENTS.md)

### In Progress
- Phase 1: Core Foundation (Tasks 1-20)
- Data models and error handling
- Communication layer

### Planned
- Phase 2: GRBL Controller (Tasks 21-35)
- Phase 3-5: Additional firmware, G-Code processing, UI
- Phase 6-8: File management, advanced features, testing

See [PLAN.md](PLAN.md) for complete roadmap with 150 tasks.

## Contributing

Contributions are welcome! Please:

1. Read [AGENTS.md](AGENTS.md) for development guidelines
2. Follow the code style (4-space indents, 100-char width)
3. Write tests for new features
4. Update documentation as needed
5. Create a pull request with clear description

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Related Projects

- [Universal G-Code Sender (Java)](https://github.com/winder/Universal-G-Code-Sender) - Original project inspiration
- [GRBL](https://github.com/gnea/grbl) - CNC firmware
- [TinyG](https://github.com/synthetos/TinyG) - CNC firmware
- [FluidNC](https://github.com/bdring/FluidNC) - Modern CNC firmware

## Support

- **Documentation**: See [SPEC.md](SPEC.md) and docs/ folder
- **Issues**: Report bugs on [GitHub Issues](https://github.com/your-username/gcodekit4/issues)
- **Discussions**: Use [GitHub Discussions](https://github.com/your-username/gcodekit4/discussions)

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and changes.

---

**Made with ❤️ by CNC enthusiasts**
