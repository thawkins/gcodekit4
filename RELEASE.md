# Release 0.63.0-alpha.0

### Added
- **Device Manager**: Added file locking mechanism to prevent race conditions during concurrent profile saves.
- **Device Manager**: Added logging for profile save operations to aid debugging.

### Fixed
- **Device Manager**: Fixed critical bug where adding a new device could reset existing device data due to UI state synchronization issues.
- **Device Manager**: Fixed issue where editing a device name caused the selection to jump to the wrong device (often the previous one), leading to accidental data overwrites.
- **Device Manager**: Fixed issue where saving a profile with an empty name was possible, now explicitly rejected with a warning.
- **Documentation**: Updated AGENTS.md with strict rules regarding pushing to remote repositories.
