# Firmware Capabilities Database

## Overview

The Firmware Capabilities Database is a comprehensive system for tracking and querying which CNC controller firmware versions support specific features. This enables GcodeKit to:

- Enable/disable UI features based on detected firmware capabilities
- Validate G-code generation against controller capabilities
- Provide version-aware warnings for unsupported operations
- Generate firmware-specific G-code variants
- Support version-aware programming

## Architecture

### Core Components

#### `FirmwareCapabilities`
Represents the complete feature set for a specific firmware type and version. Located in `src/firmware/capabilities_db.rs`.

**Key Fields:**
- `firmware_type`: The CNC controller type (GRBL, TinyG, g2core, Smoothieware, FluidNC)
- `version`: Semantic version (major.minor.patch)
- `max_axes`: Maximum number of axes supported (3-9)
- `coordinate_systems`: Number of work coordinate systems (G54-G59 equivalents)
- Feature flags for each capability category

**Feature Categories:**

1. **Core Motion**
   - `arc_support`: G2/G3 arc interpolation
   - `inverse_time_feed`: G93 inverse time feed rate mode
   - `feed_per_revolution`: G95 feed per revolution mode

2. **Spindle Control**
   - `variable_spindle`: Variable speed control (M3/M4 with PWM)
   - `spindle_direction`: Direction control (M3/M4)
   - `spindle_css`: Constant Surface Speed mode

3. **Tool Management**
   - `tool_change`: Tool change commands (M6)
   - `tool_length_offset`: Tool length offsets (G43/G49)
   - `tool_diameter_offset`: Tool diameter offsets (G41/G42)

4. **Probing**
   - `probing`: Straight probe (G38.2)
   - `probe_away`: Probe away capability (G38.4/G38.5)

5. **Coolant/Mist**
   - `coolant_control`: Coolant command (M8)
   - `mist_control`: Mist command (M7)

6. **Homing**
   - `homing_cycle`: Homing support (G28)
   - `soft_homing`: Software-based homing
   - `hard_homing`: Hardware-based homing

7. **Offsets & Compensation**
   - `local_offsets`: Local G-code offsets (G52)
   - `cutter_radius_compensation`: G41/G42 compensation

8. **Advanced Features**
   - `macro_support`: Macro/subroutine support
   - `conditional_blocks`: If/then/else logic
   - `variable_support`: Variable assignment and use

9. **Communication**
   - `status_reports`: Status query responses (?)
   - `realtime_commands`: Real-time commands (!~%C^X)
   - `flow_control`: XON/XOFF or RTS/CTS flow control

10. **Safety**
    - `soft_limits`: Software limit checking
    - `hard_limits`: Hardware limit switch support
    - `alarm_conditions`: Alarm reporting
    - `door_interlock`: Door switch interlock

#### `CapabilityInfo`
Version-aware capability information with enabled/version range tracking.

**Fields:**
- `enabled`: Whether capability is available
- `min_version`: Minimum version where capability appears
- `max_version`: Maximum version where capability is available (None = current)
- `notes`: Additional information about this capability

#### `CapabilitiesDatabase`
Main database class providing access to firmware capabilities.

**Key Methods:**
- `new()`: Create database with all built-in profiles
- `get_capabilities(firmware_type, version)`: Retrieve capabilities for specific firmware
- `supports_capability(firmware_type, version, capability)`: Check single capability
- `supported_firmware_types()`: List all supported firmware types
- `register_custom(capabilities, custom_caps)`: Add custom capabilities

## Supported Firmware

### GRBL (Open-source CNC control)
- **v0.9**: Basic motion, spindle, homing, soft limits
- **v1.0**: Adds arc support, tool change, probing, coordinate systems
- **v1.1**: Adds probe away, hard limits, status reports, real-time commands, door interlock

**Typical Use:** Small CNC mills, laser cutters, engravers

### TinyG (CNC controller for precision machines)
- **v2.0+**: Full G-code support, JSON config, macros, advanced motion control

**Features:**
- 4 axes support
- CSS (Constant Surface Speed) support
- Macro and conditional block support
- 9 coordinate systems

**Typical Use:** 3D printers, engravers, small production machines

### g2core (Next generation TinyG)
- **v3.0+**: Enhanced motion planning, 6-axis support

**Features:**
- Up to 6 axes
- Enhanced motion planning
- Tool diameter offset support
- Advanced macro support

**Typical Use:** Desktop manufacturing, production setups

### Smoothieware (Advanced CNC motion controller)
- **v1.0+**: Full G-code support, multi-axis, advanced features

**Features:**
- Up to 5 axes
- CSS support
- Macro and conditional blocks
- 9 coordinate systems
- Tool offset support

**Typical Use:** Laser cutters, CNC routers, custom machines

### FluidNC (Modern open-source CNC)
- **v3.0+**: Maximum feature set, WiFi-capable

**Features:**
- Up to 9 axes
- All advanced features enabled
- Modern communication options
- Full G-code support

**Typical Use:** New CNC builds, high-feature machines

## Usage Examples

### Check Firmware Capabilities

```rust
use gcodekit4::firmware::capabilities_db::CapabilitiesDatabase;
use gcodekit4::firmware::firmware_version::{FirmwareType, SemanticVersion};

let db = CapabilitiesDatabase::new();
let version = SemanticVersion::new(1, 1, 0);

// Get full capabilities
if let Some(caps) = db.get_capabilities(FirmwareType::Grbl, &version) {
    println!("Max axes: {}", caps.max_axes);
    println!("Supports probing: {}", caps.probing);
    println!("Supports tool change: {}", caps.tool_change);
}
```

### Enable UI Features Based on Firmware

```rust
// Check specific capability
let supports_probing = db.supports_capability(
    FirmwareType::Grbl,
    &SemanticVersion::new(1, 1, 0),
    "probing"
);

if supports_probing {
    // Enable probe button in UI
}
```

### Validate G-code Operations

```rust
// Before generating arc commands
if let Some(caps) = db.get_capabilities(fw_type, &version) {
    if caps.arc_support {
        // Can use G2/G3 commands
    } else {
        // Warn user or use linear approximation
    }
}
```

## Extending the Database

### Adding New Firmware Versions

1. **Add to `init_<firmware>_profiles()`** method:

```rust
fn init_grbl_profiles(&mut self) {
    // ... existing profiles ...
    
    // GRBL 1.2 (hypothetical future version)
    let mut grbl_1_2 = FirmwareCapabilities::new(
        FirmwareType::Grbl,
        SemanticVersion::new(1, 2, 0)
    );
    grbl_1_2.max_axes = 4;
    grbl_1_2.arc_support = true;
    // ... set all capabilities ...
    self.database.insert((FirmwareType::Grbl, "1.2".to_string()), grbl_1_2);
}
```

2. **Update Tests** to verify new profile works correctly

### Adding New Capabilities

1. **Add field to `FirmwareCapabilities` struct**:
```rust
pub my_new_capability: bool,
```

2. **Update `supports()` method**:
```rust
"my_feature" => self.my_new_capability,
```

3. **Set in firmware profiles** that support it

4. **Add tests** for new capability

## Database Loading

The database is initialized automatically when created. Profiles are loaded in this order:

1. GRBL (v0.9, v1.0, v1.1)
2. TinyG (v2.0)
3. g2core (v3.0)
4. Smoothieware (v1.0)
5. FluidNC (v3.0)

### Version Matching Strategy

When querying capabilities, the database:

1. Tries exact version match (major.minor)
2. Falls back to major.minor matching (ignores patch)
3. Returns None if not found

This allows versions like 1.1f (GRBL) to match the 1.1 profile.

## Integration Points

### Communication Layer
Used when detecting firmware type and version on connection:

```rust
// In connection handler
let db = CapabilitiesDatabase::new();
if let Some(caps) = db.get_capabilities(detected_type, &detected_version) {
    // Use capabilities for subsequent operations
}
```

### UI Layer
Enables/disables features based on detected capabilities:

```rust
// In UI panel initialization
if controller_caps.probing {
    enable_probe_panel();
}
```

### G-code Generation
Validates operations against capabilities:

```rust
// In CAM tools
if !capabilities.supports_capability(fw_type, version, "arc") {
    warn!("Firmware doesn't support arc commands");
}
```

## Performance Considerations

- Database creation is O(1) - all profiles are pre-built
- Capability queries are O(1) - hash map lookup
- Memory usage is minimal (~10KB for all profiles)
- No I/O or network operations involved

## Future Enhancements

1. **Loadable Profiles**: Support TOML/JSON configuration files for custom profiles
2. **Version Range Queries**: Query capabilities across version ranges
3. **Deprecated Features**: Track features removed in later versions
4. **Capability Groups**: Organize related capabilities into logical groups
5. **Performance Profiles**: Different optimization levels per firmware
6. **Community Profiles**: Support user-contributed firmware profiles

## Testing

Run capabilities database tests:

```bash
cargo test --lib firmware::capabilities_db
```

Verify all firmware types have profiles:

```bash
cargo test --lib firmware::capabilities_db::tests::test_supported_firmware_types
```

Test specific firmware capabilities:

```bash
cargo test --lib firmware::capabilities_db::tests::test_grbl_1_1_capabilities
cargo test --lib firmware::capabilities_db::tests::test_smoothieware_capabilities
```

## References

- [SPEC.md](../SPEC.md) - Project specification
- [src/firmware/capabilities_db.rs](../src/firmware/capabilities_db.rs) - Implementation
- [src/firmware/firmware_version.rs](../src/firmware/firmware_version.rs) - Version handling
- [tests/firmware/capabilities.rs](../tests/firmware/capabilities.rs) - Tests
