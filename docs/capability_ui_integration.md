# Firmware Capability UI Integration

## Overview

The firmware capabilities database is now integrated into the UI, enabling version-aware feature management. The UI automatically enables/disables features based on the connected controller's capabilities.

## Architecture

### Components

1. **CapabilityManager** (`src/firmware/capability_manager.rs`)
   - Manages firmware capabilities
   - Thread-safe state management
   - Provides UI-friendly capability queries

2. **UI Properties** (`src/ui.slint`)
   - `firmware-capabilities`: Summary string
   - `cap-supports-*`: Boolean flags for features
   - `cap-max-axes`: Number of supported axes
   - `cap-coordinate-systems`: Number of work coordinate systems

3. **Main Integration** (`src/main.rs`)
   - Creates CapabilityManager instance
   - Updates capabilities on firmware detection
   - Syncs capability state to UI properties

## UI Properties Added

```slint
in property <string> firmware-capabilities: "No firmware detected";
in property <bool> cap-supports-arcs: false;
in property <bool> cap-supports-probing: false;
in property <bool> cap-supports-tool-change: false;
in property <bool> cap-supports-variable-spindle: false;
in property <bool> cap-supports-homing: false;
in property <bool> cap-supports-overrides: false;
in property <int> cap-max-axes: 3;
in property <int> cap-coordinate-systems: 1;
```

## Usage in UI Panels

### Disabling Unsupported Features

```slint
Button {
    text: "Run Probing";
    enabled: root.cap-supports-probing;
    clicked => { run_probing(); }
}
```

### Conditional Display

```slint
if root.cap-supports-tool-change: VerticalBox {
    Text { text: "Tool Change"; }
    Button { text: "Change Tool"; }
}
```

### Axis Display

```slint
for axis in [0, 1, 2, 3, 4, 5]: HorizontalBox {
    visible: axis < root.cap-max-axes;
    Text { text: axis-names[axis]; }
    // ... axis controls
}
```

### Coordinate System Selection

```slint
ComboBox {
    model: root.cap-coordinate-systems;
    // Shows G54-G59 based on capabilities
}
```

## Integration Points

### Machine Control Panel

- **Homing Button**: Enabled only if `cap-supports-homing`
- **Probe Button**: Enabled only if `cap-supports-probing`
- **Tool Change**: Shown only if `cap-supports-tool-change`
- **Spindle Controls**: Variable speed only if `cap-supports-variable-spindle`

### G-Code Editor

- **Arc Commands**: Validation warns if `!cap-supports-arcs`
- **Coordinate Systems**: Dropdown limited to `cap-coordinate-systems`

### Advanced Features

- **Override Controls**: Shown only if `cap-supports-overrides`
- **Macro Support**: Enabled only if firmware supports macros

## Example: Machine Control Integration

```slint
// machine_control.slint
export component MachineControlPanel {
    in property <bool> cap-supports-homing;
    in property <bool> cap-supports-probing;
    
    // Homing section
    if cap-supports-homing: VerticalBox {
        Text { text: "Homing"; font-weight: bold; }
        HorizontalBox {
            Button { text: "Home All"; clicked => { home-all(); } }
            Button { text: "Home X"; clicked => { home-x(); } }
            Button { text: "Home Y"; clicked => { home-y(); } }
            Button { text: "Home Z"; clicked => { home-z(); } }
        }
    }
    
    // Probing section
    if cap-supports-probing: VerticalBox {
        Text { text: "Probing"; font-weight: bold; }
        Button { 
            text: "Probe Z";
            clicked => { probe-z(); }
        }
    }
    
    // If no homing support, show message
    if !cap-supports-homing: Text {
        text: "Homing not supported by this firmware";
        color: gray;
        font-size: 12px;
    }
}
```

## Capability Detection Flow

1. **Connection Established**
   - Controller sends version string (e.g., "Grbl 1.1")
   
2. **Firmware Detection**
   - Parse firmware type and version
   - Call `capability_manager.update_firmware(type, version)`
   
3. **Capability Loading**
   - CapabilityManager queries database
   - Updates internal state
   
4. **UI Update**
   - Read capability state
   - Update all UI properties
   - Features automatically enable/disable

5. **User Interaction**
   - UI components check capability properties
   - Disabled features show tooltips explaining why

## Future Enhancements

### Planned Features

1. **Tooltips**: Explain why features are disabled
   ```slint
   Button {
       text: "Run Probing";
       enabled: cap-supports-probing;
       tooltip: cap-supports-probing ? 
           "Click to start probing sequence" :
           "Probing not supported by current firmware";
   }
   ```

2. **Capability Browser**: UI panel showing all capabilities
   - Feature matrix for connected firmware
   - Comparison with other firmware versions
   - Upgrade recommendations

3. **G-Code Validation**: Real-time validation
   - Warn about unsupported commands
   - Suggest alternatives
   - Auto-fix common issues

4. **Smart Defaults**: Set defaults based on capabilities
   - Hide unsupported axis controls
   - Adjust feed rate limits
   - Configure coordinate system dropdowns

## Testing

### Manual Testing Steps

1. **Connect to GRBL 0.9**:
   - Verify arc commands are disabled
   - Verify probing is disabled
   - Verify only 1 coordinate system shown

2. **Connect to GRBL 1.1**:
   - Verify all features enabled
   - Verify 6 coordinate systems shown
   - Verify override controls visible

3. **Connect to TinyG**:
   - Verify 4 axes shown
   - Verify macro features enabled
   - Verify 9 coordinate systems shown

4. **Disconnect**:
   - Verify capabilities reset to defaults
   - Verify features disabled

### Automated Tests

```rust
#[test]
fn test_capability_ui_integration() {
    let manager = CapabilityManager::new();
    
    // Simulate GRBL 1.1 connection
    manager.update_firmware(
        FirmwareType::Grbl,
        SemanticVersion::new(1, 1, 0),
    );
    
    let state = manager.get_state();
    assert!(state.supports_arcs);
    assert!(state.supports_probing);
    assert_eq!(state.coordinate_systems, 6);
    
    // Simulate disconnect
    manager.reset();
    
    let state = manager.get_state();
    assert!(!state.detected);
    assert!(!state.supports_arcs);
}
```

## Benefits

1. **Better UX**: Users only see features their controller supports
2. **Fewer Errors**: Prevents sending unsupported commands
3. **Educational**: Shows firmware differences
4. **Future-Proof**: Easy to add new firmware versions
5. **Professional**: Industry-standard CAM software behavior

## Related Documentation

- [Firmware Capabilities Database](firmware_capabilities.md)
- [Capability Manager API](capability_manager_api.md)
- [UI Component Guidelines](ui_components.md)
