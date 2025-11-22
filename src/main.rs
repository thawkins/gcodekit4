use gcodekit4::{
    init_logging, list_ports, BoxParameters, BoxType, CapabilityManager, Communicator,
    ConnectionDriver, ConnectionParams, ConsoleListener, DeviceConsoleManager, DeviceMessageType,
    FingerJointSettings, FingerStyle, FirmwareSettingsIntegration, GcodeEditor, JigsawPuzzleMaker,
    KeyDividerType, PuzzleParameters, SerialCommunicator, SerialParity, SettingsCategory, SettingsController,
    SettingsDialog, SettingsManager, SettingsPersistence, TabbedBoxMaker, BUILD_DATE, VERSION,
    SpeedsFeedsCalculator, SpoilboardSurfacingGenerator, SpoilboardSurfacingParameters,
};
use gcodekit4_devicedb::{DeviceManager, DeviceUiController, DeviceProfileUiModel as DbDeviceProfile};
use gcodekit4_ui::EditorBridge;
use slint::{Model, VecModel};
use std::cell::RefCell;
#[allow(unused_imports)]
use std::error::Error;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::warn;

slint::include_modules!();

slint::slint! {
    export { TabbedBoxDialog } from "crates/gcodekit4-camtools/ui/tabbed_box_dialog.slint";
    export { JigsawPuzzleDialog } from "crates/gcodekit4-camtools/ui/jigsaw_puzzle_dialog.slint";
    export { SpoilboardSurfacingDialog } from "crates/gcodekit4-camtools/ui/spoilboard_surfacing_dialog.slint";
    export { LaserEngraverDialog } from "crates/gcodekit4-camtools/ui/laser_engraver_dialog.slint";
    export { VectorEngraverDialog } from "crates/gcodekit4-camtools/ui/vector_engraver_dialog.slint";
    export { ErrorDialog } from "crates/gcodekit4-ui/ui_panels/error_dialog.slint";
}

/// Copy text to clipboard using arboard crate
fn copy_to_clipboard(text: &str) -> bool {
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
fn transform_screen_to_canvas(
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
fn snap_to_mm(value: f64) -> f64 {
    (value + 0.5).floor()
}

/// Parse a GRBL setting line from $$ response
/// Format: $100=80.000
fn parse_grbl_setting_line(line: &str) -> Option<ConfigSetting> {
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
fn get_grbl_setting_info(number: i32) -> (&'static str, &'static str, &'static str, &'static str) {
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
            format!("${}", number).leak(),
            "Unknown setting",
            "",
            "Other",
        ),
    }
}

/// Sync firmware capabilities to UI properties
fn sync_capabilities_to_ui(window: &MainWindow, capability_manager: &CapabilityManager) {
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
fn update_device_info_panel(
    window: &MainWindow,
    firmware_type: gcodekit4::firmware::firmware_version::FirmwareType,
    version: gcodekit4::firmware::firmware_version::SemanticVersion,
    capability_manager: &CapabilityManager,
) {
    use slint::{ModelRc, VecModel};

    // Update capability manager with detected firmware
    capability_manager.update_firmware(firmware_type, version);

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

/// Update designer UI with current shapes from state
fn update_designer_ui(window: &MainWindow, state: &mut gcodekit4::DesignerState) {
    // Get canvas dimensions from window
    let canvas_width = window.get_designer_canvas_width().max(100.0) as u32;
    let canvas_height = window.get_designer_canvas_height().max(100.0) as u32;

    // Update viewport canvas size to match actual rendering size
    state
        .canvas
        .viewport_mut()
        .set_canvas_size(canvas_width as f64, canvas_height as f64);

    // Render canvas using SVG paths
    let crosshair_data = gcodekit4::designer::svg_renderer::render_crosshair(
        &state.canvas,
        canvas_width,
        canvas_height,
    );
    let (grid_data, grid_size) = if state.show_grid {
        gcodekit4::designer::svg_renderer::render_grid(
            &state.canvas,
            canvas_width,
            canvas_height,
        )
    } else {
        (String::new(), 0.0)
    };
    let origin_data = gcodekit4::designer::svg_renderer::render_origin(
        &state.canvas,
        canvas_width,
        canvas_height,
    );
    let shapes_data = gcodekit4::designer::svg_renderer::render_shapes(
        &state.canvas,
        canvas_width,
        canvas_height,
    );
    let selected_shapes_data = gcodekit4::designer::svg_renderer::render_selected_shapes(
        &state.canvas,
        canvas_width,
        canvas_height,
    );
    let handles_data = gcodekit4::designer::svg_renderer::render_selection_handles(
        &state.canvas,
        canvas_width,
        canvas_height,
    );

    // Update UI with SVG path data
    window.set_designer_canvas_crosshair_data(slint::SharedString::from(crosshair_data));
    window.set_designer_canvas_grid_data(slint::SharedString::from(grid_data));
    window.set_designer_canvas_origin_data(slint::SharedString::from(origin_data));
    window.set_designer_show_grid(state.show_grid);
    if grid_size > 0.0 {
        window.set_designer_grid_size(slint::SharedString::from(format!("{}mm", grid_size)));
    }
    window.set_designer_canvas_shapes_data(slint::SharedString::from(shapes_data));
    window
        .set_designer_canvas_selected_shapes_data(slint::SharedString::from(selected_shapes_data));
    window.set_designer_canvas_handles_data(slint::SharedString::from(handles_data));

    // Still update shapes array for metadata (could be used for debugging/info)
    let shapes: Vec<crate::DesignerShape> = state
        .canvas
        .shapes()
        .iter()
        .map(|obj| {
            let (x1, y1, x2, y2) = obj.shape.bounding_box();
            let shape_type = match obj.shape.shape_type() {
                gcodekit4::ShapeType::Rectangle => 0,
                gcodekit4::ShapeType::Circle => 1,
                gcodekit4::ShapeType::Line => 2,
                gcodekit4::ShapeType::Ellipse => 3,
                gcodekit4::ShapeType::Polyline => 4,
                gcodekit4::ShapeType::Path => 7,
                gcodekit4::ShapeType::Text => 6,
            };
            crate::DesignerShape {
                id: obj.id as i32,
                x: x1 as f32,
                y: y1 as f32,
                width: (x2 - x1).abs() as f32,
                height: (y2 - y1).abs() as f32,
                radius: (((x2 - x1).abs() / 2.0).max((y2 - y1).abs() / 2.0)) as f32,
                x2: x2 as f32,
                y2: y2 as f32,
                shape_type,
                selected: obj.selected,
                step_down: obj.step_down as f32,
                step_in: obj.step_in as f32,
            }
        })
        .collect();
    for _ in &shapes {}
    // Force UI to recognize the change by clearing first
    window.set_designer_shapes(slint::ModelRc::from(Rc::new(slint::VecModel::from(Vec::<
        crate::DesignerShape,
    >::new(
    )))));
    let shapes_model = Rc::new(slint::VecModel::from(shapes.clone()));
    window.set_designer_shapes(slint::ModelRc::from(shapes_model));

    // Update shape indicator with selected shape info
    if let Some(id) = state.canvas.selected_id() {
        if let Some(obj) = state.canvas.shapes().iter().find(|o| o.id == id) {
            let (x1, y1, x2, y2) = obj.shape.bounding_box();
            let width = (x2 - x1).abs();
            let height = (y2 - y1).abs();
            let radius = if let Some(c) = obj.shape.as_any().downcast_ref::<gcodekit4::designer::shapes::Circle>() {
                c.radius
            } else {
                0.0
            };
            
            let shape_type = match obj.shape.shape_type() {
                gcodekit4::ShapeType::Rectangle => 0,
                gcodekit4::ShapeType::Circle => 1,
                gcodekit4::ShapeType::Line => 2,
                gcodekit4::ShapeType::Ellipse => 3,
                gcodekit4::ShapeType::Polyline => 4,
                gcodekit4::ShapeType::Path => 5,
                gcodekit4::ShapeType::Text => 6,
            };

            window.set_designer_selected_shape_x(x1 as f32);
            window.set_designer_selected_shape_y(y1 as f32);
            window.set_designer_selected_shape_w(width as f32);
            window.set_designer_selected_shape_h(height as f32);
            window.set_designer_selected_shape_type(shape_type);
            window.set_designer_selected_shape_radius(radius as f32);
            
            let is_pocket = obj.operation_type == gcodekit4::designer::shapes::OperationType::Pocket;
            window.set_designer_selected_shape_is_pocket(is_pocket);
            window.set_designer_selected_shape_pocket_depth(obj.pocket_depth as f32);
            window.set_designer_selected_shape_step_down(obj.step_down as f32);
            window.set_designer_selected_shape_step_in(obj.step_in as f32);
            
            if let Some(text) = obj.shape.as_any().downcast_ref::<gcodekit4::designer::shapes::TextShape>() {
                window.set_designer_selected_shape_text_content(slint::SharedString::from(&text.text));
                window.set_designer_selected_shape_font_size(text.font_size as f32);
            } else {
                window.set_designer_selected_shape_text_content(slint::SharedString::from(""));
                window.set_designer_selected_shape_font_size(12.0);
            }
        }
    } else {
        // No shape selected - clear indicators
        window.set_designer_selected_shape_x(0.0);
        window.set_designer_selected_shape_y(0.0);
        window.set_designer_selected_shape_w(0.0);
        window.set_designer_selected_shape_h(0.0);
        window.set_designer_selected_shape_type(0);
        window.set_designer_selected_shape_radius(5.0);
        window.set_designer_selected_shape_is_pocket(false);
        window.set_designer_selected_shape_pocket_depth(0.0);
        window.set_designer_selected_shape_step_down(0.0);
        window.set_designer_selected_shape_text_content(slint::SharedString::from(""));
        window.set_designer_selected_shape_font_size(12.0);
    }

    // Increment update counter to force UI re-render
    let mut ui_state = window.get_designer_state();
    let counter = ui_state.update_counter + 1;
    ui_state.update_counter = counter;
    window.set_designer_state(ui_state);
}

/// Parse GRBL status response and extract position
/// Format: <Idle|MPos:10.000,20.000,0.000|WPos:10.000,20.000,0.000|...>

/// Helper function to convert editor lines to Slint TextLine and update window
fn update_visible_lines(window: &MainWindow, bridge: &EditorBridge) {
    let lines_data = bridge.get_visible_lines_data();

    let lines: Vec<TextLine> = lines_data
        .into_iter()
        .map(|(line_number, content, is_dirty)| TextLine {
            line_number,
            content: content.into(),
            is_dirty,
        })
        .collect();

    let _ = !lines.is_empty();

    window.set_visible_lines(slint::ModelRc::new(VecModel::from(lines)));
}

fn main() -> anyhow::Result<()> {
    // Initialize logging
    init_logging()?;


    let main_window = MainWindow::new().map_err(|e| anyhow::anyhow!("UI Error: {}", e))?;

    // Set window to maximized state
    let window = main_window.window();
    window.set_maximized(true);

    // Initialize about dialog properties
    main_window.set_app_version(slint::SharedString::from(VERSION));
    main_window.set_app_build_date(slint::SharedString::from(BUILD_DATE));

    // Get initial list of ports
    let ports = get_available_ports()?;
    let ports_model = Rc::new(VecModel::from(ports.clone()));
    main_window.set_available_ports(slint::ModelRc::from(ports_model.clone()));

    // Initialize selected port if we have ports
    if !ports.is_empty() {
        let first_port: slint::SharedString = ports[0].clone();
        main_window.set_selected_port(first_port);
    }

    // Initialize status panel
    main_window.set_connected(false);
    main_window.set_connection_status(slint::SharedString::from("Disconnected"));
    main_window.set_device_version(slint::SharedString::from("Not Connected"));
    main_window.set_machine_state(slint::SharedString::from("DISCONNECTED"));
    main_window.set_position_x(0.0);
    main_window.set_position_y(0.0);
    main_window.set_position_z(0.0);

    // Initialize capability properties with defaults (no firmware detected)
    main_window.set_firmware_capabilities(slint::SharedString::from("No firmware detected"));
    main_window.set_cap_supports_arcs(false);
    main_window.set_cap_supports_probing(false);
    main_window.set_cap_supports_tool_change(false);
    main_window.set_cap_supports_variable_spindle(false);
    main_window.set_cap_supports_homing(false);
    main_window.set_cap_supports_overrides(false);
    main_window.set_cap_max_axes(3);
    main_window.set_cap_coordinate_systems(1);

    // Shared state for communicator (Arc<Mutex> for thread-safe sharing)
    let communicator = Arc::new(Mutex::new(SerialCommunicator::new()));

    // Flag to control status polling
    let status_polling_active = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let status_polling_stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Shared G-code send state (for polling thread to process)
    use std::collections::VecDeque;
    #[derive(Debug)]
    struct GcodeSendState {
        lines: VecDeque<String>,
        pending_bytes: usize,
        line_lengths: VecDeque<usize>,
        total_sent: usize,
        total_lines: usize,
    }
    let gcode_send_state = Arc::new(Mutex::new(GcodeSendState {
        lines: VecDeque::new(),
        pending_bytes: 0,
        line_lengths: VecDeque::new(),
        total_sent: 0,
        total_lines: 0,
    }));

    // Initialize device console manager early to register listeners
    // Use Arc since communicator listeners need Arc for thread-safe sharing
    let console_manager = std::sync::Arc::new(DeviceConsoleManager::new());

    // Shared state for detected firmware (thread-safe)
    let detected_firmware = std::sync::Arc::new(std::sync::Mutex::new(
        None::<gcodekit4::firmware::firmware_detector::FirmwareDetectionResult>,
    ));

    // Create and register console listener with communicator and firmware state
    let console_listener = ConsoleListener::new_with_firmware_state(
        console_manager.clone(),
        detected_firmware.clone(),
    );
    communicator.lock().unwrap().add_listener(console_listener);

    // Shared state for settings dialog
    let settings_dialog = Rc::new(RefCell::new(SettingsDialog::new()));

    // Initialize Designer state (Phase 2)
    let designer_mgr = Rc::new(RefCell::new(gcodekit4::DesignerState::new()));

    // Initialize designer toolpath parameters in UI and backend
    main_window.set_designer_feed_rate(120.0);
    main_window.set_designer_spindle_speed(3000.0);
    main_window.set_designer_tool_diameter(3.175);
    main_window.set_designer_cut_depth(-5.0);

    // Sync initial values to backend
    {
        let mut state = designer_mgr.borrow_mut();
        state.toolpath_generator.set_feed_rate(120.0);
        state.toolpath_generator.set_spindle_speed(3000);
        state.toolpath_generator.set_tool_diameter(3.175);
        state.toolpath_generator.set_cut_depth(-5.0);
    }

    // Initialize Materials Manager backend
    let materials_backend = Rc::new(RefCell::new(gcodekit4::ui::MaterialsManagerBackend::new()));

    // Load initial materials into UI
    {
        let backend = materials_backend.borrow();
        let materials = backend.get_all_materials();
        let materials_ui: Vec<MaterialData> = materials
            .iter()
            .map(|m| MaterialData {
                id: m.id.0.clone().into(),
                name: m.name.clone().into(),
                category: format!("{}", m.category).into(),
                subcategory: m.subcategory.clone().into(),
                description: m.description.clone().into(),
                density: m.density,
                machinability_rating: m.machinability_rating as i32,
                tensile_strength: m.tensile_strength.unwrap_or(0.0),
                melting_point: m.melting_point.unwrap_or(0.0),
                chip_type: format!("{:?}", m.chip_type).into(),
                heat_sensitivity: format!("{:?}", m.heat_sensitivity).into(),
                abrasiveness: format!("{:?}", m.abrasiveness).into(),
                surface_finish: format!("{:?}", m.surface_finish).into(),
                dust_hazard: format!("{:?}", m.dust_hazard).into(),
                fume_hazard: format!("{:?}", m.fume_hazard).into(),
                coolant_required: m.coolant_required,
                custom: m.custom,
                notes: m.notes.clone().into(),
            })
            .collect();
        main_window.set_materials(slint::ModelRc::new(VecModel::from(materials_ui)));
    }

    // Initialize Custom G-code Editor with undo/redo
    // Start with a reasonable default viewport height
    // The viewport will adapt based on scroll position to show the right lines
    // Using 1000px (50 lines visible) as a safe default for most screens
    let editor_bridge = Rc::new(EditorBridge::new(1000.0, 20.0));

    // Initialize editor with empty content
    editor_bridge.load_text("");

    // Set initial editor state in UI
    main_window.set_can_undo(false);
    main_window.set_can_redo(false);
    main_window.set_cursor_line(1);
    main_window.set_cursor_column(1);
    main_window.set_total_lines(0);

    // Update visible lines
    update_visible_lines(&main_window, &editor_bridge);

    // Initialize CNC Tools Manager backend
    let tools_backend = Rc::new(RefCell::new(gcodekit4::ui::ToolsManagerBackend::new()));

    // Load initial tools into UI
    {
        let backend = tools_backend.borrow();
        let tools = backend.get_all_tools();
        let tools_ui: Vec<ToolData> = tools
            .iter()
            .map(|t| ToolData {
                id: t.id.0.clone().into(),
                number: t.number as i32,
                name: t.name.clone().into(),
                tool_type: format!("{}", t.tool_type).into(),
                material: format!("{}", t.material).into(),
                diameter: t.diameter,
                length: t.length,
                flute_length: t.flute_length,
                shaft_diameter: t.shaft_diameter.unwrap_or(t.diameter),
                flutes: t.flutes as i32,
                coating: t
                    .coating
                    .as_ref()
                    .map(|c| format!("{}", c))
                    .unwrap_or_else(|| "None".to_string())
                    .into(),
                manufacturer: t.manufacturer.clone().unwrap_or_default().into(),
                part_number: t.part_number.clone().unwrap_or_default().into(),
                description: t.description.clone().into(),
                custom: t.custom,
                notes: t.notes.clone().into(),
            })
            .collect();
        main_window.set_cnc_tools(slint::ModelRc::new(VecModel::from(tools_ui)));
    }

    // Shift key state for snapping in designer
    let shift_pressed = Rc::new(RefCell::new(false));

    // Initialize designer UI state with Select mode (0) as default
    let initial_designer_state = crate::DesignerState {
        mode: 0,
        zoom: 1.0,
        pan_x: 0.0,
        pan_y: 0.0,
        selected_id: 0,
        update_counter: 0,
    };
    main_window.set_designer_state(initial_designer_state);
    main_window.set_designer_show_grid(true); // Default to showing grid

    // Initial UI update to show grid/origin
    {
        let mut state = designer_mgr.borrow_mut();
        // Ensure view is reset to default (bottom-left origin) on startup
        state.reset_view();
        update_designer_ui(&main_window, &mut state);
    }

    // Handle designer toggle grid
    {
        let designer_mgr = designer_mgr.clone();
        let window_weak = main_window.as_weak();
        main_window.on_designer_toggle_grid(move || {
            if let Some(window) = window_weak.upgrade() {
                let mut state = designer_mgr.borrow_mut();
                state.toggle_grid();
                update_designer_ui(&window, &mut state);
            }
        });
    }

    // Shared state for settings persistence
    let settings_persistence = Rc::new(RefCell::new(SettingsPersistence::new()));

    // Shared state for firmware settings integration
    let firmware_integration = Rc::new(RefCell::new(FirmwareSettingsIntegration::new(
        "GRBL", "1.1",
    )));

    // Initialize capability manager for firmware-aware UI
    let capability_manager = Rc::new(CapabilityManager::new());

    // Load firmware settings
    {
        let mut fw_integration = firmware_integration.borrow_mut();
        if fw_integration.load_grbl_defaults().is_err() {
        } else {
            // Populate dialog with firmware parameters
            let mut dialog = settings_dialog.borrow_mut();
            fw_integration.populate_dialog(&mut dialog);
            drop(dialog);
        }
    }

    // Add initial messages to console
    console_manager.add_message(DeviceMessageType::Success, "GCodeKit4 initialized");
    console_manager.add_message(DeviceMessageType::Output, "Ready for operation");

    // Initialize console output in UI with initial messages
    let console_output = console_manager.get_output();
    main_window.set_console_output(slint::SharedString::from(console_output));

    // Initialize console with default max lines (500)
    console_manager.set_max_lines(500);

    // Initialize G-Code editor
    let gcode_editor = Rc::new(GcodeEditor::new());

    // Load sample G-Code content for demonstration
    // Note: Don't load sample gcode - start with empty editor
    // Let user choose to open a file or type their own

    // Load settings from config file if it exists
    {
        let mut persistence = settings_persistence.borrow_mut();
        let config_path = match SettingsManager::config_file_path() {
            Ok(path) => path,
            Err(_) => std::path::PathBuf::new(),
        };

        if config_path.exists() {
            if let Ok(loaded_persistence) = SettingsPersistence::load_from_file(&config_path) {
                *persistence = loaded_persistence;
            }
        }

        // Populate dialog with settings
        let mut dialog = settings_dialog.borrow_mut();
        persistence.populate_dialog(&mut dialog);
        drop(dialog);
    }

    // Initialize Settings Controller
    let settings_controller = Rc::new(SettingsController::new(
        settings_dialog.clone(),
        settings_persistence.clone(),
    ));

    // Initialize Device Manager
    let device_manager = std::sync::Arc::new(DeviceManager::new(PathBuf::from("devices.json")));
    if let Err(e) = device_manager.load() {
        warn!("Failed to load device profiles: {}", e);
    }
    let device_ui_controller = Rc::new(DeviceUiController::new(
        device_manager.clone(),
    ));

    // Bind Device Manager callbacks
    {
        let controller = device_ui_controller.clone();
        let window_weak = main_window.as_weak();
        main_window.on_load_device_profiles(move || {
            let profiles = controller.get_ui_profiles();
            let slint_profiles: Vec<DeviceProfileUiModel> = profiles.iter().map(|p| {
                DeviceProfileUiModel {
                    id: p.id.clone().into(),
                    name: p.name.clone().into(),
                    description: p.description.clone().into(),
                    device_type: p.device_type.clone().into(),
                    controller_type: p.controller_type.clone().into(),
                    x_min: p.x_min.clone().into(),
                    x_max: p.x_max.clone().into(),
                    y_min: p.y_min.clone().into(),
                    y_max: p.y_max.clone().into(),
                    z_min: p.z_min.clone().into(),
                    z_max: p.z_max.clone().into(),
                    has_spindle: p.has_spindle,
                    has_laser: p.has_laser,
                    has_coolant: p.has_coolant,
                    cnc_spindle_watts: p.cnc_spindle_watts.clone().into(),
                    laser_watts: p.laser_watts.clone().into(),
                    connection_type: p.connection_type.clone().into(),
                    baud_rate: p.baud_rate.clone().into(),
                    port: p.port.clone().into(),
                    tcp_port: p.tcp_port.clone().into(),
                    timeout_ms: p.timeout_ms.clone().into(),
                    auto_reconnect: p.auto_reconnect,
                    is_active: p.is_active,
                }
            }).collect();
            
            if let Some(window) = window_weak.upgrade() {
                window.set_device_profiles(slint::ModelRc::new(VecModel::from(slint_profiles)));
            }
        });
    }
    
    {
        let controller = device_ui_controller.clone();
        let window_weak = main_window.as_weak();
        main_window.on_add_device_profile(move || {
            match controller.create_new_profile() {
                Ok(new_id) => {
                    if let Some(window) = window_weak.upgrade() {
                        window.invoke_load_device_profiles();
                        
                        // Find the index of the new profile
                        let profiles = controller.get_ui_profiles();
                        if let Some(index) = profiles.iter().position(|p| p.id == new_id) {
                            window.set_selected_device_index(-1);
                            let window_weak_2 = window_weak.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(window) = window_weak_2.upgrade() {
                                    window.set_selected_device_index(index as i32);
                                }
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to create new profile: {}", e);
                    if let Some(window) = window_weak.upgrade() {
                        window.invoke_load_device_profiles();
                    }
                }
            }
        });
    }

    {
        let controller = device_ui_controller.clone();
        let window_weak = main_window.as_weak();
        main_window.on_save_device_profile(move |profile| {
            let db_profile = DbDeviceProfile {
                id: profile.id.into(),
                name: profile.name.into(),
                description: profile.description.into(),
                device_type: profile.device_type.into(),
                controller_type: profile.controller_type.into(),
                x_min: profile.x_min.into(),
                x_max: profile.x_max.into(),
                y_min: profile.y_min.into(),
                y_max: profile.y_max.into(),
                z_min: profile.z_min.into(),
                z_max: profile.z_max.into(),
                has_spindle: profile.has_spindle,
                has_laser: profile.has_laser,
                has_coolant: profile.has_coolant,
                cnc_spindle_watts: profile.cnc_spindle_watts.into(),
                laser_watts: profile.laser_watts.into(),
                connection_type: profile.connection_type.into(),
                baud_rate: profile.baud_rate.into(),
                port: profile.port.into(),
                tcp_port: profile.tcp_port.into(),
                timeout_ms: profile.timeout_ms.into(),
                auto_reconnect: profile.auto_reconnect,
                is_active: profile.is_active,
            };
            
            if let Err(e) = controller.update_profile_from_ui(db_profile) {
                warn!("Failed to save profile: {}", e);
            }
            
            // Reload profiles to update UI
            if let Some(window) = window_weak.upgrade() {
                window.invoke_load_device_profiles();
            }
        });
    }

    {
        let controller = device_ui_controller.clone();
        let window_weak = main_window.as_weak();
        main_window.on_delete_device_profile(move |id| {
            if let Err(e) = controller.delete_profile(&id) {
                warn!("Failed to delete profile: {}", e);
            }
            if let Some(window) = window_weak.upgrade() {
                window.invoke_load_device_profiles();
            }
        });
    }

    {
        let controller = device_ui_controller.clone();
        let window_weak = main_window.as_weak();
        main_window.on_set_active_device_profile(move |id| {
            if let Err(e) = controller.set_active_profile(&id) {
                warn!("Failed to set active profile: {}", e);
            }
            if let Some(window) = window_weak.upgrade() {
                window.invoke_load_device_profiles();
            }
        });
    }

    // Bind Settings Controller callbacks
    {
        let controller = settings_controller.clone();
        let window_weak = main_window.as_weak();
        main_window.on_config_retrieve_settings(move || {
            if let Some(window) = window_weak.upgrade() {
                let category_str = window.get_settings_category();
                let category = if category_str == "All" {
                    None
                } else {
                    Some(SettingsController::get_category_from_string(&category_str))
                };
                
                let settings = controller.get_settings_for_ui(category);
                
                let slint_settings: Vec<SettingItem> = settings.iter().map(|s| {
                    SettingItem {
                        id: s.id.clone().into(),
                        name: s.name.clone().into(),
                        value: s.value.clone().into(),
                        value_type: s.value_type.clone().into(),
                        category: s.category.clone().into(),
                        description: s.description.clone().into(),
                        options: slint::ModelRc::new(VecModel::from(
                            s.options.iter().map(|o| o.clone().into()).collect::<Vec<slint::SharedString>>()
                        )),
                        current_index: s.current_index,
                    }
                }).collect();
                
                window.set_current_settings(slint::ModelRc::new(VecModel::from(slint_settings)));
            }
        });
    }
    
    {
        let controller = settings_controller.clone();
        main_window.on_menu_settings_save(move || {
            match controller.save() {
                Ok(_) => warn!("Settings saved to file"),
                Err(e) => warn!("Failed to save settings: {}", e),
            }
        });
    }
    
    {
        let controller = settings_controller.clone();
        let window_weak = main_window.as_weak();
        main_window.on_menu_settings_restore_defaults(move || {
             controller.restore_defaults();
             if let Some(window) = window_weak.upgrade() {
                 window.invoke_config_retrieve_settings();
             }
        });
    }
    
    {
        let controller = settings_controller.clone();
        let window_weak = main_window.as_weak();
        main_window.on_update_setting(move |id, value| {
            controller.update_setting(&id, &value);
            // Refresh UI
            if let Some(window) = window_weak.upgrade() {
                 window.invoke_config_retrieve_settings();
            }
        });
    }
    
    {
        let window_weak = main_window.as_weak();
        main_window.on_settings_category_selected(move |_category| {
            if let Some(window) = window_weak.upgrade() {
                 window.invoke_config_retrieve_settings();
            }
        });
    }

    // Set up refresh-ports callback
    let ports_model_clone = ports_model.clone();
    main_window.on_refresh_ports(move || {
        if let Ok(ports) = get_available_ports() {
            ports_model_clone.set_vec(ports);
        }
    });

    // Set up connect callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    let polling_stop_connect = status_polling_stop.clone();
    let capability_manager_clone = capability_manager.clone();
    let gcode_send_state_connect = gcode_send_state.clone();
    main_window.on_connect(move |port: slint::SharedString, baud: i32| {
        let port_str = port.to_string();

        // Add connection attempt to console
        console_manager_clone.add_message(
            DeviceMessageType::Output,
            format!("Connecting to {} at {} baud", port_str, baud),
        );

        // Update UI with connecting status immediately
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("Connecting..."));
            window.set_device_version(slint::SharedString::from("Detecting..."));
            window.set_machine_state(slint::SharedString::from("CONNECTING"));
            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }

        // Create connection parameters
        let params = ConnectionParams {
            driver: ConnectionDriver::Serial,
            port: port_str.clone(),
            network_port: 8888,
            baud_rate: baud as u32,
            timeout_ms: 5000,
            flow_control: false,
            data_bits: 8,
            stop_bits: 1,
            parity: SerialParity::None,
            auto_reconnect: true,
            max_retries: 3,
        };

        // Try to connect
        let mut comm = communicator_clone.lock().unwrap();
        match comm.connect(&params) {
            Ok(()) => {
                console_manager_clone.add_message(
                    DeviceMessageType::Success,
                    format!("Successfully connected to {} at {} baud", port_str, baud),
                );

                if let Some(window) = window_weak.upgrade() {
                    window.set_connected(true);
                    window.set_connection_status(slint::SharedString::from("Connected"));
                    window.set_device_version(slint::SharedString::from("GRBL 1.1"));
                    window.set_machine_state(slint::SharedString::from("IDLE"));
                    let console_output = console_manager_clone.get_output();
                    window.set_console_output(slint::SharedString::from(console_output));

                    // Initialize Device Info panel with default GRBL 1.1
                    // Will be updated after firmware detection completes
                    use gcodekit4::firmware::firmware_version::{FirmwareType, SemanticVersion};
                    let firmware_type = FirmwareType::Grbl;
                    let version = SemanticVersion::new(1, 1, 0);
                    update_device_info_panel(&window, firmware_type, version, &capability_manager_clone);

                    // Set up timer to check for firmware detection and update Device Info
                    let window_weak_timer = window_weak.clone();
                    let detected_firmware_timer = detected_firmware.clone();
                    let capability_manager_timer = capability_manager_clone.clone();
                    let timer = slint::Timer::default();
                    timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(500), move || {
                        if let Some(detection) = detected_firmware_timer.lock().unwrap().as_ref().cloned() {
                            if let Some(window) = window_weak_timer.upgrade() {
                                update_device_info_panel(&window, detection.firmware_type, detection.version, &capability_manager_timer);
                                window.set_device_version(slint::SharedString::from(
                                    format!("{} {}", detection.firmware_type, detection.version)
                                ));
                            }
                            // Stop timer after updating once
                        }
                    });
                }

                // Start status polling thread
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    "Starting status polling...".to_string(),
                );

                polling_stop_connect.store(false, std::sync::atomic::Ordering::Relaxed);
                let window_weak_poll = window_weak.clone();
                let polling_active = status_polling_active.clone();
                let polling_stop = polling_stop_connect.clone();
                let communicator_poll = communicator_clone.clone();
                let console_manager_poll = console_manager_clone.clone();
                let gcode_state_poll = gcode_send_state_connect.clone();

                std::thread::spawn(move || {
                    polling_active.store(true, std::sync::atomic::Ordering::Relaxed);

                    // Send $I once at startup to get firmware version
                    {
                        let mut comm = communicator_poll.lock().unwrap();
                        if let Err(e) = comm.send_command("$I") {
                            tracing::warn!("Failed to send $I for firmware detection: {}", e);
                        } else {
                        }
                    }

                    // Wait for firmware detection to complete (listener will process the response)
                    // The UI timer will update Device Info panel automatically
                    std::thread::sleep(std::time::Duration::from_millis(1000));

                    // GRBL buffer is 128 bytes, but we use 127 for safety
                    const GRBL_RX_BUFFER_SIZE: usize = 127;
                    let mut response_buffer = String::new();

                    // Main polling loop runs at 35ms intervals
                    // - Reads responses continuously (ok, error, status reports)
                    // - Sends G-code lines using character-counting protocol
                    // - Sends ? status query every 35ms (real-time command)
                    while !polling_stop.load(std::sync::atomic::Ordering::Relaxed) {
                        std::thread::sleep(std::time::Duration::from_millis(35));

                        // Use the shared communicator instead of creating a new connection
                        // CRITICAL: Hold the lock for minimal time to allow jog commands through
                        let (response_data, is_connected) = {
                            let mut comm = communicator_poll.lock().unwrap();
                            let connected = comm.is_connected();
                            let response = if connected { comm.receive().ok() } else { None };
                            (response, connected)
                        }; // Lock released immediately after reading

                        if is_connected {
                            // Step 1: Process responses (without holding lock)
                            if let Some(response) = response_data {
                                if !response.is_empty() {
                                    response_buffer.push_str(&String::from_utf8_lossy(&response));
                                    
                                    // Process complete lines
                                    while let Some(idx) = response_buffer.find('\n') {
                                        let line = response_buffer[..idx].trim().to_string();
                                        response_buffer.drain(..idx + 1);
                                        
                                        if line.is_empty() { continue; }

                                        // Count "ok" and "error" responses for buffer management
                                        let is_ok = line.contains("ok") || line.contains("OK");
                                        let is_error = line.contains("error:");
                                        
                                        if is_ok || is_error {
                                            let mut gstate = gcode_state_poll.lock().unwrap();
                                            if let Some(len) = gstate.line_lengths.pop_front() {
                                                gstate.pending_bytes = gstate.pending_bytes.saturating_sub(len);
                                            }
                                            drop(gstate);
                                        }

                                        // Check for errors and handle them
                                        if is_error {
                                            warn!("GRBL error in response: {}", line);
                                            let error_msg = format!("GRBL error: {}", line);
                                            console_manager_poll.add_message(
                                                DeviceMessageType::Error,
                                                error_msg.clone()
                                            );

                                            // Show error dialog
                                            let wh = window_weak_poll.clone();
                                            let em = error_msg.clone();
                                            slint::invoke_from_event_loop(move || {
                                                if let Some(_w) = wh.upgrade() {
                                                    let error_dialog = ErrorDialog::new().unwrap();
                                                    error_dialog.set_error_message(slint::SharedString::from(format!(
                                                        "GRBL Error\n\nThe device reported an error.\n\n{}",
                                                        em
                                                    )));

                                                    let error_dialog_weak = error_dialog.as_weak();
                                                    error_dialog.on_close_dialog(move || {
                                                        if let Some(dlg) = error_dialog_weak.upgrade() {
                                                            dlg.hide().ok();
                                                        }
                                                    });

                                                    error_dialog.show().ok();
                                                }
                                            }).ok();
                                        }

                                        // Process status responses
                                        if line.contains("<") && line.contains(">") {
                                            // Parse full status from response
                                            use gcodekit4::firmware::grbl::status_parser::StatusParser;
                                            let full_status = StatusParser::parse_full(&line);

                                            let window_handle = window_weak_poll.clone();
                                            let raw_response = line.clone();
                                            slint::invoke_from_event_loop(move || {
                                                if let Some(window) = window_handle.upgrade() {
                                                    // Update raw status response
                                                    window.set_raw_status_response(slint::SharedString::from(raw_response.trim()));

                                                    // Update machine position
                                                    if let Some(mpos) = full_status.mpos {
                                                        window.set_position_x(mpos.x as f32);
                                                        window.set_position_y(mpos.y as f32);
                                                        window.set_position_z(mpos.z as f32);
                                                        if let Some(a) = mpos.a {
                                                            window.set_position_a(a as f32);
                                                        }
                                                        if let Some(b) = mpos.b {
                                                            window.set_position_b(b as f32);
                                                        }
                                                        if let Some(c) = mpos.c {
                                                            window.set_position_c(c as f32);
                                                        }
                                                    }

                                                    // Update machine state
                                                    if let Some(state) = full_status.machine_state {
                                                        window.set_machine_state(slint::SharedString::from(state));
                                                    }

                                                    // Update feed rate
                                                    if let Some(feed) = full_status.feed_rate {
                                                        window.set_feed_rate(feed as f32);
                                                    }

                                                    // Update spindle speed
                                                    if let Some(spindle) = full_status.spindle_speed {
                                                        window.set_spindle_speed(spindle as f32);
                                                    }
                                                }
                                            })
                                            .ok();
                                        }
                                    }
                                }
                            }

                            // Step 2: Send G-code lines if queued
                            // Send up to 10 lines per cycle, respecting buffer limits
                            {
                                let mut gstate = gcode_state_poll.lock().unwrap();
                                let mut lines_sent_this_cycle = 0;

                                while !gstate.lines.is_empty() && lines_sent_this_cycle < 10 {
                                    let line = gstate.lines.front().cloned().unwrap_or_default();
                                    let trimmed = line.trim();

                                    // Skip empty lines and comments quickly
                                    if trimmed.is_empty() || trimmed.starts_with(';') {
                                        gstate.lines.pop_front();
                                        continue; // Don't count skipped lines
                                    }

                                    // Check buffer space before sending
                                    let line_len = trimmed.len() + 1;
                                    if gstate.pending_bytes + line_len <= GRBL_RX_BUFFER_SIZE {
                                        // Acquire lock only for the actual send operation
                                        let send_result = {
                                            let mut comm = communicator_poll.lock().unwrap();
                                            comm.send(format!("{}\n", trimmed).as_bytes())
                                        }; // Lock released immediately

                                        match send_result {
                                            Ok(_) => {
                                                gstate.lines.pop_front();
                                                gstate.pending_bytes += line_len;
                                                gstate.line_lengths.push_back(line_len);
                                                gstate.total_sent += 1;
                                                lines_sent_this_cycle += 1;


                                                if gstate.total_sent.is_multiple_of(10) || gstate.lines.is_empty() {
                                                    let sent = gstate.total_sent;
                                                    let total = gstate.total_lines;
                                                    let progress = if total > 0 { (sent as f32 / total as f32) * 100.0 } else { 0.0 };
                                                    let wh = window_weak_poll.clone();
                                                    slint::invoke_from_event_loop(move || {
                                                        if let Some(w) = wh.upgrade() {
                                                            w.set_connection_status(slint::SharedString::from(
                                                                format!("Sending: {}/{}", sent, total)
                                                            ));
                                                            w.set_progress_value(progress);
                                                        }
                                                    }).ok();
                                                }
                                            }
                                            Err(e) => {
                                                let error_msg = format!("✗ Send failed at line {}: {}", gstate.total_sent + 1, e);
                                                console_manager_poll.add_message(
                                                    DeviceMessageType::Error,
                                                    error_msg.clone()
                                                );
                                                gstate.lines.clear();

                                                // Show error dialog
                                                let wh = window_weak_poll.clone();
                                                let em = error_msg.clone();
                                                slint::invoke_from_event_loop(move || {
                                                    if let Some(_w) = wh.upgrade() {
                                                        let error_dialog = ErrorDialog::new().unwrap();
                                                        error_dialog.set_error_message(slint::SharedString::from(format!(
                                                            "Send Error\n\nFailed to send G-code to device.\n\n{}",
                                                            em
                                                        )));

                                                        let error_dialog_weak = error_dialog.as_weak();
                                                        error_dialog.on_close_dialog(move || {
                                                            if let Some(dlg) = error_dialog_weak.upgrade() {
                                                                dlg.hide().ok();
                                                            }
                                                        });

                                                        error_dialog.show().ok();
                                                    }
                                                }).ok();

                                                break;
                                            }
                                        }
                                    } else {
                                        break; // Buffer full, stop sending this cycle
                                    }
                                }

                                // Check if done sending
                                if gstate.total_lines > 0 && gstate.lines.is_empty() && gstate.line_lengths.is_empty() {
                                    let total = gstate.total_sent;
                                    console_manager_poll.add_message(
                                        DeviceMessageType::Success,
                                        format!("✓ Successfully sent {} lines", total)
                                    );
                                    let wh = window_weak_poll.clone();
                                    let cm = console_manager_poll.clone();
                                    slint::invoke_from_event_loop(move || {
                                        if let Some(w) = wh.upgrade() {
                                            w.set_connection_status(slint::SharedString::from(format!("Sent: {} lines", total)));
                                            w.set_progress_value(0.0);
                                            w.set_console_output(slint::SharedString::from(cm.get_output()));
                                        }
                                    }).ok();
                                    gstate.total_lines = 0;
                                    gstate.total_sent = 0;
                                }
                            } // Release gstate lock

                            // Step 3: Send status query periodically (real-time command - doesn't use buffer)
                            // Send every 200ms (4 cycles of 50ms)
                            static mut CYCLE: u32 = 0;
                            unsafe {
                                CYCLE += 1;
                                if CYCLE.is_multiple_of(4) {
                                    // Real-time command - acquire lock briefly for send only
                                    let mut comm = communicator_poll.lock().unwrap();
                                    comm.send(b"?").ok();
                                } // Lock released immediately
                            }
                        }
                    }
                    polling_active.store(false, std::sync::atomic::Ordering::Relaxed);
                });
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                console_manager_clone.add_message(
                    DeviceMessageType::Error,
                    format!("Connection failed: {}", error_msg),
                );
                if let Some(window) = window_weak.upgrade() {
                    window.set_connected(false);
                    window.set_connection_status(slint::SharedString::from("Connection Failed".to_string()));
                    window.set_device_version(slint::SharedString::from("Not Connected"));
                    window.set_machine_state(slint::SharedString::from("DISCONNECTED"));
                    let console_output = console_manager_clone.get_output();
                    window.set_console_output(slint::SharedString::from(console_output));

                    // Show error dialog
                    let error_dialog = ErrorDialog::new().unwrap();
                    error_dialog.set_error_message(slint::SharedString::from(format!(
                        "Connection Failed\n\nUnable to connect to {} at {} baud.\n\nError: {}",
                        port_str, baud, error_msg
                    )));

                    let error_dialog_weak = error_dialog.as_weak();
                    error_dialog.on_close_dialog(move || {
                        if let Some(dlg) = error_dialog_weak.upgrade() {
                            dlg.hide().ok();
                        }
                    });

                    error_dialog.show().ok();
                }
            }
        }
    });

    // Set up disconnect callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    let polling_stop_clone = status_polling_stop.clone();
    let capability_manager_disconnect = capability_manager.clone();
    main_window.on_disconnect(move || {
        console_manager_clone.add_message(DeviceMessageType::Output, "Disconnecting from device");

        // Stop the polling thread
        polling_stop_clone.store(true, std::sync::atomic::Ordering::Relaxed);

        let mut comm = communicator_clone.lock().unwrap();
        match comm.disconnect() {
            Ok(()) => {
                // Reset the communicator to a fresh state by replacing with a new instance
                drop(comm);
                let mut new_comm = SerialCommunicator::new();
                // Re-register the console listener with the new communicator
                let console_listener = ConsoleListener::new(console_manager_clone.clone());
                new_comm.add_listener(console_listener);
                *communicator_clone.lock().unwrap() = new_comm;

                console_manager_clone
                    .add_message(DeviceMessageType::Success, "Successfully disconnected");
                if let Some(window) = window_weak.upgrade() {
                    window.set_connected(false);
                    window.set_connection_status(slint::SharedString::from("Disconnected"));
                    window.set_device_version(slint::SharedString::from("Not Connected"));
                    window.set_machine_state(slint::SharedString::from("DISCONNECTED"));
                    window.set_position_x(0.0);
                    window.set_position_y(0.0);
                    window.set_position_z(0.0);
                    let console_output = console_manager_clone.get_output();
                    window.set_console_output(slint::SharedString::from(console_output));

                    // Reset capabilities to defaults
                    capability_manager_disconnect.reset();
                    sync_capabilities_to_ui(&window, &capability_manager_disconnect);
                }
            }
            Err(e) => {
                console_manager_clone
                    .add_message(DeviceMessageType::Error, format!("Disconnect error: {}", e));
                if let Some(window) = window_weak.upgrade() {
                    window.set_connection_status(slint::SharedString::from("Disconnect error"));
                    let console_output = console_manager_clone.get_output();
                    window.set_console_output(slint::SharedString::from(console_output));

                    // Show error dialog
                    let error_dialog = ErrorDialog::new().unwrap();
                    error_dialog.set_error_message(slint::SharedString::from(format!(
                        "Disconnect Error\n\nAn error occurred while disconnecting from the device.\n\nError: {}",
                        e
                    )));

                    let error_dialog_weak = error_dialog.as_weak();
                    error_dialog.on_close_dialog(move || {
                        if let Some(dlg) = error_dialog_weak.upgrade() {
                            dlg.hide().ok();
                        }
                    });

                    error_dialog.show().ok();
                }
            }
        }
    });

    // Set up menu-file-exit callback
    let communicator_clone = communicator.clone();
    main_window.on_menu_file_exit(move || {
        // Disconnect if connected before exiting
        let mut comm = communicator_clone.lock().unwrap();
        if comm.disconnect().is_err() {}
        std::process::exit(0);
    });

    // Set up menu-file-new callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_file_new(move || {
        if let Some(window) = window_weak.upgrade() {
            // Clear the editor content and reset filename
            window.set_gcode_filename(slint::SharedString::from("unknown.gcode"));
            window.invoke_clear_editor();
        }
    });

    // Set up menu-file-open callback
    let window_weak = main_window.as_weak();
    let gcode_editor_clone = gcode_editor.clone();
    let console_manager_clone = console_manager.clone();
    let editor_bridge_open = editor_bridge.clone();
    main_window.on_menu_file_open(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_is_busy(true);
        }

        // Open file dialog and load file
        match gcode_editor_clone.open_and_load_file() {
            Ok(path) => {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let full_path = path.display().to_string();
                let line_count = gcode_editor_clone.get_line_count();

                // Log to device console
                console_manager_clone.add_message(
                    DeviceMessageType::Success,
                    format!("✓ File loaded: {}", file_name),
                );
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("  Lines: {}", line_count),
                );
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("  Path: {}", path.display()),
                );

                // Update UI with new content
                let content = gcode_editor_clone.get_plain_content();

                let preview = if content.len() > 100 {
                    format!("{}...", &content[..100])
                } else {
                    content.clone()
                };
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("CONTENT LENGTH: {} chars", content.len()),
                );
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("CONTENT PREVIEW: {}", preview),
                );

                if let Some(window) = window_weak.upgrade() {
                    // DEBUG: Log view switch
                    console_manager_clone.add_message(
                        DeviceMessageType::Output,
                        "DEBUG: Switching to gcode-editor view".to_string(),
                    );

                    // IMPORTANT: Switch to gcode-editor view to show the content
                    window.set_current_view(slint::SharedString::from("gcode-editor"));
                    window.set_gcode_focus_trigger(window.get_gcode_focus_trigger() + 1);

                    console_manager_clone.add_message(
                        DeviceMessageType::Output,
                        format!("DEBUG: Setting TextEdit content ({} chars)", content.len()),
                    );

                    // Load into custom editor
                    editor_bridge_open.load_text(&content);

                    window.set_gcode_content(slint::SharedString::from(content.clone()));

                    // Update custom editor state
                    let line_count = editor_bridge_open.line_count();
                    window.set_can_undo(editor_bridge_open.can_undo());
                    window.set_can_redo(editor_bridge_open.can_redo());
                    window.set_total_lines(line_count as i32);
                    update_visible_lines(&window, &editor_bridge_open);

                    // VERIFY: Log what was set
                    let verify_content = window.get_gcode_content();
                    console_manager_clone.add_message(
                        DeviceMessageType::Output,
                        format!(
                            "VERIFY: get_gcode_content returned {} chars",
                            verify_content.len()
                        ),
                    );

                    window.set_gcode_filename(slint::SharedString::from(&full_path));
                    window.set_connection_status(slint::SharedString::from(format!(
                        "Loaded: {} ({} lines)",
                        file_name, line_count
                    )));

                    // Render visualization in background thread to avoid blocking UI
                    let window_weak = window.as_weak();
                    let content_clone = content.clone();
                    let width = window.get_visualizer_canvas_width();
                    let height = window.get_visualizer_canvas_height();

                    // Use a channel to communicate progress from background thread
                    let (tx, rx) = std::sync::mpsc::channel();

                    std::thread::spawn(move || {
                        render_gcode_visualization_background_channel(content_clone, width as u32, height as u32, tx);
                    });

                    // Use Slint's invoke_from_event_loop to safely update UI from background thread
                    std::thread::spawn(move || {
                        while let Ok((
                            progress,
                            status,
                            path_data,
                            rapid_moves_data,
                            grid_data,
                            origin_data,
                            grid_size,
                        )) = rx.recv()
                        {
                            let window_handle = window_weak.clone();
                            let status_clone = status.clone();
                            let path_clone = path_data.clone();
                            let rapid_moves_clone = rapid_moves_data.clone();
                            let grid_clone = grid_data.clone();
                            let origin_clone = origin_data.clone();

                            slint::invoke_from_event_loop(move || {
                                if let Some(window) = window_handle.upgrade() {
                                    window.set_visualizer_progress(progress);
                                    window.set_visualizer_status(slint::SharedString::from(
                                        status_clone.clone(),
                                    ));

                                    // Set canvas path data if available
                                    if let Some(path) = path_clone {
                                        window.set_visualization_path_data(
                                            slint::SharedString::from(path),
                                        );
                                    }
                                    if let Some(rapid_moves) = rapid_moves_clone {
                                        window.set_visualization_rapid_moves_data(
                                            slint::SharedString::from(rapid_moves),
                                        );
                                    }
                                    if let Some(grid) = grid_clone {
                                        window.set_visualization_grid_data(
                                            slint::SharedString::from(grid),
                                        );
                                    }
                                    if let Some(origin) = origin_clone {
                                        window.set_visualization_origin_data(
                                            slint::SharedString::from(origin),
                                        );
                                    }
                                    if let Some(size) = grid_size {
                                        window.set_visualizer_grid_size(slint::SharedString::from(format!("{}mm", size)));
                                    }
                                }
                            })
                            .ok();
                        }
                    });

                    // DEBUG: Log console update
                    console_manager_clone.add_message(
                        DeviceMessageType::Output,
                        "DEBUG: TextEdit content set in view".to_string(),
                    );

                    // Update console display
                    let console_output = console_manager_clone.get_output();
                    window.set_console_output(slint::SharedString::from(console_output));
                }
            }
            Err(e) => {
                let error_msg = e.to_string();

                // Silently ignore dialog cancellations
                if error_msg.contains("cancelled") {
                    return;
                }

                warn!("Failed to open file: {}", e);

                // Log error to device console
                console_manager_clone.add_message(
                    DeviceMessageType::Error,
                    format!("✗ Failed to load file: {}", e),
                );

                if let Some(window) = window_weak.upgrade() {
                    window
                        .set_connection_status(slint::SharedString::from(format!("Error: {}", e)));
                    window.set_is_busy(false);

                    // Update console display
                    let console_output = console_manager_clone.get_output();
                    window.set_console_output(slint::SharedString::from(console_output));
                }
            }
        }

        // Always clear busy state at end
        if let Some(window) = window_weak.upgrade() {
            window.set_is_busy(false);
        }
    });

    // Set up menu-file-save callback
    let window_weak = main_window.as_weak();
    let gcode_editor_clone = gcode_editor.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_menu_file_save(move || {
        // Get current filename and content from window
        if let Some(window) = window_weak.upgrade() {
            let filename = window.get_gcode_filename().to_string();
            let current_content = window.get_gcode_content().to_string();

            // If it's "untitled.gcode", prompt for filename (treat as Save As)
            if filename.contains("untitled") {
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    "No file loaded. Use 'Save As' to save with a filename.",
                );
                window.set_connection_status(slint::SharedString::from(
                    "Please use 'Save As' to save the file",
                ));
                return;
            }

            // Save to current file with current content from TextEdit
            match gcode_editor_clone.save_file_with_content(&current_content) {
                Ok(_) => {
                    console_manager_clone.add_message(
                        DeviceMessageType::Success,
                        format!("✓ File saved: {}", filename),
                    );
                    window.set_connection_status(slint::SharedString::from(format!(
                        "Saved: {}",
                        filename
                    )));
                }
                Err(e) => {
                    warn!("Failed to save file: {}", e);
                    console_manager_clone
                        .add_message(DeviceMessageType::Error, format!("✗ Failed to save: {}", e));
                    window.set_connection_status(slint::SharedString::from(format!(
                        "Error saving file: {}",
                        e
                    )));
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up menu-file-save-as callback
    let window_weak = main_window.as_weak();
    let gcode_editor_clone = gcode_editor.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_menu_file_save_as(move || {
        if let Some(window) = window_weak.upgrade() {
            let current_content = window.get_gcode_content().to_string();

            // Use the editor's save_as_with_dialog_and_content method with current content
            match gcode_editor_clone.save_as_with_dialog_and_content(&current_content) {
                Ok(path) => {
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let full_path = path.display().to_string();

                    console_manager_clone.add_message(
                        DeviceMessageType::Success,
                        format!("✓ File saved as: {}", file_name),
                    );
                    console_manager_clone.add_message(
                        DeviceMessageType::Output,
                        format!("  Path: {}", path.display()),
                    );

                    // Update filename in UI
                    window.set_gcode_filename(slint::SharedString::from(full_path));
                    window.set_connection_status(slint::SharedString::from(format!(
                        "Saved as: {}",
                        file_name
                    )));
                }
                Err(e) => {
                    warn!("Failed to save file as: {}", e);
                    console_manager_clone.add_message(
                        DeviceMessageType::Error,
                        format!("✗ Failed to save as: {}", e),
                    );
                    window
                        .set_connection_status(slint::SharedString::from(format!("Error: {}", e)));
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up menu-send-to-device callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    let gcode_editor_clone = gcode_editor.clone();
    let gcode_send_state_clone = gcode_send_state.clone();
    main_window.on_menu_send_to_device(move || {
        if let Some(window) = window_weak.upgrade() {
            // Get the current content from the UI TextEdit
            let current_content = window.get_gcode_content().to_string();

            // Update the GcodeEditor with the current UI content to keep them in sync
            if let Err(e) = gcode_editor_clone.load_content(&current_content) {
                warn!("Failed to sync UI content to editor: {}", e);
            }

            // For safety, use the editor's content which should now be synchronized
            let _send_content = gcode_editor_clone.get_plain_content();

            // Check if device is connected
            let comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Send failed: Device not connected");
                console_manager_clone.add_message(
                    DeviceMessageType::Error,
                    "✗ Device not connected. Please connect before sending G-Code.",
                );
                window.set_connection_status(slint::SharedString::from(
                    "Error: Device not connected",
                ));

                // Show error dialog
                let error_dialog = ErrorDialog::new().unwrap();
                error_dialog.set_error_message(slint::SharedString::from(
                    "Device Not Connected\n\nPlease connect to a device before sending G-Code."
                ));

                let error_dialog_weak = error_dialog.as_weak();
                error_dialog.on_close_dialog(move || {
                    if let Some(dlg) = error_dialog_weak.upgrade() {
                        dlg.hide().ok();
                    }
                });

                error_dialog.show().ok();
                return;
            }
            drop(comm);

            if current_content.is_empty() {
                warn!("Send failed: No G-Code to send");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ No G-Code content to send.");
                window.set_connection_status(slint::SharedString::from("Error: No G-Code content"));

                // Show error dialog
                let error_dialog = ErrorDialog::new().unwrap();
                error_dialog.set_error_message(slint::SharedString::from(
                    "No G-Code Content\n\nThere is no G-Code loaded to send. Please load or create G-Code first."
                ));

                let error_dialog_weak = error_dialog.as_weak();
                error_dialog.on_close_dialog(move || {
                    if let Some(dlg) = error_dialog_weak.upgrade() {
                        dlg.hide().ok();
                    }
                });

                error_dialog.show().ok();
                return;
            }

            // Queue G-Code for the polling thread to send using GRBL Character-Counting Protocol
            console_manager_clone.add_message(
                DeviceMessageType::Output,
                format!(
                    "Sending G-Code to device ({} bytes) using GRBL protocol...",
                    current_content.len()
                ),
            );

            // Queue the lines for the polling thread
            let lines: Vec<String> = current_content.lines().map(|s| s.to_string()).collect();
            let line_count = lines.len();

            {
                let mut gstate = gcode_send_state_clone.lock().unwrap();
                gstate.lines = lines.into();
                gstate.total_lines = line_count;
                gstate.total_sent = 0;
                gstate.pending_bytes = 0;
                gstate.line_lengths.clear();
            }

            // Update UI
            window.set_connection_status(slint::SharedString::from(
                format!("Queued {} lines for sending...", line_count)
            ));
            window.set_progress_value(0.0);
            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));

            #[allow(dead_code)]
            struct SendState {
                lines: Vec<String>,
                line_index: usize,
                send_count: usize,
                pending_bytes: usize,
                line_lengths: Vec<usize>,
                error_occurred: bool,
                error_msg: String,
                waiting_for_acks: bool,
                timeout_count: u32,
            }
        }
    });

    // Stop transmission callback
    let window_weak = main_window.as_weak();
    let gcode_send_state_stop = gcode_send_state.clone();
    let console_manager_stop = console_manager.clone();
    main_window.on_menu_stop_transmission(move || {
        if let Some(window) = window_weak.upgrade() {
            // Clear the send queue to stop transmission
            {
                let mut gstate = gcode_send_state_stop.lock().unwrap();
                gstate.lines.clear();
                gstate.total_lines = 0;
                gstate.total_sent = 0;
                gstate.pending_bytes = 0;
                gstate.line_lengths.clear();
            }

            console_manager_stop
                .add_message(DeviceMessageType::Output, "⏹ G-Code transmission stopped");
            window.set_connection_status("G-Code transmission stopped".into());
            window.set_progress_value(0.0);

            let console_output = console_manager_stop.get_output();
            window.set_console_output(console_output.into());
        }
    });

    // Pause transmission callback
    let window_weak = main_window.as_weak();
    let communicator_pause = communicator.clone();
    let console_manager_pause = console_manager.clone();
    main_window.on_menu_pause_transmission(move || {
        if let Some(window) = window_weak.upgrade() {
            // Send pause command to device (! for feed hold in GRBL)
            let comm = communicator_pause.lock().unwrap();
            if comm.is_connected() {
                drop(comm);

                // Send pause command
                if let Err(e) = communicator_pause.lock().unwrap().send_command("!") {
                    console_manager_pause.add_message(
                        DeviceMessageType::Error,
                        format!("✗ Failed to send pause command: {}", e),
                    );
                } else {
                    console_manager_pause.add_message(
                        DeviceMessageType::Output,
                        "⏸ G-Code transmission paused (feed hold)",
                    );
                    window.set_connection_status("G-Code transmission paused".into());
                }
            } else {
                console_manager_pause
                    .add_message(DeviceMessageType::Error, "✗ Device not connected");
            }

            let console_output = console_manager_pause.get_output();
            window.set_console_output(console_output.into());
        }
    });

    // Resume transmission callback
    let window_weak = main_window.as_weak();
    let communicator_resume = communicator.clone();
    let console_manager_resume = console_manager.clone();
    main_window.on_menu_resume_transmission(move || {
        if let Some(window) = window_weak.upgrade() {
            // Send resume command to device (~ for cycle start in GRBL)
            let comm = communicator_resume.lock().unwrap();
            if comm.is_connected() {
                drop(comm);

                // Send resume command
                if let Err(e) = communicator_resume.lock().unwrap().send_command("~") {
                    console_manager_resume.add_message(
                        DeviceMessageType::Error,
                        format!("✗ Failed to send resume command: {}", e),
                    );
                } else {
                    console_manager_resume.add_message(
                        DeviceMessageType::Output,
                        "▶ G-Code transmission resumed (cycle start)",
                    );
                    window.set_connection_status("G-Code transmission resumed".into());
                }
            } else {
                console_manager_resume
                    .add_message(DeviceMessageType::Error, "✗ Device not connected");
            }

            let console_output = console_manager_resume.get_output();
            window.set_console_output(console_output.into());
        }
    });

    // Set up undo callback for custom editor
    let window_weak = main_window.as_weak();
    let editor_bridge_undo = editor_bridge.clone();
    main_window.on_undo_requested(move || {
        if editor_bridge_undo.undo() {
            if let Some(window) = window_weak.upgrade() {
                // Update UI state
                window.set_can_undo(editor_bridge_undo.can_undo());
                window.set_can_redo(editor_bridge_undo.can_redo());
                
                // Update viewport if cursor moved off-screen
                let (start_line, _end_line) = editor_bridge_undo.viewport_range();
                window.set_visible_start_line(start_line as i32);
                
                update_visible_lines(&window, &editor_bridge_undo);

                // Update g-code content
                let content = editor_bridge_undo.get_text();
                window.set_gcode_content(slint::SharedString::from(content));

                // Update cursor position
                let (line, col) = editor_bridge_undo.cursor_position();
                window.set_cursor_line((line + 1) as i32);
                window.set_cursor_column((col + 1) as i32);
            }
        }
    });

    // Set up redo callback for custom editor
    let window_weak = main_window.as_weak();
    let editor_bridge_redo = editor_bridge.clone();
    main_window.on_redo_requested(move || {
        if editor_bridge_redo.redo() {
            if let Some(window) = window_weak.upgrade() {
                // Update UI state
                window.set_can_undo(editor_bridge_redo.can_undo());
                window.set_can_redo(editor_bridge_redo.can_redo());
                
                // Update viewport if cursor moved off-screen
                let (start_line, _end_line) = editor_bridge_redo.viewport_range();
                window.set_visible_start_line(start_line as i32);
                
                update_visible_lines(&window, &editor_bridge_redo);

                // Update g-code content
                let content = editor_bridge_redo.get_text();
                window.set_gcode_content(slint::SharedString::from(content));

                // Update cursor position
                let (line, col) = editor_bridge_redo.cursor_position();
                window.set_cursor_line((line + 1) as i32);
                window.set_cursor_column((col + 1) as i32);
            }
        }
    });

    // Set up scroll callback for custom editor
    let window_weak = main_window.as_weak();
    let editor_bridge_scroll = editor_bridge.clone();
    main_window.on_scroll_changed(move |line| {
        editor_bridge_scroll.scroll_to_line(line as usize);
        if let Some(window) = window_weak.upgrade() {
            update_visible_lines(&window, &editor_bridge_scroll);
        }
    });

    // Set up text-changed callback for custom editor
    let window_weak = main_window.as_weak();
    let editor_bridge_text = editor_bridge.clone();
    main_window.on_text_changed(move |_text| {
        if let Some(window) = window_weak.upgrade() {
            window.set_can_undo(editor_bridge_text.can_undo());
            window.set_can_redo(editor_bridge_text.can_redo());
        }
    });

    // Custom editor callbacks
    let window_weak = main_window.as_weak();
    let editor_bridge_clear = editor_bridge.clone();
    main_window.on_clear_editor(move || {
        editor_bridge_clear.load_text("");
        if let Some(window) = window_weak.upgrade() {
            let line_count = editor_bridge_clear.line_count();
            window.set_total_lines(line_count as i32);
            update_visible_lines(&window, &editor_bridge_clear);
        }
    });

    let window_weak = main_window.as_weak();
    let editor_bridge_append = editor_bridge.clone();
    main_window.on_append_gcode_line(move |line| {
        let current_text = editor_bridge_append.get_text();
        let new_text = if current_text.is_empty() {
            line.to_string()
        } else {
            format!("{}\n{}", current_text, line)
        };
        editor_bridge_append.load_text(&new_text);
        if let Some(window) = window_weak.upgrade() {
            let line_count = editor_bridge_append.line_count();
            window.set_total_lines(line_count as i32);
            update_visible_lines(&window, &editor_bridge_append);
        }
    });

    let window_weak = main_window.as_weak();
    let editor_bridge_loader = editor_bridge.clone();
    main_window.on_load_editor_text(move |text| {
        let shared = text.clone();
        let content = text.to_string();
        editor_bridge_loader.load_text(&content);
        // Scroll to top
        editor_bridge_loader.scroll_to_line(0);
        if let Some(window) = window_weak.upgrade() {
            window.set_gcode_content(shared);
            window.set_can_undo(editor_bridge_loader.can_undo());
            window.set_can_redo(editor_bridge_loader.can_redo());
            window.set_total_lines(editor_bridge_loader.line_count() as i32);
            // Start cursor at position (1, 1) - line 1, column 1
            window.set_cursor_line(1);
            window.set_cursor_column(1);
            update_visible_lines(&window, &editor_bridge_loader);
        }
    });

    // Text editing callbacks
    let window_weak = main_window.as_weak();
    let editor_bridge_insert = editor_bridge.clone();
    main_window.on_text_inserted(move |line, col, text| {
        let text_str = text.to_string();
        // Move cursor to the position where text should be inserted (convert 1-based to 0-based)
        let line_0based = (line - 1).max(0) as usize;
        let col_0based = (col - 1).max(0) as usize;
        editor_bridge_insert.set_cursor(line_0based, col_0based);
        // Now insert the text at the cursor position
        editor_bridge_insert.insert_text(&text_str);
        if let Some(window) = window_weak.upgrade() {
            window.set_can_undo(editor_bridge_insert.can_undo());
            window.set_can_redo(editor_bridge_insert.can_redo());
            window.set_total_lines(editor_bridge_insert.line_count() as i32);
            // Update viewport if cursor moved off-screen
            let (start_line, _end_line) = editor_bridge_insert.viewport_range();
            window.set_visible_start_line(start_line as i32);
            update_visible_lines(&window, &editor_bridge_insert);
            let (line, col) = editor_bridge_insert.cursor_position();
            window.set_cursor_line((line + 1) as i32);
            window.set_cursor_column((col + 1) as i32);
            let content = editor_bridge_insert.get_text();
            window.set_gcode_content(slint::SharedString::from(content));
        }
    });

    let window_weak = main_window.as_weak();
    let editor_bridge_delete = editor_bridge.clone();
    main_window.on_text_deleted(move |start_line, start_col, _end_line, end_col| {
        let count = (end_col - start_col).max(0) as usize;
        if count > 0 {
            // Move cursor to the position where deletion should occur (convert 1-based to 0-based)
            let line_0based = (start_line - 1).max(0) as usize;
            let col_0based = (start_col - 1).max(0) as usize;
            editor_bridge_delete.set_cursor(line_0based, col_0based);
            // Now delete from the cursor position
            editor_bridge_delete.delete_backward(count);
            if let Some(window) = window_weak.upgrade() {
                window.set_can_undo(editor_bridge_delete.can_undo());
                window.set_can_redo(editor_bridge_delete.can_redo());
                window.set_total_lines(editor_bridge_delete.line_count() as i32);
                // Update viewport if cursor moved off-screen
                let (start_line, _end_line) = editor_bridge_delete.viewport_range();
                window.set_visible_start_line(start_line as i32);
                update_visible_lines(&window, &editor_bridge_delete);
                let (line, col) = editor_bridge_delete.cursor_position();
                window.set_cursor_line((line + 1) as i32);
                window.set_cursor_column((col + 1) as i32);
                let content = editor_bridge_delete.get_text();
                window.set_gcode_content(slint::SharedString::from(content));
            }
        }
    });

    // Cursor navigation callback
    let window_weak = main_window.as_weak();
    let editor_bridge_cursor = editor_bridge.clone();
    main_window.on_cursor_moved(move |line, col| {
        // Handle line wrapping for arrow keys
        let mut line_0based = (line - 1).max(0) as usize;
        let mut col_0based = col - 1;
        
        // If col is 0 or negative and there's a line above, move to end of previous line
        if col_0based < 0 && line_0based > 0 {
            line_0based -= 1;
            // Get the previous line's length
            if let Some(prev_line) = editor_bridge_cursor.get_line_at(line_0based) {
                col_0based = (prev_line.len() - 1).max(0) as i32;
            }
        }
        // If col is beyond line length and there's a line below, move to start of next line
        else if line_0based < editor_bridge_cursor.line_count() - 1 {
            if let Some(curr_line) = editor_bridge_cursor.get_line_at(line_0based) {
                if col_0based >= curr_line.len() as i32 {
                    line_0based += 1;
                    col_0based = 0;
                }
            }
        }
        
        let col_0based = col_0based.max(0) as usize;
        editor_bridge_cursor.set_cursor(line_0based, col_0based);
        
        if let Some(window) = window_weak.upgrade() {
            // Use the values we calculated (with wrapping), not the clamped ones
            let display_line = (line_0based + 1) as i32;
            let display_col = (col_0based + 1) as i32;
            window.set_cursor_line(display_line);
            window.set_cursor_column(display_col);
            
            // Update viewport to keep cursor visible
            let (start_line, _end_line) = editor_bridge_cursor.viewport_range();
            window.set_visible_start_line(start_line as i32);
            
            // Update visible lines to show cursor
            update_visible_lines(&window, &editor_bridge_cursor);
        }
    });

    // End key pressed - move cursor to end of current line
    let window_weak = main_window.as_weak();
    let editor_bridge_end = editor_bridge.clone();
    main_window.on_end_key_pressed(move || {
        // Get current cursor position and move to end of line
        let (line, _col) = editor_bridge_end.cursor_position();
        // Get the length of the current line
        let text = editor_bridge_end.get_text();
        let lines: Vec<&str> = text.lines().collect();
        let line_end_col = if line < lines.len() {
            lines[line].len()
        } else {
            0
        };
        
        // Move cursor to end of line (convert to 0-based cursor position)
        editor_bridge_end.set_cursor(line, line_end_col);
        
        if let Some(window) = window_weak.upgrade() {
            // Get actual cursor position
            let (actual_line, actual_col) = editor_bridge_end.cursor_position();
            // Convert to 1-based for display
            window.set_cursor_line((actual_line + 1) as i32);
            window.set_cursor_column((actual_col + 1) as i32);
            
            // Update viewport
            let (start_line, _end_line) = editor_bridge_end.viewport_range();
            window.set_visible_start_line(start_line as i32);
            
            update_visible_lines(&window, &editor_bridge_end);
        }
    });

    // Ctrl+Home: Jump to beginning of file
    let window_weak = main_window.as_weak();
    let editor_bridge_ctrl_home = editor_bridge.clone();
    main_window.on_ctrl_home_pressed(move || {
        // Move to first line, first column
        editor_bridge_ctrl_home.set_cursor(0, 0);
        
        if let Some(window) = window_weak.upgrade() {
            // Get actual cursor position
            let (actual_line, actual_col) = editor_bridge_ctrl_home.cursor_position();
            // Convert to 1-based for display
            window.set_cursor_line((actual_line + 1) as i32);
            window.set_cursor_column((actual_col + 1) as i32);
            
            // Update viewport to top
            window.set_visible_start_line(0);
            
            update_visible_lines(&window, &editor_bridge_ctrl_home);
        }
    });

    // Ctrl+End: Jump to end of file
    let window_weak = main_window.as_weak();
    let editor_bridge_ctrl_end = editor_bridge.clone();
    main_window.on_ctrl_end_pressed(move || {
        // Get total lines and last line
        let line_count = editor_bridge_ctrl_end.line_count();
        let last_line = if line_count > 0 { line_count - 1 } else { 0 };
        
        // Get the length of the last line
        let text = editor_bridge_ctrl_end.get_text();
        let lines: Vec<&str> = text.lines().collect();
        let last_col = if last_line < lines.len() {
            lines[last_line].len()
        } else {
            0
        };
        
        // Move cursor to end of last line
        editor_bridge_ctrl_end.set_cursor(last_line, last_col);
        
        if let Some(window) = window_weak.upgrade() {
            // Get actual cursor position
            let (actual_line, actual_col) = editor_bridge_ctrl_end.cursor_position();
            // Convert to 1-based for display
            window.set_cursor_line((actual_line + 1) as i32);
            window.set_cursor_column((actual_col + 1) as i32);
            
            // Update viewport to show cursor
            let (start_line, _end_line) = editor_bridge_ctrl_end.viewport_range();
            window.set_visible_start_line(start_line as i32);
            
            update_visible_lines(&window, &editor_bridge_ctrl_end);
        }
    });

    // Mouse click callback - convert pixels to line/column
    main_window.on_mouse_clicked(move |_x, _y| {
        // TODO: Implement mouse-based cursor positioning
        // Currently clicking just focuses the editor for keyboard input
    });

    // Find callback
    let window_weak = main_window.as_weak();
    main_window.on_find_requested(move |_search| {
        if let Some(_window) = window_weak.upgrade() {
            // TODO: Implement find functionality in EditorBridge
        }
    });

    // Find and replace callback
    let window_weak = main_window.as_weak();
    main_window.on_replace_requested(move |_search, _replace| {
        if let Some(_window) = window_weak.upgrade() {
            // TODO: Implement find/replace functionality in EditorBridge
        }
    });

    // Initialize settings controller
    let settings_controller = Rc::new(SettingsController::new(
        settings_dialog.clone(),
        settings_persistence.clone(),
    ));

    // Set up menu-edit-preferences callback
    let window_weak = main_window.as_weak();
    let controller_clone = settings_controller.clone();
    main_window.on_menu_edit_preferences(move || {
        let ui_settings = controller_clone.get_settings_for_ui(Some(SettingsCategory::Controller));
        
        let mut settings_items = Vec::new();
        for item in ui_settings {
            let options: Vec<slint::SharedString> = item.options.iter().map(|s| s.into()).collect();
            
            settings_items.push(slint_generatedMainWindow::SettingItem {
                id: item.id.into(),
                name: item.name.into(),
                value: item.value.into(),
                value_type: item.value_type.into(),
                category: item.category.into(),
                description: item.description.into(),
                options: slint::ModelRc::from(Rc::new(slint::VecModel::from(options))),
                current_index: item.current_index,
            });
        }

        if let Some(window) = window_weak.upgrade() {
            let model = std::rc::Rc::new(slint::VecModel::from(settings_items));
            window.set_current_settings(slint::ModelRc::new(model));
            window.set_settings_category("controller".into());
            window.set_connection_status(slint::SharedString::from("Preferences dialog opened"));
        }
    });

    // Set up settings-category-selected callback
    let window_weak = main_window.as_weak();
    let controller_clone = settings_controller.clone();
    main_window.on_settings_category_selected(move |category_str| {
        let category = SettingsController::get_category_from_string(category_str.as_str());
        let ui_settings = controller_clone.get_settings_for_ui(Some(category));
        
        let mut settings_items = Vec::new();
        for item in ui_settings {
            let options: Vec<slint::SharedString> = item.options.iter().map(|s| s.into()).collect();
            
            settings_items.push(slint_generatedMainWindow::SettingItem {
                id: item.id.into(),
                name: item.name.into(),
                value: item.value.into(),
                value_type: item.value_type.into(),
                category: item.category.into(),
                description: item.description.into(),
                options: slint::ModelRc::from(Rc::new(slint::VecModel::from(options))),
                current_index: item.current_index,
            });
        }

        if let Some(window) = window_weak.upgrade() {
            let model = std::rc::Rc::new(slint::VecModel::from(settings_items));
            window.set_current_settings(slint::ModelRc::new(model));
        }
    });

    // Set up menu-settings-save callback
    let window_weak = main_window.as_weak();
    let controller_clone = settings_controller.clone();
    main_window.on_menu_settings_save(move || {
        match controller_clone.save() {
            Ok(_) => {
                if let Some(window) = window_weak.upgrade() {
                    window.set_connection_status(slint::SharedString::from("Settings saved"));
                }
            }
            Err(e) => {
                if let Some(window) = window_weak.upgrade() {
                    window.set_connection_status(slint::SharedString::from(format!(
                        "Error saving settings: {}",
                        e
                    )));
                }
            }
        }
    });

    // Set up menu-settings-cancel callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_settings_cancel(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("Settings dialog closed"));
        }
    });

    // Set up menu-settings-restore-defaults callback
    let window_weak = main_window.as_weak();
    let controller_clone = settings_controller.clone();
    main_window.on_menu_settings_restore_defaults(move || {
        controller_clone.restore_defaults();

        if let Some(window) = window_weak.upgrade() {
            window
                .set_connection_status(slint::SharedString::from("Settings restored to defaults"));
        }
    });

    // Set up update-setting callback
    let controller_clone = settings_controller.clone();
    main_window.on_update_setting(
        move |setting_id: slint::SharedString, value: slint::SharedString| {
            controller_clone.update_setting(setting_id.as_str(), value.as_str());
        },
    );

    // Set up menu-view-fullscreen callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_fullscreen(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from(
                "Fullscreen toggle would activate here",
            ));
        }
    });

    // Set up menu-view-gcode-editor callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_gcode_editor(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("G-Code Editor activated"));
            // Trigger focus on the editor by incrementing the trigger counter
            window.set_gcode_focus_trigger(window.get_gcode_focus_trigger() + 1);
        }
    });
    
    // Debug callback for key-pressed events from editor
    main_window.on_key_pressed_event(move |_msg| {
    });
    
    // Debug callback for editor clicked events
    main_window.on_editor_clicked(move || {
    });

    // Set up menu-view-machine callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_machine(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("Machine Control activated"));
        }
    });

    // Set up machine-jog-home callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_home(move || {
        if let Some(window) = window_weak.upgrade() {
            // Check if device is connected
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Jog Home failed: Device not connected");
                console_manager_clone.add_message(
                    DeviceMessageType::Error,
                    "✗ Device not connected. Please connect before sending commands.",
                );
                window.set_connection_status(slint::SharedString::from(
                    "Error: Device not connected",
                ));
            } else {
                // Send the Home command ($H)
                console_manager_clone
                    .add_message(DeviceMessageType::Output, "Sending Home command...");

                match comm.send_command("$H") {
                    Ok(_) => {
                        console_manager_clone.add_message(
                            DeviceMessageType::Success,
                            "✓ Home command sent to device",
                        );
                        window.set_connection_status(slint::SharedString::from("Homing..."));
                    }
                    Err(e) => {
                        warn!("Failed to send Home command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Failed to send Home command: {}", e),
                        );
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Error sending Home command: {}",
                            e
                        )));
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up machine-jog-x-positive callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_x_positive(move |step_size: f32| {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Jog X+ failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in relative mode (G91) for incremental movement
                let jog_cmd = format!("$J=G91 X{} F2000", step_size);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging X+ ({} mm)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to send Jog X+ command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Jog X+ failed: {}", e),
                        );
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up machine-jog-x-negative callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_x_negative(move |step_size: f32| {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Jog X- failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in relative mode (G91) for incremental movement
                let jog_cmd = format!("$J=G91 X-{} F2000", step_size);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging X- ({} mm)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to send Jog X- command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Jog X- failed: {}", e),
                        );
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up machine-jog-y-positive callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_y_positive(move |step_size: f32| {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Jog Y+ failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in relative mode (G91) for incremental movement
                let jog_cmd = format!("$J=G91 Y{} F2000", step_size);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging Y+ ({} mm)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to send Jog Y+ command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Jog Y+ failed: {}", e),
                        );
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up machine-jog-y-negative callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_y_negative(move |step_size: f32| {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Jog Y- failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in relative mode (G91) for incremental movement
                let jog_cmd = format!("$J=G91 Y-{} F2000", step_size);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging Y- ({} mm)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to send Jog Y- command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Jog Y- failed: {}", e),
                        );
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up machine-jog-z-positive callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_z_positive(move |step_size: f32| {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Jog Z+ failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in relative mode (G91) for incremental movement
                let jog_cmd = format!("$J=G91 Z{} F2000", step_size);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging Z+ ({} mm)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to send Jog Z+ command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Jog Z+ failed: {}", e),
                        );
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up machine-jog-z-negative callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_z_negative(move |step_size: f32| {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Jog Z- failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in relative mode (G91) for incremental movement
                let jog_cmd = format!("$J=G91 Z-{} F2000", step_size);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging Z- ({} mm)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to send Jog Z- command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Jog Z- failed: {}", e),
                        );
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up machine-jog-a-positive callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_a_positive(move |step_size: f32| {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Jog A+ failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in relative mode (G91) for incremental movement
                let jog_cmd = format!("$J=G91 A{} F2000", step_size);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging A+ ({} deg)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to send Jog A+ command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Jog A+ failed: {}", e),
                        );
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up machine-jog-a-negative callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_a_negative(move |step_size: f32| {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Jog A- failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in relative mode (G91) for incremental movement
                let jog_cmd = format!("$J=G91 A-{} F2000", step_size);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging A- ({} deg)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to send Jog A- command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Jog A- failed: {}", e),
                        );
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up machine-jog-b-positive callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_b_positive(move |step_size: f32| {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Jog B+ failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in relative mode (G91) for incremental movement
                let jog_cmd = format!("$J=G91 B{} F2000", step_size);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging B+ ({} deg)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to send Jog B+ command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Jog B+ failed: {}", e),
                        );
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up machine-jog-b-negative callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_b_negative(move |step_size: f32| {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Jog B- failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in relative mode (G91) for incremental movement
                let jog_cmd = format!("$J=G91 B-{} F2000", step_size);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging B- ({} deg)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Failed to send Jog B- command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Jog B- failed: {}", e),
                        );
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up machine-unlock callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_unlock(move || {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if !comm.is_connected() {
                warn!("Unlock failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send unlock command $X to clear alarm state
                console_manager_clone
                    .add_message(DeviceMessageType::Output, "Sending unlock command ($X)...");

                match comm.send(b"$X\n") {
                    Ok(_) => {
                        console_manager_clone
                            .add_message(DeviceMessageType::Success, "✓ Unlock command sent");
                    }
                    Err(e) => {
                        warn!("Failed to send unlock command: {}", e);
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Unlock failed: {}", e),
                        );
                    }
                }
            }

            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));
        }
    });

    // Set up menu-view-device-console callback
    let window_weak = main_window.as_weak();
    let console_manager_weak = std::sync::Arc::downgrade(&console_manager);
    main_window.on_menu_view_device_console(move || {
        if let Some(window) = window_weak.upgrade() {
            if let Some(console_mgr) = console_manager_weak.upgrade() {
                let output = console_mgr.get_output();
                window.set_console_output(slint::SharedString::from(output));
                window.set_connection_status(slint::SharedString::from("Device Console activated"));
            }
        }
    });

    // ═════════════════════════════════════════════════════════════
    // Configuration Settings Callbacks
    // ═════════════════════════════════════════════════════════════

    let communicator_clone = communicator.clone();
    let window_weak = main_window.as_weak();
    main_window.on_config_retrieve_settings(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_config_status_message(slint::SharedString::from(
                "Retrieving settings from controller...",
            ));

            // Send $$ command to query settings
            let mut comm = communicator_clone.lock().unwrap();
            if let Err(e) = comm.send(b"$$\n") {
                window
                    .set_config_status_message(slint::SharedString::from(format!("Error: {}", e)));
                return;
            }

            // Brief delay for response
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Read response
            match comm.receive() {
                Ok(response) => {
                    let response_str = String::from_utf8_lossy(&response);
                    let mut settings = Vec::new();

                    // Parse each line
                    for line in response_str.lines() {
                        if let Some(setting) = parse_grbl_setting_line(line) {
                            settings.push(setting);
                        }
                    }

                    // Convert to Slint model
                    let settings_model = Rc::new(VecModel::from(settings.clone()));
                    window.set_config_settings(slint::ModelRc::from(settings_model));

                    // Initialize filtered settings to show all
                    let filtered_model = Rc::new(VecModel::from(settings));
                    window.set_config_filtered_settings(slint::ModelRc::from(filtered_model));

                    window.set_config_has_loaded_settings(true);
                    window.set_config_status_message(slint::SharedString::from(format!(
                        "Retrieved {} settings from controller",
                        window.get_config_settings().row_count()
                    )));
                }
                Err(e) => {
                    window.set_config_status_message(slint::SharedString::from(format!(
                        "Error reading response: {}",
                        e
                    )));
                }
            }
        }
    });

    let window_weak = main_window.as_weak();
    main_window.on_config_save_to_file(move || {
        if let Some(window) = window_weak.upgrade() {
            // Check if we have settings to save
            if window.get_config_settings().row_count() == 0 {
                window.set_config_status_message(slint::SharedString::from(
                    "No settings to save. Retrieve settings first.",
                ));
                return;
            }

            // Open file dialog
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("grbl_config.json")
                .add_filter("JSON", &["json"])
                .save_file()
            {
                // Build JSON structure
                let settings_model = window.get_config_settings();
                let mut settings_json = Vec::new();

                for i in 0..settings_model.row_count() {
                    if let Some(setting) = settings_model.row_data(i) {
                        settings_json.push(serde_json::json!({
                            "number": setting.number,
                            "name": setting.name.as_str(),
                            "value": setting.value.as_str(),
                            "unit": setting.unit.as_str(),
                            "description": setting.description.as_str(),
                            "category": setting.category.as_str(),
                        }));
                    }
                }

                let backup = serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "firmware_type": "GRBL",
                    "firmware_version": window.get_device_version().as_str(),
                    "machine_name": "CNC Machine",
                    "settings": settings_json,
                    "notes": "Configuration backup from GCodeKit4",
                });

                // Write to file
                match std::fs::write(&path, serde_json::to_string_pretty(&backup).unwrap()) {
                    Ok(_) => {
                        window.set_config_status_message(slint::SharedString::from(format!(
                            "Saved {} settings to {}",
                            settings_model.row_count(),
                            path.display()
                        )));
                    }
                    Err(e) => {
                        window.set_config_status_message(slint::SharedString::from(format!(
                            "Error saving file: {}",
                            e
                        )));
                    }
                }
            }
        }
    });

    let window_weak = main_window.as_weak();
    main_window.on_config_load_from_file(move || {
        if let Some(window) = window_weak.upgrade() {
            // Open file dialog
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file()
            {
                // Read and parse JSON file
                match std::fs::read_to_string(&path) {
                    Ok(contents) => {
                        match serde_json::from_str::<serde_json::Value>(&contents) {
                            Ok(json) => {
                                let mut settings = Vec::new();

                                if let Some(settings_array) = json["settings"].as_array() {
                                    for setting_json in settings_array {
                                        if let (Some(number), Some(name), Some(value)) = (
                                            setting_json["number"].as_i64(),
                                            setting_json["name"].as_str(),
                                            setting_json["value"].as_str(),
                                        ) {
                                            settings.push(ConfigSetting {
                                                number: number as i32,
                                                name: slint::SharedString::from(name),
                                                value: slint::SharedString::from(value),
                                                unit: slint::SharedString::from(
                                                    setting_json["unit"].as_str().unwrap_or(""),
                                                ),
                                                description: slint::SharedString::from(
                                                    setting_json["description"]
                                                        .as_str()
                                                        .unwrap_or(""),
                                                ),
                                                category: slint::SharedString::from(
                                                    setting_json["category"]
                                                        .as_str()
                                                        .unwrap_or("Other"),
                                                ),
                                                read_only: false,
                                            });
                                        }
                                    }
                                }

                                // Update UI
                                let settings_model = Rc::new(VecModel::from(settings.clone()));
                                window.set_config_settings(slint::ModelRc::from(settings_model));

                                // Initialize filtered settings to show all
                                let filtered_model = Rc::new(VecModel::from(settings));
                                window.set_config_filtered_settings(slint::ModelRc::from(
                                    filtered_model,
                                ));

                                window.set_config_has_loaded_settings(true);
                                window.set_config_status_message(slint::SharedString::from(
                                    format!(
                                        "Loaded {} settings from {}",
                                        window.get_config_settings().row_count(),
                                        path.display()
                                    ),
                                ));
                            }
                            Err(e) => {
                                window.set_config_status_message(slint::SharedString::from(
                                    format!("Error parsing JSON: {}", e),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        window.set_config_status_message(slint::SharedString::from(format!(
                            "Error reading file: {}",
                            e
                        )));
                    }
                }
            }
        }
    });

    let communicator_clone = communicator.clone();
    let window_weak = main_window.as_weak();
    main_window.on_config_restore_to_device(move || {
        if let Some(window) = window_weak.upgrade() {
            // Check if we have settings to restore
            if window.get_config_settings().row_count() == 0 {
                window.set_config_status_message(slint::SharedString::from(
                    "No settings to restore. Load settings first.",
                ));
                return;
            }

            window.set_config_status_message(slint::SharedString::from(
                "Restoring settings to controller...",
            ));

            // Get all settings from model
            let settings_model = window.get_config_settings();
            let mut success_count = 0;
            let mut error_count = 0;

            let mut comm = communicator_clone.lock().unwrap();

            for i in 0..settings_model.row_count() {
                if let Some(setting) = settings_model.row_data(i) {
                    // Skip read-only settings
                    if setting.read_only {
                        continue;
                    }

                    // Send $n=value command
                    let command = format!("${}={}\n", setting.number, setting.value.as_str());

                    match comm.send(command.as_bytes()) {
                        Ok(_) => {
                            // Brief delay between commands
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            success_count += 1;
                        }
                        Err(_) => {
                            error_count += 1;
                        }
                    }
                }
            }

            // Show results
            if error_count == 0 {
                window.set_config_status_message(slint::SharedString::from(format!(
                    "Successfully restored {} settings to controller",
                    success_count
                )));
            } else {
                window.set_config_status_message(slint::SharedString::from(format!(
                    "Restored {} settings, {} errors",
                    success_count, error_count
                )));
            }
        }
    });

    let window_weak = main_window.as_weak();
    main_window.on_config_edit_setting(move |number: i32| {
        if let Some(window) = window_weak.upgrade() {
            // Find the setting by number
            let settings_model = window.get_config_settings();
            for i in 0..settings_model.row_count() {
                if let Some(setting) = settings_model.row_data(i) {
                    if setting.number == number {
                        // Populate edit dialog
                        window.set_edit_setting_number(number);
                        window.set_edit_setting_name(setting.name);
                        window.set_edit_setting_value(setting.value);
                        window.set_edit_setting_unit(setting.unit);
                        window.set_edit_setting_description(setting.description);
                        window.set_show_edit_dialog(true);
                        break;
                    }
                }
            }
        }
    });

    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_config_save_edited_setting(move |number: i32, value: slint::SharedString| {
        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.lock().unwrap();
            if comm.is_connected() {
                // Send GRBL command to update setting: $<number>=<value>
                let command = format!("${}={}\n", number, value);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Updating setting ${} to {}", number, value),
                );

                match comm.send(command.as_bytes()) {
                    Ok(_) => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        if let Ok(response) = comm.receive() {
                            let response_str = String::from_utf8_lossy(&response);
                            if response_str.contains("ok") {
                                window.set_config_status_message(slint::SharedString::from(
                                    format!("✓ Setting ${} updated successfully", number),
                                ));
                                console_manager_clone.add_message(
                                    DeviceMessageType::Success,
                                    format!("✓ Setting ${} updated to {}", number, value),
                                );

                                // Update the setting in the model
                                let settings_model = window.get_config_settings();
                                let mut updated_settings = Vec::new();
                                for i in 0..settings_model.row_count() {
                                    if let Some(mut setting) = settings_model.row_data(i) {
                                        if setting.number == number {
                                            setting.value = value.clone();
                                        }
                                        updated_settings.push(setting);
                                    }
                                }
                                use slint::{Model, ModelRc, VecModel};
                                let new_model = Rc::new(VecModel::from(updated_settings));
                                window.set_config_settings(ModelRc::from(new_model));

                                // Trigger filter update to refresh display
                                window.invoke_config_filter_changed();
                            } else {
                                window.set_config_status_message(slint::SharedString::from(
                                    format!("✗ Failed to update setting ${}", number),
                                ));
                                console_manager_clone.add_message(
                                    DeviceMessageType::Error,
                                    format!("✗ Setting update failed: {}", response_str.trim()),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        window.set_config_status_message(slint::SharedString::from(format!(
                            "✗ Communication error: {}",
                            e
                        )));
                        console_manager_clone.add_message(
                            DeviceMessageType::Error,
                            format!("✗ Failed to send command: {}", e),
                        );
                    }
                }

                let console_output = console_manager_clone.get_output();
                window.set_console_output(slint::SharedString::from(console_output));
            } else {
                window
                    .set_config_status_message(slint::SharedString::from("✗ Device not connected"));
            }
        }
    });

    let window_weak = main_window.as_weak();
    main_window.on_config_filter_changed(move || {
        if let Some(window) = window_weak.upgrade() {
            // Get filter text and category
            let filter_text = window.get_config_filter_text();
            let category = window.get_config_selected_category();
            let settings_model = window.get_config_settings();
            let mut filtered = Vec::new();

            let filter_lower = filter_text.to_lowercase();


            for i in 0..settings_model.row_count() {
                if let Some(setting) = settings_model.row_data(i) {
                    // Check category filter
                    let category_match =
                        category == "All" || setting.category.as_str() == category.as_str();

                    // Check text filter (matches ID, name, value, or description)
                    let text_match = filter_lower.is_empty()
                        || format!("${}", setting.number)
                            .to_lowercase()
                            .contains(&filter_lower)
                        || setting.name.to_lowercase().contains(&filter_lower)
                        || setting.value.to_lowercase().contains(&filter_lower)
                        || setting.description.to_lowercase().contains(&filter_lower);

                    if category_match && text_match {
                        filtered.push(setting);
                    }
                }
            }

            // Update filtered settings
            let filtered_model = Rc::new(VecModel::from(filtered));
            window.set_config_filtered_settings(slint::ModelRc::from(filtered_model));

            // Update status message to show filter is working
            window.set_config_status_message(slint::SharedString::from(format!(
                "Showing {} of {} settings",
                window.get_config_filtered_settings().row_count(),
                settings_model.row_count()
            )));
        }
    });

    // Set up menu-view-designer callback (Phase 2)
    let _designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_designer(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_current_view(slint::SharedString::from("designer"));
            window.set_connection_status(slint::SharedString::from("Designer tool activated"));
        }
    });

    // Set up menu-view-gcode-visualizer callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_gcode_visualizer(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_current_view(slint::SharedString::from("gcode-visualizer"));
            window.set_connection_status(slint::SharedString::from("Visualizer panel activated"));

            // Defer refresh to allow canvas to be laid out first
            let window_weak_timer = window.as_weak();
            slint::Timer::single_shot(std::time::Duration::from_millis(200), move || {
                if let Some(window) = window_weak_timer.upgrade() {
                    let canvas_width = window.get_visualizer_canvas_width();
                    let canvas_height = window.get_visualizer_canvas_height();
                    if canvas_width < 100.0 || canvas_height < 100.0 {
                        tracing::warn!("Canvas too small, retrying refresh in 200ms");
                        // Retry one more time
                        let window_weak_retry = window.as_weak();
                        slint::Timer::single_shot(std::time::Duration::from_millis(200), move || {
                            if let Some(window) = window_weak_retry.upgrade() {
                                let canvas_width = window.get_visualizer_canvas_width();
                                let canvas_height = window.get_visualizer_canvas_height();
                                window.invoke_refresh_visualization(canvas_width, canvas_height);
                            }
                        });
                    } else {
                        window.invoke_refresh_visualization(canvas_width, canvas_height);
                    }
                }
            });
        }
    });

    // Set up menu-view-materials callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_materials(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_current_view(slint::SharedString::from("materials"));
            window.set_connection_status(slint::SharedString::from("Materials Manager activated"));
        }
    });

    // Set up materials manager callbacks
    let materials_backend_clone = materials_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_load_materials(move || {
        if let Some(window) = window_weak.upgrade() {
            let backend = materials_backend_clone.borrow();
            let materials = backend.get_all_materials();
            let materials_ui: Vec<MaterialData> = materials
                .iter()
                .map(|m| MaterialData {
                    id: m.id.0.clone().into(),
                    name: m.name.clone().into(),
                    category: format!("{}", m.category).into(),
                    subcategory: m.subcategory.clone().into(),
                    description: m.description.clone().into(),
                    density: m.density,
                    machinability_rating: m.machinability_rating as i32,
                    tensile_strength: m.tensile_strength.unwrap_or(0.0),
                    melting_point: m.melting_point.unwrap_or(0.0),
                    chip_type: format!("{:?}", m.chip_type).into(),
                    heat_sensitivity: format!("{:?}", m.heat_sensitivity).into(),
                    abrasiveness: format!("{:?}", m.abrasiveness).into(),
                    surface_finish: format!("{:?}", m.surface_finish).into(),
                    dust_hazard: format!("{:?}", m.dust_hazard).into(),
                    fume_hazard: format!("{:?}", m.fume_hazard).into(),
                    coolant_required: m.coolant_required,
                    custom: m.custom,
                    notes: m.notes.clone().into(),
                })
                .collect();
            window.set_materials(slint::ModelRc::new(VecModel::from(materials_ui)));
        }
    });

    let materials_backend_clone = materials_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_create_material(move |material_data| {
        if let Some(window) = window_weak.upgrade() {
            let mut backend = materials_backend_clone.borrow_mut();

            // Convert UI material to backend material
            if let Some(category) = gcodekit4::ui::materials_manager_backend::string_to_category(
                material_data.category.as_ref(),
            ) {
                let mut material = gcodekit4::data::materials::Material::new(
                    gcodekit4::data::materials::MaterialId(material_data.id.to_string()),
                    material_data.name.to_string(),
                    category,
                    material_data.subcategory.to_string(),
                );

                material.description = material_data.description.to_string();
                material.density = material_data.density;
                material.machinability_rating = material_data.machinability_rating as u8;
                material.tensile_strength = if material_data.tensile_strength > 0.0 {
                    Some(material_data.tensile_strength)
                } else {
                    None
                };
                material.melting_point = if material_data.melting_point > 0.0 {
                    Some(material_data.melting_point)
                } else {
                    None
                };
                material.chip_type = gcodekit4::ui::materials_manager_backend::string_to_chip_type(
                    material_data.chip_type.as_ref(),
                );
                material.heat_sensitivity =
                    gcodekit4::ui::materials_manager_backend::string_to_heat_sensitivity(
                        material_data.heat_sensitivity.as_ref(),
                    );
                material.abrasiveness =
                    gcodekit4::ui::materials_manager_backend::string_to_abrasiveness(
                        material_data.abrasiveness.as_ref(),
                    );
                material.surface_finish =
                    gcodekit4::ui::materials_manager_backend::string_to_surface_finish(
                        material_data.surface_finish.as_ref(),
                    );
                material.dust_hazard =
                    gcodekit4::ui::materials_manager_backend::string_to_hazard_level(
                        material_data.dust_hazard.as_ref(),
                    );
                material.fume_hazard =
                    gcodekit4::ui::materials_manager_backend::string_to_hazard_level(
                        material_data.fume_hazard.as_ref(),
                    );
                material.coolant_required = material_data.coolant_required;
                material.custom = true;
                material.notes = material_data.notes.to_string();

                backend.add_material(material);

                // Reload the materials list
                let materials = backend.get_all_materials();
                let materials_ui: Vec<MaterialData> = materials
                    .iter()
                    .map(|m| MaterialData {
                        id: m.id.0.clone().into(),
                        name: m.name.clone().into(),
                        category: format!("{}", m.category).into(),
                        subcategory: m.subcategory.clone().into(),
                        description: m.description.clone().into(),
                        density: m.density,
                        machinability_rating: m.machinability_rating as i32,
                        tensile_strength: m.tensile_strength.unwrap_or(0.0),
                        melting_point: m.melting_point.unwrap_or(0.0),
                        chip_type: format!("{:?}", m.chip_type).into(),
                        heat_sensitivity: format!("{:?}", m.heat_sensitivity).into(),
                        abrasiveness: format!("{:?}", m.abrasiveness).into(),
                        surface_finish: format!("{:?}", m.surface_finish).into(),
                        dust_hazard: format!("{:?}", m.dust_hazard).into(),
                        fume_hazard: format!("{:?}", m.fume_hazard).into(),
                        coolant_required: m.coolant_required,
                        custom: m.custom,
                        notes: m.notes.clone().into(),
                    })
                    .collect();
                window.set_materials(slint::ModelRc::new(VecModel::from(materials_ui)));
            }
        }
    });

    let materials_backend_clone = materials_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_update_material(move |material_data| {
        if let Some(window) = window_weak.upgrade() {
            // Same as create_material since add_material will replace if ID exists
            let mut backend = materials_backend_clone.borrow_mut();

            if let Some(category) = gcodekit4::ui::materials_manager_backend::string_to_category(
                material_data.category.as_ref(),
            ) {
                let mut material = gcodekit4::data::materials::Material::new(
                    gcodekit4::data::materials::MaterialId(material_data.id.to_string()),
                    material_data.name.to_string(),
                    category,
                    material_data.subcategory.to_string(),
                );

                material.description = material_data.description.to_string();
                material.density = material_data.density;
                material.machinability_rating = material_data.machinability_rating as u8;
                material.tensile_strength = if material_data.tensile_strength > 0.0 {
                    Some(material_data.tensile_strength)
                } else {
                    None
                };
                material.melting_point = if material_data.melting_point > 0.0 {
                    Some(material_data.melting_point)
                } else {
                    None
                };
                material.chip_type = gcodekit4::ui::materials_manager_backend::string_to_chip_type(
                    material_data.chip_type.as_ref(),
                );
                material.heat_sensitivity =
                    gcodekit4::ui::materials_manager_backend::string_to_heat_sensitivity(
                        material_data.heat_sensitivity.as_ref(),
                    );
                material.abrasiveness =
                    gcodekit4::ui::materials_manager_backend::string_to_abrasiveness(
                        material_data.abrasiveness.as_ref(),
                    );
                material.surface_finish =
                    gcodekit4::ui::materials_manager_backend::string_to_surface_finish(
                        material_data.surface_finish.as_ref(),
                    );
                material.dust_hazard =
                    gcodekit4::ui::materials_manager_backend::string_to_hazard_level(
                        material_data.dust_hazard.as_ref(),
                    );
                material.fume_hazard =
                    gcodekit4::ui::materials_manager_backend::string_to_hazard_level(
                        material_data.fume_hazard.as_ref(),
                    );
                material.coolant_required = material_data.coolant_required;
                material.custom = material_data.custom;
                material.notes = material_data.notes.to_string();

                backend.add_material(material);

                // Reload the materials list
                let materials = backend.get_all_materials();
                let materials_ui: Vec<MaterialData> = materials
                    .iter()
                    .map(|m| MaterialData {
                        id: m.id.0.clone().into(),
                        name: m.name.clone().into(),
                        category: format!("{}", m.category).into(),
                        subcategory: m.subcategory.clone().into(),
                        description: m.description.clone().into(),
                        density: m.density,
                        machinability_rating: m.machinability_rating as i32,
                        tensile_strength: m.tensile_strength.unwrap_or(0.0),
                        melting_point: m.melting_point.unwrap_or(0.0),
                        chip_type: format!("{:?}", m.chip_type).into(),
                        heat_sensitivity: format!("{:?}", m.heat_sensitivity).into(),
                        abrasiveness: format!("{:?}", m.abrasiveness).into(),
                        surface_finish: format!("{:?}", m.surface_finish).into(),
                        dust_hazard: format!("{:?}", m.dust_hazard).into(),
                        fume_hazard: format!("{:?}", m.fume_hazard).into(),
                        coolant_required: m.coolant_required,
                        custom: m.custom,
                        notes: m.notes.clone().into(),
                    })
                    .collect();
                window.set_materials(slint::ModelRc::new(VecModel::from(materials_ui)));
            }
        }
    });

    let materials_backend_clone = materials_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_delete_material(move |id| {
        if let Some(window) = window_weak.upgrade() {
            let mut backend = materials_backend_clone.borrow_mut();
            backend.remove_material(&gcodekit4::data::materials::MaterialId(id.to_string()));

            // Reload the materials list
            let materials = backend.get_all_materials();
            let materials_ui: Vec<MaterialData> = materials
                .iter()
                .map(|m| MaterialData {
                    id: m.id.0.clone().into(),
                    name: m.name.clone().into(),
                    category: format!("{}", m.category).into(),
                    subcategory: m.subcategory.clone().into(),
                    description: m.description.clone().into(),
                    density: m.density,
                    machinability_rating: m.machinability_rating as i32,
                    tensile_strength: m.tensile_strength.unwrap_or(0.0),
                    melting_point: m.melting_point.unwrap_or(0.0),
                    chip_type: format!("{:?}", m.chip_type).into(),
                    heat_sensitivity: format!("{:?}", m.heat_sensitivity).into(),
                    abrasiveness: format!("{:?}", m.abrasiveness).into(),
                    surface_finish: format!("{:?}", m.surface_finish).into(),
                    dust_hazard: format!("{:?}", m.dust_hazard).into(),
                    fume_hazard: format!("{:?}", m.fume_hazard).into(),
                    coolant_required: m.coolant_required,
                    custom: m.custom,
                    notes: m.notes.clone().into(),
                })
                .collect();
            window.set_materials(slint::ModelRc::new(VecModel::from(materials_ui)));
        }
    });

    let materials_backend_clone = materials_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_search_materials(move |query| {
        if let Some(window) = window_weak.upgrade() {
            let backend = materials_backend_clone.borrow();
            let materials = backend.search_materials(query.as_ref());
            let materials_ui: Vec<MaterialData> = materials
                .iter()
                .map(|m| MaterialData {
                    id: m.id.0.clone().into(),
                    name: m.name.clone().into(),
                    category: format!("{}", m.category).into(),
                    subcategory: m.subcategory.clone().into(),
                    description: m.description.clone().into(),
                    density: m.density,
                    machinability_rating: m.machinability_rating as i32,
                    tensile_strength: m.tensile_strength.unwrap_or(0.0),
                    melting_point: m.melting_point.unwrap_or(0.0),
                    chip_type: format!("{:?}", m.chip_type).into(),
                    heat_sensitivity: format!("{:?}", m.heat_sensitivity).into(),
                    abrasiveness: format!("{:?}", m.abrasiveness).into(),
                    surface_finish: format!("{:?}", m.surface_finish).into(),
                    dust_hazard: format!("{:?}", m.dust_hazard).into(),
                    fume_hazard: format!("{:?}", m.fume_hazard).into(),
                    coolant_required: m.coolant_required,
                    custom: m.custom,
                    notes: m.notes.clone().into(),
                })
                .collect();
            window.set_materials(slint::ModelRc::new(VecModel::from(materials_ui)));
        }
    });

    let materials_backend_clone = materials_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_filter_by_category(move |category| {
        if let Some(window) = window_weak.upgrade() {
            let backend = materials_backend_clone.borrow();
            if let Some(mat_category) =
                gcodekit4::ui::materials_manager_backend::string_to_category(category.as_ref())
            {
                let materials = backend.filter_by_category(mat_category);
                let materials_ui: Vec<MaterialData> = materials
                    .iter()
                    .map(|m| MaterialData {
                        id: m.id.0.clone().into(),
                        name: m.name.clone().into(),
                        category: format!("{}", m.category).into(),
                        subcategory: m.subcategory.clone().into(),
                        description: m.description.clone().into(),
                        density: m.density,
                        machinability_rating: m.machinability_rating as i32,
                        tensile_strength: m.tensile_strength.unwrap_or(0.0),
                        melting_point: m.melting_point.unwrap_or(0.0),
                        chip_type: format!("{:?}", m.chip_type).into(),
                        heat_sensitivity: format!("{:?}", m.heat_sensitivity).into(),
                        abrasiveness: format!("{:?}", m.abrasiveness).into(),
                        surface_finish: format!("{:?}", m.surface_finish).into(),
                        dust_hazard: format!("{:?}", m.dust_hazard).into(),
                        fume_hazard: format!("{:?}", m.fume_hazard).into(),
                        coolant_required: m.coolant_required,
                        custom: m.custom,
                        notes: m.notes.clone().into(),
                    })
                    .collect();
                window.set_materials(slint::ModelRc::new(VecModel::from(materials_ui)));
            }
        }
    });

    let window_weak = main_window.as_weak();
    main_window.on_select_material(move |_id| {
        if let Some(_window) = window_weak.upgrade() {
            // Material selection is handled in the UI
        }
    });

    // Set up CNC Tools Manager callbacks
    let tools_backend_clone = tools_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_load_tools(move || {
        if let Some(window) = window_weak.upgrade() {
            let backend = tools_backend_clone.borrow();
            let tools = backend.get_all_tools();
            let tools_ui: Vec<ToolData> = tools
                .iter()
                .map(|t| ToolData {
                    id: t.id.0.clone().into(),
                    number: t.number as i32,
                    name: t.name.clone().into(),
                    tool_type: format!("{}", t.tool_type).into(),
                    material: format!("{}", t.material).into(),
                    diameter: t.diameter,
                    length: t.length,
                    flute_length: t.flute_length,
                    shaft_diameter: t.shaft_diameter.unwrap_or(t.diameter),
                    flutes: t.flutes as i32,
                    coating: t
                        .coating
                        .as_ref()
                        .map(|c| format!("{}", c))
                        .unwrap_or_else(|| "None".to_string())
                        .into(),
                    manufacturer: t.manufacturer.clone().unwrap_or_default().into(),
                    part_number: t.part_number.clone().unwrap_or_default().into(),
                    description: t.description.clone().into(),
                    custom: t.custom,
                    notes: t.notes.clone().into(),
                })
                .collect();
            window.set_cnc_tools(slint::ModelRc::new(VecModel::from(tools_ui)));
        }
    });

    let tools_backend_clone = tools_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_create_tool(move |tool_data| {
        if let Some(window) = window_weak.upgrade() {
            let mut backend = tools_backend_clone.borrow_mut();

            if let Some(tool_type) = gcodekit4::ui::tools_manager_backend::string_to_tool_type(
                tool_data.tool_type.as_ref(),
            ) {
                let tool_id =
                    gcodekit4::data::tools::ToolId(format!("custom_{}", tool_data.number));

                let mut tool = gcodekit4::data::tools::Tool::new(
                    tool_id,
                    tool_data.number as u32,
                    tool_data.name.to_string(),
                    tool_type,
                    tool_data.diameter,
                    tool_data.length,
                );

                tool.flute_length = tool_data.flute_length;
                tool.shaft_diameter = Some(tool_data.shaft_diameter);
                tool.flutes = tool_data.flutes as u32;

                if let Some(material) =
                    gcodekit4::ui::tools_manager_backend::string_to_tool_material(
                        tool_data.material.as_ref(),
                    )
                {
                    tool.material = material;
                }

                tool.manufacturer = Some(tool_data.manufacturer.to_string());
                tool.part_number = Some(tool_data.part_number.to_string());
                tool.description = tool_data.description.to_string();
                tool.notes = tool_data.notes.to_string();
                tool.custom = true;

                backend.add_tool(tool);

                // Reload tools list
                let tools = backend.get_all_tools();
                let tools_ui: Vec<ToolData> = tools
                    .iter()
                    .map(|t| ToolData {
                        id: t.id.0.clone().into(),
                        number: t.number as i32,
                        name: t.name.clone().into(),
                        tool_type: format!("{}", t.tool_type).into(),
                        material: format!("{}", t.material).into(),
                        diameter: t.diameter,
                        length: t.length,
                        flute_length: t.flute_length,
                        shaft_diameter: t.shaft_diameter.unwrap_or(t.diameter),
                        flutes: t.flutes as i32,
                        coating: t
                            .coating
                            .as_ref()
                            .map(|c| format!("{}", c))
                            .unwrap_or_else(|| "None".to_string())
                            .into(),
                        manufacturer: t.manufacturer.clone().unwrap_or_default().into(),
                        part_number: t.part_number.clone().unwrap_or_default().into(),
                        description: t.description.clone().into(),
                        custom: t.custom,
                        notes: t.notes.clone().into(),
                    })
                    .collect();
                window.set_cnc_tools(slint::ModelRc::new(VecModel::from(tools_ui)));
            }
        }
    });

    let tools_backend_clone = tools_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_update_tool(move |tool_data| {
        if let Some(window) = window_weak.upgrade() {
            let mut backend = tools_backend_clone.borrow_mut();

            if let Some(tool_type) = gcodekit4::ui::tools_manager_backend::string_to_tool_type(
                tool_data.tool_type.as_ref(),
            ) {
                let tool_id = gcodekit4::data::tools::ToolId(tool_data.id.to_string());

                let mut tool = gcodekit4::data::tools::Tool::new(
                    tool_id,
                    tool_data.number as u32,
                    tool_data.name.to_string(),
                    tool_type,
                    tool_data.diameter,
                    tool_data.length,
                );

                tool.flute_length = tool_data.flute_length;
                tool.shaft_diameter = Some(tool_data.shaft_diameter);
                tool.flutes = tool_data.flutes as u32;

                if let Some(material) =
                    gcodekit4::ui::tools_manager_backend::string_to_tool_material(
                        tool_data.material.as_ref(),
                    )
                {
                    tool.material = material;
                }

                tool.manufacturer = Some(tool_data.manufacturer.to_string());
                tool.part_number = Some(tool_data.part_number.to_string());
                tool.description = tool_data.description.to_string();
                tool.notes = tool_data.notes.to_string();
                tool.custom = tool_data.custom;

                backend.add_tool(tool);

                // Reload tools list
                let tools = backend.get_all_tools();
                let tools_ui: Vec<ToolData> = tools
                    .iter()
                    .map(|t| ToolData {
                        id: t.id.0.clone().into(),
                        number: t.number as i32,
                        name: t.name.clone().into(),
                        tool_type: format!("{}", t.tool_type).into(),
                        material: format!("{}", t.material).into(),
                        diameter: t.diameter,
                        length: t.length,
                        flute_length: t.flute_length,
                        shaft_diameter: t.shaft_diameter.unwrap_or(t.diameter),
                        flutes: t.flutes as i32,
                        coating: t
                            .coating
                            .as_ref()
                            .map(|c| format!("{}", c))
                            .unwrap_or_else(|| "None".to_string())
                            .into(),
                        manufacturer: t.manufacturer.clone().unwrap_or_default().into(),
                        part_number: t.part_number.clone().unwrap_or_default().into(),
                        description: t.description.clone().into(),
                        custom: t.custom,
                        notes: t.notes.clone().into(),
                    })
                    .collect();
                window.set_cnc_tools(slint::ModelRc::new(VecModel::from(tools_ui)));
            }
        }
    });

    let tools_backend_clone = tools_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_delete_tool(move |id| {
        if let Some(window) = window_weak.upgrade() {
            let mut backend = tools_backend_clone.borrow_mut();
            backend.remove_tool(&gcodekit4::data::tools::ToolId(id.to_string()));

            // Reload tools list
            let tools = backend.get_all_tools();
            let tools_ui: Vec<ToolData> = tools
                .iter()
                .map(|t| ToolData {
                    id: t.id.0.clone().into(),
                    number: t.number as i32,
                    name: t.name.clone().into(),
                    tool_type: format!("{}", t.tool_type).into(),
                    material: format!("{}", t.material).into(),
                    diameter: t.diameter,
                    length: t.length,
                    flute_length: t.flute_length,
                    shaft_diameter: t.shaft_diameter.unwrap_or(t.diameter),
                    flutes: t.flutes as i32,
                    coating: t
                        .coating
                        .as_ref()
                        .map(|c| format!("{}", c))
                        .unwrap_or_else(|| "None".to_string())
                        .into(),
                    manufacturer: t.manufacturer.clone().unwrap_or_default().into(),
                    part_number: t.part_number.clone().unwrap_or_default().into(),
                    description: t.description.clone().into(),
                    custom: t.custom,
                    notes: t.notes.clone().into(),
                })
                .collect();
            window.set_cnc_tools(slint::ModelRc::new(VecModel::from(tools_ui)));
        }
    });

    let tools_backend_clone = tools_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_search_tools(move |query| {
        if let Some(window) = window_weak.upgrade() {
            let backend = tools_backend_clone.borrow();
            let tools = backend.search_tools(query.as_ref());
            let tools_ui: Vec<ToolData> = tools
                .iter()
                .map(|t| ToolData {
                    id: t.id.0.clone().into(),
                    number: t.number as i32,
                    name: t.name.clone().into(),
                    tool_type: format!("{}", t.tool_type).into(),
                    material: format!("{}", t.material).into(),
                    diameter: t.diameter,
                    length: t.length,
                    flute_length: t.flute_length,
                    shaft_diameter: t.shaft_diameter.unwrap_or(t.diameter),
                    flutes: t.flutes as i32,
                    coating: t
                        .coating
                        .as_ref()
                        .map(|c| format!("{}", c))
                        .unwrap_or_else(|| "None".to_string())
                        .into(),
                    manufacturer: t.manufacturer.clone().unwrap_or_default().into(),
                    part_number: t.part_number.clone().unwrap_or_default().into(),
                    description: t.description.clone().into(),
                    custom: t.custom,
                    notes: t.notes.clone().into(),
                })
                .collect();
            window.set_cnc_tools(slint::ModelRc::new(VecModel::from(tools_ui)));
        }
    });

    let tools_backend_clone = tools_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_filter_by_tool_type(move |tool_type| {
        if let Some(window) = window_weak.upgrade() {
            let backend = tools_backend_clone.borrow();
            if let Some(tt) =
                gcodekit4::ui::tools_manager_backend::string_to_tool_type(tool_type.as_ref())
            {
                let tools = backend.filter_by_type(tt);
                let tools_ui: Vec<ToolData> = tools
                    .iter()
                    .map(|t| ToolData {
                        id: t.id.0.clone().into(),
                        number: t.number as i32,
                        name: t.name.clone().into(),
                        tool_type: format!("{}", t.tool_type).into(),
                        material: format!("{}", t.material).into(),
                        diameter: t.diameter,
                        length: t.length,
                        flute_length: t.flute_length,
                        shaft_diameter: t.shaft_diameter.unwrap_or(t.diameter),
                        flutes: t.flutes as i32,
                        coating: t
                            .coating
                            .as_ref()
                            .map(|c| format!("{}", c))
                            .unwrap_or_else(|| "None".to_string())
                            .into(),
                        manufacturer: t.manufacturer.clone().unwrap_or_default().into(),
                        part_number: t.part_number.clone().unwrap_or_default().into(),
                        description: t.description.clone().into(),
                        custom: t.custom,
                        notes: t.notes.clone().into(),
                    })
                    .collect();
                window.set_cnc_tools(slint::ModelRc::new(VecModel::from(tools_ui)));
            }
        }
    });

    let window_weak = main_window.as_weak();
    main_window.on_select_tool(move |_id| {
        if let Some(_window) = window_weak.upgrade() {
            // Tool selection is handled in the UI
        }
    });

    let tools_backend_clone = tools_backend.clone();
    let window_weak = main_window.as_weak();
    main_window.on_import_gtc_package(move |_file_path| {
        if let Some(window) = window_weak.upgrade() {
            // Open file dialog for GTC package or JSON
            let file_result = rfd::FileDialog::new()
                .add_filter("GTC Package", &["zip"])
                .add_filter("GTC JSON", &["json"])
                .add_filter("All Files", &["*"])
                .set_title("Import GTC Tool Catalog")
                .pick_file();

            if let Some(file_path) = file_result {
                let mut backend = tools_backend_clone.borrow_mut();

                // Determine file type and import
                let extension = file_path.extension().and_then(|s| s.to_str());
                let result = match extension {
                    Some("zip") => backend.import_gtc_package(&file_path),
                    Some("json") => backend.import_gtc_json(&file_path),
                    _ => {
                        tracing::warn!("Unsupported file type for GTC import: {:?}", extension);
                        return;
                    }
                };

                match result {
                    Ok(import_result) => {

                        if !import_result.errors.is_empty() {
                            tracing::warn!("Import errors:");
                            for error in &import_result.errors {
                                tracing::warn!("  {}", error);
                            }
                        }

                        // Reload tools list
                        let tools = backend.get_all_tools();
                        let tools_ui: Vec<ToolData> = tools
                            .iter()
                            .map(|t| ToolData {
                                id: t.id.0.clone().into(),
                                number: t.number as i32,
                                name: t.name.clone().into(),
                                tool_type: format!("{}", t.tool_type).into(),
                                material: format!("{}", t.material).into(),
                                diameter: t.diameter,
                                length: t.length,
                                flute_length: t.flute_length,
                                shaft_diameter: t.shaft_diameter.unwrap_or(t.diameter),
                                flutes: t.flutes as i32,
                                coating: t
                                    .coating
                                    .as_ref()
                                    .map(|c| format!("{}", c))
                                    .unwrap_or_else(|| "None".to_string())
                                    .into(),
                                manufacturer: t.manufacturer.clone().unwrap_or_default().into(),
                                part_number: t.part_number.clone().unwrap_or_default().into(),
                                description: t.description.clone().into(),
                                custom: t.custom,
                                notes: t.notes.clone().into(),
                            })
                            .collect();
                        window.set_cnc_tools(slint::ModelRc::new(VecModel::from(tools_ui)));

                        // Show success message
                    }
                    Err(e) => {
                        tracing::error!("Failed to import GTC catalog: {}", e);
                    }
                }
            }
        }
    });

    // Set up menu-help-about callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_help_about(move || {
        if let Some(window) = window_weak.upgrade() {
            let about_msg = format!(
                "GCodeKit4 v{}\n\nUniversal G-Code Sender for CNC Machines",
                VERSION
            );
            window.set_connection_status(slint::SharedString::from(about_msg));
        }
    });

    // Set up console-clear-clicked callback
    let console_manager_clone = console_manager.clone();
    let window_weak = main_window.as_weak();
    main_window.on_console_clear_clicked(move || {
        console_manager_clone.clear();
        if let Some(window) = window_weak.upgrade() {
            window.set_console_output(slint::SharedString::from(""));
            window.set_connection_status(slint::SharedString::from("Console cleared"));
        }
    });

    // Set up console-copy-clicked callback
    let console_manager_clone = console_manager.clone();
    let window_weak = main_window.as_weak();
    main_window.on_console_copy_clicked(move || {
        let output = console_manager_clone.get_output();

        let success = copy_to_clipboard(&output);

        if let Some(window) = window_weak.upgrade() {
            if success {
                window.set_connection_status(slint::SharedString::from(format!(
                    "Copied {} characters to clipboard",
                    output.len()
                )));
            } else {
                window.set_connection_status(slint::SharedString::from(
                    "Failed to copy to clipboard",
                ));
            }
        }
    });

    // Set up send-command callback
    let console_manager_clone = console_manager.clone();
    let communicator_clone = communicator.clone();
    let window_weak = main_window.as_weak();
    main_window.on_send_command(move |command: slint::SharedString| {
        let cmd = command.to_string();

        // Send command to device
        let mut comm = communicator_clone.lock().unwrap();
        if comm.is_connected() {
            match comm.send_command(&cmd) {
                Ok(_) => {
                    console_manager_clone
                        .add_message(DeviceMessageType::Command, format!(">>> {}", cmd));
                    if let Some(window) = window_weak.upgrade() {
                        let console_output = console_manager_clone.get_output();
                        window.set_console_output(slint::SharedString::from(console_output));
                    }
                }
                Err(e) => {
                    console_manager_clone.add_message(
                        DeviceMessageType::Error,
                        format!("Failed to send command: {}", e),
                    );
                    if let Some(window) = window_weak.upgrade() {
                        let console_output = console_manager_clone.get_output();
                        window.set_console_output(slint::SharedString::from(console_output));
                    }
                }
            }
        } else {
            console_manager_clone.add_message(DeviceMessageType::Error, "Not connected to device");
            if let Some(window) = window_weak.upgrade() {
                let console_output = console_manager_clone.get_output();
                window.set_console_output(slint::SharedString::from(console_output));
            }
        }
    });

    // ═════════════════════════════════════════════════════════════
    // Designer Tool Callbacks (Phase 2)
    // ═════════════════════════════════════════════════════════════

    // Designer: Set Mode callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_set_mode(move |mode: i32| {
        let mut state = designer_mgr_clone.borrow_mut();
        state.set_mode(mode);
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from(format!(
                "Drawing mode: {}",
                match mode {
                    0 => "Select",
                    1 => "Rectangle",
                    2 => "Circle",
                    3 => "Line",
                    _ => "Unknown",
                }
            )));
            // Update UI state to reflect mode change
            window.set_designer_state(crate::DesignerState {
                mode,
                zoom: state.canvas.zoom() as f32,
                pan_x: 0.0,
                pan_y: 0.0,
                selected_id: -1,
                update_counter: 0,
            });
        }
    });

    // Designer: Zoom In callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_zoom_in(move || {
        let mut state = designer_mgr_clone.borrow_mut();
        state.zoom_in();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);
            // Create UI state struct from Rust state
            let ui_state = crate::DesignerState {
                mode: state.canvas.mode() as i32,
                zoom: state.canvas.zoom() as f32,
                pan_x: state.canvas.pan_offset().0 as f32,
                pan_y: state.canvas.pan_offset().1 as f32,
                selected_id: state.canvas.selected_id().unwrap_or(0) as i32,
                update_counter: 0,
            };
            window.set_designer_state(ui_state);
        }
    });

    // Designer: Zoom Out callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_zoom_out(move || {
        let mut state = designer_mgr_clone.borrow_mut();
        state.zoom_out();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);
            let ui_state = crate::DesignerState {
                mode: state.canvas.mode() as i32,
                zoom: state.canvas.zoom() as f32,
                pan_x: state.canvas.pan_offset().0 as f32,
                pan_y: state.canvas.pan_offset().1 as f32,
                selected_id: state.canvas.selected_id().unwrap_or(0) as i32,
                update_counter: 0,
            };
            window.set_designer_state(ui_state);
        }
    });

    // Designer: Zoom Fit callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_zoom_fit(move || {
        let mut state = designer_mgr_clone.borrow_mut();
        state.zoom_fit();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);
            let ui_state = crate::DesignerState {
                mode: state.canvas.mode() as i32,
                zoom: state.canvas.zoom() as f32,
                pan_x: state.canvas.pan_offset().0 as f32,
                pan_y: state.canvas.pan_offset().1 as f32,
                selected_id: state.canvas.selected_id().unwrap_or(0) as i32,
                update_counter: 0,
            };
            window.set_designer_state(ui_state);
        }
    });

    // Designer: Reset View callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_reset_view(move || {
        let mut state = designer_mgr_clone.borrow_mut();
        state.reset_view();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);
            let ui_state = crate::DesignerState {
                mode: state.canvas.mode() as i32,
                zoom: state.canvas.zoom() as f32,
                pan_x: state.canvas.pan_offset().0 as f32,
                pan_y: state.canvas.pan_offset().1 as f32,
                selected_id: state.canvas.selected_id().unwrap_or(0) as i32,
                update_counter: 0,
            };
            window.set_designer_state(ui_state);
        }
    });

    // Designer: Delete Selected callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_delete_selected(move || {
        let mut state = designer_mgr_clone.borrow_mut();
        state.delete_selected();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);
            window.set_connection_status(slint::SharedString::from(format!(
                "Shapes: {}",
                state.canvas.shapes().len()
            )));
        }
    });

    // Designer: Clear Canvas callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_clear_canvas(move || {
        let mut state = designer_mgr_clone.borrow_mut();
        state.clear_canvas();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);
            window.set_designer_gcode_generated(false);
            window.set_connection_status(slint::SharedString::from("Canvas cleared"));
        }
    });

    // Designer: G-code generated callback (called from thread via invoke_from_event_loop)
    let window_weak = main_window.as_weak();
    let editor_bridge_designer = editor_bridge.clone();
    main_window.on_invoke_designer_gcode_generated(move |gcode| {
        if let Some(window) = window_weak.upgrade() {
            let gcode_str = gcode.to_string();
            
            window.set_designer_generated_gcode(gcode.clone());
            window.set_designer_gcode_generated(true);
            
            // Load into editor and switch view
            editor_bridge_designer.load_text(&gcode_str);
            window.set_total_lines(editor_bridge_designer.line_count() as i32);
            update_visible_lines(&window, &editor_bridge_designer);
            
            window.set_gcode_content(gcode);
            window.set_current_view(slint::SharedString::from("gcode-editor"));
            window.set_gcode_focus_trigger(window.get_gcode_focus_trigger() + 1);
            
            window.set_connection_status(slint::SharedString::from(
                "G-code generated and loaded into editor",
            ));
            window.set_is_busy(false);
        }
    });

    // Designer: Generate Toolpath callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_generate_toolpath(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_is_busy(true);
            
            let designer_mgr_inner = designer_mgr_clone.clone();
            let window_weak_inner = window.as_weak();
            
            // Clone state to offload to thread
            let mut state_clone = {
                let state = designer_mgr_inner.borrow();
                state.clone()
            };

            std::thread::spawn(move || {
                let gcode = state_clone.generate_gcode();
                let gcode_shared = slint::SharedString::from(gcode);
                
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(window) = window_weak_inner.upgrade() {
                        window.invoke_invoke_designer_gcode_generated(gcode_shared);
                    }
                });
            });
        }
    });

    // Designer: Import DXF callback
    let window_weak = main_window.as_weak();
    let designer_mgr_clone = designer_mgr.clone();
    main_window.on_designer_import_dxf(move || {
        use gcodekit4::designer::{DxfImporter, DxfParser};
        use rfd::FileDialog;

        if let Some(path) = FileDialog::new()
            .add_filter("DXF Files", &["dxf"])
            .set_title("Import DXF File")
            .pick_file()
        {
            if let Some(window) = window_weak.upgrade() {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match DxfParser::parse(&content) {
                        Ok(dxf_file) => {
                            let importer = DxfImporter::new(1.0, 0.0, 0.0);
                            match importer.import_string(&content) {
                                Ok(design) => {
                                    let mut state = designer_mgr_clone.borrow_mut();
                                    for shape in design.shapes {
                                        state.canvas.add_shape(shape);
                                    }
                                    window.set_connection_status(slint::SharedString::from(
                                        format!(
                                            "DXF imported: {} entities from {} layers",
                                            dxf_file.entity_count(),
                                            dxf_file.layer_names().len()
                                        ),
                                    ));
                                    update_designer_ui(&window, &mut state);
                                }
                                Err(e) => {
                                    window.set_connection_status(slint::SharedString::from(
                                        format!("DXF import failed: {}", e),
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            window.set_connection_status(slint::SharedString::from(format!(
                                "DXF parse error: {}",
                                e
                            )));
                        }
                    },
                    Err(e) => {
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Failed to read file: {}",
                            e
                        )));
                    }
                }
            }
        }
    });

    // Designer: Import SVG callback
    let window_weak = main_window.as_weak();
    let designer_mgr_clone = designer_mgr.clone();
    main_window.on_designer_import_svg(move || {
        use gcodekit4::designer::SvgImporter;
        use rfd::FileDialog;

        if let Some(path) = FileDialog::new()
            .add_filter("SVG Files", &["svg"])
            .set_title("Import SVG File")
            .pick_file()
        {
            if let Some(window) = window_weak.upgrade() {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        let importer = SvgImporter::new(1.0, 0.0, 0.0);
                        match importer.import_string(&content) {
                            Ok(design) => {
                                let shape_count = design.shapes.len();
                                let layer_count = design.layer_count;
                                let mut state = designer_mgr_clone.borrow_mut();
                                for shape in design.shapes {
                                    state.canvas.add_shape(shape);
                                }
                                window.set_connection_status(slint::SharedString::from(format!(
                                    "SVG imported: {} shapes from {} layers",
                                    shape_count, layer_count
                                )));
                                update_designer_ui(&window, &mut state);
                            }
                            Err(e) => {
                                window.set_connection_status(slint::SharedString::from(format!(
                                    "SVG import failed: {}",
                                    e
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Failed to read file: {}",
                            e
                        )));
                    }
                }
            }
        }
    });

    // Designer: Export Design callback
    let window_weak = main_window.as_weak();
    main_window.on_designer_export_design(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("Design export: Ready to save"));
        }
    });

    // Designer: File New callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_file_new(move || {
        let mut state = designer_mgr_clone.borrow_mut();
        state.new_design();

        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);
            window.set_connection_status(slint::SharedString::from("New design created"));
        }
    });

    // Designer: File Open callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_file_open(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("GCodeKit4 Design", &["gck4", "json"])
            .pick_file()
        {
            let mut state = designer_mgr_clone.borrow_mut();
            match state.load_from_file(&path) {
                Ok(()) => {
                    if let Some(window) = window_weak.upgrade() {
                        update_designer_ui(&window, &mut state);
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Opened: {}",
                            path.display()
                        )));
                    }
                }
                Err(e) => {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Error opening file: {}",
                            e
                        )));
                    }
                }
            }
        }
    });

    // Designer: File Save callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_file_save(move || {
        let mut state = designer_mgr_clone.borrow_mut();

        // If no current file, prompt for filename
        let path = if let Some(existing_path) = &state.current_file_path {
            existing_path.clone()
        } else if let Some(new_path) = rfd::FileDialog::new()
            .add_filter("GCodeKit4 Design", &["gck4"])
            .set_file_name("design.gck4")
            .save_file()
        {
            new_path
        } else {
            return; // User cancelled
        };

        match state.save_to_file(&path) {
            Ok(()) => {
                if let Some(window) = window_weak.upgrade() {
                    window.set_connection_status(slint::SharedString::from(format!(
                        "Saved: {}",
                        path.display()
                    )));
                }
            }
            Err(e) => {
                if let Some(window) = window_weak.upgrade() {
                    window.set_connection_status(slint::SharedString::from(format!(
                        "Error saving file: {}",
                        e
                    )));
                }
            }
        }
    });

    // Designer: File Save As callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_file_save_as(move || {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("GCodeKit4 Design", &["gck4"])
            .set_file_name("design.gck4")
            .save_file()
        {
            let mut state = designer_mgr_clone.borrow_mut();
            match state.save_to_file(&path) {
                Ok(()) => {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Saved as: {}",
                            path.display()
                        )));
                    }
                }
                Err(e) => {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Error saving file: {}",
                            e
                        )));
                    }
                }
            }
        }
    });

    // Designer: Canvas Click callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_canvas_click(move |x: f32, y: f32| {
        let mut state = designer_mgr_clone.borrow_mut();

        // Convert pixel coordinates to world coordinates
        let world_point = state.canvas.pixel_to_world(x as f64, y as f64);

        // If in Select mode, try to select a shape; otherwise add a new shape
        if state.canvas.mode() == gcodekit4::DrawingMode::Select {
            // Try to select - this will deselect any other shapes
            let _ = state.canvas.select_at(&world_point);
        } else {
            state.add_shape_at(world_point.x, world_point.y);
        }

        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);

            // Update UI state with selected shape ID
            let mut ui_state = window.get_designer_state();
            ui_state.selected_id = state.canvas.selected_id().unwrap_or(0) as i32;
            window.set_designer_state(ui_state);

            window.set_connection_status(slint::SharedString::from(format!(
                "Shapes: {}",
                state.canvas.shapes().len()
            )));
        }
    });

    // Designer: Shape drag callback (move selected shape)
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_shape_drag(move |_shape_id: i32, dx: f32, dy: f32| {
        let mut state = designer_mgr_clone.borrow_mut();

        // Convert pixel delta to world delta using viewport zoom
        // At zoom level Z, moving 1 pixel is equivalent to 1/Z world units
        // Note: Y-axis is flipped - positive pixel dy (down) = negative world dy
        let viewport = state.canvas.viewport();
        let world_dx = dx as f64 / viewport.zoom();
        let world_dy = -(dy as f64) / viewport.zoom(); // Flip Y direction

        state.move_selected(world_dx, world_dy);

        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);
        }
    });

    // Designer: Detect handle callback (check if click is on a resize handle)
    // Returns handle index (-1 if not on a handle): 0=TL, 1=TR, 2=BL, 3=BR, 4=Center
    let designer_mgr_clone = designer_mgr.clone();
    main_window.on_designer_detect_handle(move |x: f32, y: f32| -> i32 {
        let state = designer_mgr_clone.borrow();
        let mut dragging_handle = -1;

        // Convert pixel coordinates to world coordinates
        let world_point = state.canvas.pixel_to_world(x as f64, y as f64);

        if let Some(selected_id) = state.canvas.selected_id() {
            if let Some(obj) = state.canvas.shapes().iter().find(|o| o.id == selected_id) {
                let (x1, y1, x2, y2) = obj.shape.bounding_box();

                // Handle size in world coordinates (scaled by zoom)
                let viewport = state.canvas.viewport();
                let handle_size = 8.0 / viewport.zoom();

                let cx = (x1 + x2) / 2.0;
                let cy = (y1 + y2) / 2.0;

                // Check each handle position
                if (world_point.x - x1).abs() < handle_size
                    && (world_point.y - y1).abs() < handle_size
                {
                    dragging_handle = 0; // Top-left
                } else if (world_point.x - x2).abs() < handle_size
                    && (world_point.y - y1).abs() < handle_size
                {
                    dragging_handle = 1; // Top-right
                } else if (world_point.x - x1).abs() < handle_size
                    && (world_point.y - y2).abs() < handle_size
                {
                    dragging_handle = 2; // Bottom-left
                } else if (world_point.x - x2).abs() < handle_size
                    && (world_point.y - y2).abs() < handle_size
                {
                    dragging_handle = 3; // Bottom-right
                } else if (world_point.x - cx).abs() < handle_size
                    && (world_point.y - cy).abs() < handle_size
                {
                    dragging_handle = 4; // Center (move handle)
                }
            }
        }

        dragging_handle
    });

    // Designer: Handle drag callback (move or resize via handles)
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    let shift_pressed_clone = shift_pressed.clone();
    main_window.on_designer_handle_drag(move |_shape_id: i32, handle: i32, dx: f32, dy: f32| {
        let mut state = designer_mgr_clone.borrow_mut();

        // Convert pixel delta to world delta using viewport zoom
        // Note: Y-axis is flipped - positive pixel dy (down) = negative world dy
        let viewport = state.canvas.viewport();
        let mut world_dx = dx as f64 / viewport.zoom();
        let mut world_dy = -(dy as f64) / viewport.zoom(); // Flip Y direction

        // If Shift is pressed and this is a MOVE (not resize), snap deltas to whole mm
        if *shift_pressed_clone.borrow() && (handle == -1 || handle == 4) {
            world_dx = snap_to_mm(world_dx);
            world_dy = snap_to_mm(world_dy);
        }

        if handle == -1 || handle == 4 {
            // handle=-1 or handle=4 (center handle) means move the entire shape
            state.move_selected(world_dx, world_dy);

            // For moves, also snap the final position to whole mm if Shift is pressed
            if *shift_pressed_clone.borrow() {
                state.snap_selected_to_mm();
            }
        } else {
            // Resize via specific handle (0=top-left, 1=top-right, 2=bottom-left, 3=bottom-right)
            state.resize_selected(handle as usize, world_dx, world_dy);

            // For resizes, also snap the final dimensions to whole mm if Shift is pressed
            if *shift_pressed_clone.borrow() {
                state.snap_selected_to_mm();
            }
        }

        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);
        }
    });

    // Designer: Deselect all callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_deselect_all(move || {
        let mut state = designer_mgr_clone.borrow_mut();
        state.deselect_all();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);

            // Update UI state with no selected shape
            let mut ui_state = window.get_designer_state();
            ui_state.selected_id = 0;
            window.set_designer_state(ui_state);
        }
    });

    // Designer: Shift key state callback
    let shift_pressed_clone = shift_pressed.clone();
    main_window.on_designer_set_shift_pressed(move |pressed: bool| {
        *shift_pressed_clone.borrow_mut() = pressed;
    });

    // Designer: Save shape properties
    // (x, y, w, h, radius, is_pocket, pocket_depth, text_content, font_size, step_down, step_in)
    let pending_properties = Rc::new(RefCell::new((0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, false, 0.0f64, String::new(), 12.0f64, 0.0f64, 0.0f64)));

    let pending_clone = pending_properties.clone();
    main_window.on_designer_update_shape_property(move |prop_id: i32, value: f32| {
        let mut props = pending_clone.borrow_mut();
        match prop_id {
            0 => props.0 = value as f64, // x
            1 => props.1 = value as f64, // y
            2 => props.2 = value as f64, // w
            3 => props.3 = value as f64, // h
            4 => props.4 = value as f64, // radius
            5 => props.6 = value as f64, // pocket_depth
            6 => props.8 = value as f64, // font_size
            7 => props.9 = value as f64, // step_down
            8 => props.10 = value as f64, // step_in
            _ => {}
        }
    });

    let pending_clone_bool = pending_properties.clone();
    main_window.on_designer_update_shape_property_bool(move |prop_id: i32, value: bool| {
        let mut props = pending_clone_bool.borrow_mut();
        match prop_id {
            0 => props.5 = value, // is_pocket
            _ => {}
        }
    });

    let pending_clone_string = pending_properties.clone();
    main_window.on_designer_update_shape_property_string(move |prop_id: i32, value: slint::SharedString| {
        let mut props = pending_clone_string.borrow_mut();
        match prop_id {
            0 => props.7 = value.to_string(), // text_content
            _ => {}
        }
    });

    let designer_mgr_clone2 = designer_mgr.clone();
    let window_weak2 = main_window.as_weak();
    let pending_clone2 = pending_properties.clone();
    main_window.on_designer_save_shape_properties(move || {
        let props = pending_clone2.borrow();
        let mut state = designer_mgr_clone2.borrow_mut();
        state.set_selected_position_and_size(props.0, props.1, props.2, props.3);
        state.set_selected_pocket_properties(props.5, props.6);
        state.set_selected_text_properties(&props.7, props.8);
        state.set_selected_step_down(props.9);
        state.set_selected_step_in(props.10);

        if let Some(window) = window_weak2.upgrade() {
            update_designer_ui(&window, &mut state);
        }
    });

    // Designer: Canvas pan callback (drag on empty canvas)
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    let shift_pressed_clone = shift_pressed.clone();
    main_window.on_designer_canvas_pan(move |dx: f32, dy: f32| {
        let mut state = designer_mgr_clone.borrow_mut();

        // Pan is in pixel space - direct pan offset adjustment
        // Note: Since Y-axis is flipped in world coordinates, pan_y follows screen coordinates
        // Dragging down (positive dy) increases pan_y to show content that was higher up
        // No need to flip Y for panning - pan offsets are in screen space
        let mut pan_dx = dx as f64;
        let mut pan_dy = dy as f64;

        // Apply snapping to whole pixels if Shift is pressed
        if *shift_pressed_clone.borrow() {
            pan_dx = pan_dx.round();
            pan_dy = pan_dy.round();
        }

        state.canvas.pan_by(pan_dx, pan_dy);

        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &mut state);
            // Update UI state with new pan values
            let ui_state = crate::DesignerState {
                mode: state.canvas.mode() as i32,
                zoom: state.canvas.zoom() as f32,
                pan_x: state.canvas.pan_offset().0 as f32,
                pan_y: state.canvas.pan_offset().1 as f32,
                selected_id: state.canvas.selected_id().unwrap_or(0) as i32,
                update_counter: 0,
            };
            window.set_designer_state(ui_state);
        }
    });

    // Designer: Update feed rate
    let designer_mgr_clone = designer_mgr.clone();
    main_window.on_designer_update_feed_rate(move |rate: f32| {
        let mut state = designer_mgr_clone.borrow_mut();
        state.toolpath_generator.set_feed_rate(rate as f64);
    });

    // Designer: Update spindle speed
    let designer_mgr_clone = designer_mgr.clone();
    main_window.on_designer_update_spindle_speed(move |speed: f32| {
        let mut state = designer_mgr_clone.borrow_mut();
        state.toolpath_generator.set_spindle_speed(speed as u32);
    });

    // Designer: Update tool diameter
    let designer_mgr_clone = designer_mgr.clone();
    main_window.on_designer_update_tool_diameter(move |diameter: f32| {
        let mut state = designer_mgr_clone.borrow_mut();
        state.toolpath_generator.set_tool_diameter(diameter as f64);
    });

    // Designer: Update cut depth
    let designer_mgr_clone = designer_mgr.clone();
    main_window.on_designer_update_cut_depth(move |depth: f32| {
        let mut state = designer_mgr_clone.borrow_mut();
        state.toolpath_generator.set_cut_depth(depth as f64);
    });

    // Designer: Update step in
    let designer_mgr_clone = designer_mgr.clone();
    main_window.on_designer_update_step_in(move |step_in: f32| {
        let mut state = designer_mgr_clone.borrow_mut();
        state.toolpath_generator.set_step_in(step_in as f64);
    });


    // Tabbed Box Maker: Open dialog window
    let window_weak = main_window.as_weak();
    let dialog_holder: Rc<RefCell<Option<TabbedBoxDialog>>> = Rc::new(RefCell::new(None));
    main_window.on_generate_tabbed_box(move || {
        if let Some(main_win) = window_weak.upgrade() {
            let dialog = TabbedBoxDialog::new().unwrap();

            // Store dialog in holder to keep it alive
            *dialog_holder.borrow_mut() = Some(dialog.clone_strong());

            // Initialize dialog with current values from main window
            dialog.set_box_x(main_win.get_tbox_x());
            dialog.set_box_y(main_win.get_tbox_y());
            dialog.set_box_h(main_win.get_tbox_h());
            dialog.set_material_thickness(main_win.get_tbox_thickness());
            dialog.set_burn(main_win.get_tbox_burn());
            dialog.set_finger_width(main_win.get_tbox_finger_width());
            dialog.set_space_width(main_win.get_tbox_space_width());
            dialog.set_surrounding_spaces(main_win.get_tbox_surrounding_spaces());
            dialog.set_play(main_win.get_tbox_play());
            dialog.set_extra_length(main_win.get_tbox_extra_length());
            dialog.set_dimple_height(main_win.get_tbox_dimple_height());
            dialog.set_dimple_length(main_win.get_tbox_dimple_length());
            dialog.set_finger_style(main_win.get_tbox_finger_style());
            dialog.set_box_type(main_win.get_tbox_box_type());
            dialog.set_outside_dimensions(main_win.get_tbox_outside_dims());
            dialog.set_laser_passes(main_win.get_tbox_laser_passes());
            dialog.set_laser_power(main_win.get_tbox_laser_power());
            dialog.set_feed_rate(main_win.get_tbox_feed_rate());
            dialog.set_offset_x(main_win.get_tbox_offset_x());
            dialog.set_offset_y(main_win.get_tbox_offset_y());
            dialog.set_dividers_x(main_win.get_tbox_dividers_x());
            dialog.set_dividers_y(main_win.get_tbox_dividers_y());
            dialog.set_optimize_layout(main_win.get_tbox_optimize_layout());
            dialog.set_key_divider_type(main_win.get_tbox_key_divider_type());

            // Cancel button callback
            let dialog_weak_cancel = dialog.as_weak();
            dialog.on_cancel_dialog(move || {
                if let Some(dlg) = dialog_weak_cancel.upgrade() {
                    dlg.hide().ok();
                }
            });

            // Generate button callback
            let main_win_clone = main_win.as_weak();
            let dialog_weak = dialog.as_weak();
            dialog.on_generate_box(move || {
                if let Some(window) = main_win_clone.upgrade() {
                    if let Some(dlg) = dialog_weak.upgrade() {
                        // Read values from dialog
                        let x = dlg.get_box_x().parse::<f32>().unwrap_or(100.0);
                        let y = dlg.get_box_y().parse::<f32>().unwrap_or(100.0);
                        let h = dlg.get_box_h().parse::<f32>().unwrap_or(100.0);
                        let thickness = dlg.get_material_thickness().parse::<f32>().unwrap_or(3.0);
                        let burn = dlg.get_burn().parse::<f32>().unwrap_or(0.1);
                        let finger_width = dlg.get_finger_width().parse::<f32>().unwrap_or(2.0);
                        let space_width = dlg.get_space_width().parse::<f32>().unwrap_or(2.0);
                        let surrounding_spaces =
                            dlg.get_surrounding_spaces().parse::<f32>().unwrap_or(2.0);
                        let play = dlg.get_play().parse::<f32>().unwrap_or(0.0);
                        let extra_length = dlg.get_extra_length().parse::<f32>().unwrap_or(0.0);
                        let dimple_height = dlg.get_dimple_height().parse::<f32>().unwrap_or(0.0);
                        let dimple_length = dlg.get_dimple_length().parse::<f32>().unwrap_or(0.0);
                        let finger_style = FingerStyle::from(dlg.get_finger_style());
                        let box_type = BoxType::from(dlg.get_box_type());
                        let outside = dlg.get_outside_dimensions();
                        let laser_passes =
                            dlg.get_laser_passes().parse::<i32>().unwrap_or(3).max(1);
                        let laser_power =
                            dlg.get_laser_power().parse::<i32>().unwrap_or(1000).max(0);
                        let feed_rate =
                            dlg.get_feed_rate().parse::<f32>().unwrap_or(500.0).max(1.0);
                        let offset_x = dlg.get_offset_x().parse::<f32>().unwrap_or(10.0);
                        let offset_y = dlg.get_offset_y().parse::<f32>().unwrap_or(10.0);
                        let dividers_x = dlg.get_dividers_x().parse::<u32>().unwrap_or(0);
                        let dividers_y = dlg.get_dividers_y().parse::<u32>().unwrap_or(0);
                        let optimize_layout = dlg.get_optimize_layout();
                        let key_divider_type = KeyDividerType::from(dlg.get_key_divider_type());

                        // Save values back to main window for next time
                        window.set_tbox_x(dlg.get_box_x());
                        window.set_tbox_y(dlg.get_box_y());
                        window.set_tbox_h(dlg.get_box_h());
                        window.set_tbox_thickness(dlg.get_material_thickness());
                        window.set_tbox_burn(dlg.get_burn());
                        window.set_tbox_finger_width(dlg.get_finger_width());
                        window.set_tbox_space_width(dlg.get_space_width());
                        window.set_tbox_surrounding_spaces(dlg.get_surrounding_spaces());
                        window.set_tbox_play(dlg.get_play());
                        window.set_tbox_extra_length(dlg.get_extra_length());
                        window.set_tbox_dimple_height(dlg.get_dimple_height());
                        window.set_tbox_dimple_length(dlg.get_dimple_length());
                        window.set_tbox_finger_style(dlg.get_finger_style());
                        window.set_tbox_box_type(dlg.get_box_type());
                        window.set_tbox_outside_dims(dlg.get_outside_dimensions());
                        window.set_tbox_laser_passes(dlg.get_laser_passes());
                        window.set_tbox_laser_power(dlg.get_laser_power());
                        window.set_tbox_feed_rate(dlg.get_feed_rate());
                        window.set_tbox_offset_x(dlg.get_offset_x());
                        window.set_tbox_offset_y(dlg.get_offset_y());
                        window.set_tbox_dividers_x(dlg.get_dividers_x());
                        window.set_tbox_dividers_y(dlg.get_dividers_y());
                        window.set_tbox_optimize_layout(dlg.get_optimize_layout());
                        window.set_tbox_key_divider_type(dlg.get_key_divider_type());

                        let finger_joint = FingerJointSettings {
                            finger: finger_width,
                            space: space_width,
                            surrounding_spaces,
                            play,
                            extra_length,
                            style: finger_style,
                            dimple_height,
                            dimple_length,
                        };

                        let params = BoxParameters {
                            x,
                            y,
                            h,
                            thickness,
                            outside,
                            box_type,
                            finger_joint,
                            burn,
                            laser_passes,
                            laser_power,
                            feed_rate,
                            offset_x,
                            offset_y,
                            dividers_x,
                            dividers_y,
                            optimize_layout,
                            key_divider_type,
                        };

                        // Show progress and spawn background thread
                        window.set_connection_status("Generating tabbed box G-code...".into());
                        window.set_progress_value(0.1); // 10% - Starting

                        // Close dialog immediately
                        dlg.hide().ok();

                        // Spawn background thread for generation
                        let window_weak_thread = window.as_weak();
                        std::thread::spawn(move || {
                            let result = TabbedBoxMaker::new(params.clone())
                                .and_then(|mut maker| {
                                    // Update progress: 30% - Parameters validated
                                    let _ = slint::invoke_from_event_loop({
                                        let ww = window_weak_thread.clone();
                                        move || {
                                            if let Some(w) = ww.upgrade() {
                                                w.set_progress_value(0.3);
                                            }
                                        }
                                    });

                                    maker.generate().map(|_| maker)
                                })
                                .map(|maker| {
                                    // Update progress: 70% - Generation complete
                                    let _ = slint::invoke_from_event_loop({
                                        let ww = window_weak_thread.clone();
                                        move || {
                                            if let Some(w) = ww.upgrade() {
                                                w.set_progress_value(0.7);
                                            }
                                        }
                                    });

                                    maker.to_gcode()
                                });

                            // Update UI from main thread
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(win) = window_weak_thread.upgrade() {
                                    match result {
                                        Ok(gcode) => {
                                            win.invoke_load_editor_text(slint::SharedString::from(
                                                gcode.clone(),
                                            ));

                                            win.set_gcode_filename(slint::SharedString::from(
                                                format!(
                                                    "box_{}x{}x{}.gcode",
                                                    x as i32, y as i32, h as i32
                                                ),
                                            ));
                                            win.set_current_view(slint::SharedString::from(
                                                "gcode-editor",
                                            ));
                                            win.set_connection_status(slint::SharedString::from(
                                                "Tabbed box G-code generated successfully",
                                            ));
                                            win.set_progress_value(1.0); // 100%

                                            // Show success dialog
                                            let success_dialog = ErrorDialog::new().unwrap();
                                            success_dialog.set_error_message(
                                                slint::SharedString::from("Tabbed box G-code has been generated and loaded into the editor."),
                                            );

                                            let success_dialog_weak = success_dialog.as_weak();
                                            success_dialog.on_close_dialog(move || {
                                                if let Some(dlg) = success_dialog_weak.upgrade() {
                                                    dlg.hide().ok();
                                                }
                                            });

                                            success_dialog.show().ok();

                                            // Hide progress after 1 second
                                            let win_weak = win.as_weak();
                                            slint::Timer::single_shot(
                                                std::time::Duration::from_secs(1),
                                                move || {
                                                    if let Some(w) = win_weak.upgrade() {
                                                        w.set_progress_value(0.0);
                                                    }
                                                },
                                            );
                                        }
                                        Err(e) => {
                                            let error_msg =
                                                format!("Failed to generate box: {}", e);
                                            win.set_connection_status(slint::SharedString::from(
                                                &error_msg,
                                            ));
                                            win.set_progress_value(0.0); // Hide progress

                                            // Show error dialog
                                            let error_dialog = ErrorDialog::new().unwrap();
                                            error_dialog.set_error_message(
                                                slint::SharedString::from(&error_msg),
                                            );

                                            let error_dialog_weak = error_dialog.as_weak();
                                            error_dialog.on_close_dialog(move || {
                                                if let Some(dlg) = error_dialog_weak.upgrade() {
                                                    dlg.hide().ok();
                                                }
                                            });

                                            error_dialog.show().ok();
                                        }
                                    }
                                }
                            });
                        });
                    }
                }
            });

            dialog.show().unwrap();
        }
    });

    // Jigsaw Puzzle Maker
    let window_weak = main_window.as_weak();
    let dialog_holder: Rc<RefCell<Option<JigsawPuzzleDialog>>> = Rc::new(RefCell::new(None));
    main_window.on_generate_jigsaw_puzzle(move || {
        if let Some(main_win) = window_weak.upgrade() {
            let dialog = JigsawPuzzleDialog::new().unwrap();

            // Store dialog in holder to keep it alive
            *dialog_holder.borrow_mut() = Some(dialog.clone_strong());

            // Initialize dialog with current values from main window
            dialog.set_puzzle_width(main_win.get_puzzle_width());
            dialog.set_puzzle_height(main_win.get_puzzle_height());
            dialog.set_pieces_across(main_win.get_puzzle_pieces_across());
            dialog.set_pieces_down(main_win.get_puzzle_pieces_down());
            dialog.set_kerf(main_win.get_puzzle_kerf());
            dialog.set_laser_passes(main_win.get_puzzle_laser_passes());
            dialog.set_laser_power(main_win.get_puzzle_laser_power());
            dialog.set_feed_rate(main_win.get_puzzle_feed_rate());
            dialog.set_seed(main_win.get_puzzle_seed());
            dialog.set_tab_size(main_win.get_puzzle_tab_size());
            dialog.set_jitter(main_win.get_puzzle_jitter());
            dialog.set_corner_radius(main_win.get_puzzle_corner_radius());

            // Cancel button callback
            let dialog_weak_cancel = dialog.as_weak();
            dialog.on_cancel_dialog(move || {
                if let Some(dlg) = dialog_weak_cancel.upgrade() {
                    dlg.hide().ok();
                }
            });

            // Generate button callback
            let main_win_clone = main_win.as_weak();
            let dialog_weak = dialog.as_weak();
            dialog.on_generate_puzzle(move || {
                if let Some(window) = main_win_clone.upgrade() {
                    if let Some(dlg) = dialog_weak.upgrade() {
                        // Read values from dialog
                        let width = dlg.get_puzzle_width().parse::<f32>().unwrap_or(200.0);
                        let height = dlg.get_puzzle_height().parse::<f32>().unwrap_or(150.0);
                        let pieces_across = dlg.get_pieces_across().parse::<i32>().unwrap_or(4);
                        let pieces_down = dlg.get_pieces_down().parse::<i32>().unwrap_or(3);
                        let kerf = dlg.get_kerf().parse::<f32>().unwrap_or(0.5);
                        let laser_passes =
                            dlg.get_laser_passes().parse::<i32>().unwrap_or(3).max(1);
                        let laser_power =
                            dlg.get_laser_power().parse::<i32>().unwrap_or(1000).max(0);
                        let feed_rate =
                            dlg.get_feed_rate().parse::<f32>().unwrap_or(500.0).max(1.0);
                        let seed = dlg.get_seed().parse::<u32>().unwrap_or(42);
                        let tab_size_percent = dlg
                            .get_tab_size()
                            .parse::<f32>()
                            .unwrap_or(20.0)
                            .clamp(10.0, 30.0);
                        let jitter_percent = dlg
                            .get_jitter()
                            .parse::<f32>()
                            .unwrap_or(4.0)
                            .clamp(0.0, 13.0);
                        let corner_radius = dlg
                            .get_corner_radius()
                            .parse::<f32>()
                            .unwrap_or(2.0)
                            .clamp(0.0, 10.0);
                        let offset_x = dlg.get_offset_x().parse::<f32>().unwrap_or(10.0);
                        let offset_y = dlg.get_offset_y().parse::<f32>().unwrap_or(10.0);

                        // Save values back to main window for next time
                        window.set_puzzle_width(dlg.get_puzzle_width());
                        window.set_puzzle_height(dlg.get_puzzle_height());
                        window.set_puzzle_pieces_across(dlg.get_pieces_across());
                        window.set_puzzle_pieces_down(dlg.get_pieces_down());
                        window.set_puzzle_kerf(dlg.get_kerf());
                        window.set_puzzle_laser_passes(dlg.get_laser_passes());
                        window.set_puzzle_laser_power(dlg.get_laser_power());
                        window.set_puzzle_feed_rate(dlg.get_feed_rate());
                        window.set_puzzle_seed(dlg.get_seed());
                        window.set_puzzle_tab_size(dlg.get_tab_size());
                        window.set_puzzle_jitter(dlg.get_jitter());
                        window.set_puzzle_corner_radius(dlg.get_corner_radius());
                        window.set_puzzle_offset_x(dlg.get_offset_x());
                        window.set_puzzle_offset_y(dlg.get_offset_y());

                        let params = PuzzleParameters {
                            width,
                            height,
                            pieces_across,
                            pieces_down,
                            kerf,
                            laser_passes,
                            laser_power,
                            feed_rate,
                            seed,
                            tab_size_percent,
                            jitter_percent,
                            corner_radius,
                            offset_x,
                            offset_y,
                        };

                        // Show progress and spawn background thread
                        window.set_connection_status("Generating jigsaw puzzle G-code...".into());
                        window.set_progress_value(0.1); // 10% - Starting

                        // Close dialog immediately
                        dlg.hide().ok();

                        // Spawn background thread for generation
                        let window_weak_thread = window.as_weak();
                        std::thread::spawn(move || {
                            let result = JigsawPuzzleMaker::new(params.clone())
                                .and_then(|mut maker| {
                                    // Update progress: 30% - Parameters validated
                                    let _ = slint::invoke_from_event_loop({
                                        let ww = window_weak_thread.clone();
                                        move || {
                                            if let Some(w) = ww.upgrade() {
                                                w.set_progress_value(0.3);
                                            }
                                        }
                                    });

                                    maker.generate().map(|_| maker)
                                })
                                .map(|maker| {
                                    // Update progress: 70% - Generation complete
                                    let _ = slint::invoke_from_event_loop({
                                        let ww = window_weak_thread.clone();
                                        move || {
                                            if let Some(w) = ww.upgrade() {
                                                w.set_progress_value(0.7);
                                            }
                                        }
                                    });

                                    maker.to_gcode(300.0, 3.0)
                                });

                            // Update UI from main thread
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(win) = window_weak_thread.upgrade() {
                                    match result {
                                        Ok(gcode) => {
                                            win.invoke_load_editor_text(slint::SharedString::from(
                                                gcode.clone(),
                                            ));

                                            win.set_gcode_filename(slint::SharedString::from(
                                                format!(
                                                    "puzzle_{}x{}_{}x{}.gcode",
                                                    width as i32,
                                                    height as i32,
                                                    pieces_across,
                                                    pieces_down
                                                ),
                                            ));
                                            win.set_current_view(slint::SharedString::from(
                                                "gcode-editor",
                                            ));
                                            win.set_connection_status(slint::SharedString::from(
                                                "Jigsaw puzzle G-code generated successfully",
                                            ));
                                            win.set_progress_value(1.0); // 100%

                                            // Show success dialog
                                            let success_dialog = ErrorDialog::new().unwrap();
                                            success_dialog.set_error_message(
                                                slint::SharedString::from("Jigsaw puzzle G-code has been generated and loaded into the editor."),
                                            );

                                            let success_dialog_weak = success_dialog.as_weak();
                                            success_dialog.on_close_dialog(move || {
                                                if let Some(dlg) = success_dialog_weak.upgrade() {
                                                    dlg.hide().ok();
                                                }
                                            });

                                            success_dialog.show().ok();

                                            // Hide progress after 1 second
                                            let win_weak = win.as_weak();
                                            slint::Timer::single_shot(
                                                std::time::Duration::from_secs(1),
                                                move || {
                                                    if let Some(w) = win_weak.upgrade() {
                                                        w.set_progress_value(0.0);
                                                    }
                                                },
                                            );
                                        }
                                        Err(e) => {
                                            let error_msg =
                                                format!("Failed to generate puzzle: {}", e);
                                            win.set_connection_status(slint::SharedString::from(
                                                &error_msg,
                                            ));
                                            win.set_progress_value(0.0); // Hide progress

                                            // Show error dialog
                                            let error_dialog = ErrorDialog::new().unwrap();
                                            error_dialog.set_error_message(
                                                slint::SharedString::from(&error_msg),
                                            );

                                            let error_dialog_weak = error_dialog.as_weak();
                                            error_dialog.on_close_dialog(move || {
                                                if let Some(dlg) = error_dialog_weak.upgrade() {
                                                    dlg.hide().ok();
                                                }
                                            });

                                            error_dialog.show().ok();
                                        }
                                    }
                                }
                            });
                        });
                    }
                }
            });

            dialog.show().unwrap();
        }
    });

    // Laser Image Engraver
    let window_weak = main_window.as_weak();
    let dialog_holder: Rc<RefCell<Option<LaserEngraverDialog>>> = Rc::new(RefCell::new(None));
    let editor_bridge_laser = editor_bridge.clone();
    main_window.on_generate_laser_engraving(move || {
        if let Some(main_win) = window_weak.upgrade() {
            let dialog = LaserEngraverDialog::new().unwrap();

            // Store dialog in holder to keep it alive
            *dialog_holder.borrow_mut() = Some(dialog.clone_strong());

            // Initialize dialog with default values
            dialog.set_width_mm(100.0);
            dialog.set_feed_rate(1000.0);
            dialog.set_travel_rate(3000.0);
            dialog.set_min_power(0.0);
            dialog.set_max_power(100.0);
            dialog.set_pixels_per_mm(10.0);
            dialog.set_scan_direction("Horizontal".into());
            dialog.set_bidirectional(true);
            dialog.set_invert(false);
            dialog.set_line_spacing(1.0);
            dialog.set_power_scale(1000.0);

            // Load image callback
            let dialog_weak_load = dialog.as_weak();
            dialog.on_load_image(move || {
                if let Some(dlg) = dialog_weak_load.upgrade() {
                    // Open file dialog to select image
                    use rfd::FileDialog;
                    if let Some(path) = FileDialog::new()
                        .add_filter("Image Files", &["png", "jpg", "jpeg", "bmp", "gif", "tiff"])
                        .pick_file()
                    {
                        dlg.set_image_path(path.display().to_string().into());

                        // Load and convert image to Slint format for preview
                        if let Ok(img) = image::open(&path) {
                            // Convert to RGB8 for display
                            let rgb_img = img.to_rgb8();
                            let width = rgb_img.width();
                            let height = rgb_img.height();

                            // Create Slint image buffer
                            let buffer =
                                slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
                                    rgb_img.as_raw(),
                                    width,
                                    height,
                                );
                            dlg.set_preview_image(slint::Image::from_rgb8(buffer));

                            // Calculate and display output size
                            let _pixels_per_mm = dlg.get_pixels_per_mm();
                            let width_mm = dlg.get_width_mm();
                            let aspect_ratio = height as f32 / width as f32;
                            let height_mm = width_mm * aspect_ratio;
                            dlg.set_output_size(
                                format!("{:.1} x {:.1} mm", width_mm, height_mm).into(),
                            );
                        }
                    }
                }
            });

            // Update preview callback (when parameters change)
            let dialog_weak_update = dialog.as_weak();
            dialog.on_update_preview(move || {
                if let Some(dlg) = dialog_weak_update.upgrade() {
                    let image_path = dlg.get_image_path().to_string();
                    if !image_path.is_empty() {
                        if let Ok(img) = image::open(&image_path) {
                            let width_mm = dlg.get_width_mm();
                            let pixels_per_mm = dlg.get_pixels_per_mm();
                            let feed_rate = dlg.get_feed_rate();
                            let travel_rate = dlg.get_travel_rate();
                            let bidirectional = dlg.get_bidirectional();
                            let line_spacing = dlg.get_line_spacing();

                            // Calculate output dimensions
                            let aspect_ratio = img.height() as f32 / img.width() as f32;
                            let height_mm = width_mm * aspect_ratio;
                            dlg.set_output_size(
                                format!("{:.1} x {:.1} mm", width_mm, height_mm).into(),
                            );

                            // Estimate engraving time
                            let num_lines = (height_mm * pixels_per_mm / line_spacing) as u32;
                            let engrave_time = (width_mm * num_lines as f32) / feed_rate * 60.0;
                            let travel_time = if bidirectional {
                                (height_mm / travel_rate) * 60.0
                            } else {
                                (width_mm * num_lines as f32) / travel_rate * 60.0
                            };
                            let total_seconds = engrave_time + travel_time;
                            let minutes = (total_seconds / 60.0) as i32;
                            let seconds = (total_seconds % 60.0) as i32;
                            dlg.set_estimated_time(format!("{}:{:02}", minutes, seconds).into());
                        }
                    }
                }
            });

            // Generate G-code callback
            let main_win_clone = main_win.as_weak();
            let dialog_weak_generate = dialog.as_weak();
            let _editor_bridge_engraver = editor_bridge_laser.clone();
            dialog.on_generate_gcode(move || {
                if let Some(window) = main_win_clone.upgrade() {
                    if let Some(dlg) = dialog_weak_generate.upgrade() {
                        let image_path = dlg.get_image_path().to_string();

                        if image_path.is_empty() {
                            let error_dialog = ErrorDialog::new().unwrap();
                            error_dialog.set_error_message(
                                "No Image Selected\n\nPlease select an image file first.".into(),
                            );

                            let error_dialog_weak = error_dialog.as_weak();
                            error_dialog.on_close_dialog(move || {
                                if let Some(dlg) = error_dialog_weak.upgrade() {
                                    dlg.hide().ok();
                                }
                            });

                            error_dialog.show().ok();
                            return;
                        }

                        // Create engraving parameters - collect all data before spawning thread
                        use gcodekit4_camtools::{
                            EngravingParameters, ImageTransformations, BitmapImageEngraver, ScanDirection,
                            RotationAngle, HalftoneMethod,
                        };

                        let width_mm = dlg.get_width_mm();
                        let feed_rate = dlg.get_feed_rate();
                        let travel_rate = dlg.get_travel_rate();
                        let min_power = dlg.get_min_power();
                        let max_power = dlg.get_max_power();
                        let pixels_per_mm = dlg.get_pixels_per_mm();
                        let scan_dir = dlg.get_scan_direction().to_string();
                        let bidirectional = dlg.get_bidirectional();
                        let invert = dlg.get_invert();
                        let line_spacing = dlg.get_line_spacing();
                        let power_scale = dlg.get_power_scale();
                        let offset_x = dlg.get_offset_x().parse::<f32>().unwrap_or(10.0);
                        let offset_y = dlg.get_offset_y().parse::<f32>().unwrap_or(10.0);
                        
                        // Get transformation parameters from dialog
                        let mirror_x = false;
                        let mirror_y = false;
                        let rotation_str = "0°".to_string();
                        let halftone_str = dlg.get_halftone();
                        let halftone_dot_size = dlg.get_halftone_dot_size() as usize;
                        let halftone_threshold = 127;

                        // Show status message and initial progress
                        window.set_connection_status("Generating laser engraving G-code...".into());
                        window.set_progress_value(0.0); // Starting

                        // Close dialog immediately
                        dlg.hide().ok();

                        // Spawn thread FIRST, before any UI operations
                        let window_weak_thread = window.as_weak();
                        let image_path_clone = image_path.clone();
                        std::thread::spawn(move || {

                            let params = EngravingParameters {
                                width_mm,
                                height_mm: None,
                                feed_rate,
                                travel_rate,
                                min_power,
                                max_power,
                                pixels_per_mm,
                                scan_direction: if scan_dir == "Horizontal" {
                                    ScanDirection::Horizontal
                                } else {
                                    ScanDirection::Vertical
                                },
                                bidirectional,
                                line_spacing,
                                power_scale,
                                transformations: ImageTransformations {
                                    mirror_x,
                                    mirror_y,
                                    rotation: match rotation_str.as_str() {
                                        "90°" => RotationAngle::Degrees90,
                                        "180°" => RotationAngle::Degrees180,
                                        "270°" => RotationAngle::Degrees270,
                                        _ => RotationAngle::Degrees0,
                                    },
                                    halftone: match halftone_str.as_str() {
                                        "Circle" => HalftoneMethod::Circle,
                                        "Cross" => HalftoneMethod::Cross,
                                        "Ellipse" => HalftoneMethod::Ellipse,
                                        "Line" => HalftoneMethod::Line,
                                        "Inverted Line" => HalftoneMethod::InvertedLine,
                                        _ => HalftoneMethod::None,
                                    },
                                    halftone_dot_size,
                                    halftone_threshold: halftone_threshold as u8,
                                    invert,
                                },
                                offset_x,
                                offset_y,
                            };

                            let result = BitmapImageEngraver::from_file(&image_path_clone, params)
                                .and_then(|engraver| {
                                    // Generate G-code with progress updates (0-100%)
                                    let gcode =
                                        engraver.generate_gcode_with_progress(|progress| {
                                            // Map internal progress (0.0-1.0) to 0-100% range
                                            let overall_progress = progress * 100.0;
                                            let _ = slint::invoke_from_event_loop({
                                                let ww = window_weak_thread.clone();
                                                move || {
                                                    if let Some(w) = ww.upgrade() {
                                                        w.set_progress_value(overall_progress);
                                                    }
                                                }
                                            });
                                        })?;

                                    Ok(gcode)
                                });


                            // Update UI from the main thread using slint::invoke_from_event_loop
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(win) = window_weak_thread.upgrade() {
                                    match result {
                                        Ok(gcode) => {
                                            win.set_progress_value(95.0); // Show progress before UI update
                                            win.set_connection_status(
                                                "Loading G-code into editor...".into(),
                                            );

                                            // Load into custom editor using callbacks
                                            win.invoke_load_editor_text(slint::SharedString::from(
                                                gcode.clone(),
                                            ));

                                            // Switch to editor view
                                            win.set_current_view("gcode-editor".into());
                                            win.set_gcode_focus_trigger(win.get_gcode_focus_trigger() + 1);
                                            win.set_connection_status(
                                                "Laser engraving G-code generated successfully"
                                                    .into(),
                                            );
                                            win.set_progress_value(100.0); // 100%

                                            // Show success dialog
                                            let success_dialog = ErrorDialog::new().unwrap();
                                            success_dialog.set_error_message(
                                                slint::SharedString::from("Laser engraving G-code has been generated and loaded into the editor."),
                                            );

                                            let success_dialog_weak = success_dialog.as_weak();
                                            success_dialog.on_close_dialog(move || {
                                                if let Some(dlg) = success_dialog_weak.upgrade() {
                                                    dlg.hide().ok();
                                                }
                                            });

                                            success_dialog.show().ok();

                                            // Hide progress after 1 second
                                            let win_weak = win.as_weak();
                                            slint::Timer::single_shot(
                                                std::time::Duration::from_secs(1),
                                                move || {
                                                    if let Some(w) = win_weak.upgrade() {
                                                        w.set_progress_value(0.0);
                                                    }
                                                },
                                            );
                                        }
                                        Err(e) => {
                                            // Build detailed error message with full chain
                                            let mut error_details = String::new();
                                            error_details.push_str("G-code Generation Failed\n\n");
                                            
                                            // Add root error
                                            error_details.push_str(&format!("Error: {}\n", e));
                                            
                                            // Add error chain if available (anyhow provides this)
                                            let mut source = e.source();
                                            let mut depth = 0;
                                            while let Some(err) = source {
                                                depth += 1;
                                                error_details.push_str(&format!("  └─ {}: {}\n", depth, err));
                                                source = err.source();
                                            }
                                            
                                            let error_msg = format!("Failed to generate laser engraving: {}", e);
                                            win.set_connection_status(error_msg.clone().into());
                                            win.set_progress_value(0.0); // Hide progress
                                            tracing::error!("G-code generation error:\n{}", error_details);

                                            let error_dialog = ErrorDialog::new().unwrap();
                                            error_dialog.set_error_message(error_details.into());

                                            let error_dialog_weak = error_dialog.as_weak();
                                            error_dialog.on_close_dialog(move || {
                                                if let Some(dlg) = error_dialog_weak.upgrade() {
                                                    dlg.hide().ok();
                                                }
                                            });

                                            error_dialog.show().ok();
                                        }
                                    }
                                }
                            });
                        });
                    }
                }
            });

            // Close dialog callback
            let dialog_weak_close = dialog.as_weak();
            dialog.on_close_dialog(move || {
                if let Some(dlg) = dialog_weak_close.upgrade() {
                    dlg.hide().ok();
                }
            });

            dialog.show().unwrap();
        }
    });

    // Vector Image Engraver
    let window_weak = main_window.as_weak();
    let dialog_holder_vector: Rc<RefCell<Option<VectorEngraverDialog>>> = Rc::new(RefCell::new(None));
    let editor_bridge_vector = editor_bridge.clone();
    main_window.on_generate_vector_engraving(move || {
        if let Some(main_win) = window_weak.upgrade() {
            let dialog = VectorEngraverDialog::new().unwrap();

            // Store dialog in holder to keep it alive
            *dialog_holder_vector.borrow_mut() = Some(dialog.clone_strong());

            // Initialize dialog with default values
            dialog.set_feed_rate(600.0);
            dialog.set_travel_rate(3000.0);
            dialog.set_cut_power(100.0);
            dialog.set_engrave_power(50.0);
            dialog.set_power_scale(1000.0);
            dialog.set_multi_pass(false);
            dialog.set_num_passes(1);
            dialog.set_z_increment(0.5);
            dialog.set_invert_power(false);

            // Load vector file callback
            let dialog_weak_load = dialog.as_weak();
            dialog.on_load_vector_file(move || {
                if let Some(dlg) = dialog_weak_load.upgrade() {
                    // Open file dialog to select vector file
                    use rfd::FileDialog;
                    if let Some(path) = FileDialog::new()
                        .add_filter("Vector Files", &["svg", "dxf"])
                        .pick_file()
                    {
                        dlg.set_vector_path(path.display().to_string().into());

                        // Display file info
                        let file_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown");
                        let file_info = format!("{} ({})", file_name, 
                            path.extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("unknown")
                                .to_uppercase());
                        dlg.set_file_info(file_info.into());
                    }
                }
            });

            // Update preview callback (when parameters change)
            let dialog_weak_update = dialog.as_weak();
            dialog.on_update_preview(move || {
                if let Some(dlg) = dialog_weak_update.upgrade() {
                    let vector_path = dlg.get_vector_path().to_string();
                    if !vector_path.is_empty() {
                        let _feed_rate = dlg.get_feed_rate();
                        let _travel_rate = dlg.get_travel_rate();
                        let multi_pass = dlg.get_multi_pass();
                        let num_passes = dlg.get_num_passes();

                        // Estimate cutting time (placeholder)
                        let base_time = 60.0; // seconds - would be calculated from vector analysis
                        let total_time = if multi_pass {
                            base_time * num_passes as f32
                        } else {
                            base_time
                        };
                        let minutes = (total_time / 60.0) as i32;
                        let seconds = (total_time % 60.0) as i32;
                        dlg.set_estimated_time(format!("{}:{:02}", minutes, seconds).into());
                    }
                }
            });

            // Generate G-code callback
            let main_win_clone = main_win.as_weak();
            let dialog_weak_generate = dialog.as_weak();
            let _editor_bridge_engraver_vector = editor_bridge_vector.clone();
            dialog.on_generate_gcode(move || {
                if let Some(window) = main_win_clone.upgrade() {
                    if let Some(dlg) = dialog_weak_generate.upgrade() {
                        let vector_path = dlg.get_vector_path().to_string();

                        if vector_path.is_empty() {
                            let error_dialog = ErrorDialog::new().unwrap();
                            error_dialog.set_error_message(
                                "No Vector File Selected\n\nPlease select an SVG or DXF file first.".into(),
                            );

                            let error_dialog_weak = error_dialog.as_weak();
                            error_dialog.on_close_dialog(move || {
                                if let Some(dlg) = error_dialog_weak.upgrade() {
                                    dlg.hide().ok();
                                }
                            });

                            error_dialog.show().ok();
                            return;
                        }

                        // Create vector engraving parameters
                        use gcodekit4_camtools::{
                            VectorEngraver, VectorEngravingParameters,
                        };

                        let feed_rate = dlg.get_feed_rate();
                        let travel_rate = dlg.get_travel_rate();
                        let cut_power = dlg.get_cut_power();
                        let engrave_power = dlg.get_engrave_power();
                        let power_scale = dlg.get_power_scale();
                        let multi_pass = dlg.get_multi_pass();
                        let num_passes = dlg.get_num_passes() as u32;
                        let z_increment = dlg.get_z_increment();
                        let invert_power = dlg.get_invert_power();
                        let desired_width = dlg.get_desired_width();
                        let offset_x = dlg.get_offset_x().parse::<f32>().unwrap_or(10.0);
                        let offset_y = dlg.get_offset_y().parse::<f32>().unwrap_or(10.0);
                        let enable_hatch = dlg.get_enable_hatch();
                        let hatch_angle = dlg.get_hatch_angle();
                        let hatch_spacing = dlg.get_hatch_spacing();
                        let hatch_tolerance = dlg.get_hatch_tolerance();
                        let cross_hatch = dlg.get_cross_hatch();
                        let enable_dwell = dlg.get_enable_dwell();
                        let dwell_time = dlg.get_dwell_time();

                        // Show status message and initial progress
                        window.set_connection_status("Generating vector engraving G-code...".into());
                        window.set_progress_value(0.0); // Starting

                        // Close dialog immediately
                        dlg.hide().ok();

                        // Spawn thread FIRST, before any UI operations
                        let window_weak_thread = window.as_weak();
                        let vector_path_clone = vector_path.clone();
                        std::thread::spawn(move || {

                            let params = VectorEngravingParameters {
                                feed_rate,
                                travel_rate,
                                cut_power,
                                engrave_power,
                                power_scale,
                                multi_pass,
                                num_passes,
                                z_increment,
                                invert_power,
                                desired_width,
                                offset_x,
                                offset_y,
                                enable_hatch,
                                hatch_angle,
                                hatch_spacing,
                                hatch_tolerance,
                                cross_hatch,
                                enable_dwell,
                                dwell_time,
                            };

                            let result = VectorEngraver::from_file(&vector_path_clone, params)
                                .and_then(|engraver| {
                                    // Generate G-code with progress updates (0-100%)
                                    let gcode =
                                        engraver.generate_gcode_with_progress(|progress| {
                                            // Map internal progress (0.0-1.0) to 0-100% range
                                            let overall_progress = progress * 100.0;
                                            let _ = slint::invoke_from_event_loop({
                                                let ww = window_weak_thread.clone();
                                                move || {
                                                    if let Some(w) = ww.upgrade() {
                                                        w.set_progress_value(overall_progress);
                                                    }
                                                }
                                            });
                                        })?;

                                    Ok(gcode)
                                });


                            // Update UI from the main thread using slint::invoke_from_event_loop
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(win) = window_weak_thread.upgrade() {
                                    match result {
                                        Ok(gcode) => {
                                            win.set_progress_value(95.0); // Show progress before UI update
                                            win.set_connection_status(
                                                "Loading G-code into editor...".into(),
                                            );

                                            // Load into custom editor using callbacks
                                            win.invoke_load_editor_text(slint::SharedString::from(
                                                gcode.clone(),
                                            ));

                                            // Switch to editor view
                                            win.set_current_view("gcode-editor".into());
                                            win.set_gcode_focus_trigger(win.get_gcode_focus_trigger() + 1);
                                            win.set_connection_status(
                                                "Vector engraving G-code generated successfully"
                                                    .into(),
                                            );
                                            win.set_progress_value(100.0); // 100%

                                            // Trigger visualizer refresh after switching view
                                            let win_weak_viz = win.as_weak();
                                            slint::Timer::single_shot(
                                                std::time::Duration::from_millis(100),
                                                move || {
                                                    if let Some(w) = win_weak_viz.upgrade() {
                                                        let canvas_width = w.get_visualizer_canvas_width();
                                                        let canvas_height = w.get_visualizer_canvas_height();
                                                        w.invoke_refresh_visualization(canvas_width, canvas_height);
                                                    }
                                                },
                                            );

                                            // Show success dialog
                                            let success_dialog = ErrorDialog::new().unwrap();
                                            success_dialog.set_error_message(
                                                slint::SharedString::from("Vector engraving G-code has been generated and loaded into the editor."),
                                            );

                                            let success_dialog_weak = success_dialog.as_weak();
                                            success_dialog.on_close_dialog(move || {
                                                if let Some(dlg) = success_dialog_weak.upgrade() {
                                                    dlg.hide().ok();
                                                }
                                            });

                                            success_dialog.show().ok();

                                            // Hide progress after 1 second
                                            let win_weak = win.as_weak();
                                            slint::Timer::single_shot(
                                                std::time::Duration::from_secs(1),
                                                move || {
                                                    if let Some(w) = win_weak.upgrade() {
                                                        w.set_progress_value(0.0);
                                                    }
                                                },
                                            );
                                        }
                                        Err(e) => {
                                            // Build detailed error message with full chain
                                            let mut error_details = String::new();
                                            error_details.push_str("Vector G-code Generation Failed\n\n");
                                            
                                            // Add root error
                                            error_details.push_str(&format!("Error: {}\n", e));
                                            
                                            // Add error chain if available
                                            let mut source = e.source();
                                            let mut depth = 0;
                                            while let Some(err) = source {
                                                depth += 1;
                                                error_details.push_str(&format!("  └─ {}: {}\n", depth, err));
                                                source = err.source();
                                            }
                                            
                                            let error_msg = format!("Failed to generate vector engraving: {}", e);
                                            win.set_connection_status(error_msg.clone().into());
                                            win.set_progress_value(0.0); // Hide progress
                                            tracing::error!("Vector G-code generation error:\n{}", error_details);

                                            let error_dialog = ErrorDialog::new().unwrap();
                                            error_dialog.set_error_message(error_details.into());

                                            let error_dialog_weak = error_dialog.as_weak();
                                            error_dialog.on_close_dialog(move || {
                                                if let Some(dlg) = error_dialog_weak.upgrade() {
                                                    dlg.hide().ok();
                                                }
                                            });

                                            error_dialog.show().ok();
                                        }
                                    }
                                }
                            });
                        });
                    }
                }
            });

            // Close dialog callback
            let dialog_weak_close = dialog.as_weak();
            dialog.on_close_dialog(move || {
                if let Some(dlg) = dialog_weak_close.upgrade() {
                    dlg.hide().ok();
                }
            });

            dialog.show().unwrap();
        }
    });

    // Speeds and Feeds Calculator
    let window_weak = main_window.as_weak();
    let materials_backend_sf = materials_backend.clone();
    let tools_backend_sf = tools_backend.clone();
    let device_manager_sf = device_manager.clone();
    
    main_window.on_load_speeds_feeds_data(move || {
        if let Some(window) = window_weak.upgrade() {
            // Load Materials
            let backend = materials_backend_sf.borrow();
            let materials = backend.get_all_materials();
            let material_names: Vec<slint::SharedString> = materials.iter()
                .map(|m| slint::SharedString::from(m.name.clone()))
                .collect();
            window.set_sf_materials(slint::ModelRc::new(VecModel::from(material_names)));
            
            // Load Tools
            let tool_backend = tools_backend_sf.borrow();
            let tools = tool_backend.get_all_tools();
            let tool_names: Vec<slint::SharedString> = tools.iter()
                .map(|t| slint::SharedString::from(format!("{} - {}mm {}", t.name, t.diameter, t.tool_type)))
                .collect();
            window.set_sf_tools(slint::ModelRc::new(VecModel::from(tool_names)));
            
            // Load Devices
            let devices = device_manager_sf.get_all_profiles();
            let device_names: Vec<slint::SharedString> = devices.iter()
                .map(|d| slint::SharedString::from(d.name.clone()))
                .collect();
            window.set_sf_devices(slint::ModelRc::new(VecModel::from(device_names)));
            
            // Set defaults if empty
            if window.get_sf_selected_material_index() == -1 && !materials.is_empty() {
                window.set_sf_selected_material_index(0);
            }
            if window.get_sf_selected_tool_index() == -1 && !tools.is_empty() {
                window.set_sf_selected_tool_index(0);
            }
            if window.get_sf_selected_device_index() == -1 && !devices.is_empty() {
                // Try to find active device
                if let Some(active) = device_manager_sf.get_active_profile() {
                    if let Some(idx) = devices.iter().position(|d| d.id == active.id) {
                        window.set_sf_selected_device_index(idx as i32);
                    } else {
                        window.set_sf_selected_device_index(0);
                    }
                } else {
                    window.set_sf_selected_device_index(0);
                }
            }
        }
    });
    
    let window_weak = main_window.as_weak();
    let materials_backend_sf = materials_backend.clone();
    let tools_backend_sf = tools_backend.clone();
    let device_manager_sf = device_manager.clone();
    
    main_window.on_calculate_speeds_feeds(move || {
        if let Some(window) = window_weak.upgrade() {
            let mat_idx = window.get_sf_selected_material_index();
            let tool_idx = window.get_sf_selected_tool_index();
            let dev_idx = window.get_sf_selected_device_index();
            
            if mat_idx < 0 || tool_idx < 0 || dev_idx < 0 {
                return;
            }
            
            let mat_backend = materials_backend_sf.borrow();
            let materials = mat_backend.get_all_materials();
            let tool_backend = tools_backend_sf.borrow();
            let tools = tool_backend.get_all_tools();
            let devices = device_manager_sf.get_all_profiles();
            
            if let (Some(material), Some(tool), Some(device)) = (
                materials.get(mat_idx as usize),
                tools.get(tool_idx as usize),
                devices.get(dev_idx as usize)
            ) {
                let result = SpeedsFeedsCalculator::calculate(material, tool, device);
                
                window.set_sf_result_rpm(slint::SharedString::from(format!("{}", result.rpm)));
                if let Some(unclamped) = result.unclamped_rpm {
                    window.set_sf_result_unclamped_rpm(slint::SharedString::from(format!("{}", unclamped)));
                } else {
                    window.set_sf_result_unclamped_rpm(slint::SharedString::from(""));
                }
                
                window.set_sf_result_feed(slint::SharedString::from(format!("{:.0}", result.feed_rate)));
                if let Some(unclamped) = result.unclamped_feed_rate {
                    window.set_sf_result_unclamped_feed(slint::SharedString::from(format!("{:.0}", unclamped)));
                } else {
                    window.set_sf_result_unclamped_feed(slint::SharedString::from(""));
                }

                window.set_sf_result_surface_speed(slint::SharedString::from(format!("{:.1}", result.surface_speed)));
                window.set_sf_result_chip_load(slint::SharedString::from(format!("{:.4}", result.chip_load)));
                window.set_sf_result_source(slint::SharedString::from(result.source));
                
                if result.warnings.is_empty() {
                    window.set_sf_result_warnings(slint::SharedString::from(""));
                } else {
                    window.set_sf_result_warnings(slint::SharedString::from(result.warnings.join("\n")));
                }
            }
        }
    });

    // Spoilboard Surfacing Tool
    let window_weak = main_window.as_weak();
    let device_manager_ss = device_manager.clone();
    let editor_bridge_ss = editor_bridge.clone();
    let tools_backend_ss = tools_backend.clone();
    
    main_window.on_generate_spoilboard_surfacing(move || {
        if let Some(window) = window_weak.upgrade() {
            let dialog = SpoilboardSurfacingDialog::new().unwrap();
            
            // Populate Devices
            let mut profiles = device_manager_ss.get_all_profiles();
            profiles.sort_by(|a, b| a.name.cmp(&b.name));
            let device_names: Vec<slint::SharedString> = profiles.iter()
                .map(|p| slint::SharedString::from(p.name.clone()))
                .collect();
            dialog.set_devices(slint::ModelRc::new(VecModel::from(device_names)));
            
            // Set active device if available
            if let Some(active) = device_manager_ss.get_active_profile() {
                if let Some(idx) = profiles.iter().position(|p| p.id == active.id) {
                    dialog.set_selected_device_index(idx as i32);
                    let width = active.x_axis.max - active.x_axis.min;
                    let height = active.y_axis.max - active.y_axis.min;
                    dialog.set_width_mm(slint::SharedString::from(format!("{:.1}", width)));
                    dialog.set_height_mm(slint::SharedString::from(format!("{:.1}", height)));
                }
            }
            
            // Populate Tool Categories
            let categories: Vec<String> = gcodekit4_core::data::tools::ToolType::all()
                .iter()
                .map(|t| t.to_string())
                .collect();
            
            let category_names: Vec<slint::SharedString> = categories.iter()
                .map(|c| slint::SharedString::from(c.as_str()))
                .collect();
            dialog.set_tool_categories(slint::ModelRc::new(VecModel::from(category_names)));
            
            // Default to Specialty if available, else Flat End Mill
            let default_cat_idx = categories.iter().position(|c| c == "Specialty").unwrap_or(0);
            dialog.set_selected_category_index(default_cat_idx as i32);
            
            // Helper to populate tools based on category
            let categories_clone = categories.clone();
            let populate_tools = {
                let dialog_weak = dialog.as_weak();
                let tools_backend = tools_backend_ss.clone();
                move |category_idx: i32| {
                    if let Some(dlg) = dialog_weak.upgrade() {
                        if category_idx < 0 || category_idx >= categories_clone.len() as i32 {
                            dlg.set_tools(slint::ModelRc::new(VecModel::from(vec![])));
                            return;
                        }
                        
                        let category = &categories_clone[category_idx as usize];
                        let tool_type = gcodekit4::ui::tools_manager_backend::string_to_tool_type(category);
                        
                        if let Some(tt) = tool_type {
                            let backend = tools_backend.borrow();
                            let tools = backend.filter_by_type(tt);
                            let tool_names: Vec<slint::SharedString> = tools.iter()
                                .map(|t| slint::SharedString::from(format!("{} (D{:.1}mm)", t.name, t.diameter)))
                                .collect();
                            dlg.set_tools(slint::ModelRc::new(VecModel::from(tool_names)));
                            
                            // Select first tool if available
                            if !tools.is_empty() {
                                dlg.set_selected_tool_index(0);
                                // Trigger tool selection update manually since we can't call callback directly easily
                                // We'll just update the fields directly here
                                let t = tools[0];
                                dlg.set_tool_diameter(slint::SharedString::from(format!("{:.3}", t.diameter)));
                                dlg.set_feed_rate(slint::SharedString::from(format!("{:.0}", t.params.feed_rate)));
                                dlg.set_spindle_speed(slint::SharedString::from(format!("{}", t.params.rpm)));
                                dlg.set_stepover(slint::SharedString::from(format!("{:.1}", t.params.stepover_percent)));
                                dlg.set_cut_depth(slint::SharedString::from(format!("{:.3}", t.params.depth_per_pass)));
                            } else {
                                dlg.set_selected_tool_index(-1);
                            }
                        }
                    }
                }
            };
            
            // Initial population
            populate_tools(default_cat_idx as i32);
            
            // Callbacks
            let dialog_weak = dialog.as_weak();
            let device_manager_cb = device_manager_ss.clone();
            dialog.on_device_selected(move |idx| {
                if let Some(dlg) = dialog_weak.upgrade() {
                    let mut profiles = device_manager_cb.get_all_profiles();
                    profiles.sort_by(|a, b| a.name.cmp(&b.name));
                    if let Some(profile) = profiles.get(idx as usize) {
                        let width = profile.x_axis.max - profile.x_axis.min;
                        let height = profile.y_axis.max - profile.y_axis.min;
                        dlg.set_width_mm(slint::SharedString::from(format!("{:.1}", width)));
                        dlg.set_height_mm(slint::SharedString::from(format!("{:.1}", height)));
                    }
                }
            });
            
            let populate_tools_cb = populate_tools.clone();
            dialog.on_category_selected(move |idx| {
                populate_tools_cb(idx);
            });
            
            let dialog_weak = dialog.as_weak();
            let tools_backend_cb = tools_backend_ss.clone();
            let categories_clone_2 = categories.clone();
            dialog.on_tool_selected(move |idx| {
                if let Some(dlg) = dialog_weak.upgrade() {
                    let cat_idx = dlg.get_selected_category_index();
                    if cat_idx >= 0 && idx >= 0 {
                        let category = &categories_clone_2[cat_idx as usize];
                        if let Some(tt) = gcodekit4::ui::tools_manager_backend::string_to_tool_type(category) {
                            let backend = tools_backend_cb.borrow();
                            let tools = backend.filter_by_type(tt);
                            if let Some(t) = tools.get(idx as usize) {
                                dlg.set_tool_diameter(slint::SharedString::from(format!("{:.3}", t.diameter)));
                                dlg.set_feed_rate(slint::SharedString::from(format!("{:.0}", t.params.feed_rate)));
                                dlg.set_spindle_speed(slint::SharedString::from(format!("{}", t.params.rpm)));
                                dlg.set_stepover(slint::SharedString::from(format!("{:.1}", t.params.stepover_percent)));
                                dlg.set_cut_depth(slint::SharedString::from(format!("{:.3}", t.params.depth_per_pass)));
                            }
                        }
                    }
                }
            });
            
            let dialog_weak = dialog.as_weak();
            let window_weak_gen = window.as_weak();
            let _editor_bridge_gen = editor_bridge_ss.clone();
            
            dialog.on_generate_gcode(move || {
                if let Some(dlg) = dialog_weak.upgrade() {
                    // Parse parameters
                    let width = dlg.get_width_mm().parse::<f64>().unwrap_or(300.0);
                    let height = dlg.get_height_mm().parse::<f64>().unwrap_or(180.0);
                    let tool_diameter = dlg.get_tool_diameter().parse::<f64>().unwrap_or(25.4);
                    let feed_rate = dlg.get_feed_rate().parse::<f64>().unwrap_or(1000.0);
                    let spindle_speed = dlg.get_spindle_speed().parse::<f64>().unwrap_or(12000.0);
                    let cut_depth = dlg.get_cut_depth().parse::<f64>().unwrap_or(0.5);
                    let stepover_percent = dlg.get_stepover().parse::<f64>().unwrap_or(40.0);
                    let safe_z = dlg.get_safe_z().parse::<f64>().unwrap_or(5.0);
                    
                    let params = SpoilboardSurfacingParameters {
                        width,
                        height,
                        tool_diameter,
                        feed_rate,
                        spindle_speed,
                        cut_depth,
                        stepover_percent,
                        safe_z,
                    };
                    
                    // Close dialog
                    dlg.hide().unwrap();
                    
                    // Generate G-code
                    let generator = SpoilboardSurfacingGenerator::new(params);
                    match generator.generate() {
                        Ok(gcode) => {
                            if let Some(win) = window_weak_gen.upgrade() {
                                // Load into editor
                                win.invoke_load_editor_text(slint::SharedString::from(gcode));
                                win.set_current_view(slint::SharedString::from("gcode-editor"));
                                win.set_gcode_focus_trigger(win.get_gcode_focus_trigger() + 1);
                                
                                // Show success message
                                let success_dialog = ErrorDialog::new().unwrap();
                                success_dialog.set_error_message(
                                    slint::SharedString::from("Spoilboard surfacing G-code generated successfully."),
                                );
                                let success_weak = success_dialog.as_weak();
                                success_dialog.on_close_dialog(move || {
                                    if let Some(d) = success_weak.upgrade() {
                                        d.hide().unwrap();
                                    }
                                });
                                success_dialog.show().unwrap();
                            }
                        }
                        Err(e) => {
                            if let Some(_win) = window_weak_gen.upgrade() {
                                let error_dialog = ErrorDialog::new().unwrap();
                                error_dialog.set_error_message(
                                    slint::SharedString::from(format!("Failed to generate G-code: {}", e)),
                                );
                                let error_weak = error_dialog.as_weak();
                                error_dialog.on_close_dialog(move || {
                                    if let Some(d) = error_weak.upgrade() {
                                        d.hide().unwrap();
                                    }
                                });
                                error_dialog.show().unwrap();
                            }
                        }
                    }
                }
            });
            
            let dialog_weak_cancel = dialog.as_weak();
            dialog.on_cancel_dialog(move || {
                if let Some(dlg) = dialog_weak_cancel.upgrade() {
                    dlg.hide().unwrap();
                }
            });
            
            dialog.show().unwrap();
        }
    });
    use std::sync::{Arc, Mutex};
    let zoom_scale = Arc::new(Mutex::new(1.0f32));
    let pan_offset = Arc::new(Mutex::new((0.0f32, 0.0f32)));

    // Handle refresh visualization button
    let window_weak = main_window.as_weak();
    let zoom_for_refresh = zoom_scale.clone();
    let pan_for_refresh = pan_offset.clone();
    main_window.on_refresh_visualization(move |canvas_width, canvas_height| {
        // Skip if canvas dimensions are invalid (not yet laid out)
        if canvas_width < 100.0 || canvas_height < 100.0 {
            return;
        }

        // Get the current G-code content
        if let Some(window) = window_weak.upgrade() {
            let content = window.get_gcode_content();
            let _current_view = window.get_current_view();


            // Reset progress
            window.set_visualizer_progress(0.0);

            if content.is_empty() {
                tracing::warn!("Content is empty, clearing paths");
                // No G-code loaded, but still generate grid and origin
                window.set_visualizer_status(slint::SharedString::from("Ready"));
                window.set_visualization_path_data(slint::SharedString::from(""));
                window.set_visualization_rapid_moves_data(slint::SharedString::from(""));

                // Generate empty visualizer with just grid and origin
                use gcodekit4::visualizer::{
                    render_grid_to_path, render_origin_to_path, Visualizer2D,
                };
                let mut visualizer = Visualizer2D::new();
                visualizer.set_default_view(canvas_width, canvas_height);
                let show_grid = window.get_visualizer_show_grid();
                let (grid_data, grid_size) = if show_grid {
                    render_grid_to_path(&visualizer, canvas_width as u32, canvas_height as u32)
                } else {
                    (String::new(), 0.0)
                };
                let origin_data =
                    render_origin_to_path(&visualizer, canvas_width as u32, canvas_height as u32);
                window.set_visualization_grid_data(slint::SharedString::from(grid_data));
                window.set_visualizer_grid_size(slint::SharedString::from(format!("{}mm", grid_size)));
                window.set_visualizer_bounding_box_info(slint::SharedString::from(""));
                window.set_visualization_origin_data(slint::SharedString::from(origin_data));
                return;
            }

            window.set_visualizer_status(slint::SharedString::from("Refreshing..."));

            // Spawn rendering thread
            let content_owned = content.to_string();

            // Message format: (progress, status, path_data, rapid_moves_data, grid_data, origin_data, grid_size, bbox_info)
            let (tx, rx) = std::sync::mpsc::channel::<(
                f32,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<f32>,
                Option<String>,
            )>();
            let window_weak_render = window_weak.clone();
            let zoom_scale_render = zoom_for_refresh.clone();
            let pan_offset_render = pan_for_refresh.clone();
            let zoom_scale_for_msg = zoom_for_refresh.clone();
            let pan_offset_for_msg = pan_for_refresh.clone();

            // Get show_grid state before spawning thread
            let show_grid = window.get_visualizer_show_grid();

            std::thread::spawn(move || {
                if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                use gcodekit4::visualizer::{
                    render_grid_to_path, render_origin_to_path, render_rapid_moves_to_path,
                    render_toolpath_to_path, Visualizer2D,
                };

                let _ = tx.send((0.1, "Parsing G-code...".to_string(), None, None, None, None, None, None));

                let mut visualizer = Visualizer2D::new();
                visualizer.show_grid = show_grid;

                visualizer.parse_gcode(&content_owned);

                // Set default view to position origin at bottom-left
                visualizer.set_default_view(canvas_width, canvas_height);

                // Apply zoom scale
                if let Ok(scale) = zoom_scale_render.lock() {
                    visualizer.zoom_scale = *scale;
                }

                // Apply pan offsets
                if let Ok(offsets) = pan_offset_render.lock() {
                    visualizer.x_offset = offsets.0;
                    visualizer.y_offset = offsets.1;
                }

                let _ = tx.send((0.3, "Rendering...".to_string(), None, None, None, None, None, None));

                // Generate canvas path data
                let path_data =
                    render_toolpath_to_path(&visualizer, canvas_width as u32, canvas_height as u32);
                let rapid_moves_data = render_rapid_moves_to_path(
                    &visualizer,
                    canvas_width as u32,
                    canvas_height as u32,
                );
                let (grid_data, grid_size) = if show_grid {
                    render_grid_to_path(&visualizer, canvas_width as u32, canvas_height as u32)
                } else {
                    (String::new(), 0.0)
                };
                let origin_data =
                    render_origin_to_path(&visualizer, canvas_width as u32, canvas_height as u32);

                // Calculate bounding box info
                let bbox_info = if let Some((min_x, max_x, min_y, max_y)) = visualizer.get_cutting_bounds() {
                    let width = max_x - min_x;
                    let height = max_y - min_y;
                    Some(format!("W: {:.1}mm H: {:.1}mm at X:{:.1} Y:{:.1}", width, height, min_x, min_y))
                } else {
                    None
                };

                if !path_data.is_empty() || !rapid_moves_data.is_empty() || !grid_data.is_empty() {
                    let _ = tx.send((
                        1.0,
                        "Complete".to_string(),
                        Some(path_data),
                        Some(rapid_moves_data),
                        Some(grid_data),
                        Some(origin_data),
                        Some(grid_size),
                        bbox_info,
                    ));
                } else {
                    tracing::error!("ERROR: no render data generated!");
                    let _ = tx.send((1.0, "Error: no data".to_string(), None, None, None, None, None, None));
                }
                })) {
                    tracing::error!("Render thread panicked: {:?}", e);
                }
            });

            // Process messages from rendering thread
            std::thread::spawn(move || {
                while let Ok((
                    progress,
                    status,
                    path_data,
                    rapid_moves_data,
                    grid_data,
                    origin_data,
                    grid_size,
                    bbox_info,
                )) = rx.recv()
                {
                    let window_handle = window_weak_render.clone();
                    let status_clone = status.clone();
                    let path_clone = path_data.clone();
                    let rapid_moves_clone = rapid_moves_data.clone();
                    let grid_clone = grid_data.clone();
                    let origin_clone = origin_data.clone();
                    let bbox_info_clone = bbox_info.clone();
                    let zoom_for_closure = zoom_scale_for_msg.clone();
                    let pan_for_closure = pan_offset_for_msg.clone();

                    slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_handle.upgrade() {
                            window.set_visualizer_progress(progress);
                            window.set_visualizer_status(slint::SharedString::from(
                                status_clone.clone(),
                            ));

                            // Set canvas path data if available
                            if let Some(path) = path_clone {
                                window.set_visualization_path_data(slint::SharedString::from(path));
                            }
                            if let Some(rapid_moves) = rapid_moves_clone {
                                window.set_visualization_rapid_moves_data(
                                    slint::SharedString::from(rapid_moves),
                                );
                            }
                            if let Some(grid) = grid_clone {
                                window.set_visualization_grid_data(slint::SharedString::from(grid));
                            }
                            if let Some(origin) = origin_clone {
                                window.set_visualization_origin_data(slint::SharedString::from(
                                    origin,
                                ));
                            }
                            if let Some(size) = grid_size {
                                window.set_visualizer_grid_size(slint::SharedString::from(format!("{}mm", size)));
                            }
                            if let Some(info) = bbox_info_clone {
                                window.set_visualizer_bounding_box_info(slint::SharedString::from(info));
                            } else if progress == 1.0 {
                                // Clear if no info and finished
                                window.set_visualizer_bounding_box_info(slint::SharedString::from(""));
                            }

                            // Update indicator properties
                            if let Ok(scale) = zoom_for_closure.lock() {
                                window.set_visualizer_zoom_scale(*scale);
                            }
                            if let Ok(offsets) = pan_for_closure.lock() {
                                window.set_visualizer_x_offset(offsets.0);
                                window.set_visualizer_y_offset(offsets.1);
                            }
                        }
                    })
                    .ok();
                }
            });
        }
    });

    // Handle zoom in button
    let zoom_scale_in = zoom_scale.clone();
    let window_weak_zoom_in = main_window.as_weak();
    main_window.on_zoom_in(move |_canvas_width, _canvas_height| {
        if let Ok(mut scale) = zoom_scale_in.lock() {
            *scale *= 1.1;
            // Clamp to reasonable values (10% to 5000%)
            if *scale > 50.0 {
                *scale = 50.0;
            }

            // Update UI immediately
            if let Some(window) = window_weak_zoom_in.upgrade() {
                window.set_visualizer_zoom_scale(*scale);
                let canvas_width = window.get_visualizer_canvas_width();
                let canvas_height = window.get_visualizer_canvas_height();
                window.invoke_refresh_visualization(canvas_width, canvas_height);
            }
        }
    });

    // Handle zoom out button
    let zoom_scale_out = zoom_scale.clone();
    let window_weak_zoom_out = main_window.as_weak();
    main_window.on_zoom_out(move |_canvas_width, _canvas_height| {
        if let Ok(mut scale) = zoom_scale_out.lock() {
            *scale /= 1.1;
            // Clamp to reasonable values (10% to 5000%)
            if *scale < 0.1 {
                *scale = 0.1;
            }

            // Update UI immediately
            if let Some(window) = window_weak_zoom_out.upgrade() {
                window.set_visualizer_zoom_scale(*scale);
                let canvas_width = window.get_visualizer_canvas_width();
                let canvas_height = window.get_visualizer_canvas_height();
                window.invoke_refresh_visualization(canvas_width, canvas_height);
            }
        }
    });

    // Handle reset view button
    let zoom_scale_reset = zoom_scale.clone();
    let pan_offset_reset = pan_offset.clone();
    let window_weak_reset = main_window.as_weak();
    main_window.on_reset_view(move |_canvas_width, _canvas_height| {
        if let Ok(mut scale) = zoom_scale_reset.lock() {
            *scale = 1.0;
        }

        // Reset is handled by the visualizer's reset_pan() which will be called during refresh
        if let Ok(mut offsets) = pan_offset_reset.lock() {
            offsets.0 = 0.0;
            offsets.1 = 0.0;
        }

        // Update UI immediately
        if let Some(window) = window_weak_reset.upgrade() {
            window.set_visualizer_zoom_scale(1.0);
            window.set_visualizer_x_offset(0.0);
            window.set_visualizer_y_offset(0.0);
            let canvas_width = window.get_visualizer_canvas_width();
            let canvas_height = window.get_visualizer_canvas_height();
            window.invoke_refresh_visualization(canvas_width, canvas_height);
        }
    });

    // Handle fit to view button
    let window_weak_fit = main_window.as_weak();
    let zoom_for_fit = zoom_scale.clone();
    let pan_for_fit = pan_offset.clone();
    main_window.on_fit_to_view(move |canvas_width, canvas_height| {
        if let Some(window) = window_weak_fit.upgrade() {
            let content = window.get_gcode_content();

            if content.is_empty() {
                window.set_visualizer_status(slint::SharedString::from("No G-code loaded"));
                return;
            }

            // Reset progress
            window.set_visualizer_progress(0.0);
            window.set_visualizer_status(slint::SharedString::from("Fitting to view..."));

            // Spawn calculation thread
            let content_owned = content.to_string();
            let (tx, rx) = std::sync::mpsc::channel();
            let window_weak_render = window_weak_fit.clone();
            let zoom_fit_render = zoom_for_fit.clone();
            let pan_fit_render = pan_for_fit.clone();

            // Get show_grid state before spawning thread
            let show_grid = window.get_visualizer_show_grid();

            std::thread::spawn(move || {
                use gcodekit4::visualizer::Visualizer2D;

                let _ = tx.send((0.1, "Calculating fit...".to_string()));

                let mut visualizer = Visualizer2D::new();
                visualizer.show_grid = show_grid;
                visualizer.parse_gcode(&content_owned);

                // Calculate fit parameters using actual canvas dimensions
                visualizer.fit_to_view(canvas_width, canvas_height);

                // Apply fit parameters to shared state
                if let Ok(mut scale) = zoom_fit_render.lock() {
                    *scale = visualizer.zoom_scale;
                }

                if let Ok(mut offsets) = pan_fit_render.lock() {
                    offsets.0 = visualizer.x_offset;
                    offsets.1 = visualizer.y_offset;
                }

                let _ = tx.send((1.0, "Complete".to_string()));
            });

            // Process messages from rendering thread
            let zoom_for_closure_fit = zoom_for_fit.clone();
            let pan_for_closure_fit = pan_for_fit.clone();
            std::thread::spawn(move || {
                while let Ok((progress, status)) = rx.recv() {
                    let window_handle = window_weak_render.clone();
                    let status_clone = status.clone();
                    let zoom_for_closure = zoom_for_closure_fit.clone();
                    let pan_for_closure = pan_for_closure_fit.clone();

                    slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_handle.upgrade() {
                            window.set_visualizer_progress(progress);
                            window.set_visualizer_status(slint::SharedString::from(status_clone));

                            // Update indicator properties
                            if let Ok(scale) = zoom_for_closure.lock() {
                                window.set_visualizer_zoom_scale(*scale);
                            }
                            if let Ok(offsets) = pan_for_closure.lock() {
                                window.set_visualizer_x_offset(offsets.0);
                                window.set_visualizer_y_offset(offsets.1);
                            }

                            // Trigger refresh to re-render with new zoom/pan
                            if progress >= 1.0 {
                                let canvas_width = window.get_visualizer_canvas_width();
                                let canvas_height = window.get_visualizer_canvas_height();
                                window.invoke_refresh_visualization(canvas_width, canvas_height);
                            }
                        }
                    })
                    .ok();
                }
            });
        }
    });

    // Handle mouse pan (drag)
    let pan_mouse_clone = pan_offset.clone();
    let window_weak_pan_mouse = main_window.as_weak();
    main_window.on_pan_by_mouse(move |dx, dy| {
        if let Ok(mut offsets) = pan_mouse_clone.lock() {
            // Apply mouse delta directly to offsets
            offsets.0 += dx;
            offsets.1 += dy;

            if let Some(window) = window_weak_pan_mouse.upgrade() {
                window.set_visualizer_x_offset(offsets.0);
                window.set_visualizer_y_offset(offsets.1);
                let canvas_width = window.get_visualizer_canvas_width();
                let canvas_height = window.get_visualizer_canvas_height();
                window.invoke_refresh_visualization(canvas_width, canvas_height);
            }
        }
    });

    // Toggle grid callback
    let window_weak_grid = main_window.as_weak();
    main_window.on_toggle_grid(move || {
        if let Some(window) = window_weak_grid.upgrade() {
            let canvas_width = window.get_visualizer_canvas_width();
            let canvas_height = window.get_visualizer_canvas_height();
            window.invoke_refresh_visualization(canvas_width, canvas_height);
        }
    });

    // Cursor blink callback - updates the property
    let window_weak_callback = main_window.as_weak();
    main_window.on_set_cursor_blink_visible(move |visible| {
        if let Some(window) = window_weak_callback.upgrade() {
            window.set_cursor_blink_visible(visible);
        }
    });

    // Start cursor blink animation timer
    let window_weak_blink = main_window.as_weak();
    std::thread::spawn(move || {
        let mut visible = true;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(400));
            visible = !visible;
            let window_weak_invoke = window_weak_blink.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(window) = window_weak_invoke.upgrade() {
                    window.invoke_set_cursor_blink_visible(visible);
                }
            }).ok();
        }
    });

    main_window
        .show()
        .map_err(|e| anyhow::anyhow!("UI Show Error: {}", e))?;
    
    main_window
        .run()
        .map_err(|e| anyhow::anyhow!("UI Runtime Error: {}", e))?;

    Ok(())
}

/// Get list of available serial ports
fn get_available_ports() -> anyhow::Result<Vec<slint::SharedString>> {
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

/// Render G-code visualization in background thread using message passing
fn render_gcode_visualization_background_channel(
    gcode_content: String,
    width: u32,
    height: u32,
    tx: std::sync::mpsc::Sender<(
        f32,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<f32>,
    )>,
) {
    use gcodekit4::visualizer::{
        render_grid_to_path, render_origin_to_path, render_rapid_moves_to_path,
        render_toolpath_to_path, Visualizer2D,
    };

    let _ = tx.send((0.1, "Parsing G-code...".to_string(), None, None, None, None, None));

    // Parse G-code
    let mut visualizer = Visualizer2D::new();
    visualizer.parse_gcode(&gcode_content);

    // Set default view to position origin at bottom-left
    visualizer.set_default_view(width as f32, height as f32);

    let _ = tx.send((0.3, "Rendering...".to_string(), None, None, None, None, None));

    // Generate canvas path data
    let path_data = render_toolpath_to_path(&visualizer, width, height);
    let rapid_moves_data = render_rapid_moves_to_path(&visualizer, width, height);
    let (grid_data, grid_size) = render_grid_to_path(&visualizer, width, height);
    let origin_data = render_origin_to_path(&visualizer, width, height);


    if !path_data.is_empty() || !rapid_moves_data.is_empty() || !grid_data.is_empty() {
        let _ = tx.send((
            1.0,
            "Complete".to_string(),
            Some(path_data),
            Some(rapid_moves_data),
            Some(grid_data),
            Some(origin_data),
            Some(grid_size),
        ));
    } else {
        let _ = tx.send((1.0, "Error: no data".to_string(), None, None, None, None, None));
    }
}
