// On Windows, hide the console window for GUI applications
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use std::sync::{Arc, Mutex};
use gcodekit4::{
    init_logging, list_ports, CapabilityManager, Communicator,
    ConsoleListener, DeviceConsoleManager, DeviceMessageType,
    FirmwareSettingsIntegration, SerialCommunicator,
    SettingsController, SettingsDialog, SettingsManager, SettingsPersistence,
    SettingsCategory,
    BUILD_DATE, VERSION,
};
use gcodekit4_ui::GcodeEditor;
use gcodekit4_devicedb::{
    DeviceManager, DeviceProfileUiModel as DbDeviceProfile, DeviceUiController,
};
use gcodekit4_ui::EditorBridge;
use slint::{Model, VecModel};
use std::cell::RefCell;
#[allow(unused_imports)]
use std::error::Error;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::warn;

mod app;
use app::helpers::*;
use crate::app::designer::update_designer_ui;

slint::include_modules!();





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
    use crate::app::types::GcodeSendState;
    let gcode_send_state = Arc::new(Mutex::new(GcodeSendState {
        lines: std::collections::VecDeque::new(),
        pending_bytes: 0,
        line_lengths: std::collections::VecDeque::new(),
        total_sent: 0,
        total_lines: 0,
        start_time: None,
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
        can_undo: false,
        can_redo: false, can_group: false, can_ungroup: false,
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

    // Setup designer callbacks
    app::callbacks::designer::setup_designer_callbacks(
        &main_window,
        designer_mgr.clone(),
        editor_bridge.clone(),
        shift_pressed.clone(),
    );

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
        
        // Apply UI settings to window
        // main_window.set_show_menu_shortcuts(persistence.config().ui.show_menu_shortcuts);
    }

    // Initialize Settings Controller
    let settings_controller = Rc::new(SettingsController::new(
        settings_dialog.clone(),
        settings_persistence.clone(),
    ));

    // Initialize Device Manager
    let config_dir = SettingsManager::config_directory().unwrap_or_else(|_| PathBuf::from("."));
    let device_manager = std::sync::Arc::new(DeviceManager::new(config_dir.join("devices.json")));
    if let Err(e) = device_manager.load() {
        warn!("Failed to load device profiles: {}", e);
    }
    let device_ui_controller = Rc::new(DeviceUiController::new(device_manager.clone()));

    // Bind Device Manager callbacks
    {
        let controller = device_ui_controller.clone();
        let window_weak = main_window.as_weak();
        main_window.on_load_device_profiles(move || {
            let profiles = controller.get_ui_profiles();
            let slint_profiles: Vec<DeviceProfileUiModel> = profiles
                .iter()
                .map(|p| DeviceProfileUiModel {
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
                })
                .collect();

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

    // Register callbacks
    app::callbacks::settings::register_callbacks(&main_window, settings_controller.clone());
    app::callbacks::machine::register_callbacks(
        &main_window,
        communicator.clone(),
        console_manager.clone(),
        ports_model.clone(),
        status_polling_stop.clone(),
        status_polling_active.clone(),
        capability_manager.clone(),
        gcode_send_state.clone(),
        detected_firmware.clone(),
    );
    app::callbacks::editor::register_callbacks(
        &main_window,
        gcode_editor.clone(),
        console_manager.clone(),
        editor_bridge.clone(),
        communicator.clone(),
    );
    app::callbacks::cam::register_callbacks(
        &main_window,
        gcode_editor.clone(),
        console_manager.clone(),
        editor_bridge.clone(),
        materials_backend.clone(),
        tools_backend.clone(),
        device_manager.clone(),
    );





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
                gstate.start_time = Some(std::time::Instant::now());
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
    main_window.on_menu_settings_save(move || match controller_clone.save() {
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
    main_window.on_key_pressed_event(move |_msg| {});

    // Debug callback for editor clicked events
    main_window.on_editor_clicked(move || {});

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
            let result: gcodekit4_core::Result<Vec<u8>> = comm.receive();
            match result {
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
                        let result: gcodekit4_core::Result<Vec<u8>> = comm.receive();
                        if let Ok(response) = result {
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
    let designer_mgr_clone = designer_mgr.clone();
    let window_weak = main_window.as_weak();
    main_window.on_menu_view_designer(move || {
        if let Some(window) = window_weak.upgrade() {
            window.set_current_view(slint::SharedString::from("designer"));
            window.set_connection_status(slint::SharedString::from("Designer tool activated"));
            
            // Defer fit-to-view to allow layout to settle
            let window_weak_timer = window.as_weak();
            let designer_mgr_timer = designer_mgr_clone.clone();
            slint::Timer::single_shot(std::time::Duration::from_millis(100), move || {
                if let Some(window) = window_weak_timer.upgrade() {
                    let mut state = designer_mgr_timer.borrow_mut();
                    // First update to set correct viewport size from UI
                    update_designer_ui(&window, &mut state);
                    // Then fit to view
                    state.zoom_fit();
                    // Then update again to render with new zoom/pan
                    update_designer_ui(&window, &mut state);
                }
            });
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
                        slint::Timer::single_shot(
                            std::time::Duration::from_millis(200),
                            move || {
                                if let Some(window) = window_weak_retry.upgrade() {
                                    let canvas_width = window.get_visualizer_canvas_width();
                                    let canvas_height = window.get_visualizer_canvas_height();
                                    let max_intensity = window.get_visualizer_max_intensity();
                                    window
                                        .invoke_refresh_visualization(canvas_width, canvas_height, max_intensity);
                                }
                            },
                        );
                    } else {
                        let max_intensity = window.get_visualizer_max_intensity(); window.invoke_refresh_visualization(canvas_width, canvas_height, max_intensity);
                    }
                }
            });
        }
    });

    // Initialize Visualizer
    let visualizer = Rc::new(RefCell::new(gcodekit4::visualizer::Visualizer2D::new()));

    // Set up refresh-visualization callback
    let window_weak = main_window.as_weak();
    let visualizer_refresh = visualizer.clone();
    main_window.on_refresh_visualization(move |width, height, max_intensity| {
        if let Some(window) = window_weak.upgrade() {
            let gcode = window.get_gcode_content();
            let mut vis = visualizer_refresh.borrow_mut();
            
            // Parse G-code
            vis.parse_gcode(&gcode);
            
            // Update toolpath data
            window.set_visualization_path_data(vis.toolpath_svg().into());
            window.set_visualization_rapid_moves_data(vis.rapid_svg().into());
            window.set_visualization_g1_data(vis.g1_svg().into());
            window.set_visualization_g2_data(vis.g2_svg().into());
            window.set_visualization_g3_data(vis.g3_svg().into());
            window.set_visualization_g4_data(vis.g4_svg().into());
            
            // Update intensity layers
            let layers = gcodekit4::visualizer::render_intensity_overlay(&vis, width as u32, height as u32, max_intensity);
            let model = Rc::new(VecModel::from(layers.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>()));
            window.set_visualization_intensity_layers(slint::ModelRc::from(model));

            // Update grid and origin
            let (grid, grid_size) = gcodekit4::visualizer::render_grid_to_path(&vis, width as u32, height as u32);
            window.set_visualization_grid_data(grid.into());
            window.set_visualizer_grid_size(format!("{:.1}mm", grid_size).into());
            
            let origin = gcodekit4::visualizer::render_origin_to_path(&vis, width as u32, height as u32);
            window.set_visualization_origin_data(origin.into());
            
            // Update viewbox
            let (vb_x, vb_y, vb_w, vb_h) = vis.get_viewbox(width, height);
            window.set_visualizer_viewbox_x(vb_x);
            window.set_visualizer_viewbox_y(vb_y);
            window.set_visualizer_viewbox_width(vb_w);
            window.set_visualizer_viewbox_height(vb_h);
            
            // Update status
            window.set_visualizer_status("Ready".into());
            window.set_visualizer_zoom_scale(vis.zoom_scale);
            window.set_visualizer_x_offset(vis.x_offset);
            window.set_visualizer_y_offset(vis.y_offset);
            
            // Update bounds info
            let (min_x, max_x, min_y, max_y) = vis.get_bounds();
            window.set_visualizer_bounding_box_info(format!("X: {:.1} to {:.1}\nY: {:.1} to {:.1}", min_x, max_x, min_y, max_y).into());
        }
    });

    // Set up zoom-in callback
    let window_weak = main_window.as_weak();
    let visualizer_zoom_in = visualizer.clone();
    main_window.on_zoom_in(move |width, height| {
        if let Some(window) = window_weak.upgrade() {
            let mut vis = visualizer_zoom_in.borrow_mut();
            vis.zoom_in();
            
            // Update viewbox
            let (vb_x, vb_y, vb_w, vb_h) = vis.get_viewbox(width, height);
            window.set_visualizer_viewbox_x(vb_x);
            window.set_visualizer_viewbox_y(vb_y);
            window.set_visualizer_viewbox_width(vb_w);
            window.set_visualizer_viewbox_height(vb_h);
            
            // Update grid and origin
            let (grid, grid_size) = gcodekit4::visualizer::render_grid_to_path(&vis, width as u32, height as u32);
            window.set_visualization_grid_data(grid.into());
            window.set_visualizer_grid_size(format!("{:.1}mm", grid_size).into());
            
            let origin = gcodekit4::visualizer::render_origin_to_path(&vis, width as u32, height as u32);
            window.set_visualization_origin_data(origin.into());
            
            window.set_visualizer_zoom_scale(vis.zoom_scale);
        }
    });

    // Set up zoom-out callback
    let window_weak = main_window.as_weak();
    let visualizer_zoom_out = visualizer.clone();
    main_window.on_zoom_out(move |width, height| {
        if let Some(window) = window_weak.upgrade() {
            let mut vis = visualizer_zoom_out.borrow_mut();
            vis.zoom_out();
            
            // Update viewbox
            let (vb_x, vb_y, vb_w, vb_h) = vis.get_viewbox(width, height);
            window.set_visualizer_viewbox_x(vb_x);
            window.set_visualizer_viewbox_y(vb_y);
            window.set_visualizer_viewbox_width(vb_w);
            window.set_visualizer_viewbox_height(vb_h);
            
            // Update grid and origin
            let (grid, grid_size) = gcodekit4::visualizer::render_grid_to_path(&vis, width as u32, height as u32);
            window.set_visualization_grid_data(grid.into());
            window.set_visualizer_grid_size(format!("{:.1}mm", grid_size).into());
            
            let origin = gcodekit4::visualizer::render_origin_to_path(&vis, width as u32, height as u32);
            window.set_visualization_origin_data(origin.into());
            
            window.set_visualizer_zoom_scale(vis.zoom_scale);
        }
    });

    // Set up reset-view callback
    let window_weak = main_window.as_weak();
    let visualizer_reset = visualizer.clone();
    main_window.on_reset_view(move |width, height| {
        if let Some(window) = window_weak.upgrade() {
            let mut vis = visualizer_reset.borrow_mut();
            vis.reset_zoom();
            vis.reset_pan();
            vis.set_default_view(width, height);
            
            // Update viewbox
            let (vb_x, vb_y, vb_w, vb_h) = vis.get_viewbox(width, height);
            window.set_visualizer_viewbox_x(vb_x);
            window.set_visualizer_viewbox_y(vb_y);
            window.set_visualizer_viewbox_width(vb_w);
            window.set_visualizer_viewbox_height(vb_h);
            
            // Update grid and origin
            let (grid, grid_size) = gcodekit4::visualizer::render_grid_to_path(&vis, width as u32, height as u32);
            window.set_visualization_grid_data(grid.into());
            window.set_visualizer_grid_size(format!("{:.1}mm", grid_size).into());
            
            let origin = gcodekit4::visualizer::render_origin_to_path(&vis, width as u32, height as u32);
            window.set_visualization_origin_data(origin.into());
            
            window.set_visualizer_zoom_scale(vis.zoom_scale);
            window.set_visualizer_x_offset(vis.x_offset);
            window.set_visualizer_y_offset(vis.y_offset);
        }
    });

    // Set up fit-to-view callback
    let window_weak = main_window.as_weak();
    let visualizer_fit = visualizer.clone();
    main_window.on_fit_to_view(move |width, height| {
        if let Some(window) = window_weak.upgrade() {
            let mut vis = visualizer_fit.borrow_mut();
            vis.fit_to_view(width, height);
            
            // Update viewbox
            let (vb_x, vb_y, vb_w, vb_h) = vis.get_viewbox(width, height);
            window.set_visualizer_viewbox_x(vb_x);
            window.set_visualizer_viewbox_y(vb_y);
            window.set_visualizer_viewbox_width(vb_w);
            window.set_visualizer_viewbox_height(vb_h);
            
            // Update grid and origin
            let (grid, grid_size) = gcodekit4::visualizer::render_grid_to_path(&vis, width as u32, height as u32);
            window.set_visualization_grid_data(grid.into());
            window.set_visualizer_grid_size(format!("{:.1}mm", grid_size).into());
            
            let origin = gcodekit4::visualizer::render_origin_to_path(&vis, width as u32, height as u32);
            window.set_visualization_origin_data(origin.into());
            
            window.set_visualizer_zoom_scale(vis.zoom_scale);
            window.set_visualizer_x_offset(vis.x_offset);
            window.set_visualizer_y_offset(vis.y_offset);
        }
    });

    // Set up toggle-grid callback
    let window_weak = main_window.as_weak();
    let visualizer_grid = visualizer.clone();
    main_window.on_toggle_grid(move || {
        if let Some(window) = window_weak.upgrade() {
            let mut vis = visualizer_grid.borrow_mut();
            vis.toggle_grid();
            window.set_visualizer_show_grid(vis.is_grid_visible());
        }
    });

    // Set up pan-by-mouse callback
    let window_weak = main_window.as_weak();
    let visualizer_pan = visualizer.clone();
    main_window.on_pan_by_mouse(move |dx, dy| {
        if let Some(window) = window_weak.upgrade() {
            let mut vis = visualizer_pan.borrow_mut();
            // Update offsets (Slint coordinates are top-left, but our visualizer handles it)
            // Actually, we need to adjust x_offset and y_offset in visualizer
            // The visualizer's x_offset/y_offset are in pixels
            vis.x_offset += dx;
            vis.y_offset += dy;
            
            let width = window.get_visualizer_canvas_width();
            let height = window.get_visualizer_canvas_height();
            
            // Update viewbox
            let (vb_x, vb_y, vb_w, vb_h) = vis.get_viewbox(width, height);
            window.set_visualizer_viewbox_x(vb_x);
            window.set_visualizer_viewbox_y(vb_y);
            window.set_visualizer_viewbox_width(vb_w);
            window.set_visualizer_viewbox_height(vb_h);
            
            // Update grid and origin
            let (grid, grid_size) = gcodekit4::visualizer::render_grid_to_path(&vis, width as u32, height as u32);
            window.set_visualization_grid_data(grid.into());
            window.set_visualizer_grid_size(format!("{:.1}mm", grid_size).into());
            
            let origin = gcodekit4::visualizer::render_origin_to_path(&vis, width as u32, height as u32);
            window.set_visualization_origin_data(origin.into());
            
            window.set_visualizer_x_offset(vis.x_offset);
            window.set_visualizer_y_offset(vis.y_offset);
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
#[allow(dead_code)]
fn render_gcode_visualization_background_channel(
    gcode_content: String,
    width: u32,
    height: u32,
    max_intensity: f32,
    tx: std::sync::mpsc::Sender<(
        f32,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<f32>,
        Option<String>,
        Option<(f32, f32, f32, f32)>,
        Option<Vec<String>>,
    )>,
) {
    use gcodekit4::visualizer::{
        render_grid_to_path, render_origin_to_path, render_rapid_moves_to_path,
        render_g1_to_path, render_g2_to_path, render_g3_to_path, render_g4_to_path,
        Visualizer2D,
    };
    // Import render_intensity_overlay from canvas_renderer via crate root re-export if available, 
    // or we might need to update lib.rs to export it.
    // Assuming I'll update lib.rs next. For now, let's assume it's available.
    // Actually, I should check lib.rs exports.
    // I'll use the full path if needed or update lib.rs.
    // Let's assume I'll update lib.rs to export `render_intensity_overlay`.

    let _ = tx.send((
        0.1,
        "Parsing G-code...".to_string(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ));

    // Parse G-code
    let mut visualizer = Visualizer2D::new();
    visualizer.parse_gcode(&gcode_content);

    // Set default view to position origin at bottom-left
    visualizer.set_default_view(width as f32, height as f32);

    let _ = tx.send((
        0.3,
        "Rendering...".to_string(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ));

    // Generate canvas path data
    let g1_data = render_g1_to_path(&visualizer, width, height);
    let g2_data = render_g2_to_path(&visualizer, width, height);
    let g3_data = render_g3_to_path(&visualizer, width, height);
    let g4_data = render_g4_to_path(&visualizer, width, height);
    let rapid_moves_data = render_rapid_moves_to_path(&visualizer, width, height);
    let (grid_data, grid_size) = render_grid_to_path(&visualizer, width, height);
    let origin_data = render_origin_to_path(&visualizer, width, height);
    
    // Generate intensity layers
    // We need to access render_intensity_overlay. It's in canvas_renderer.
    // I'll need to make sure it's exported.
    // For now, I'll use the crate path if possible, but I'm in main.rs so I use `gcodekit4::visualizer::...`
    // I'll update lib.rs to export it.
    let intensity_layers = gcodekit4::visualizer::render_intensity_overlay(&visualizer, width, height, max_intensity);

    let viewbox = visualizer.get_viewbox(width as f32, height as f32);

    if !g1_data.is_empty() || !g2_data.is_empty() || !g3_data.is_empty() || !g4_data.is_empty() || !rapid_moves_data.is_empty() || !grid_data.is_empty() {
        let _ = tx.send((
            1.0,
            "Complete".to_string(),
            Some(g1_data),
            Some(g2_data),
            Some(g3_data),
            Some(g4_data),
            Some(rapid_moves_data),
            Some(grid_data),
            Some(origin_data),
            Some(grid_size),
            None,
            Some(viewbox),
            Some(intensity_layers),
        ));
    } else {
        let _ = tx.send((
            1.0,
            "Error: no data".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));
    }
}
