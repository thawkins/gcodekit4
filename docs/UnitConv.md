# Implementing Selectable Units (Metric/Imperial)

This guide describes the steps required to implement selectable measurement units (Metric vs. Imperial) for a dialog or panel in GCodeKit4. The system allows users to view and enter values in their preferred unit system (mm or inches), while the application internally maintains all values in millimeters.

## 1. UI Implementation (.slint)

### Add Unit Label Property
Add a property to your component to hold the current unit label. This will be set by the backend ("mm" or "in").

```slint
export component MyDialog inherits Window {
    // ... other properties
    
    // Property to hold the unit label
    in property <string> unit-label: "mm";
    
    // ...
}
```

### Remove Spinboxes ###
Spinboxes cant handle imperial values, so convert all spinboxes to TextEdit controls. 

### Update Input Properties
Ensure all dimension-related properties are of type `string`. This is necessary to support fractional Imperial inputs (e.g., "1 1/2") and formatted decimal strings.

```slint
    // Change from <float> or <length> to <string>
    in-out property <string> width-val: "300.0";
    in-out property <string> height-val: "180.0";
```

### Update UI Labels
Update text labels to dynamically display the unit.

```slint
    Text { 
        text: "Width (" + root.unit-label + "):"; 
    }
```

### Update Input Fields
Change `input-type` to `text` to allow for flexible input formats.

```slint
    LineEdit { 
        text <=> width-val;
        placeholder-text: "300.0";
        input-type: text; // Changed from decimal/number
    }
```

## 2. Backend Implementation (Rust)

### Imports
Import the unit conversion utilities from the core crate.

```rust
use gcodekit4_core::units::{MeasurementSystem, to_display_string, parse_from_string, get_unit_label};
use std::str::FromStr;
```

### Retrieve Measurement System
Get the user's preferred measurement system from the settings configuration.

```rust
let system = {
    let persistence = settings_persistence.borrow();
    let sys_str = &persistence.config().ui.measurement_system;
    MeasurementSystem::from_str(sys_str).unwrap_or(MeasurementSystem::Metric)
};
```

### Initialize Dialog
Set the unit label and convert default/initial values to the display format.

```rust
// Set the unit label on the dialog
dialog.set_unit_label(get_unit_label(system).into());

// Initialize values (converting from internal mm to display units)
let default_width_mm = 300.0;
dialog.set_width_val(to_display_string(default_width_mm, system).into());
```

### Processing User Input
When generating G-code or processing data, parse the string inputs back to millimeters.

```rust
dialog.on_generate_gcode(move || {
    if let Some(d) = dialog_weak.upgrade() {
        // Parse input string to mm
        // parse_from_string handles decimals and fractions (e.g., "1.5", "1 1/2")
        let width_mm = parse_from_string(&d.get_width_val(), system).unwrap_or(300.0);
        
        // Use width_mm (which is now guaranteed to be in mm) for logic/generation
        let params = MyParams {
            width: width_mm,
            // ...
        };
        
        // ...
    }
});
```

### Handling Dynamic Updates
If your dialog updates values programmatically (e.g., selecting a preset or device), ensure you convert those values to the current display system.

```rust
dialog.on_preset_selected(move |index| {
    // ... get preset ...
    let preset_width_mm = preset.width;
    
    // Convert to display string before setting on dialog
    dlg.set_width_val(to_display_string(preset_width_mm, system).into());
});
```

## Summary Checklist

- [ ] **UI**: Add `unit-label` property.
- [ ] **UI**: Convert dimension properties to `string`.
- [ ] **UI**: Update labels to use `unit-label`.
- [ ] **UI**: Set `input-type: text` on LineEdits.
- [ ] **Rust**: Import `gcodekit4_core::units` modules.
- [ ] **Rust**: Get `MeasurementSystem` from settings.
- [ ] **Rust**: Set `unit_label` on dialog.
- [ ] **Rust**: Use `to_display_string` for initialization and updates.
- [ ] **Rust**: Use `parse_from_string` for reading inputs.
