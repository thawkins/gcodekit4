use gcodekit4::{init_logging, VERSION, BUILD_DATE, list_ports, SerialCommunicator, ConnectionParams, ConnectionDriver, SerialParity, Communicator};
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
    main_window.on_connect(move |port: slint::SharedString, baud: i32| {
        let port_str = port.to_string();
        info!("Connect requested for port: {} at {} baud", port_str, baud);
        
        // Update UI with connecting status immediately
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("Connecting..."));
            window.set_device_version(slint::SharedString::from("Detecting..."));
            window.set_machine_state(slint::SharedString::from("CONNECTING"));
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
                if let Some(window) = window_weak.upgrade() {
                    window.set_connected(true);
                    window.set_connection_status(slint::SharedString::from("Connected"));
                    window.set_device_version(slint::SharedString::from("GRBL 1.1+"));
                    window.set_machine_state(slint::SharedString::from("IDLE"));
                }
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                info!("Failed to connect: {}", error_msg);
                if let Some(window) = window_weak.upgrade() {
                    window.set_connected(false);
                    window.set_connection_status(slint::SharedString::from(format!("Connection Failed")));
                    window.set_device_version(slint::SharedString::from("Not Connected"));
                    window.set_machine_state(slint::SharedString::from("DISCONNECTED"));
                }
            }
        }
    });
    
    // Set up disconnect callback
    let window_weak = main_window.as_weak();
    let communicator_clone = communicator.clone();
    main_window.on_disconnect(move || {
        info!("Disconnect requested");
        let mut comm = communicator_clone.borrow_mut();
        match comm.disconnect() {
            Ok(()) => {
                info!("Successfully disconnected");
                if let Some(window) = window_weak.upgrade() {
                    window.set_connected(false);
                    window.set_connection_status(slint::SharedString::from("Disconnected"));
                    window.set_device_version(slint::SharedString::from("Not Connected"));
                    window.set_machine_state(slint::SharedString::from("DISCONNECTED"));
                    window.set_position_x(0.0);
                    window.set_position_y(0.0);
                    window.set_position_z(0.0);
                }
            }
            Err(e) => {
                info!("Failed to disconnect: {}", e);
                if let Some(window) = window_weak.upgrade() {
                    window.set_connection_status(slint::SharedString::from("Disconnect error"));
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
    main_window.on_menu_edit_preferences(move || {
        info!("Menu: Edit > Preferences selected");
        if let Some(window) = window_weak.upgrade() {
            window.set_connection_status(slint::SharedString::from("Preferences dialog would open here"));
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
