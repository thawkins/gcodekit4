use gcodekit4::{init_logging, VERSION, list_ports};
use tracing::info;
use slint::VecModel;
use std::rc::Rc;

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
    
    // Get initial list of ports
    let ports = get_available_ports()?;
    let ports_model = Rc::new(VecModel::from(ports.clone()));
    main_window.set_available_ports(slint::ModelRc::from(ports_model.clone()));
    
    // Set up refresh-ports callback
    let ports_model_clone = ports_model.clone();
    main_window.on_refresh_ports(move || {
        if let Ok(ports) = get_available_ports() {
            ports_model_clone.set_vec(ports);
        }
    });
    
    // Set up connect callback
    main_window.on_connect(|port: slint::SharedString, baud: i32| {
        info!("Connect requested for port: {} at {} baud", port, baud);
    });
    
    // Set up disconnect callback
    main_window.on_disconnect(|| {
        info!("Disconnect requested");
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
