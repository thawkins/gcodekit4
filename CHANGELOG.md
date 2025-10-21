# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2024-10-21

### Added

#### GitHub Issues and Milestones
- Created 150 GitHub issues from PLAN.md task list
  * Phase 1: Core Foundation (Issues 1-20)
  * Phase 2: GRBL Controller (Issues 21-35)
  * Phase 3: Additional Firmware (Issues 36-50)
  * Phase 4: G-Code Processing (Issues 51-65)
  * Phase 5: UI Implementation (Issues 66-90)
  * Phase 6: File Management (Issues 91-100)
  * Phase 7: Advanced Features (Issues 101-125)
  * Phase 8: Testing & Documentation (Issues 126-150)

- Created 4 milestone definitions:
  * Milestone 1: MVP (v0.2.0) - Due: Dec 31, 2024
  * Milestone 2: Functional Release (v0.3.0) - Due: Mar 31, 2025
  * Milestone 3: Feature Complete (v0.4.0) - Due: Jun 30, 2025
  * Milestone 4: Production Ready (v1.0.0) - Due: Sep 30, 2025

#### Core Implementation Started
- Task 1: Project Initialization
  * Completed Rust project structure setup
  * Configured dependencies in Cargo.toml
  * Set up logging infrastructure (tracing)
  * Created initial module structure

- Task 2: Data Models - Position and Coordinates
  * Implemented Position struct (X, Y, Z, A, B, C coordinates)
  * Implemented PartialPosition for partial axis updates
  * Implemented CNCPoint base structure
  * Added unit support (MM, INCH, UNKNOWN)
  * Created unit conversion utilities

### Changed
- Updated version to 0.3.0
- Updated SPEC.md to version 0.3.0
- Updated README.md development status

### Documentation
- PLAN.md: 150 tasks across 8 phases (1,147 lines)
- SPEC.md: Complete system specification (1,380 lines, v0.3.0)
- AGENTS.md: Development guidelines with code standards
- CHANGELOG.md: Version history documentation

## [0.1.1] - 2024-10-21

### Added

#### GitHub Milestones Configuration
- Complete milestone definitions (docs/MILESTONES.md)
  * Milestone 1: MVP (v0.2.0) - 70 tasks, Due: Dec 31, 2024
  * Milestone 2: Functional Release (v0.3.0) - 60 tasks, Due: Mar 31, 2025
  * Milestone 3: Feature Complete (v0.4.0) - 65 tasks, Due: Jun 30, 2025
  * Milestone 4: Production Ready (v1.0.0) - 30+ tasks, Due: Sep 30, 2025
  * Success criteria for each milestone
  * Task assignments and deliverables

- Milestone setup guide (docs/MILESTONES_SETUP.md)
  * Quick start instructions
  * GitHub CLI commands for manual creation
  * Bash script for automated setup
  * Best practices for progress tracking
  * Troubleshooting guide

#### Task-to-Milestone Mapping
- Milestone 1: Tasks 1-20, 21-35, 66-74, 91-94
- Milestone 2: Tasks 36-50, 51-65, 75-83, 101-107
- Milestone 3: Tasks 84-90, 95-100, 108-121, 126-137
- Milestone 4: Tasks 122-125, 138-150

---

## [0.1.0] - 2024-10-21

### Added

#### Documentation
- Complete technical specification (SPEC.md) - 1,379 lines
  - System architecture with 8 major components
  - Complete UI specifications for 11 major panels
  - State machine with 9 controller states
  - Core functionality specifications for all major features
  - Non-functional requirements (performance, reliability, security)
  - G-Code command matrix (20+ G-codes, 11+ M-codes)
  - Firmware capabilities matrix (5 controllers, 13 features)
  - Error handling and recovery strategies
  - Macro and script system design

- Implementation roadmap (PLAN.md) - 1,147 lines
  - 150 tasks organized in 8 phases
  - Phase 1: Core Foundation (Tasks 1-20)
  - Phase 2: GRBL Controller Implementation (Tasks 21-35)
  - Phase 3: Additional Firmware Support (Tasks 36-50)
  - Phase 4: Advanced G-Code Processing (Tasks 51-65)
  - Phase 5: UI Implementation with Slint (Tasks 66-90)
  - Phase 6: File Management and Processing (Tasks 91-100)
  - Phase 7: Advanced Features (Tasks 101-125)
  - Phase 8: Testing and Documentation (Tasks 126-150)
  - Priority milestones and success criteria
  - Complete dependency list for Cargo.toml
  - File structure and organization

- Development guidelines (AGENTS.md)
  - Technology stack specifications
  - Build commands with timeouts
  - Test organization requirements
  - Code style guidelines (4-space, 100-char width)
  - Documentation standards
  - Issue handling process
  - GitHub workflow

- README.md with project overview and quick start guide

#### Project Structure
- Specification of complete module hierarchy
  - core/ - Controller and state management
  - communication/ - Serial/TCP/WebSocket communication
  - gcode/ - G-Code parsing and preprocessing
  - firmware/ - Controller-specific implementations
  - models/ - Data structures and types
  - ui/ - Slint-based user interface
  - visualizer/ - 3D rendering with wgpu
  - utils/ - Helper functions and utilities
  - tests/ - Test organization

#### Architecture & Design
- Modular Rust architecture with trait-based abstractions
- Event-driven state management system
- Async-first design with tokio runtime
- Pluggable preprocessor pipeline (14 processor types)
- Multi-protocol support (text-based, JSON, WebSocket)
- Firmware auto-detection and capability querying

#### Features Specified
- 5 CNC controller firmware support (GRBL, TinyG, g2core, Smoothieware, FluidNC)
- 3 connection types (Serial/USB, TCP/IP, WebSocket)
- 11 major UI panels with detailed specifications
- 14 G-Code preprocessing operations
- Real-time machine overrides (feed rate, rapid, spindle)
- Work coordinate systems (G54-G59)
- Jogging (continuous and incremental)
- Probing (single-point and multi-point)
- Tool change management
- Macro and script system
- 3D visualization with interactive controls
- Performance monitoring and diagnostics

#### Non-Functional Requirements
- Performance targets documented
- Reliability and robustness specifications
- Accessibility requirements (WCAG 2.1 AA)
- Security constraints and considerations
- Code quality metrics (>80% test coverage)
- Cross-platform support (Linux, Windows, macOS)

### Specification Details

#### Supported Controllers
- **GRBL v0.9, v1.0, v1.1**: Character counting protocol, real-time commands
- **TinyG**: JSON protocol, 6-axis motion
- **g2core**: Advanced JSON, file system support
- **Smoothieware**: RepRap dialect, network connectivity
- **FluidNC**: JSON/WebSocket, WiFi, modern kinematics

#### G-Code Support
- Motion commands: G0, G1, G2, G3, G4, G10, G17-G19, G20-G21, G28, G30, G38.x
- Coordinate systems: G53, G54-G59
- Machine commands: M0-M2, M3-M9, M30
- Tool selection: T0-T99

#### UI Components
1. Connection Panel - Port selection, baud rate, connection status
2. DRO Panel - Machine/work coordinates, state, feed rate, spindle speed
3. Jog Panel - Incremental/continuous jogging with keyboard shortcuts
4. File Operations - Browser, drag-drop, statistics
5. G-Code Editor - Syntax highlighting, line numbers, search/replace
6. Console - Color-coded messages, filtering, history
7. Control Panel - Start/Pause/Stop, Home, Reset, Unlock
8. Overrides Panel - Feed rate, rapid, spindle sliders
9. Coordinate System - WCS selection, offset management
10. Macros Panel - Macro execution and editing
11. 3D Visualizer - Toolpath preview, interactive camera

#### State Machine
- DISCONNECTED → CONNECTING → IDLE → RUN/HOLD/JOG/HOME/ALARM
- 9 distinct states with defined transitions
- Safety states (DOOR, CHECK, SLEEP)
- Error recovery paths

### Development Milestones

#### Milestone 1 (MVP)
- Core foundation implementation
- GRBL controller support
- Basic UI with essential panels
- File management basics

#### Milestone 2 (Functional Release)
- Additional firmware support (TinyG, g2core, Smoothieware, FluidNC)
- Advanced G-Code processing
- Complete UI features
- Basic advanced features (probing, tool change)

#### Milestone 3 (Feature Complete)
- UI polish and accessibility
- Advanced file features
- Advanced features (macros, calibration, diagnostics)
- Comprehensive testing

#### Milestone 4 (Production Ready)
- Plugin system
- Extensibility features
- Complete documentation
- Performance optimization

### Future Enhancements (Post-MVP)

#### Phase 2 Features
- Plugin system for third-party extensions
- Remote access via REST API and WebSocket
- Advanced collision detection
- Tool library management
- Automatic tool length offset probing
- Auto-leveling mesh generation

#### Phase 3 Features
- Kinematics support (non-Cartesian machines)
- Multi-head support
- Advanced debugging UI
- Machine health monitoring
- Performance profiling tools

#### Phase 4 Features
- Mobile app support (iOS/Android)
- Augmented reality visualization
- Machine learning-based optimization
- Enterprise integration (MES systems)
- 3D CAM integration

---

## Version Guidelines

This project uses [Semantic Versioning](https://semver.org/):
- **MAJOR**: Incompatible API changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Pre-release versions use format: `X.Y.Z-alpha`, `X.Y.Z-beta`, etc.

---

**Repository**: https://github.com/your-username/gcodekit4
**License**: GNU General Public License v3.0
