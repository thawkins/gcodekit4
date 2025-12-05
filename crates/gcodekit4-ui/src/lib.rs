//! # GCodeKit4 UI
//!
//! Slint-based user interface for GCodeKit4.
//! Provides UI panels, visualizer, settings, and editor components.

pub mod testing;
pub mod ui;

pub use ui::{
    ConsoleEvent, ConsoleListener, DeviceConsoleManager, DeviceMessageType, FirmwareSettingsIntegration,
    GcodeEditor, GcodeLine, KeyboardShortcut, Setting, SettingUiModel, SettingValue,
    SettingsCategory, SettingsController, SettingsDialog, SettingsPersistence, Token, TokenType,
};

pub use gcodekit4_settings::{
    Config, ConnectionSettings, ConnectionType, FileProcessingSettings, FirmwareSettings,
    MachineSettings, SettingsManager, UiSettings,
};

pub use gcodekit4_gcodeeditor::{
    EditorBridge, EditorState, SlintTextLine, TextBuffer, TextChange, TextLine, UndoManager,
    Viewport,
};
