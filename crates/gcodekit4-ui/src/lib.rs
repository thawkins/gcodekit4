//! # GCodeKit4 UI
//!
//! Slint-based user interface for GCodeKit4.
//! Provides UI panels, visualizer, settings, and editor components.

pub mod ui;
pub mod visualizer;
pub mod config;
pub mod testing;

pub use ui::{
    ConsoleListener, DeviceConsoleManager, DeviceMessageType, FirmwareSettingsIntegration,
    GcodeEditor, GcodeLine, KeyboardShortcut, Setting, SettingValue, SettingsCategory,
    SettingsDialog, SettingsPersistence, Token, TokenType,
};

pub use config::{
    Config, ConnectionSettings, ConnectionType, FileProcessingSettings, FirmwareSettings,
    MachineSettings, SettingsManager, UiSettings,
};
