use crate::{CapabilityItem, ConfigSetting, MainWindow};
use gcodekit4::{CapabilityManager, list_ports};
use slint::Model;
use gcodekit4_ui::EditorBridge;
use crate::TextLine;

/// Get list of available serial ports
pub fn get_available_ports() -> anyhow::Result<Vec<slint::SharedString>> {
    match list_ports() {
        Ok(ports) => {
            let port_names: Vec<slint::SharedString> = ports
                .iter()
                .map(|p| slint::SharedString::from(p.port_name.clone()))
                .collect();

            if port_names.is_empty() {
                Ok(vec![slint::SharedString::from("No ports available")])
            } else {
                Ok(port_names)
            }
        }
        Err(_) => Ok(vec![slint::SharedString::from("Error reading ports")]),
    }
}

/// Copy text to clipboard using arboard crate
pub fn copy_to_clipboard(text: &str) -> bool {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            match clipboard.set_text(text.to_string()) {
                Ok(_) => {
                    // Keep clipboard alive for a moment to ensure managers see it
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    true
                }
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// Transform screen coordinates to canvas coordinates
/// Used when the displayed image size doesn't match the rendered image size
#[allow(dead_code)]
pub fn transform_screen_to_canvas(
    screen_x: f32,
    screen_y: f32,
    display_width: f32,
    display_height: f32,
    canvas_width: f32,
    canvas_height: f32,
) -> (f32, f32) {
    // Calculate scaling factors
    let scale_x = canvas_width / display_width;
    let scale_y = canvas_height / display_height;

    // Transform coordinates
    let canvas_x = screen_x * scale_x;
    let canvas_y = screen_y * scale_y;

    (canvas_x, canvas_y)
}

/// Snap world coordinates to whole millimeters
/// Rounds to nearest 1.0 mm
pub fn snap_to_mm(value: f64) -> f64 {
    (value + 0.5).floor()
}

/// Parse a GRBL setting line from $$ response
/// Format: $100=80.000
pub fn parse_grbl_setting_line(line: &str) -> Option<ConfigSetting> {
    let line = line.trim();
    if !line.starts_with('$') {
        return None;
    }

    // Remove the $ and split on =
    let line = &line[1..];
    let parts: Vec<&str> = line.split('=').collect();
    if parts.len() != 2 {
        return None;
    }

    let number = parts[0].parse::<i32>().ok()?;
    let value = parts[1].to_string();

    // Get metadata for the setting
    let (name, desc, unit, category) = get_grbl_setting_info(number);

    Some(ConfigSetting {
        number,
        name: slint::SharedString::from(name),
        value: slint::SharedString::from(value),
        unit: slint::SharedString::from(unit),
        description: slint::SharedString::from(desc),
        category: slint::SharedString::from(category),
        read_only: false,
    })
}

/// Get metadata for a GRBL setting number
pub fn get_grbl_setting_info(number: i32) -> (&'static str, &'static str, &'static str, &'static str) {
    match number {
        0 => (
            "Step pulse time",
            "Step pulse duration in microseconds",
            "μs",
            "System",
        ),
        1 => (
            "Step idle delay",
            "Step idle delay in milliseconds",
            "ms",
            "System",
        ),
        2 => ("Step pulse invert", "Step pulse invert mask", "", "System"),
        3 => (
            "Step direction invert",
            "Step direction invert mask",
            "",
            "System",
        ),
        4 => ("Invert step enable", "Invert step enable pin", "", "System"),
        5 => ("Invert limit pins", "Invert limit pins", "", "Limits"),
        6 => ("Invert probe pin", "Invert probe pin", "", "System"),
        10 => ("Status report", "Status report mask", "", "System"),
        11 => (
            "Junction deviation",
            "Junction deviation in mm",
            "mm",
            "System",
        ),
        12 => ("Arc tolerance", "Arc tolerance in mm", "mm", "System"),
        13 => ("Report in inches", "Report in inches", "", "System"),
        20 => ("Soft limits", "Enable soft limits", "", "Limits"),
        21 => ("Hard limits", "Enable hard limits", "", "Limits"),
        22 => ("Homing cycle", "Enable homing cycle", "", "Homing"),
        23 => (
            "Homing direction",
            "Homing direction invert mask",
            "",
            "Homing",
        ),
        24 => (
            "Homing locate feed",
            "Homing locate feed rate",
            "mm/min",
            "Homing",
        ),
        25 => (
            "Homing search seek",
            "Homing search seek rate",
            "mm/min",
            "Homing",
        ),
        26 => (
            "Homing debounce",
            "Homing switch debounce delay",
            "ms",
            "Homing",
        ),
        27 => (
            "Homing pull-off",
            "Homing switch pull-off distance",
            "mm",
            "Homing",
        ),
        30 => (
            "Max spindle speed",
            "Maximum spindle speed",
            "RPM",
            "Spindle",
        ),
        31 => (
            "Min spindle speed",
            "Minimum spindle speed",
            "RPM",
            "Spindle",
        ),
        32 => ("Laser mode", "Enable laser mode", "", "Spindle"),
        100 => (
            "X steps/mm",
            "X-axis steps per millimeter",
            "steps/mm",
            "Steps Per Unit",
        ),
        101 => (
            "Y steps/mm",
            "Y-axis steps per millimeter",
            "steps/mm",
            "Steps Per Unit",
        ),
        102 => (
            "Z steps/mm",
            "Z-axis steps per millimeter",
            "steps/mm",
            "Steps Per Unit",
        ),
        110 => ("X max rate", "X-axis maximum rate", "mm/min", "Max Rate"),
        111 => ("Y max rate", "Y-axis maximum rate", "mm/min", "Max Rate"),
        112 => ("Z max rate", "Z-axis maximum rate", "mm/min", "Max Rate"),
        120 => (
            "X acceleration",
            "X-axis acceleration",
            "mm/sec²",
            "Acceleration",
        ),
        121 => (
            "Y acceleration",
            "Y-axis acceleration",
            "mm/sec²",
            "Acceleration",
        ),
        122 => (
            "Z acceleration",
            "Z-axis acceleration",
            "mm/sec²",
            "Acceleration",
        ),
        130 => ("X max travel", "X-axis maximum travel", "mm", "Max Travel"),
        131 => ("Y max travel", "Y-axis maximum travel", "mm", "Max Travel"),
        132 => ("Z max travel", "Z-axis maximum travel", "mm", "Max Travel"),
        _ => (
            Box::leak(format!("${}", number).into_boxed_str()),
            "Unknown setting",
            "",
            "Other",
        ),
    }
}

/// Sync firmware capabilities to UI properties
pub fn sync_capabilities_to_ui(window: &MainWindow, capability_manager: &CapabilityManager) {
    let state = capability_manager.get_state();

    window.set_firmware_capabilities(slint::SharedString::from(state.get_summary()));
    window.set_cap_supports_arcs(state.supports_arcs);
    window.set_cap_supports_probing(state.supports_probing);
    window.set_cap_supports_tool_change(state.supports_tool_change);
    window.set_cap_supports_variable_spindle(state.supports_variable_spindle);
    window.set_cap_supports_homing(state.supports_homing);
    window.set_cap_supports_overrides(state.supports_overrides);
    window.set_cap_max_axes(state.max_axes as i32);
    window.set_cap_coordinate_systems(state.coordinate_systems as i32);
}

/// Update device info panel with firmware and capabilities
pub fn update_device_info_panel(
    window: &MainWindow,
    firmware_type: gcodekit4::firmware::firmware_version::FirmwareType,
    version: gcodekit4::firmware::firmware_version::SemanticVersion,
    capability_manager: &CapabilityManager,
) {
    use slint::{ModelRc, VecModel};
    use std::rc::Rc;

    // Update capability manager with detected firmware
    capability_manager.update_firmware(firmware_type, version.clone());

    // Set firmware type and version
    window.set_device_firmware_type(slint::SharedString::from(format!("{:?}", firmware_type)));
    window.set_device_firmware_version(slint::SharedString::from(version.to_string()));
    window.set_device_name(slint::SharedString::from(format!(
        "{:?} Device",
        firmware_type
    )));

    // Get capabilities state
    let state = capability_manager.get_state();

    // Build capabilities list
    let mut capabilities = Vec::new();

    capabilities.push(CapabilityItem {
        name: "Arc Support (G2/G3)".into(),
        enabled: state.supports_arcs,
        notes: "Circular interpolation commands".into(),
    });

    capabilities.push(CapabilityItem {
        name: "Variable Spindle (M3/M4 S)".into(),
        enabled: state.supports_variable_spindle,
        notes: "PWM spindle speed control".into(),
    });

    capabilities.push(CapabilityItem {
        name: "Probing (G38.x)".into(),
        enabled: state.supports_probing,
        notes: "Touch probe operations".into(),
    });

    capabilities.push(CapabilityItem {
        name: "Tool Change (M6 T)".into(),
        enabled: state.supports_tool_change,
        notes: "Automatic tool changing".into(),
    });

    capabilities.push(CapabilityItem {
        name: "Homing Cycle ($H)".into(),
        enabled: state.supports_homing,
        notes: "Machine homing to limit switches".into(),
    });

    capabilities.push(CapabilityItem {
        name: "Feed/Spindle Overrides".into(),
        enabled: state.supports_overrides,
        notes: "Real-time adjustment of feed and spindle".into(),
    });

    capabilities.push(CapabilityItem {
        name: "Laser Mode (M3/M4)".into(),
        enabled: state.supports_laser,
        notes: "Dynamic laser power control for engraving/cutting".into(),
    });

    capabilities.push(CapabilityItem {
        name: format!("{} Axes Support", state.max_axes).into(),
        enabled: state.max_axes > 0,
        notes: format!("Maximum {} axes (X,Y,Z,A,B,C)", state.max_axes).into(),
    });

    capabilities.push(CapabilityItem {
        name: format!("{} Coordinate Systems", state.coordinate_systems).into(),
        enabled: state.coordinate_systems > 0,
        notes: "Work coordinate systems (G54-G59)".into(),
    });

    let capabilities_model = Rc::new(VecModel::from(capabilities));
    window.set_device_capabilities(ModelRc::from(capabilities_model));
}

/// Update visible lines in the editor
pub fn update_visible_lines(window: &MainWindow, editor_bridge: &EditorBridge) {
    let (start_line, end_line) = editor_bridge.viewport_range();
    let mut visible_lines = Vec::new();
    for i in start_line..end_line {
        if let Some(content) = editor_bridge.get_line_at(i) {
            visible_lines.push(TextLine {
                line_number: (i + 1) as i32,
                content: slint::SharedString::from(content),
                is_dirty: false,
            });
        }
    }
    let model = std::rc::Rc::new(slint::VecModel::from(visible_lines));
    window.set_visible_lines(slint::ModelRc::new(model));
}

