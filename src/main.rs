use gcodekit4::{
    init_logging, list_ports, CapabilityManager, Communicator, ConnectionDriver,
    ConnectionParams, ConsoleListener, DeviceConsoleManager, DeviceMessageType,
    FirmwareSettingsIntegration, GcodeEditor, SerialCommunicator, SerialParity, SettingValue,
    SettingsDialog, SettingsPersistence, BUILD_DATE, VERSION,
};
use slint::{Model, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, warn};

slint::include_modules!();

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
        0 => ("Step pulse time", "Step pulse duration in microseconds", "μs", "System"),
        1 => ("Step idle delay", "Step idle delay in milliseconds", "ms", "System"),
        2 => ("Step pulse invert", "Step pulse invert mask", "", "System"),
        3 => ("Step direction invert", "Step direction invert mask", "", "System"),
        4 => ("Invert step enable", "Invert step enable pin", "", "System"),
        5 => ("Invert limit pins", "Invert limit pins", "", "Limits"),
        6 => ("Invert probe pin", "Invert probe pin", "", "System"),
        10 => ("Status report", "Status report mask", "", "System"),
        11 => ("Junction deviation", "Junction deviation in mm", "mm", "System"),
        12 => ("Arc tolerance", "Arc tolerance in mm", "mm", "System"),
        13 => ("Report in inches", "Report in inches", "", "System"),
        20 => ("Soft limits", "Enable soft limits", "", "Limits"),
        21 => ("Hard limits", "Enable hard limits", "", "Limits"),
        22 => ("Homing cycle", "Enable homing cycle", "", "Homing"),
        23 => ("Homing direction", "Homing direction invert mask", "", "Homing"),
        24 => ("Homing locate feed", "Homing locate feed rate", "mm/min", "Homing"),
        25 => ("Homing search seek", "Homing search seek rate", "mm/min", "Homing"),
        26 => ("Homing debounce", "Homing switch debounce delay", "ms", "Homing"),
        27 => ("Homing pull-off", "Homing switch pull-off distance", "mm", "Homing"),
        30 => ("Max spindle speed", "Maximum spindle speed", "RPM", "Spindle"),
        31 => ("Min spindle speed", "Minimum spindle speed", "RPM", "Spindle"),
        32 => ("Laser mode", "Enable laser mode", "", "Spindle"),
        100 => ("X steps/mm", "X-axis steps per millimeter", "steps/mm", "Steps Per Unit"),
        101 => ("Y steps/mm", "Y-axis steps per millimeter", "steps/mm", "Steps Per Unit"),
        102 => ("Z steps/mm", "Z-axis steps per millimeter", "steps/mm", "Steps Per Unit"),
        110 => ("X max rate", "X-axis maximum rate", "mm/min", "Max Rate"),
        111 => ("Y max rate", "Y-axis maximum rate", "mm/min", "Max Rate"),
        112 => ("Z max rate", "Z-axis maximum rate", "mm/min", "Max Rate"),
        120 => ("X acceleration", "X-axis acceleration", "mm/sec²", "Acceleration"),
        121 => ("Y acceleration", "Y-axis acceleration", "mm/sec²", "Acceleration"),
        122 => ("Z acceleration", "Z-axis acceleration", "mm/sec²", "Acceleration"),
        130 => ("X max travel", "X-axis maximum travel", "mm", "Max Travel"),
        131 => ("Y max travel", "Y-axis maximum travel", "mm", "Max Travel"),
        132 => ("Z max travel", "Z-axis maximum travel", "mm", "Max Travel"),
        _ => (format!("${}", number).leak(), "Unknown setting", "", "Other"),
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

/// Update designer UI with current shapes from state
fn update_designer_ui(window: &MainWindow, state: &mut gcodekit4::DesignerState) {
    // Get canvas dimensions from window (or use defaults)
    let canvas_width = 800u32;  // Will be overridden by actual canvas size
    let canvas_height = 600u32;
    
    // Update viewport canvas size to match actual rendering size
    state.canvas.viewport_mut().set_canvas_size(canvas_width as f64, canvas_height as f64);
    
    // Render canvas using SVG paths
    let crosshair_data = gcodekit4::designer::svg_renderer::render_crosshair(
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
    window.set_designer_canvas_shapes_data(slint::SharedString::from(shapes_data));
    window.set_designer_canvas_selected_shapes_data(slint::SharedString::from(selected_shapes_data));
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
                gcodekit4::ShapeType::Polygon => 4,
                gcodekit4::ShapeType::RoundRectangle => 5,
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
    if let Some(selected_shape) = shapes.iter().find(|s| s.selected) {
        window.set_designer_selected_shape_x(selected_shape.x);
        window.set_designer_selected_shape_y(selected_shape.y);
        window.set_designer_selected_shape_w(selected_shape.width);
        window.set_designer_selected_shape_h(selected_shape.height);
        window.set_designer_selected_shape_type(selected_shape.shape_type);
        window.set_designer_selected_shape_radius(selected_shape.radius);
    } else {
        // No shape selected - clear indicators
        window.set_designer_selected_shape_x(0.0);
        window.set_designer_selected_shape_y(0.0);
        window.set_designer_selected_shape_w(0.0);
        window.set_designer_selected_shape_h(0.0);
        window.set_designer_selected_shape_type(0);
        window.set_designer_selected_shape_radius(5.0);
    }

    // Increment update counter to force UI re-render
    let mut ui_state = window.get_designer_state();
    let counter = ui_state.update_counter + 1;
    ui_state.update_counter = counter;
    window.set_designer_state(ui_state);
}

/// Parse GRBL status response and extract position
/// Format: <Idle|MPos:10.000,20.000,0.000|WPos:10.000,20.000,0.000|...>
fn parse_grbl_status(response: &str) -> Option<(f64, f64, f64)> {
    // Look for MPos (Machine Position)
    if let Some(mpos_start) = response.find("MPos:") {
        let mpos_data = &response[mpos_start + 5..];
        if let Some(mpos_end) = mpos_data.find('|') {
            let coords = &mpos_data[..mpos_end];
            let parts: Vec<&str> = coords.split(',').collect();
            if parts.len() >= 3 {
                if let (Ok(x), Ok(y), Ok(z)) = (
                    parts[0].trim().parse::<f64>(),
                    parts[1].trim().parse::<f64>(),
                    parts[2].trim().parse::<f64>(),
                ) {
                    return Some((x, y, z));
                }
            }
        }
    }
    None
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

    // Initialize device console manager early to register listeners
    // Use Arc since communicator listeners need Arc for thread-safe sharing
    let console_manager = std::sync::Arc::new(DeviceConsoleManager::new());

    // Create and register console listener with communicator
    let console_listener = ConsoleListener::new(console_manager.clone());
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
        if let Err(_) = fw_integration.load_grbl_defaults() {
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
        let config_path = match gcodekit4::config::SettingsManager::config_file_path() {
            Ok(path) => path,
            Err(_) => std::path::PathBuf::new(),
        };

        if config_path.exists() {
            match SettingsPersistence::load_from_file(&config_path) {
                Ok(loaded_persistence) => {
                    *persistence = loaded_persistence;
                }
                Err(_) => {}
            }
        } 

        // Populate dialog with settings
        let mut dialog = settings_dialog.borrow_mut();
        persistence.populate_dialog(&mut dialog);
        drop(dialog);
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
                    window.set_device_version(slint::SharedString::from("GRBL 1.1+"));
                    window.set_machine_state(slint::SharedString::from("IDLE"));
                    let console_output = console_manager_clone.get_output();
                    window.set_console_output(slint::SharedString::from(console_output));
                    
                    // Detect firmware and update capabilities
                    // TODO: Replace with actual firmware detection from response
                    // For now, assume GRBL 1.1 as default
                    use gcodekit4::firmware::firmware_version::{FirmwareType, SemanticVersion};
                    let firmware_type = FirmwareType::Grbl;
                    let version = SemanticVersion::new(1, 1, 0);
                    capability_manager_clone.update_firmware(firmware_type, version);
                    sync_capabilities_to_ui(&window, &capability_manager_clone);
                }

                // Start status polling thread
                polling_stop_connect.store(false, std::sync::atomic::Ordering::Relaxed);
                let port_clone = port_str.clone();
                let baud_clone = baud;
                let window_weak_poll = window_weak.clone();
                let polling_active = status_polling_active.clone();
                let polling_stop = polling_stop_connect.clone();

                std::thread::spawn(move || {
                    polling_active.store(true, std::sync::atomic::Ordering::Relaxed);
                    use gcodekit4::SerialCommunicator;
                    let mut poll_comm = SerialCommunicator::new();

                    // Connect the polling communicator
                    let params = gcodekit4::ConnectionParams {
                        driver: gcodekit4::ConnectionDriver::Serial,
                        port: port_clone.clone(),
                        network_port: 8888,
                        baud_rate: baud_clone as u32,
                        timeout_ms: 5000,
                        flow_control: false,
                        data_bits: 8,
                        stop_bits: 1,
                        parity: gcodekit4::SerialParity::None,
                        auto_reconnect: true,
                        max_retries: 3,
                    };

                    if let Ok(()) = poll_comm.connect(&params) {
                        while !polling_stop.load(std::sync::atomic::Ordering::Relaxed) {
                            std::thread::sleep(std::time::Duration::from_millis(200));

                            if poll_comm.is_connected() {
                                // Send status query '?'
                                if poll_comm.send(b"?").is_ok() {
                                    std::thread::sleep(std::time::Duration::from_millis(10));
                                    if let Ok(response) = poll_comm.receive() {
                                        if !response.is_empty() {
                                            let response_str = String::from_utf8_lossy(&response);

                                            // Parse position from response
                                            if let Some((x, y, z)) =
                                                parse_grbl_status(&response_str)
                                            {
                                                let window_handle = window_weak_poll.clone();
                                                slint::invoke_from_event_loop(move || {
                                                    if let Some(window) = window_handle.upgrade() {
                                                        window.set_position_x(x as f32);
                                                        window.set_position_y(y as f32);
                                                        window.set_position_z(z as f32);
                                                    }
                                                })
                                                .ok();
                                            }
                                        }
                                    }
                                }
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
                }
            }
        }
    });

    // Set up menu-file-exit callback
    let communicator_clone = communicator.clone();
    main_window.on_menu_file_exit(move || {
        // Disconnect if connected before exiting
        let mut comm = communicator_clone.lock().unwrap();
        if let Err(_) = comm.disconnect() {}
        std::process::exit(0);
    });

    // Set up menu-file-open callback
    let window_weak = main_window.as_weak();
    let gcode_editor_clone = gcode_editor.clone();
    let console_manager_clone = console_manager.clone();
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

                    console_manager_clone.add_message(
                        DeviceMessageType::Output,
                        format!("DEBUG: Setting TextEdit content ({} chars)", content.len()),
                    );

                    window.set_gcode_content(slint::SharedString::from(content.clone()));

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

                    // Use a channel to communicate progress from background thread
                    let (tx, rx) = std::sync::mpsc::channel();

                    std::thread::spawn(move || {
                        render_gcode_visualization_background_channel(content_clone, tx);
                    });

                    // Use Slint's invoke_from_event_loop to safely update UI from background thread
                    std::thread::spawn(move || {
                        while let Ok((progress, status, path_data, grid_data, origin_data)) = rx.recv() {
                            let window_handle = window_weak.clone();
                            let status_clone = status.clone();
                            let path_clone = path_data.clone();
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
                                        window.set_visualization_path_data(slint::SharedString::from(path));
                                    }
                                    if let Some(grid) = grid_clone {
                                        window.set_visualization_grid_data(slint::SharedString::from(grid));
                                    }
                                    if let Some(origin) = origin_clone {
                                        window.set_visualization_origin_data(slint::SharedString::from(origin));
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
                return;
            }
            drop(comm);

            if current_content.is_empty() {
                warn!("Send failed: No G-Code to send");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ No G-Code content to send.");
                window.set_connection_status(slint::SharedString::from("Error: No G-Code content"));
                return;
            }

            // Send G-Code content to device using GRBL Character-Counting Protocol
            // This protocol ensures the 127-character serial RX buffer is used efficiently
            console_manager_clone.add_message(
                DeviceMessageType::Output,
                format!(
                    "Sending G-Code to device ({} bytes) using GRBL protocol...",
                    current_content.len()
                ),
            );

            // Update UI to show sending is in progress
            window.set_connection_status(slint::SharedString::from("Sending G-Code..."));
            let console_output = console_manager_clone.get_output();
            window.set_console_output(slint::SharedString::from(console_output));

            // Use slint timer to run sending in small chunks to avoid blocking UI
            // Store state in shared Rc<RefCell<>> for the timer callback
            use std::cell::RefCell;
            use std::rc::Rc;

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

            let lines: Vec<String> = current_content.lines().map(|s| s.to_string()).collect();
            let send_state = Rc::new(RefCell::new(SendState {
                lines,
                line_index: 0,
                send_count: 0,
                pending_bytes: 0,
                line_lengths: Vec::new(),
                error_occurred: false,
                error_msg: String::new(),
                waiting_for_acks: false,
                timeout_count: 0,
            }));

            let window_weak_timer = window_weak.clone();
            let communicator_timer = communicator_clone.clone();
            let console_manager_timer = console_manager_clone.clone();
            let send_state_timer = send_state.clone();

            const GRBL_RX_BUFFER_SIZE: usize = 127;
            const MAX_TIMEOUT_ITERATIONS: u32 = 300000;

            // Use a timer that fires every 50ms to process sending without blocking UI
            // This reduces contention on the communicator lock
            let timer = Rc::new(slint::Timer::default());
            let timer_clone = timer.clone();

            timer.start(
                slint::TimerMode::Repeated,
                std::time::Duration::from_millis(50),
                move || {
                    // Step 1: Handle incoming responses (minimize lock duration)
                    {
                        let mut comm = communicator_timer.lock().unwrap();
                        match comm.receive() {
                            Ok(response) => {
                                if !response.is_empty() {
                                    let resp_str = String::from_utf8_lossy(&response);
                                    let ok_count = resp_str.matches("ok").count()
                                        + resp_str.matches("OK").count();

                                    let mut state = send_state_timer.borrow_mut();
                                    for _ in 0..ok_count {
                                        if !state.line_lengths.is_empty() {
                                            let processed_length = state.line_lengths.remove(0);
                                            state.pending_bytes = state
                                                .pending_bytes
                                                .saturating_sub(processed_length);
                                        }
                                    }
                                    state.timeout_count = 0;
                                }
                            }
                            Err(_) => {
                                let mut state = send_state_timer.borrow_mut();
                                if state.waiting_for_acks {
                                    state.timeout_count += 1;
                                }
                            }
                        }
                    } // Release communicator lock here

                    let mut state = send_state_timer.borrow_mut();

                    // Check if we're waiting for final acknowledgments
                    if state.waiting_for_acks {
                        if state.line_lengths.is_empty()
                            || state.timeout_count >= MAX_TIMEOUT_ITERATIONS
                        {
                            // Done - update UI
                            drop(state);

                            let final_state = send_state_timer.borrow();
                            if let Some(window) = window_weak_timer.upgrade() {
                                if final_state.timeout_count >= MAX_TIMEOUT_ITERATIONS
                                    && !final_state.line_lengths.is_empty()
                                {
                                    warn!(
                                        "Timeout waiting for {} line acknowledgments",
                                        final_state.line_lengths.len()
                                    );
                                    console_manager_timer.add_message(
                                        DeviceMessageType::Error,
                                        format!(
                                            "⚠ Timeout: {} lines may not have been received",
                                            final_state.line_lengths.len()
                                        ),
                                    );
                                }

                                if !final_state.error_occurred {
                                    console_manager_timer.add_message(
                                        DeviceMessageType::Success,
                                        format!(
                                            "✓ Successfully sent {} lines to device",
                                            final_state.send_count
                                        ),
                                    );
                                    window.set_connection_status(slint::SharedString::from(
                                        format!("Sent: {} lines", final_state.send_count),
                                    ));
                                } else {
                                    window.set_connection_status(slint::SharedString::from(
                                        format!(
                                            "Send stopped at line {} ({})",
                                            final_state.send_count + 1,
                                            final_state.error_msg
                                        ),
                                    ));
                                }

                                let console_output = console_manager_timer.get_output();
                                window
                                    .set_console_output(slint::SharedString::from(console_output));
                            }

                            timer_clone.stop();
                        }
                        return;
                    }

                    // Step 2: Send multiple lines in batches (up to 5 lines per timer tick)
                    for _ in 0..5 {
                        if state.line_index >= state.lines.len() {
                            state.waiting_for_acks = true;
                            break;
                        }

                        let trimmed = state.lines[state.line_index].trim().to_string();

                        if trimmed.is_empty() {
                            state.line_index += 1;
                            continue;
                        }

                        let line_length = trimmed.len() + 1;

                        // Check if there's room in the buffer
                        if state.pending_bytes + line_length <= GRBL_RX_BUFFER_SIZE {
                            // Scope the communicator borrow to minimal duration
                            let send_result = {
                                let mut comm = communicator_timer.lock().unwrap();
                                let command_bytes = format!("{}\n", trimmed);
                                comm.send(command_bytes.as_bytes())
                            };

                            match send_result {
                                Ok(_bytes_sent) => {
                                    state.send_count += 1;
                                    state.pending_bytes += line_length;
                                    state.line_lengths.push(line_length);
                                    state.line_index += 1;

                                    debug!(
                                        "Sent line {} ({} bytes): {} [pending: {}/{}]",
                                        state.send_count,
                                        line_length,
                                        trimmed,
                                        state.pending_bytes,
                                        GRBL_RX_BUFFER_SIZE
                                    );
                                }
                                Err(e) => {
                                    let error_msg = format!("{}", e);
                                    let line_count = state.send_count + 1;
                                    warn!("Failed to send line {}: {}", line_count, error_msg);

                                    state.error_msg = error_msg.clone();
                                    state.error_occurred = true;

                                    drop(state);

                                    console_manager_timer.add_message(
                                        DeviceMessageType::Error,
                                        format!(
                                            "✗ Failed to send line {}: {}",
                                            line_count, error_msg
                                        ),
                                    );

                                    if let Some(window) = window_weak_timer.upgrade() {
                                        window.set_connection_status(slint::SharedString::from(
                                            format!(
                                                "Send stopped at line {} ({})",
                                                line_count, error_msg
                                            ),
                                        ));
                                        let console_output = console_manager_timer.get_output();
                                        window.set_console_output(slint::SharedString::from(
                                            console_output,
                                        ));
                                    }

                                    timer_clone.stop();
                                    return;
                                }
                            }
                        } else {
                            // Buffer full, wait for acknowledgments
                            state.waiting_for_acks = true;
                            break;
                        }
                    }
                },
            );
        }
    });

    // Set up menu-edit-preferences callback
    let window_weak = main_window.as_weak();
    let settings_dialog_clone = settings_dialog.clone();
    main_window.on_menu_edit_preferences(move || {
        // Get reference to settings dialog
        let dialog = settings_dialog_clone.borrow();

        // Build settings array for UI display - using generated SettingItem type
        let mut settings_items = Vec::new();
        for setting in dialog.settings.values() {
            let value_type = match &setting.value {
                SettingValue::Boolean(_) => "Boolean",
                SettingValue::Integer(_) => "Integer",
                SettingValue::Float(_) => "Float",
                _ => "String",
            };

            let category = format!("{}", setting.category);

            // Build options list for enum-type settings
            let options: Vec<slint::SharedString> = if value_type == "String" {
                // For now, no options for string settings
                Vec::new()
            } else {
                Vec::new()
            };

            // Find the current index in the options list
            let current_index = options
                .iter()
                .position(|o| *o == setting.value.as_str())
                .unwrap_or(0) as i32;

            settings_items.push(slint_generatedMainWindow::SettingItem {
                id: setting.id.clone().into(),
                name: setting.name.clone().into(),
                value: setting.value.as_str().into(),
                value_type: value_type.into(),
                category: category.into(),
                description: setting
                    .description
                    .clone()
                    .map(|s| s.into())
                    .unwrap_or_default(),
                options: slint::ModelRc::from(Rc::new(slint::VecModel::from(options))),
                current_index,
            });
        }

        if let Some(window) = window_weak.upgrade() {
            let model = std::rc::Rc::new(slint::VecModel::from(settings_items));
            window.set_all_settings(slint::ModelRc::new(model));
            window.set_connection_status(slint::SharedString::from("Preferences dialog opened"));
        }
    });

    // Set up menu-settings-save callback
    let window_weak = main_window.as_weak();
    let settings_dialog_clone = settings_dialog.clone();
    let settings_persistence_clone = settings_persistence.clone();
    main_window.on_menu_settings_save(move || {
        let dialog = settings_dialog_clone.borrow();

        // Check for unsaved changes
        if dialog.has_changes() {
            // Log all changed settings
            for setting in dialog.settings.values() {
                if setting.is_changed() {
                    // Setting changed, will be saved
                }
            }

            // Save to disk
            {
                let mut persistence = settings_persistence_clone.borrow_mut();

                // Load settings from dialog into config
                if let Err(e) = persistence.load_from_dialog(&dialog) {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Error saving settings: {}",
                            e
                        )));
                    }
                    return;
                }

                // Save to file
                let config_path = match gcodekit4::config::SettingsManager::config_file_path() {
                    Ok(path) => path,
                    Err(e) => {
                        if let Some(window) = window_weak.upgrade() {
                            window.set_connection_status(slint::SharedString::from(format!(
                                "Error: Could not determine config path: {}",
                                e
                            )));
                        }
                        return;
                    }
                };

                // Ensure config directory exists
                if let Err(e) = gcodekit4::config::SettingsManager::ensure_config_dir() {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Error: Failed to create config directory: {}",
                            e
                        )));
                    }
                    return;
                }

                if let Err(e) = persistence.save_to_file(&config_path) {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Error saving settings: {}",
                            e
                        )));
                    }
                } else {
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from(
                            "Settings saved successfully",
                        ));
                    }

                    // Reset the unsaved changes flag and sync defaults
                    drop(dialog); // Release the borrow
                    let mut dialog_mut = settings_dialog_clone.borrow_mut();
                    for setting in dialog_mut.settings.values_mut() {
                        setting.default = setting.value.clone();
                    }
                    dialog_mut.has_unsaved_changes = false;
                }
            }
        } else if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("No changes to save"));
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
    let settings_dialog_clone = settings_dialog.clone();
    main_window.on_menu_settings_restore_defaults(move || {
        let mut dialog = settings_dialog_clone.borrow_mut();
        dialog.reset_all_to_defaults();

        if let Some(window) = window_weak.upgrade() {
            window
                .set_connection_status(slint::SharedString::from("Settings restored to defaults"));
        }
    });

    // Set up update-setting callback
    let settings_dialog_clone = settings_dialog.clone();
    main_window.on_update_setting(
        move |setting_id: slint::SharedString, value: slint::SharedString| {
            let mut dialog = settings_dialog_clone.borrow_mut();
            let setting_id_str = setting_id.to_string();
            let value_str = value.to_string();

            if let Some(setting) = dialog.get_setting_mut(&setting_id_str) {
                let new_value = match &setting.value {
                    gcodekit4::ui::settings_dialog::SettingValue::String(_) => {
                        gcodekit4::ui::settings_dialog::SettingValue::String(value_str)
                    }
                    gcodekit4::ui::settings_dialog::SettingValue::Integer(_) => {
                        if let Ok(i) = value_str.parse::<i32>() {
                            gcodekit4::ui::settings_dialog::SettingValue::Integer(i)
                        } else {
                            return;
                        }
                    }
                    gcodekit4::ui::settings_dialog::SettingValue::Float(_) => {
                        if let Ok(f) = value_str.parse::<f64>() {
                            gcodekit4::ui::settings_dialog::SettingValue::Float(f)
                        } else {
                            return;
                        }
                    }
                    gcodekit4::ui::settings_dialog::SettingValue::Boolean(_) => {
                        let b = matches!(value_str.to_lowercase().as_str(), "true" | "1" | "yes");
                        gcodekit4::ui::settings_dialog::SettingValue::Boolean(b)
                    }
                    gcodekit4::ui::settings_dialog::SettingValue::Enum(_, ref options) => {
                        if options.contains(&value_str) {
                            gcodekit4::ui::settings_dialog::SettingValue::Enum(
                                value_str,
                                options.clone(),
                            )
                        } else {
                            return;
                        }
                    }
                };

                setting.value = new_value;
                dialog.has_unsaved_changes = true;
            } 
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
        }
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
                // Send jog command in positive X direction using step size and 2000mm/min feedrate
                let jog_cmd = format!("$J=X{} F2000", step_size);
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
                // Send jog command in negative X direction using step size and 2000mm/min feedrate
                let jog_cmd = format!("$J=X-{} F2000", step_size);
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
            window.set_config_status_message(slint::SharedString::from("Retrieving settings from controller..."));
            
            // Send $$ command to query settings
            let mut comm = communicator_clone.lock().unwrap();
            if let Err(e) = comm.send(b"$$\n") {
                window.set_config_status_message(slint::SharedString::from(format!("Error: {}", e)));
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
                    window.set_config_status_message(slint::SharedString::from(
                        format!("Retrieved {} settings from controller", window.get_config_settings().row_count())
                    ));
                }
                Err(e) => {
                    window.set_config_status_message(slint::SharedString::from(format!("Error reading response: {}", e)));
                }
            }
        }
    });

    let window_weak = main_window.as_weak();
    main_window.on_config_save_to_file(move || {
        if let Some(window) = window_weak.upgrade() {
            // Check if we have settings to save
            if window.get_config_settings().row_count() == 0 {
                window.set_config_status_message(slint::SharedString::from("No settings to save. Retrieve settings first."));
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
                                                    setting_json["unit"].as_str().unwrap_or("")
                                                ),
                                                description: slint::SharedString::from(
                                                    setting_json["description"].as_str().unwrap_or("")
                                                ),
                                                category: slint::SharedString::from(
                                                    setting_json["category"].as_str().unwrap_or("Other")
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
                                window.set_config_filtered_settings(slint::ModelRc::from(filtered_model));
                                
                                window.set_config_has_loaded_settings(true);
                                window.set_config_status_message(slint::SharedString::from(format!(
                                    "Loaded {} settings from {}",
                                    window.get_config_settings().row_count(),
                                    path.display()
                                )));
                            }
                            Err(e) => {
                                window.set_config_status_message(slint::SharedString::from(format!(
                                    "Error parsing JSON: {}",
                                    e
                                )));
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
                window.set_config_status_message(slint::SharedString::from("No settings to restore. Load settings first."));
                return;
            }
            
            window.set_config_status_message(slint::SharedString::from("Restoring settings to controller..."));
            
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
                    success_count,
                    error_count
                )));
            }
        }
    });

    let window_weak = main_window.as_weak();
    main_window.on_config_edit_setting(move |_number: i32| {
        if let Some(window) = window_weak.upgrade() {
            window.set_config_status_message(slint::SharedString::from("Edit setting dialog not yet implemented"));
        }
    });

    let window_weak = main_window.as_weak();
    main_window.on_config_filter_changed(move || {
        if let Some(window) = window_weak.upgrade() {
            // Get filter text and category
            let filter_text = window.get_config_filter_text();
            let category = window.get_config_selected_category();
            
            debug!("Filter changed - text: '{}', category: '{}'", filter_text, category);
            
            let settings_model = window.get_config_settings();
            let mut filtered = Vec::new();
            
            let filter_lower = filter_text.to_lowercase();
            
            debug!("Total settings: {}", settings_model.row_count());
            
            for i in 0..settings_model.row_count() {
                if let Some(setting) = settings_model.row_data(i) {
                    // Check category filter
                    let category_match = category == "All" || setting.category.as_str() == category.as_str();
                    
                    // Check text filter (matches ID, name, value, or description)
                    let text_match = filter_lower.is_empty() || 
                        format!("${}", setting.number).to_lowercase().contains(&filter_lower) ||
                        setting.name.to_lowercase().contains(&filter_lower) ||
                        setting.value.to_lowercase().contains(&filter_lower) ||
                        setting.description.to_lowercase().contains(&filter_lower);
                    
                    if category_match && text_match {
                        filtered.push(setting);
                    }
                }
            }
            
            // Update filtered settings
            debug!("Filtered settings count: {}", filtered.len());
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

    // Set up menu-view-gtools callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_gtools(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_current_view(slint::SharedString::from("gtools"));
            window.set_connection_status(slint::SharedString::from("GTools panel activated"));
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
            slint::Timer::single_shot(std::time::Duration::from_millis(50), move || {
                if let Some(window) = window_weak_timer.upgrade() {
                    let canvas_width = window.get_visualizer_canvas_width();
                    let canvas_height = window.get_visualizer_canvas_height();
                    tracing::debug!(
                        "Deferred visualizer refresh with canvas {}x{}",
                        canvas_width,
                        canvas_height
                    );
                    window.invoke_refresh_visualization(canvas_width, canvas_height);
                }
            });
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
        // Reset zoom to 1.0 (100%)
        state.canvas.set_zoom(1.0);
        // Reset pan to 0,0
        let (pan_x, pan_y) = state.canvas.pan_offset();
        state.canvas.pan(-pan_x, -pan_y);
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

    // Designer: Generate Toolpath callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_generate_toolpath(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_is_busy(true);
        }
        
        let window_weak_inner = window_weak.clone();
        let designer_mgr_inner = designer_mgr_clone.clone();
        
        // Use timer to allow UI to update cursor before blocking operation
        slint::Timer::single_shot(std::time::Duration::from_millis(50), move || {
            let gcode = {
                let mut state = designer_mgr_inner.borrow_mut();
                state.generate_gcode()
            };
            
            if let Some(window) = window_weak_inner.upgrade() {
                window.set_designer_generated_gcode(slint::SharedString::from(gcode));
                window.set_designer_gcode_generated(true);
                window.set_connection_status(slint::SharedString::from("G-code generated successfully"));
                window.set_is_busy(false);
            }
        });
    });

    // Designer: Export G-code callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_export_gcode(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_is_busy(true);
        }
        
        let window_weak_inner = window_weak.clone();
        let designer_mgr_inner = designer_mgr_clone.clone();
        
        // Use timer to allow UI to update cursor before blocking operation
        slint::Timer::single_shot(std::time::Duration::from_millis(50), move || {
            let gcode = {
                let mut state = designer_mgr_inner.borrow_mut();
                state.generate_gcode()
            };
            
            if let Some(window) = window_weak_inner.upgrade() {
                window.set_gcode_content(slint::SharedString::from(gcode));
                window.set_current_view(slint::SharedString::from("gcode-editor"));
                window.set_connection_status(slint::SharedString::from("G-code exported to editor"));
                window.set_is_busy(false);
            }
        });
    });

    // Designer: Import DXF callback
    let window_weak = main_window.as_weak();
    let designer_mgr_clone = designer_mgr.clone();
    main_window.on_designer_import_dxf(move || {
        use rfd::FileDialog;
        use gcodekit4::designer::{DxfImporter, DxfParser};
        
        if let Some(path) = FileDialog::new()
            .add_filter("DXF Files", &["dxf"])
            .set_title("Import DXF File")
            .pick_file()
        {
            if let Some(window) = window_weak.upgrade() {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        match DxfParser::parse(&content) {
                            Ok(dxf_file) => {
                                let importer = DxfImporter::new(1.0, 0.0, 0.0);
                                match importer.import_string(&content) {
                                    Ok(design) => {
                                        let mut state = designer_mgr_clone.borrow_mut();
                                        for shape in design.shapes {
                                            state.canvas.add_shape(shape);
                                        }
                                        window.set_connection_status(slint::SharedString::from(
                                            format!("DXF imported: {} entities from {} layers", 
                                                   dxf_file.entity_count(), 
                                                   dxf_file.layer_names().len())
                                        ));
                                        update_designer_ui(&window, &mut state);
                                    }
                                    Err(e) => {
                                        window.set_connection_status(slint::SharedString::from(
                                            format!("DXF import failed: {}", e)
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                window.set_connection_status(slint::SharedString::from(
                                    format!("DXF parse error: {}", e)
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        window.set_connection_status(slint::SharedString::from(
                            format!("Failed to read file: {}", e)
                        ));
                    }
                }
            }
        }
    });

    // Designer: Import SVG callback
    let window_weak = main_window.as_weak();
    let designer_mgr_clone = designer_mgr.clone();
    main_window.on_designer_import_svg(move || {
        use rfd::FileDialog;
        use gcodekit4::designer::SvgImporter;
        
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
                                window.set_connection_status(slint::SharedString::from(
                                    format!("SVG imported: {} shapes from {} layers", 
                                           shape_count, 
                                           layer_count)
                                ));
                                update_designer_ui(&window, &mut state);
                            }
                            Err(e) => {
                                window.set_connection_status(slint::SharedString::from(
                                    format!("SVG import failed: {}", e)
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        window.set_connection_status(slint::SharedString::from(
                            format!("Failed to read file: {}", e)
                        ));
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
        } else {
            if let Some(new_path) = rfd::FileDialog::new()
                .add_filter("GCodeKit4 Design", &["gck4"])
                .set_file_name("design.gck4")
                .save_file()
            {
                new_path
            } else {
                return; // User cancelled
            }
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
    
    // Designer: Save shape properties (corner radius)
    let pending_properties = Rc::new(RefCell::new((0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64)));
    
    let pending_clone = pending_properties.clone();
    main_window.on_designer_update_shape_property(move |prop_id: i32, value: f32| {
        let mut props = pending_clone.borrow_mut();
        match prop_id {
            0 => props.0 = value as f64, // x
            1 => props.1 = value as f64, // y
            2 => props.2 = value as f64, // w
            3 => props.3 = value as f64, // h
            4 => props.4 = value as f64, // radius
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
        state.set_selected_corner_radius(props.4);
        
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

    // Zoom state tracking (Arc for shared state across closures)
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
            tracing::debug!(
                "Skipping visualization refresh: canvas too small ({}x{})",
                canvas_width,
                canvas_height
            );
            return;
        }
        
        // Get the current G-code content
        if let Some(window) = window_weak.upgrade() {
            let content = window.get_gcode_content();

            // Reset progress
            window.set_visualizer_progress(0.0);
            
            if content.is_empty() {
                // No G-code loaded, but still generate grid and origin
                window.set_visualizer_status(slint::SharedString::from("Ready"));
                window.set_visualization_path_data(slint::SharedString::from(""));
                window.set_visualization_rapid_moves_data(slint::SharedString::from(""));
                
                // Generate empty visualizer with just grid and origin
                use gcodekit4::visualizer::{Visualizer2D, render_grid_to_path, render_origin_to_path};
                let visualizer = Visualizer2D::new();
                let show_grid = window.get_visualizer_show_grid();
                let grid_data = if show_grid {
                    render_grid_to_path(&visualizer, canvas_width as u32, canvas_height as u32)
                } else {
                    String::new()
                };
                let origin_data = render_origin_to_path(&visualizer, canvas_width as u32, canvas_height as u32);
                window.set_visualization_grid_data(slint::SharedString::from(grid_data));
                window.set_visualization_origin_data(slint::SharedString::from(origin_data));
                return;
            }

            window.set_visualizer_status(slint::SharedString::from("Refreshing..."));

            // Spawn rendering thread
            let content_owned = content.to_string();
            
            // Message format: (progress, status, path_data, rapid_moves_data, grid_data, origin_data)
            let (tx, rx) = std::sync::mpsc::channel::<(f32, String, Option<String>, Option<String>, Option<String>, Option<String>)>();
            let window_weak_render = window_weak.clone();
            let zoom_scale_render = zoom_for_refresh.clone();
            let pan_offset_render = pan_for_refresh.clone();
            let zoom_scale_for_msg = zoom_for_refresh.clone();
            let pan_offset_for_msg = pan_for_refresh.clone();

            // Get show_grid state before spawning thread
            let show_grid = window.get_visualizer_show_grid();

            std::thread::spawn(move || {
                use gcodekit4::visualizer::{Visualizer2D, render_toolpath_to_path, render_rapid_moves_to_path, render_grid_to_path, render_origin_to_path};

                let _ = tx.send((0.1, "Parsing G-code...".to_string(), None, None, None, None));

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

                let _ = tx.send((0.3, "Rendering...".to_string(), None, None, None, None));

                // Generate canvas path data
                let path_data = render_toolpath_to_path(&visualizer, canvas_width as u32, canvas_height as u32);
                let rapid_moves_data = render_rapid_moves_to_path(&visualizer, canvas_width as u32, canvas_height as u32);
                let grid_data = if show_grid {
                    render_grid_to_path(&visualizer, canvas_width as u32, canvas_height as u32)
                } else {
                    String::new()
                };
                let origin_data = render_origin_to_path(&visualizer, canvas_width as u32, canvas_height as u32);

                if !path_data.is_empty() {
                    let _ = tx.send((1.0, "Complete".to_string(), Some(path_data), Some(rapid_moves_data), Some(grid_data), Some(origin_data)));
                } else {
                    let _ = tx.send((1.0, "Error: no data".to_string(), None, None, None, None));
                }
            });

            // Process messages from rendering thread
            std::thread::spawn(move || {
                while let Ok((progress, status, path_data, rapid_moves_data, grid_data, origin_data)) = rx.recv() {
                    let window_handle = window_weak_render.clone();
                    let status_clone = status.clone();
                    let path_clone = path_data.clone();
                    let rapid_moves_clone = rapid_moves_data.clone();
                    let grid_clone = grid_data.clone();
                    let origin_clone = origin_data.clone();
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
                                window.set_visualization_rapid_moves_data(slint::SharedString::from(rapid_moves));
                            }
                            if let Some(grid) = grid_clone {
                                window.set_visualization_grid_data(slint::SharedString::from(grid));
                            }
                            if let Some(origin) = origin_clone {
                                window.set_visualization_origin_data(slint::SharedString::from(origin));
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
            // Clamp to reasonable values (10% to 500%)
            if *scale > 5.0 {
                *scale = 5.0;
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
            // Clamp to reasonable values (10% to 500%)
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
    tx: std::sync::mpsc::Sender<(f32, String, Option<String>, Option<String>, Option<String>)>,
) {
    use gcodekit4::visualizer::{Visualizer2D, render_toolpath_to_path, render_grid_to_path, render_origin_to_path};

    let _ = tx.send((0.1, "Parsing G-code...".to_string(), None, None, None));

    // Parse G-code
    let mut visualizer = Visualizer2D::new();
    visualizer.parse_gcode(&gcode_content);

    let _ = tx.send((0.3, "Rendering...".to_string(), None, None, None));

    // Generate canvas path data
    let path_data = render_toolpath_to_path(&visualizer, 1600, 1200);
    let grid_data = render_grid_to_path(&visualizer, 1600, 1200);
    let origin_data = render_origin_to_path(&visualizer, 1600, 1200);

    if !path_data.is_empty() {
        let _ = tx.send((1.0, "Complete".to_string(), Some(path_data), Some(grid_data), Some(origin_data)));
    } else {
        let _ = tx.send((1.0, "Error: no data".to_string(), None, None, None));
    }
}
