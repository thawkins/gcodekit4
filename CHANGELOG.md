# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

