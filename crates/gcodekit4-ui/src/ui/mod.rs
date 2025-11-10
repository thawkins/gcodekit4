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

pub mod advanced_features_panel;
pub mod architecture;
pub mod components;
pub mod connection_panel;
pub mod console_panel;
pub mod control_buttons;
pub mod coordinate_system;
pub mod device_console_manager;
pub mod dro_panel;
pub mod events;
pub mod file_management;
pub mod file_operations;
pub mod file_validation_panel;
pub mod firmware_integration;
pub mod firmware_settings_panel;
pub mod gcode_editor;
pub mod help_system;
pub mod jog_controller;
pub mod keyboard_shortcuts;
pub mod layout_manager;
pub mod macros_panel;
pub mod main_window;
pub mod materials_manager_backend;
pub mod notifications;
pub mod tools_manager_backend;
pub mod overrides_panel;
pub mod progress_indicators;
pub mod safety_diagnostics_panel;
pub mod settings_dialog;
pub mod settings_persistence;
pub mod state;
pub mod themes;
pub mod ui_polish;

pub use advanced_features_panel::{
    AdvancedFeaturesPanel, PerformanceMetrics, SimulationState, SoftLimits, Tool, ToolChangeMode,
    WorkCoordinateSystem,
};
pub use architecture::UiArchitecture;
pub use connection_panel::ConnectionPanel;
pub use console_panel::ConsolePanel;
pub use control_buttons::ControlButtonsPanel;
pub use coordinate_system::CoordinateSystemPanel;
pub use device_console_manager::{
    ConsoleEvent, ConsoleListener, DeviceConsoleManager, DeviceMessageType,
};
pub use dro_panel::DROPanel;
pub use events::{UiEvent, UiEventBus};
pub use file_management::{FileReader, FileStatistics, RecentFilesManager, TemplateLibrary};
pub use file_operations::FileOperationsPanel;
pub use file_validation_panel::{FileValidationPanel, ValidationIssue, ValidationSeverity};
pub use firmware_integration::FirmwareSettingsIntegration;
pub use firmware_settings_panel::{FirmwareParameter, FirmwareSettingsPanel, ParameterType};
pub use gcode_editor::{GcodeEditor, GcodeLine, Token, TokenType};
pub use help_system::{AppInfo, HelpSystem, HelpTopic, ShortcutReference, TooltipProvider};
pub use jog_controller::JogControllerPanel;
pub use keyboard_shortcuts::{KeyBinding, KeyModifiers, KeyboardAction, KeyboardManager};
pub use layout_manager::{Layout, LayoutManager, LayoutPreset, PanelId, PanelLocation, PanelState};
pub use macros_panel::{GcodeMacro, MacroVariable, MacrosPanel};
pub use main_window::MainWindow;
pub use materials_manager_backend::MaterialsManagerBackend;
pub use notifications::{Notification, NotificationLevel, NotificationManager as NotificationMgr};
pub use tools_manager_backend::ToolsManagerBackend;
pub use overrides_panel::OverridesPanel;
pub use progress_indicators::{ProgressDisplay, StreamProgress};
pub use safety_diagnostics_panel::{
    BufferDiagnostics, CommunicationDiagnostics, DiagnosticEvent, EmergencyStopDisplay,
    FeedHoldState, MotionInterlockState, PerformanceDiagnostics, SafetyDiagnosticsPanel,
};
pub use settings_dialog::{
    KeyboardShortcut, Setting, SettingValue, SettingsCategory, SettingsDialog,
};
pub use settings_persistence::SettingsPersistence;
pub use state::UiState;
pub use themes::{Color, FontConfig, Theme, ThemeColors, ThemeId, ThemeManager};
pub use ui_polish::{I18nManager, KeyboardShortcutManager, NotificationManager, ProgressIndicator};
