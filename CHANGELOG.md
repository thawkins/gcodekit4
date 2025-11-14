# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.25.6-alpha] - 2025-11-14

### Added
- **Custom G-Code Text Editor - Phase 1 (COMPLETE)**
  - Full keyboard input system with proper event handling through Slint callback chain
  - Text insertion with automatic cursor advancement for each character typed
  - Text deletion via Backspace and Delete keys with cursor repositioning
  - Complete arrow key navigation (left, right, up, down) with proper boundary checking
  - Home and End keys for jumping to line boundaries
  - PageUp and PageDown for viewport scrolling (10 lines per page)
  - Undo/Redo support triggered by Ctrl+Z (undo) and Ctrl+Y (redo)
  - Tab key inserts 4 spaces for automatic indentation
  - Enter/Return key for newline insertion at cursor position
  - Virtual scrolling system supporting 100+ line files efficiently
  - Line number display with synchronized scrolling
  - Real-time cursor position tracking displayed in status bar
  - Text buffer updates on every keystroke, automatically saved to file operations
  - Complete integration: keyboard events → CustomTextEdit → GcodeEditorPanel → MainWindow → Rust EditorBridge

### Technical Implementation
- Slint callback architecture with proper hyphenated naming conventions
- MainWindow FocusScope handles keyboard events and routes to text_inserted() Rust callback
- CustomTextEdit.key-pressed handler recognizes special keys using Key namespace constants
- Proper event forwarding through callback chain: CustomTextEdit → GcodeEditorPanel → MainWindow → Rust
- Line-based cursor tracking (0-based internally, 1-based for user display)
- EditorBridge integration for persistent text buffer management

### Fixed
- Keyboard event handling in custom components through proper FocusScope implementation
- Callback naming consistency across Slint (.slint with hyphens) and Rust (with underscores)
- Event propagation from child components to parent through explicit root.callback() calls
- Text cursor initialization and boundary checking during navigation

### Known Limitations (Phase 1)
- No text selection yet (Phase 2 feature)
- No copy/paste support (Phase 2 feature)
- No find/replace functionality (Phase 2 feature)
- No syntax highlighting (Phase 2+ feature)
- No multi-level undo/redo (Phase 2 feature)

## [0.25.5-alpha] - 2025-11-13


### Changed
- **Tabbed Box Generator**: Complete rewrite using boxes.py algorithm from https://github.com/florianfesti/boxes
  - Replaced previous finger joint implementation with production-proven boxes.py approach
  - Added configurable finger joint settings: finger width, space width, surrounding spaces, play tolerance
  - Improved finger joint algorithm with automatic calculation of optimal finger count
  - Added multiple finger joint styles: Rectangular (default), Springs, Barbs, Snap
  - Enhanced parameter controls in UI with finger/space multiples of thickness
  - Fixed coordinate transformation issues for proper closed rectangular paths
  - Implemented duplicate point checking to eliminate corner gaps
  - Added proper edge reversal for top and left edges
  - Corrected finger orientation on all four edges (fingers point outward correctly)

### Added
- New `FingerJointSettings` structure with configurable parameters
- `FingerStyle` enum supporting multiple finger joint types
- Enhanced CAM Tool dialog with additional finger joint parameters
- Better G-code generation with cleaner paths and proper edge transitions

### Fixed
- Diagonal jump vectors in generated G-code paths
- Incorrect finger orientations on top and left edges
- Corner connection issues causing open paths
- Edge transformation coordinate calculation errors
- Path generation now produces cuttable, mostly-closed shapes

## [0.25.4-alpha] - 2025-11-01

### Added
- Initial tabbed box generator implementation
- Basic finger joint calculations
- G-code export for laser cutting

