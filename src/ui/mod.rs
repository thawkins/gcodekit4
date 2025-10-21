//! User interface module - Slint-based
//!
//! **Phase 5: UI Implementation (Tasks 66-90)**
//!
//! This module contains:
//! - UI Architecture setup (Task 66)
//! - Component hierarchy and communication
//! - State management
//! - Event handling
//! - 11 major UI panels (Tasks 67-76)
//! - Macros Panel (Task 77)
//! - Settings/Preferences Dialog (Task 78)
//! - Firmware Settings Panel (Task 79)
//! - Additional UI features (Tasks 80-90)

pub mod architecture;
pub mod components;
pub mod state;
pub mod events;
pub mod main_window;
pub mod connection_panel;
pub mod dro_panel;
pub mod jog_controller;
pub mod file_operations;
pub mod gcode_viewer;
pub mod console_panel;
pub mod control_buttons;
pub mod overrides_panel;
pub mod coordinate_system;
pub mod macros_panel;
pub mod settings_dialog;
pub mod firmware_settings_panel;

pub use architecture::UiArchitecture;
pub use state::UiState;
pub use events::{UiEvent, UiEventBus};
pub use main_window::MainWindow;
pub use connection_panel::ConnectionPanel;
pub use dro_panel::DROPanel;
pub use jog_controller::JogControllerPanel;
pub use file_operations::FileOperationsPanel;
pub use gcode_viewer::GCodeViewerPanel;
pub use console_panel::ConsolePanel;
pub use control_buttons::ControlButtonsPanel;
pub use overrides_panel::OverridesPanel;
pub use coordinate_system::CoordinateSystemPanel;
pub use macros_panel::{MacrosPanel, GcodeMacro, MacroVariable};
pub use settings_dialog::{SettingsDialog, Setting, SettingValue, SettingsCategory, KeyboardShortcut};
pub use firmware_settings_panel::{FirmwareSettingsPanel, FirmwareParameter, ParameterType};
