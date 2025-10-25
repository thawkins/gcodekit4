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

    // Initialize device console manager early to register listeners
    // Use Arc since communicator listeners need Arc for thread-safe sharing
    let console_manager = std::sync::Arc::new(DeviceConsoleManager::new());
    info!("Device console manager initialized");

    // Create and register console listener with communicator
    let console_listener = ConsoleListener::new(console_manager.clone());
    communicator.borrow_mut().add_listener(console_listener);

    // Shared state for settings dialog
    let settings_dialog = Rc::new(RefCell::new(SettingsDialog::new()));

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
    main_window.on_disconnect(move || {
        info!("Disconnect requested");
        console_manager_clone.add_message(DeviceMessageType::Output, "Disconnecting from device");
        let mut comm = communicator_clone.borrow_mut();
        match comm.disconnect() {
            Ok(()) => {
                info!("Successfully disconnected");
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
    main_window.on_menu_file_exit(move || {
        info!("Menu: File > Exit selected");
        // Disconnect if connected before exiting
        let mut comm = communicator.borrow_mut();
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
            window.set_connection_status(slint::SharedString::from("Settings changes discarded"));
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
