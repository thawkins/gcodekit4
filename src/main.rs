use gcodekit4::{init_logging, VERSION, BUILD_DATE, list_ports, SerialCommunicator, ConnectionParams, ConnectionDriver, SerialParity, Communicator, SettingsDialog, SettingsPersistence, FirmwareSettingsIntegration, SettingValue, DeviceConsoleManager, DeviceMessageType, GcodeEditor};
use tracing::info;
use slint::VecModel;
use std::rc::Rc;
use std::cell::RefCell;

slint::include_modules!();

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
    
    // Shared state for settings dialog
    let settings_dialog = Rc::new(RefCell::new(SettingsDialog::new()));
    
    // Shared state for settings persistence
    let settings_persistence = Rc::new(RefCell::new(SettingsPersistence::new()));
    
    // Shared state for firmware settings integration
    let firmware_integration = Rc::new(RefCell::new(FirmwareSettingsIntegration::new("GRBL", "1.1")));
    
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
    
    // Initialize device console manager
    let console_manager = Rc::new(DeviceConsoleManager::new());
    info!("Device console manager initialized");
    
    // Add initial messages to console
    console_manager.add_message(DeviceMessageType::Success, "GCodeKit4 initialized");
    console_manager.add_message(DeviceMessageType::Output, "Ready for operation");
    
    // Initialize console output in UI with initial messages
    let console_output = console_manager.get_output();
    main_window.set_console_output(slint::SharedString::from(console_output));
    
    // Initialize G-Code editor
    let gcode_editor = Rc::new(GcodeEditor::new());
    info!("G-Code editor initialized");
    
    // Load sample G-Code content for demonstration
    let sample_gcode = "; Sample G-Code\nG00 X10 Y10\nG01 Z-5 F100\nG00 Z5\nM30\n";
    if let Err(e) = gcode_editor.load_content(sample_gcode) {
        info!("Warning: Could not load sample G-Code: {}", e);
    } else {
        let editor_content = gcode_editor.get_display_content();
        main_window.set_gcode_content(slint::SharedString::from(editor_content));
    }
    
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
        console_manager_clone.add_message(DeviceMessageType::Output, format!("Connecting to {} at {} baud", port_str, baud));
        
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
                console_manager_clone.add_message(DeviceMessageType::Success, format!("Successfully connected to {} at {} baud", port_str, baud));
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
                console_manager_clone.add_message(DeviceMessageType::Error, format!("Connection failed: {}", error_msg));
                if let Some(window) = window_weak.upgrade() {
                    window.set_connected(false);
                    window.set_connection_status(slint::SharedString::from(format!("Connection Failed")));
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
                console_manager_clone.add_message(DeviceMessageType::Success, "Successfully disconnected");
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
                console_manager_clone.add_message(DeviceMessageType::Error, format!("Disconnect error: {}", e));
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
    main_window.on_menu_file_open(move || {
        info!("Menu: File > Open selected");
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("File dialog would open here"));
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
                description: setting.description.clone().map(|s| s.into()).unwrap_or_default(),
            });
        }
        
        info!("Settings prepared: {} items for UI display", settings_items.len());
        
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
                    info!("Setting changed: {} = {}", setting.name, setting.value.as_str());
                }
            }
            
            // Save to disk
            {
                let mut persistence = settings_persistence_clone.borrow_mut();
                
                // Load settings from dialog into config
                if let Err(e) = persistence.load_from_dialog(&dialog) {
                    info!("Error loading settings from dialog: {}", e);
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from(
                            format!("Error saving settings: {}", e)
                        ));
                    }
                    return;
                }
                
                // Save to file
                let config_path = match gcodekit4::config::SettingsManager::config_file_path() {
                    Ok(path) => path,
                    Err(e) => {
                        info!("Could not determine config path: {}", e);
                        if let Some(window) = window_weak.upgrade() {
                            window.set_connection_status(slint::SharedString::from(
                                format!("Error: Could not determine config path: {}", e)
                            ));
                        }
                        return;
                    }
                };
                
                if let Err(e) = persistence.save_to_file(&config_path) {
                    info!("Error saving settings to file: {}", e);
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from(
                            format!("Error saving settings: {}", e)
                        ));
                    }
                } else {
                    info!("Settings saved to disk successfully");
                    if let Some(window) = window_weak.upgrade() {
                        window.set_connection_status(slint::SharedString::from("Settings saved successfully"));
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
            window.set_connection_status(slint::SharedString::from("Settings restored to defaults"));
        }
    });
    
    // Set up menu-view-fullscreen callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_fullscreen(move || {
        info!("Menu: View > Fullscreen selected");
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("Fullscreen toggle would activate here"));
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
    
    // Set up menu-view-device-console callback
    let window_weak = main_window.as_weak();
    let console_manager_weak = Rc::downgrade(&console_manager);
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
    
    // Set up menu-help-about callback
    let window_weak = main_window.as_weak();
    main_window.on_menu_help_about(move || {
        info!("Menu: Help > About selected");
        if let Some(window) = window_weak.upgrade() {
            let about_msg = format!("GCodeKit4 v{}\n\nUniversal G-Code Sender for CNC Machines", VERSION);
            window.set_connection_status(slint::SharedString::from(about_msg));
        }
    });
    
    main_window.show().map_err(|e| anyhow::anyhow!("UI Show Error: {}", e))?;
    main_window.run().map_err(|e| anyhow::anyhow!("UI Runtime Error: {}", e))?;

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
