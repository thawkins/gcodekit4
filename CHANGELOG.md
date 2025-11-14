# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.25.6-alpha] - 2025-11-14

### Added
- **New `load_editor_text()` callback**: Optimized method for bulk loading generated G-code into editor
  - Single operation replaces line-by-line appending for better performance
  - Automatically scrolls viewport to top (line 0) on load
- **Success dialogs for generators**: All G-code generators (TabbedBox, JigsawPuzzle, LaserEngraving) now show completion messages

### Changed
- **CustomTextEdit alignment fixes**: 
  - Added `spacing: 0px` to both VerticalLayout components for consistent line spacing
  - Wrapped Text elements in VerticalLayout with `alignment: center` for proper vertical centering
  - Line numbers and content now properly aligned vertically
- **G-code generation performance**: Tabbed box, jigsaw puzzle, and laser engraving generators now use `load_editor_text()` instead of line-by-line insertion
  - Eliminates UI thread blocking during large G-code insertions
  - Progress bar continues updating smoothly throughout generation

### Fixed
- **"New" button on G-code editor**: Now properly clears editor content by calling `clear_editor()` callback
- **Text line vertical alignment**: Half-line displacement between line numbers and text content resolved
- **Content not showing until scroll**: New loaded content now displays immediately at top of viewport
- **UI stall during G-code insertion**: ImageEngraving and other generators no longer block UI thread

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

