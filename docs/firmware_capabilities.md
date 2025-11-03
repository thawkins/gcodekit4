# Firmware Capabilities Database

## Overview

The firmware capabilities database tracks which features are supported by each CNC controller firmware version. This enables GCodeKit4 to provide version-aware UI and G-code generation that's tailored to the connected controller's capabilities.

## Purpose

Different firmware versions support different features. For example:
- GRBL 0.9 doesn't support arc commands (G2/G3)
- GRBL 1.1 added status reports and real-time overrides
- TinyG supports macros and conditional blocks
- FluidNC supports up to 9 axes

The capabilities database allows GCodeKit4 to:
1. Enable/disable UI features based on what the controller supports
2. Generate firmware-appropriate G-code
3. Show warnings for unsupported operations
4. Provide accurate feature descriptions

## Supported Firmware

### GRBL

**Version 0.9** (Basic)
- 3 axes (X, Y, Z)
- Variable spindle speed
- Spindle direction control
- Homing cycle
- Soft limits
- Alarm conditions

**Version 1.0** (Standard)
- All v0.9 features plus:
- Arc interpolation (G2/G3)
- Tool change support (M6)
- Probing (G38.2)
- 6 work coordinate systems (G54-G59)
- Flow control

**Version 1.1** (Enhanced)
- All v1.0 features plus:
- Probe away commands (G38.4/G38.5)
- Cutter radius compensation (G41/G42)
- Hard limits
- Door interlock
- Status reports (?)
- Real-time commands (!~%C^X)

**Version 1.2 & 1.3** (Current)
- Same as v1.1 with bug fixes

### TinyG

**Version 2.x** (Full Featured)
- 4 axes (X, Y, Z, A)
- All GRBL features plus:
- Inverse time feed rate (G93)
- Feed per revolution (G95)
- Constant surface speed (CSS)
- Tool length offsets (G43/G49)
- Coolant and mist control (M7/M8)
- 9 work coordinate systems
- Local offsets (G52)
- Macro support
- Conditional blocks (if/then)
- Variable support

### g2core

**Version 3.x** (Advanced)
- 6 axes (X, Y, Z, A, B, C)
- All TinyG features plus:
- Tool diameter offsets (G41/G42)
- Enhanced motion planning
- Advanced safety features

### Smoothieware

**Version 1.x** (Multi-Axis)
- 5 axes
- Advanced multi-axis support
- Custom module system
- Full G-code feature set
- Network connectivity
- File system support

### FluidNC

**Version 3.x** (Modern)
- Up to 9 axes
- WiFi support
- Web interface
- Modern feature set
- YAML configuration
- Real-time monitoring

## Data Structure

### FirmwareCapabilities

```rust
pub struct FirmwareCapabilities {
    pub firmware_type: FirmwareType,
    pub version: SemanticVersion,
    
    // Core Motion
    pub max_axes: u8,
    pub arc_support: bool,
    pub inverse_time_feed: bool,
    pub feed_per_revolution: bool,
    
    // Spindle
    pub variable_spindle: bool,
    pub spindle_direction: bool,
    pub spindle_css: bool,
    
    // Tool Management
    pub tool_change: bool,
    pub tool_length_offset: bool,
    pub tool_diameter_offset: bool,
    
    // Probing
    pub probing: bool,
    pub probe_away: bool,
    
    // Coolant
    pub coolant_control: bool,
    pub mist_control: bool,
    
    // Homing
    pub homing_cycle: bool,
    pub soft_homing: bool,
    pub hard_homing: bool,
    
    // Offsets
    pub coordinate_systems: u8,
    pub local_offsets: bool,
    pub cutter_radius_compensation: bool,
    
    // Advanced
    pub macro_support: bool,
    pub conditional_blocks: bool,
    pub variable_support: bool,
    
    // Communication
    pub status_reports: bool,
    pub realtime_commands: bool,
    pub flow_control: bool,
    
    // Safety
    pub soft_limits: bool,
    pub hard_limits: bool,
    pub alarm_conditions: bool,
    pub door_interlock: bool,
    
    // Custom capabilities
    pub custom: HashMap<String, bool>,
}
```

## Usage

### In Rust Code

```rust
use gcodekit4::firmware::capabilities_db::CapabilitiesDatabase;
use gcodekit4::firmware::firmware_version::{FirmwareType, SemanticVersion};

// Create database (loads built-in profiles)
let db = CapabilitiesDatabase::new();

// Get capabilities for specific firmware
let grbl_version = SemanticVersion::new(1, 1, 0);
if let Some(caps) = db.get_capabilities(FirmwareType::Grbl, &grbl_version) {
    println!("Max axes: {}", caps.max_axes);
    println!("Arc support: {}", caps.arc_support);
    println!("Coordinate systems: {}", caps.coordinate_systems);
}

// Check specific capability
if db.supports_capability(FirmwareType::Grbl, &grbl_version, "arc") {
    // Generate arc commands
} else {
    // Use line approximation
}

// List all supported firmware types
let types = db.supported_firmware_types();
```

### Version Detection

When connecting to a controller:

```rust
// Detect firmware version from controller response
let firmware_type = detect_firmware_type(&response);
let version = parse_version(&response);

// Load capabilities for that version
let caps = db.get_capabilities(firmware_type, &version)?;

// Enable/disable UI features
ui.enable_arc_commands(caps.arc_support);
ui.enable_probing(caps.probing);
ui.set_max_coordinate_systems(caps.coordinate_systems);
```

## Feature Matrix

| Feature | GRBL 0.9 | GRBL 1.1 | TinyG 2.x | g2core 3.x | Smoothie | FluidNC |
|---------|----------|----------|-----------|------------|----------|---------|
| Max Axes | 3 | 3 | 4 | 6 | 5 | 9 |
| Arc Motion | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Probing | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tool Change | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Status Reports | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Overrides | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Macros | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| Coord Systems | 1 | 6 | 9 | 9 | 9 | 9 |
| WiFi | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |

## Extending the Database

### Adding Custom Capabilities

```rust
use std::collections::HashMap;

let mut db = CapabilitiesDatabase::new();

// Create custom capabilities
let mut caps = FirmwareCapabilities::new(
    FirmwareType::Custom,
    SemanticVersion::new(1, 0, 0)
);

caps.max_axes = 4;
caps.arc_support = true;
// ... set other fields

// Add custom capabilities
let mut custom_caps = HashMap::new();
custom_caps.insert("laser_mode".to_string(), true);
custom_caps.insert("rotary_axis".to_string(), true);

db.register_custom(caps, custom_caps);
```

### Adding New Firmware Version

To add support for a new firmware version, edit `src/firmware/capabilities_db.rs` and add to the appropriate init function:

```rust
fn init_grbl_profiles(&mut self) {
    // ... existing profiles ...
    
    // GRBL 1.4 (hypothetical)
    let mut grbl_1_4 = FirmwareCapabilities::new(
        FirmwareType::Grbl,
        SemanticVersion::new(1, 4, 0)
    );
    grbl_1_4.max_axes = 3;
    grbl_1_4.arc_support = true;
    // ... configure capabilities ...
    
    self.database.insert(
        (FirmwareType::Grbl, "1.4".to_string()),
        grbl_1_4
    );
}
```

## Testing

The database includes comprehensive unit tests:

```bash
cargo test --lib capabilities_db
```

Tests verify:
- Database initialization
- Capability queries for each firmware
- Version matching
- Feature detection
- Custom capability registration

## Files

- **`src/firmware/capabilities.rs`** - Capability trait and enums
- **`src/firmware/capabilities_db.rs`** - Database implementation
- **`src/firmware/device_db.rs`** - Integration with device database
- **`docs/firmware_capabilities.md`** - This documentation

## Future Enhancements

Planned improvements:
1. TOML/JSON configuration file for user overrides
2. Capability discovery from controller (query firmware features)
3. Per-machine capability customization
4. Capability change notifications
5. UI tooltips showing why features are disabled
6. G-code validation against capabilities
7. Automatic firmware update recommendations

## Related Documentation

- [Firmware Version Detection](firmware_version.md)
- [Device Database](device_db.md)
- [Controller Communication](../communication/README.md)
