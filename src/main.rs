use gcodekit4::{
    init_logging, list_ports, Communicator, ConnectionDriver, ConnectionParams, ConsoleListener,
    DeviceConsoleManager, DeviceMessageType, FirmwareSettingsIntegration, GcodeEditor,
    SerialCommunicator, SerialParity, SettingValue, SettingsDialog, SettingsPersistence,
    BUILD_DATE, VERSION,
};
use slint::VecModel;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::{debug, info, warn};

slint::include_modules!();

/// Copy text to clipboard using arboard crate
fn copy_to_clipboard(text: &str) -> bool {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            match clipboard.set_text(text.to_string()) {
                Ok(_) => {
                    info!("Copied {} characters to clipboard", text.len());
                    // Keep clipboard alive for a moment to ensure managers see it
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    true
                }
                Err(e) => {
                    info!("Error copying to clipboard: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            info!("Could not access clipboard: {}", e);
            false
        }
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

/// Update designer UI with current shapes from state
fn update_designer_ui(
    window: &MainWindow,
    state: &gcodekit4::DesignerState,
) {
    tracing::info!("update_designer_ui called with {} shapes", state.canvas.shapes().len());
    
    // Render canvas to image - using larger size to reduce stretching artifacts
    let canvas_width = 1600u32;
    let canvas_height = 1200u32;
    let img = gcodekit4::designer::renderer::render_canvas(
        &state.canvas,
        canvas_width,
        canvas_height,
        state.canvas.zoom() as f32,
        state.canvas.pan_offset().0 as f32,
        state.canvas.pan_offset().1 as f32,
    );
    
    // Convert to Slint image
    let buffer = slint::SharedPixelBuffer::<slint::Rgb8Pixel>::clone_from_slice(
        img.as_raw(),
        canvas_width,
        canvas_height,
    );
    let slint_image = slint::Image::from_rgb8(buffer);
    window.set_designer_canvas_image(slint_image);
    
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
    tracing::info!("Updating UI with {} shapes", shapes.len());
    for shape in &shapes {
        tracing::info!("  Shape {}: x={}, y={}, w={}, h={}", shape.id, shape.x, shape.y, shape.width, shape.height);
    }
    // Force UI to recognize the change by clearing first
    window.set_designer_shapes(slint::ModelRc::from(Rc::new(slint::VecModel::from(Vec::<crate::DesignerShape>::new()))));
    let shapes_model = Rc::new(slint::VecModel::from(shapes));
    window.set_designer_shapes(slint::ModelRc::from(shapes_model));
    
    // Increment update counter to force UI re-render
    let mut ui_state = window.get_designer_state();
    let counter = ui_state.update_counter + 1;
    ui_state.update_counter = counter;
    window.set_designer_state(ui_state);
    
    tracing::info!("Designer shapes updated in UI (counter: {})", counter);
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

    info!("═══════════════════════════════════════════════════════════");
    info!("GCodeKit4 v{}", VERSION);
    info!("Universal G-Code Sender for CNC Machines");
    info!("═══════════════════════════════════════════════════════════");

    let main_window = MainWindow::new().map_err(|e| anyhow::anyhow!("UI Error: {}", e))?;
    info!("UI window created successfully");

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
        let first_port: slint::SharedString = ports[0].clone().into();
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

    // Shared state for communicator
    let communicator = Rc::new(RefCell::new(SerialCommunicator::new()));

    // Flag to control status polling
    let status_polling_active = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let status_polling_stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Initialize device console manager early to register listeners
    // Use Arc since communicator listeners need Arc for thread-safe sharing
    let console_manager = std::sync::Arc::new(DeviceConsoleManager::new());
    info!("Device console manager initialized");

    // Create and register console listener with communicator
    let console_listener = ConsoleListener::new(console_manager.clone());
    communicator.borrow_mut().add_listener(console_listener);

    // Shared state for settings dialog
    let settings_dialog = Rc::new(RefCell::new(SettingsDialog::new()));

    // Initialize Designer state (Phase 2)
    let designer_mgr = Rc::new(RefCell::new(gcodekit4::DesignerState::new()));
    info!("Designer state initialized");
    
    // Initialize designer UI state with Select mode (0) as default
    let initial_designer_state = crate::DesignerState {
        mode: 0,
        zoom: 1.0,
        pan_x: 0.0,
        pan_y: 0.0,
        selected_id: -1,
        update_counter: 0,
    };
    main_window.set_designer_state(initial_designer_state);

    // Shared state for settings persistence
    let settings_persistence = Rc::new(RefCell::new(SettingsPersistence::new()));

    // Shared state for firmware settings integration
    let firmware_integration = Rc::new(RefCell::new(FirmwareSettingsIntegration::new(
        "GRBL", "1.1",
    )));

    // Load firmware settings
    {
        let mut fw_integration = firmware_integration.borrow_mut();
        if let Err(e) = fw_integration.load_grbl_defaults() {
            info!("Warning: Could not load firmware settings: {}", e);
        } else {
            info!("Firmware settings loaded successfully");

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

    // Initialize Phase 6 panels with sample data
    // File Validation Panel
    main_window.set_validation_summary(slint::SharedString::from("Ready"));
    main_window.set_validation_error_count(0);
    main_window.set_validation_warning_count(0);
    main_window.set_validation_issues(Default::default());

    // Advanced Features Panel
    main_window.set_advanced_features_mode(slint::SharedString::from("None"));
    main_window.set_simulation_state(slint::SharedString::from("Idle"));

    // Safety & Diagnostics Panel
    main_window.set_emergency_stop_armed(true);
    main_window.set_safety_status(slint::SharedString::from("Safe"));
    main_window.set_diagnostics_info(slint::SharedString::from("System Ready"));

    // Initialize G-Code editor
    let gcode_editor = Rc::new(GcodeEditor::new());
    info!("G-Code editor initialized");

    // Load sample G-Code content for demonstration
    // Note: Don't load sample gcode - start with empty editor
    // Let user choose to open a file or type their own

    // Load settings from config file if it exists
    {
        let mut persistence = settings_persistence.borrow_mut();
        let config_path = match gcodekit4::config::SettingsManager::config_file_path() {
            Ok(path) => path,
            Err(e) => {
                info!("Could not determine config path: {}", e);
                std::path::PathBuf::new()
            }
        };

        if config_path.exists() {
            match SettingsPersistence::load_from_file(&config_path) {
                Ok(loaded_persistence) => {
                    *persistence = loaded_persistence;
                    info!("Settings loaded from {:?}", config_path);
                }
                Err(e) => {
                    info!("Could not load settings from {:?}: {}", config_path, e);
                }
            }
        } else {
            info!("No existing config file at {:?}", config_path);
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
    main_window.on_connect(move |port: slint::SharedString, baud: i32| {
        let port_str = port.to_string();
        info!("Connect requested for port: {} at {} baud", port_str, baud);

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
        let mut comm = communicator_clone.borrow_mut();
        match comm.connect(&params) {
            Ok(()) => {
                info!("Successfully connected to {}", port_str);
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
                                if let Ok(_) = poll_comm.send(&[b'?']) {
                                    std::thread::sleep(std::time::Duration::from_millis(10));
                                    if let Ok(response) = poll_comm.receive() {
                                        if !response.is_empty() {
                                            let response_str = String::from_utf8_lossy(&response);
                                            debug!("Status response: {}", response_str);

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
                        let _ = poll_comm.disconnect();
                    }
                    polling_active.store(false, std::sync::atomic::Ordering::Relaxed);
                });
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                info!("Failed to connect: {}", error_msg);
                console_manager_clone.add_message(
                    DeviceMessageType::Error,
                    format!("Connection failed: {}", error_msg),
                );
                if let Some(window) = window_weak.upgrade() {
                    window.set_connected(false);
                    window.set_connection_status(slint::SharedString::from(format!(
                        "Connection Failed"
                    )));
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
    main_window.on_disconnect(move || {
        info!("Disconnect requested");
        console_manager_clone.add_message(DeviceMessageType::Output, "Disconnecting from device");

        // Stop the polling thread
        polling_stop_clone.store(true, std::sync::atomic::Ordering::Relaxed);

        let mut comm = communicator_clone.borrow_mut();
        match comm.disconnect() {
            Ok(()) => {
                info!("Successfully disconnected");
                // Reset the communicator to a fresh state by replacing with a new instance
                drop(comm);
                let mut new_comm = SerialCommunicator::new();
                // Re-register the console listener with the new communicator
                let console_listener = ConsoleListener::new(console_manager_clone.clone());
                new_comm.add_listener(console_listener);
                *communicator_clone.borrow_mut() = new_comm;

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
                }
            }
            Err(e) => {
                info!("Failed to disconnect: {}", e);
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
        info!("Menu: File > Exit selected");
        // Disconnect if connected before exiting
        let mut comm = communicator_clone.borrow_mut();
        if let Err(e) = comm.disconnect() {
            info!("Warning: Failed to disconnect during exit: {}", e);
        }
        info!("Exiting application");
        std::process::exit(0);
    });

    // Set up menu-file-open callback
    let window_weak = main_window.as_weak();
    let gcode_editor_clone = gcode_editor.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_menu_file_open(move || {
        info!("Menu: File > Open selected");

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

                info!("File loaded: {:?}", path);

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

                // DEBUG: Log the content that will be set in TextEdit
                info!("TextEdit content length: {} characters", content.len());
                let preview = if content.len() > 100 {
                    format!("{}...", &content[..100])
                } else {
                    content.clone()
                };
                info!("TextEdit content preview:\n{}", preview);
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
                    info!("Switching to gcode-editor view");
                    console_manager_clone.add_message(
                        DeviceMessageType::Output,
                        format!("DEBUG: Switching to gcode-editor view"),
                    );

                    // IMPORTANT: Switch to gcode-editor view to show the content
                    window.set_current_view(slint::SharedString::from("gcode-editor"));

                    // DEBUG: Log TextEdit update
                    info!(
                        "Setting gcode-content property with {} chars",
                        content.len()
                    );
                    console_manager_clone.add_message(
                        DeviceMessageType::Output,
                        format!("DEBUG: Setting TextEdit content ({} chars)", content.len()),
                    );

                    window.set_gcode_content(slint::SharedString::from(content.clone()));

                    // VERIFY: Log what was set
                    let verify_content = window.get_gcode_content();
                    info!(
                        "VERIFY - After set_gcode_content, get_gcode_content returns {} chars",
                        verify_content.len()
                    );
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
                        while let Ok((progress, status, image_data)) = rx.recv() {
                            // Use invoke_from_event_loop to update UI safely
                            let window_handle = window_weak.clone();
                            let status_clone = status.clone();
                            let image_clone = image_data.clone();

                            slint::invoke_from_event_loop(move || {
                                if let Some(window) = window_handle.upgrade() {
                                    window.set_visualizer_progress(progress);
                                    window.set_visualizer_status(slint::SharedString::from(
                                        status_clone.clone(),
                                    ));

                                    if let Some(png_bytes) = image_clone {
                                        if let Ok(img) = image::load_from_memory_with_format(
                                            &png_bytes,
                                            image::ImageFormat::Png,
                                        ) {
                                            let rgba_img = img.to_rgba8();
                                            let img_buffer = slint::Image::from_rgba8(
                                                slint::SharedPixelBuffer::clone_from_slice(
                                                    &rgba_img,
                                                    rgba_img.width(),
                                                    rgba_img.height(),
                                                ),
                                            );
                                            window.set_visualization_image(img_buffer);
                                        }
                                    }
                                }
                            })
                            .ok();
                        }
                    });

                    // DEBUG: Log console update
                    info!("Updating console output");
                    console_manager_clone.add_message(
                        DeviceMessageType::Output,
                        format!("DEBUG: TextEdit content set in view"),
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
                    debug!("File dialog cancelled by user");
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

                    // Update console display
                    let console_output = console_manager_clone.get_output();
                    window.set_console_output(slint::SharedString::from(console_output));
                }
            }
        }
    });

    // Set up menu-file-save callback
    let window_weak = main_window.as_weak();
    let gcode_editor_clone = gcode_editor.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_menu_file_save(move || {
        info!("Menu: File > Save selected");

        // Get current filename and content from window
        if let Some(window) = window_weak.upgrade() {
            let filename = window.get_gcode_filename().to_string();
            let current_content = window.get_gcode_content().to_string();

            // If it's "untitled.gcode", prompt for filename (treat as Save As)
            if filename.contains("untitled") {
                info!("No current file, treating Save as Save As");
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
                    info!("File saved: {}", filename);
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
        info!("Menu: File > Save As selected");

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

                    info!("File saved as: {}", file_name);
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
        info!("Send to Device: User clicked Send button");

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
            let comm = communicator_clone.borrow();
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
            info!(
                "Sending {} bytes of G-Code to device using character-counting protocol",
                current_content.len()
            );
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

            // Use a timer that fires every 1ms to process sending without blocking UI
            let timer = Rc::new(slint::Timer::default());
            let timer_clone = timer.clone();

            timer.start(
                slint::TimerMode::Repeated,
                std::time::Duration::from_millis(1),
                move || {
                    let mut state = send_state_timer.borrow_mut();
                    let mut comm = communicator_timer.borrow_mut();

                    // First, always try to receive and process acknowledgments
                    match comm.receive() {
                        Ok(response) => {
                            if !response.is_empty() {
                                let resp_str = String::from_utf8_lossy(&response);
                                debug!("Response: {}", resp_str);

                                let ok_count =
                                    resp_str.matches("ok").count() + resp_str.matches("OK").count();
                                for _ in 0..ok_count {
                                    if !state.line_lengths.is_empty() {
                                        let processed_length = state.line_lengths.remove(0);
                                        state.pending_bytes =
                                            state.pending_bytes.saturating_sub(processed_length);
                                        debug!(
                                            "ACK received: pending_bytes now {}",
                                            state.pending_bytes
                                        );
                                    }
                                }
                                state.timeout_count = 0;
                            }
                        }
                        Err(_) => {
                            if state.waiting_for_acks {
                                state.timeout_count += 1;
                            }
                        }
                    }

                    // Check if we're waiting for final acknowledgments
                    if state.waiting_for_acks {
                        if state.line_lengths.is_empty()
                            || state.timeout_count >= MAX_TIMEOUT_ITERATIONS
                        {
                            // Done - update UI
                            drop(comm);
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
                                    info!(
                                        "Successfully sent {} lines to device",
                                        final_state.send_count
                                    );
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

                    // Try to send next line if we have room in buffer
                    if state.line_index < state.lines.len() {
                        let trimmed = state.lines[state.line_index].trim().to_string();

                        if trimmed.is_empty() {
                            state.line_index += 1;
                            return;
                        }

                        let line_length = trimmed.len() + 1;

                        // Check if there's room in the buffer
                        if state.pending_bytes + line_length <= GRBL_RX_BUFFER_SIZE {
                            let command_bytes = format!("{}\n", trimmed);
                            let line_num = state.send_count + 1;
                            let pending_after = state.pending_bytes + line_length;

                            match comm.send(command_bytes.as_bytes()) {
                                Ok(bytes_sent) => {
                                    state.send_count += 1;
                                    state.pending_bytes += line_length;
                                    state.line_lengths.push(line_length);
                                    state.line_index += 1;

                                    info!(
                                        "Sent line {} ({} bytes): {} [pending: {}/{}]",
                                        line_num,
                                        bytes_sent,
                                        trimmed,
                                        pending_after,
                                        GRBL_RX_BUFFER_SIZE
                                    );
                                }
                                Err(e) => {
                                    let error_msg = format!("{}", e);
                                    let line_count = state.send_count + 1;
                                    warn!("Failed to send line {}: {}", line_count, error_msg);

                                    state.error_msg = error_msg.clone();
                                    state.error_occurred = true;

                                    drop(comm);
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
                                }
                            }
                        }
                    } else if !state.waiting_for_acks {
                        // All lines sent, now wait for final acknowledgments
                        let ack_count = state.line_lengths.len();
                        info!("All lines sent, waiting for {} acknowledgments", ack_count);
                        state.waiting_for_acks = true;
                        state.timeout_count = 0;
                    }
                },
            );
        }
    });

    // Set up menu-edit-preferences callback
    let window_weak = main_window.as_weak();
    let settings_dialog_clone = settings_dialog.clone();
    main_window.on_menu_edit_preferences(move || {
        info!("Menu: Edit > Preferences selected");

        // Get reference to settings dialog
        let dialog = settings_dialog_clone.borrow();

        // Build settings array for UI display - using generated SettingItem type
        let mut settings_items = Vec::new();
        for (_, setting) in &dialog.settings {
            let value_type = match &setting.value {
                SettingValue::Boolean(_) => "Boolean",
                SettingValue::Integer(_) => "Integer",
                SettingValue::Float(_) => "Float",
                _ => "String",
            };

            let category = format!("{}", setting.category);

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
            });
        }

        info!(
            "Settings prepared: {} items for UI display",
            settings_items.len()
        );

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
        info!("Menu: Settings Save clicked");

        let dialog = settings_dialog_clone.borrow();

        // Check for unsaved changes
        if dialog.has_changes() {
            info!("Settings have been modified");

            // Log all changed settings
            for setting in dialog.settings.values() {
                if setting.is_changed() {
                    info!(
                        "Setting changed: {} = {}",
                        setting.name,
                        setting.value.as_str()
                    );
                }
            }

            // Save to disk
            {
                let mut persistence = settings_persistence_clone.borrow_mut();

                // Load settings from dialog into config
                if let Err(e) = persistence.load_from_dialog(&dialog) {
                    info!("Error loading settings from dialog: {}", e);
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
                        info!("Could not determine config path: {}", e);
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
                    info!("Failed to create config directory: {}", e);
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Error: Failed to create config directory: {}",
                            e
                        )));
                    }
                    return;
                }

                if let Err(e) = persistence.save_to_file(&config_path) {
                    info!("Error saving settings to file: {}", e);
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from(format!(
                            "Error saving settings: {}",
                            e
                        )));
                    }
                } else {
                    info!("Settings saved to disk successfully");
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
                    info!("Dialog defaults synced with saved values");
                }
            }
        } else {
            info!("No settings changes to save");
            if let Some(window) = window_weak.upgrade() {
                window.set_connection_status(slint::SharedString::from("No changes to save"));
            }
        }
    });

    // Set up menu-settings-cancel callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_settings_cancel(move || {
        info!("Menu: Settings Cancel clicked");
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("Settings dialog closed"));
        }
    });

    // Set up menu-settings-restore-defaults callback
    let window_weak = main_window.as_weak();
    let settings_dialog_clone = settings_dialog.clone();
    main_window.on_menu_settings_restore_defaults(move || {
        info!("Menu: Settings Restore Defaults clicked");

        let mut dialog = settings_dialog_clone.borrow_mut();
        dialog.reset_all_to_defaults();

        info!("All settings reset to defaults");
        if let Some(window) = window_weak.upgrade() {
            window
                .set_connection_status(slint::SharedString::from("Settings restored to defaults"));
        }
    });

    // Set up update-setting callback
    let settings_dialog_clone = settings_dialog.clone();
    main_window.on_update_setting(
        move |setting_id: slint::SharedString, value: slint::SharedString| {
            debug!("Setting updated: {} = {}", setting_id, value);

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
                debug!("Setting {} updated successfully", setting_id_str);
            } else {
                debug!("Setting {} not found in dialog", setting_id_str);
            }
        },
    );

    // Set up menu-view-fullscreen callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_fullscreen(move || {
        info!("Menu: View > Fullscreen selected");
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from(
                "Fullscreen toggle would activate here",
            ));
        }
    });

    // Set up menu-view-gcode-editor callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_gcode_editor(move || {
        info!("Menu: View > G-Code Editor selected");
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("G-Code Editor activated"));
        }
    });

    // Set up menu-view-machine callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_machine(move || {
        info!("Menu: View > Machine Control selected");
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("Machine Control activated"));
        }
    });

    // Set up machine-jog-home callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    let console_manager_clone = console_manager.clone();
    main_window.on_machine_jog_home(move || {
        info!("Machine: Jog Home button clicked - sending $H command");

        if let Some(window) = window_weak.upgrade() {
            // Check if device is connected
            let mut comm = communicator_clone.borrow_mut();
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
                info!("Sending Home command: $H");
                console_manager_clone
                    .add_message(DeviceMessageType::Output, "Sending Home command...");

                match comm.send_command("$H") {
                    Ok(_) => {
                        info!("Home command sent successfully");
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
        info!(
            "Machine: Jog X+ button clicked - sending jog command with step size {}",
            step_size
        );

        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.borrow_mut();
            if !comm.is_connected() {
                warn!("Jog X+ failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in positive X direction using step size and 2000mm/min feedrate
                let jog_cmd = format!("$J=X{} F2000", step_size);
                info!("Sending jog command: {}", jog_cmd);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging X+ ({} mm)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {
                        info!("Jog X+ command sent successfully");
                    }
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
        info!(
            "Machine: Jog X- button clicked - sending jog command with step size {}",
            step_size
        );

        if let Some(window) = window_weak.upgrade() {
            let mut comm = communicator_clone.borrow_mut();
            if !comm.is_connected() {
                warn!("Jog X- failed: Device not connected");
                console_manager_clone
                    .add_message(DeviceMessageType::Error, "✗ Device not connected.");
            } else {
                // Send jog command in negative X direction using step size and 2000mm/min feedrate
                let jog_cmd = format!("$J=X-{} F2000", step_size);
                info!("Sending jog command: {}", jog_cmd);
                console_manager_clone.add_message(
                    DeviceMessageType::Output,
                    format!("Jogging X- ({} mm)...", step_size),
                );

                match comm.send(format!("{}\n", jog_cmd).as_bytes()) {
                    Ok(_) => {
                        info!("Jog X- command sent successfully");
                    }
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
        info!("Menu: View > Device Console selected");
        if let Some(window) = window_weak.upgrade() {
            if let Some(console_mgr) = console_manager_weak.upgrade() {
                let output = console_mgr.get_output();
                window.set_console_output(slint::SharedString::from(output));
                window.set_connection_status(slint::SharedString::from("Device Console activated"));
            }
        }
    });

    // Set up menu-view-file-validation callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_file_validation(move || {
        info!("Menu: View > File Validation selected");
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from(
                "File Validation panel activated",
            ));
        }
    });

    // Set up menu-view-advanced-features callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_advanced_features(move || {
        info!("Menu: View > Advanced Features selected");
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from(
                "Advanced Features panel activated",
            ));
        }
    });

    // Set up menu-view-safety-diagnostics callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_safety_diagnostics(move || {
        info!("Menu: View > Safety & Diagnostics selected");
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from(
                "Safety & Diagnostics panel activated",
            ));
        }
    });

    // Set up menu-view-designer callback (Phase 2)
    let _designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_designer(move || {
        info!("Menu: View > Designer selected");
        if let Some(window) = window_weak.upgrade() {
            window.set_current_view(slint::SharedString::from("designer"));
            window.set_connection_status(slint::SharedString::from(
                "Designer tool activated",
            ));
        }
    });

    // Set up menu-help-about callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_help_about(move || {
        info!("Menu: Help > About selected");
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
        info!("Console Clear button clicked");
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
        info!("Console Copy button clicked");
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
        info!("Designer: Set mode to {}", mode);
        let mut state = designer_mgr_clone.borrow_mut();
        state.set_mode(mode);
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from(
                format!("Drawing mode: {}", 
                    match mode {
                        0 => "Select",
                        1 => "Rectangle",
                        2 => "Circle",
                        3 => "Line",
                        _ => "Unknown"
                    }
                )
            ));
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
        info!("Designer: Zoom in");
        let mut state = designer_mgr_clone.borrow_mut();
        state.zoom_in();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &state);
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
        info!("Designer: Zoom out");
        let mut state = designer_mgr_clone.borrow_mut();
        state.zoom_out();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &state);
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
        info!("Designer: Zoom fit");
        let mut state = designer_mgr_clone.borrow_mut();
        state.zoom_fit();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &state);
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
        info!("Designer: Reset view");
        let mut state = designer_mgr_clone.borrow_mut();
        // Reset zoom to 1.0 (100%)
        state.canvas.set_zoom(1.0);
        // Reset pan to 0,0
        let (pan_x, pan_y) = state.canvas.pan_offset();
        state.canvas.pan(-pan_x, -pan_y);
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &state);
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
        info!("Designer: Delete selected");
        let mut state = designer_mgr_clone.borrow_mut();
        state.delete_selected();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &state);
            window.set_connection_status(slint::SharedString::from(
                format!("Shapes: {}", state.canvas.shapes().len())
            ));
        }
    });

    // Designer: Clear Canvas callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_clear_canvas(move || {
        info!("Designer: Clear canvas");
        let mut state = designer_mgr_clone.borrow_mut();
        state.clear_canvas();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &state);
            window.set_designer_gcode_generated(false);
            window.set_connection_status(slint::SharedString::from("Canvas cleared"));
        }
    });

    // Designer: Generate Toolpath callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_generate_toolpath(move || {
        info!("Designer: Generate toolpath");
        let mut state = designer_mgr_clone.borrow_mut();
        let gcode = state.generate_gcode();
        if let Some(window) = window_weak.upgrade() {
            window.set_designer_generated_gcode(slint::SharedString::from(gcode));
            window.set_designer_gcode_generated(true);
            window.set_connection_status(slint::SharedString::from(
                "G-code generated successfully"
            ));
        }
    });

    // Designer: Export G-code callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_export_gcode(move || {
        info!("Designer: Export G-code to editor");
        let mut state = designer_mgr_clone.borrow_mut();
        let gcode = state.generate_gcode();
        if let Some(window) = window_weak.upgrade() {
            window.set_gcode_content(slint::SharedString::from(gcode));
            window.set_current_view(slint::SharedString::from("gcode-editor"));
            window.set_connection_status(slint::SharedString::from(
                "G-code exported to editor"
            ));
        }
    });

    // Designer: Canvas Click callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_canvas_click(move |x: f32, y: f32| {
        info!("Designer: Canvas clicked at pixel ({}, {})", x, y);
        let mut state = designer_mgr_clone.borrow_mut();
        
        // Convert pixel coordinates to world coordinates
        let world_point = state.canvas.pixel_to_world(x as f64, y as f64);
        info!("Converted to world coordinates: ({}, {})", world_point.x, world_point.y);
        
        // If in Select mode, try to select a shape; otherwise add a new shape
        if state.canvas.mode() == gcodekit4::DrawingMode::Select {
            // Check if we clicked on the already selected shape - if so, don't deselect
            info!("Checking shapes for selection at world point ({}, {})", world_point.x, world_point.y);
            for (i, shape) in state.canvas.shapes().iter().enumerate() {
                let (x1, y1, x2, y2) = shape.shape.bounding_box();
                info!("  Shape {}: bounds=({}, {}) to ({}, {})", i, x1, y1, x2, y2);
            }
            if !state.canvas.is_point_in_selected(&world_point) {
                let selected = state.canvas.select_at(&world_point);
                info!("Selection result: {:?}", selected);
            }
        } else {
            state.add_shape_at(world_point.x, world_point.y);
        }
        
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &state);
            window.set_connection_status(slint::SharedString::from(
                format!("Shapes: {}", state.canvas.shapes().len())
            ));
        }
    });

    // Designer: Shape drag callback (move selected shape)
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_shape_drag(move |_shape_id: i32, dx: f32, dy: f32| {
        info!("Designer: Shape drag - id={}, pixel delta=({}, {})", _shape_id, dx, dy);
        let mut state = designer_mgr_clone.borrow_mut();
        
        // Convert pixel delta to world delta using viewport zoom
        // At zoom level Z, moving 1 pixel is equivalent to 1/Z world units
        let viewport = state.canvas.viewport();
        let world_dx = dx as f64 / viewport.zoom();
        let world_dy = dy as f64 / viewport.zoom();
        
        info!("Converted to world delta: ({}, {})", world_dx, world_dy);
        state.move_selected(world_dx, world_dy);
        
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &state);
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
                if (world_point.x - x1).abs() < handle_size && (world_point.y - y1).abs() < handle_size {
                    dragging_handle = 0; // Top-left
                } else if (world_point.x - x2).abs() < handle_size && (world_point.y - y1).abs() < handle_size {
                    dragging_handle = 1; // Top-right
                } else if (world_point.x - x1).abs() < handle_size && (world_point.y - y2).abs() < handle_size {
                    dragging_handle = 2; // Bottom-left
                } else if (world_point.x - x2).abs() < handle_size && (world_point.y - y2).abs() < handle_size {
                    dragging_handle = 3; // Bottom-right
                } else if (world_point.x - cx).abs() < handle_size && (world_point.y - cy).abs() < handle_size {
                    dragging_handle = 4; // Center (move handle)
                }
                
                info!("Designer: Detect handle at ({}, {}) - handle={}", x, y, dragging_handle);
            }
        }
        
        dragging_handle
    });

    // Designer: Handle drag callback (move or resize via handles)
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_handle_drag(move |_shape_id: i32, handle: i32, dx: f32, dy: f32| {
        let mut state = designer_mgr_clone.borrow_mut();
        
        // Convert pixel delta to world delta using viewport zoom
        let viewport = state.canvas.viewport();
        let world_dx = dx as f64 / viewport.zoom();
        let world_dy = dy as f64 / viewport.zoom();
        
        if handle == -1 || handle == 4 {
            // handle=-1 or handle=4 (center handle) means move the entire shape
            info!("Designer: Move shape - world delta=({}, {})", world_dx, world_dy);
            state.move_selected(world_dx, world_dy);
        } else {
            // Resize via specific handle (0=top-left, 1=top-right, 2=bottom-left, 3=bottom-right)
            info!("Designer: Resize handle {} - world delta=({}, {})", handle, world_dx, world_dy);
            state.resize_selected(handle as usize, world_dx, world_dy);
        }
        
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &state);
        }
    });

    // Designer: Deselect all callback
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_deselect_all(move || {
        info!("Designer: Deselect all");
        let mut state = designer_mgr_clone.borrow_mut();
        state.deselect_all();
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &state);
        }
    });

    // Designer: Canvas pan callback (drag on empty canvas)
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_designer_canvas_pan(move |dx: f32, dy: f32| {
        info!("Designer: Canvas pan - pixel delta=({}, {})", dx, dy);
        let mut state = designer_mgr_clone.borrow_mut();
        
        // Pan is in pixel space, convert to world space
        // Note: Pan direction is inverted - dragging right pans left (shows content to the right)
        let viewport = state.canvas.viewport();
        let world_dx = -dx as f64 / viewport.zoom();
        let world_dy = -dy as f64 / viewport.zoom();
        
        info!("Converted to world delta: ({}, {})", world_dx, world_dy);
        state.canvas.pan_by(world_dx, world_dy);
        
        if let Some(window) = window_weak.upgrade() {
            update_designer_ui(&window, &state);
        }
    });

    // Zoom state tracking (Arc for shared state across closures)
    use std::sync::{Arc, Mutex};
    let zoom_scale = Arc::new(Mutex::new(1.0f32));
    let pan_offset = Arc::new(Mutex::new((0.0f32, 0.0f32)));

    // Handle refresh visualization button
    let window_weak = main_window.as_weak();
    let zoom_for_refresh = zoom_scale.clone();
    let pan_for_refresh = pan_offset.clone();
    main_window.on_refresh_visualization(move || {
        info!("Refresh visualization requested");

        // Get the current G-code content
        if let Some(window) = window_weak.upgrade() {
            let content = window.get_gcode_content();

            if content.is_empty() {
                window.set_visualizer_status(slint::SharedString::from("No G-code loaded"));
                return;
            }

            info!("Re-rendering visualization with {} chars", content.len());

            // Reset progress
            window.set_visualizer_progress(0.0);
            window.set_visualizer_status(slint::SharedString::from("Refreshing..."));

            // Clear current image to show loading state
            window.set_visualization_image(slint::Image::default());

            // Spawn rendering thread
            let content_owned = content.to_string();
            let (tx, rx) = std::sync::mpsc::channel();
            let window_weak_render = window_weak.clone();
            let zoom_scale_render = zoom_for_refresh.clone();
            let pan_offset_render = pan_for_refresh.clone();
            let zoom_scale_for_msg = zoom_for_refresh.clone();
            let pan_offset_for_msg = pan_for_refresh.clone();

            // Get show_grid state before spawning thread
            let show_grid = window.get_visualizer_show_grid();

            std::thread::spawn(move || {
                use gcodekit4::visualizer::Visualizer2D;

                let _ = tx.send((0.1, "Parsing G-code...".to_string(), None));

                let mut visualizer = Visualizer2D::new();
                visualizer.show_grid = show_grid;

                visualizer.parse_gcode(&content_owned);

                // Apply zoom scale
                if let Ok(scale) = zoom_scale_render.lock() {
                    visualizer.zoom_scale = *scale;
                }

                // Apply pan offsets
                if let Ok(offsets) = pan_offset_render.lock() {
                    visualizer.x_offset = offsets.0;
                    visualizer.y_offset = offsets.1;
                }

                let _ = tx.send((0.3, "Rendering image...".to_string(), None));

                let image_bytes = visualizer.render(800, 600);

                if !image_bytes.is_empty() {
                    let _ = tx.send((0.7, "Encoding PNG...".to_string(), None));
                    let _ = tx.send((1.0, "Complete".to_string(), Some(image_bytes)));
                } else {
                    let _ = tx.send((1.0, "Error: no image data".to_string(), None));
                }
            });

            // Process messages from rendering thread
            std::thread::spawn(move || {
                while let Ok((progress, status, image_data)) = rx.recv() {
                    let window_handle = window_weak_render.clone();
                    let status_clone = status.clone();
                    let image_clone = image_data.clone();
                    let zoom_for_closure = zoom_scale_for_msg.clone();
                    let pan_for_closure = pan_offset_for_msg.clone();

                    slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_handle.upgrade() {
                            window.set_visualizer_progress(progress);
                            window.set_visualizer_status(slint::SharedString::from(
                                status_clone.clone(),
                            ));

                            if let Some(png_bytes) = image_clone {
                                if let Ok(img) = image::load_from_memory_with_format(
                                    &png_bytes,
                                    image::ImageFormat::Png,
                                ) {
                                    let rgba_img = img.to_rgba8();
                                    let img_buffer = slint::Image::from_rgba8(
                                        slint::SharedPixelBuffer::clone_from_slice(
                                            &rgba_img,
                                            rgba_img.width(),
                                            rgba_img.height(),
                                        ),
                                    );
                                    window.set_visualization_image(img_buffer);

                                    // Update indicator properties
                                    if let Ok(scale) = zoom_for_closure.lock() {
                                        window.set_visualizer_zoom_scale(*scale);
                                    }
                                    if let Ok(offsets) = pan_for_closure.lock() {
                                        window.set_visualizer_x_offset(offsets.0);
                                        window.set_visualizer_y_offset(offsets.1);
                                    }
                                }
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
    main_window.on_zoom_in(move || {
        info!("Zoom in requested");

        if let Ok(mut scale) = zoom_scale_in.lock() {
            *scale *= 1.1;
            // Clamp to reasonable values (10% to 500%)
            if *scale > 5.0 {
                *scale = 5.0;
            }
            info!("New zoom scale: {}%", (*scale * 100.0).round() as u32);

            // Update UI immediately
            if let Some(window) = window_weak_zoom_in.upgrade() {
                window.set_visualizer_zoom_scale(*scale);
                window.invoke_refresh_visualization();
            }
        }
    });

    // Handle zoom out button
    let zoom_scale_out = zoom_scale.clone();
    let window_weak_zoom_out = main_window.as_weak();
    main_window.on_zoom_out(move || {
        info!("Zoom out requested");

        if let Ok(mut scale) = zoom_scale_out.lock() {
            *scale /= 1.1;
            // Clamp to reasonable values (10% to 500%)
            if *scale < 0.1 {
                *scale = 0.1;
            }
            info!("New zoom scale: {}%", (*scale * 100.0).round() as u32);

            // Update UI immediately
            if let Some(window) = window_weak_zoom_out.upgrade() {
                window.set_visualizer_zoom_scale(*scale);
                window.invoke_refresh_visualization();
            }
        }
    });

    // Handle reset view button
    let zoom_scale_reset = zoom_scale.clone();
    let pan_offset_reset = pan_offset.clone();
    let window_weak_reset = main_window.as_weak();
    main_window.on_reset_view(move || {
        info!("Reset view requested");

        if let Ok(mut scale) = zoom_scale_reset.lock() {
            *scale = 1.0;
            info!("Zoom scale reset to 100%");
        }

        if let Ok(mut offsets) = pan_offset_reset.lock() {
            offsets.0 = 0.0;
            offsets.1 = 0.0;
            info!("Pan offsets reset to (0, 0)");
        }

        // Update UI immediately
        if let Some(window) = window_weak_reset.upgrade() {
            window.set_visualizer_zoom_scale(1.0);
            window.set_visualizer_x_offset(0.0);
            window.set_visualizer_y_offset(0.0);
            window.invoke_refresh_visualization();
        }
    });

    // Handle fit to view button
    let window_weak_fit = main_window.as_weak();
    let zoom_for_fit = zoom_scale.clone();
    let pan_for_fit = pan_offset.clone();
    main_window.on_fit_to_view(move || {
        info!("Fit to view requested");

        if let Some(window) = window_weak_fit.upgrade() {
            let content = window.get_gcode_content();

            if content.is_empty() {
                window.set_visualizer_status(slint::SharedString::from("No G-code loaded"));
                return;
            }

            info!("Calculating fit-to-view for {} chars", content.len());

            // Reset progress
            window.set_visualizer_progress(0.0);
            window.set_visualizer_status(slint::SharedString::from("Fitting to view..."));

            // Clear current image to show loading state
            window.set_visualization_image(slint::Image::default());

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

                let _ = tx.send((0.1, "Parsing G-code...".to_string(), None));

                let mut visualizer = Visualizer2D::new();
                visualizer.show_grid = show_grid;

                visualizer.parse_gcode(&content_owned);

                let _ = tx.send((0.2, "Calculating bounding box...".to_string(), None));

                // Calculate fit parameters (canvas is 800x600)
                visualizer.fit_to_view(800.0, 600.0);

                // Apply fit parameters to shared state
                if let Ok(mut scale) = zoom_fit_render.lock() {
                    *scale = visualizer.zoom_scale;
                    info!("Fit zoom scale: {}%", (*scale * 100.0).round() as u32);
                }

                if let Ok(mut offsets) = pan_fit_render.lock() {
                    offsets.0 = visualizer.x_offset;
                    offsets.1 = visualizer.y_offset;
                    info!("Fit offsets: x={}, y={}", offsets.0, offsets.1);
                }

                let _ = tx.send((0.3, "Rendering image...".to_string(), None));

                let image_bytes = visualizer.render(800, 600);

                if !image_bytes.is_empty() {
                    let _ = tx.send((0.7, "Encoding PNG...".to_string(), None));
                    let _ = tx.send((1.0, "Complete".to_string(), Some(image_bytes)));
                } else {
                    let _ = tx.send((1.0, "Error: no image data".to_string(), None));
                }
            });

            // Process messages from rendering thread
            let zoom_for_closure_fit = zoom_for_fit.clone();
            let pan_for_closure_fit = pan_for_fit.clone();
            std::thread::spawn(move || {
                while let Ok((progress, status, image_data)) = rx.recv() {
                    let window_handle = window_weak_render.clone();
                    let status_clone = status.clone();
                    let image_clone = image_data.clone();
                    let zoom_for_closure = zoom_for_closure_fit.clone();
                    let pan_for_closure = pan_for_closure_fit.clone();

                    slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_handle.upgrade() {
                            window.set_visualizer_progress(progress);
                            window.set_visualizer_status(slint::SharedString::from(status_clone));

                            if let Some(png_bytes) = image_clone {
                                if let Ok(img) = image::load_from_memory_with_format(
                                    &png_bytes,
                                    image::ImageFormat::Png,
                                ) {
                                    let rgba_img = img.to_rgba8();
                                    let img_buffer = slint::Image::from_rgba8(
                                        slint::SharedPixelBuffer::clone_from_slice(
                                            &rgba_img,
                                            rgba_img.width(),
                                            rgba_img.height(),
                                        ),
                                    );
                                    window.set_visualization_image(img_buffer);

                                    // Update indicator properties
                                    if let Ok(scale) = zoom_for_closure.lock() {
                                        window.set_visualizer_zoom_scale(*scale);
                                    }
                                    if let Ok(offsets) = pan_for_closure.lock() {
                                        window.set_visualizer_x_offset(offsets.0);
                                        window.set_visualizer_y_offset(offsets.1);
                                    }
                                }
                            }
                        }
                    })
                    .ok();
                }
            });
        }
    });

    // Handle pan left button
    let pan_left_clone = pan_offset.clone();
    let window_weak_pan_left = main_window.as_weak();
    main_window.on_pan_left(move || {
        info!("Pan left requested");

        if let Ok(mut offsets) = pan_left_clone.lock() {
            offsets.0 -= 80.0; // 10% of 800px canvas
            info!("Pan offset: x={}, y={}", offsets.0, offsets.1);

            if let Some(window) = window_weak_pan_left.upgrade() {
                window.set_visualizer_x_offset(offsets.0);
                window.set_visualizer_y_offset(offsets.1);
                window.invoke_refresh_visualization();
            }
        }
    });

    // Handle pan right button
    let pan_right_clone = pan_offset.clone();
    let window_weak_pan_right = main_window.as_weak();
    main_window.on_pan_right(move || {
        info!("Pan right requested");

        if let Ok(mut offsets) = pan_right_clone.lock() {
            offsets.0 += 80.0; // 10% of 800px canvas
            info!("Pan offset: x={}, y={}", offsets.0, offsets.1);

            if let Some(window) = window_weak_pan_right.upgrade() {
                window.set_visualizer_x_offset(offsets.0);
                window.set_visualizer_y_offset(offsets.1);
                window.invoke_refresh_visualization();
            }
        }
    });

    // Handle pan up button
    let pan_up_clone = pan_offset.clone();
    let window_weak_pan_up = main_window.as_weak();
    main_window.on_pan_up(move || {
        info!("Pan up requested");

        if let Ok(mut offsets) = pan_up_clone.lock() {
            offsets.1 -= 60.0; // 10% of 600px canvas (down in screen coords)
            info!("Pan offset: x={}, y={}", offsets.0, offsets.1);

            if let Some(window) = window_weak_pan_up.upgrade() {
                window.set_visualizer_x_offset(offsets.0);
                window.set_visualizer_y_offset(offsets.1);
                window.invoke_refresh_visualization();
            }
        }
    });

    // Handle pan down button
    let pan_down_clone = pan_offset.clone();
    let window_weak_pan_down = main_window.as_weak();
    main_window.on_pan_down(move || {
        info!("Pan down requested");

        if let Ok(mut offsets) = pan_down_clone.lock() {
            offsets.1 += 60.0; // 10% of 600px canvas (up in screen coords)
            info!("Pan offset: x={}, y={}", offsets.0, offsets.1);

            if let Some(window) = window_weak_pan_down.upgrade() {
                window.set_visualizer_x_offset(offsets.0);
                window.set_visualizer_y_offset(offsets.1);
                window.invoke_refresh_visualization();
            }
        }
    });

    // Toggle grid callback
    let window_weak_grid = main_window.as_weak();
    main_window.on_toggle_grid(move || {
        info!("Toggle grid requested");
        if let Some(window) = window_weak_grid.upgrade() {
            window.invoke_refresh_visualization();
        }
    });

    main_window
        .show()
        .map_err(|e| anyhow::anyhow!("UI Show Error: {}", e))?;
    main_window
        .run()
        .map_err(|e| anyhow::anyhow!("UI Runtime Error: {}", e))?;

    info!("Application closed");
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
                info!("No serial ports found");
                Ok(vec![slint::SharedString::from("No ports available")])
            } else {
                info!("Found {} serial ports", port_names.len());
                Ok(port_names)
            }
        }
        Err(e) => {
            info!("Error listing serial ports: {}", e);
            Ok(vec![slint::SharedString::from("Error reading ports")])
        }
    }
}

/// Render G-code visualization to an image
#[allow(dead_code)]
fn render_gcode_visualization(window: &MainWindow, gcode_content: &str) {
    use gcodekit4::visualizer::Visualizer2D;

    info!("Rendering 2D G-code visualization");

    // Parse G-code
    let mut visualizer = Visualizer2D::new();
    visualizer.parse_gcode(gcode_content);

    info!("Parsed {} G-code commands", visualizer.get_command_count());

    // Render to image
    let image_bytes = visualizer.render(800, 600);

    if !image_bytes.is_empty() {
        // Convert PNG bytes to slint Image
        if let Ok(img) = image::load_from_memory_with_format(&image_bytes, image::ImageFormat::Png)
        {
            let rgba_img = img.to_rgba8();
            let img_buffer = slint::Image::from_rgba8(slint::SharedPixelBuffer::clone_from_slice(
                &rgba_img,
                rgba_img.width(),
                rgba_img.height(),
            ));
            window.set_visualization_image(img_buffer);
            info!("Visualization rendered successfully");
        } else {
            info!("Failed to load rendered image");
        }
    } else {
        info!("Failed to render visualization - no image bytes");
    }
}

/// Render G-code visualization in background thread using message passing
fn render_gcode_visualization_background_channel(
    gcode_content: String,
    tx: std::sync::mpsc::Sender<(f32, String, Option<Vec<u8>>)>,
) {
    use gcodekit4::visualizer::Visualizer2D;

    let _ = tx.send((0.1, "Parsing G-code...".to_string(), None));

    // Parse G-code
    let mut visualizer = Visualizer2D::new();
    visualizer.parse_gcode(&gcode_content);

    let _ = tx.send((0.3, "Rendering image...".to_string(), None));

    // Render to image
    let image_bytes = visualizer.render(800, 600);

    if !image_bytes.is_empty() {
        let _ = tx.send((0.7, "Encoding PNG...".to_string(), None));
        let _ = tx.send((1.0, "Complete".to_string(), Some(image_bytes)));
    } else {
        let _ = tx.send((1.0, "Error: no image data".to_string(), None));
    }
}

/// Render G-code visualization in background thread
#[allow(dead_code)]
fn render_gcode_visualization_background(
    _window_weak: slint::Weak<MainWindow>,
    _gcode_content: String,
) {
    // This function is deprecated - use render_gcode_visualization_background_channel instead
}
