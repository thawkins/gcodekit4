//! Settings Persistence
//!
//! Handles loading and saving application settings from/to configuration files.
//! Bridges SettingsDialog (UI) with Config (persistence layer).
//! Provides validation, migration, and synchronization of settings.

use crate::config::Config;
use crate::view_model::{
    KeyboardShortcut, Setting, SettingValue, SettingsCategory, SettingsDialog,
};
use gcodekit4_core::Result;
use std::path::Path;

/// Settings persistence layer
#[derive(Debug, Clone)]
pub struct SettingsPersistence {
    config: Config,
}

impl SettingsPersistence {
    /// Create new persistence layer with default config
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Load settings from file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let config = Config::load_from_file(path)?;
        Ok(Self { config })
    }

    /// Save settings to file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        self.config.save_to_file(path)?;
        Ok(())
    }

    /// Populate SettingsDialog from config
    pub fn populate_dialog(&self, dialog: &mut SettingsDialog) {
        // Connection Settings - Moved to DeviceDB
        // self.add_connection_settings(dialog);

        // UI Settings
        self.add_ui_settings(dialog);

        // File Processing Settings
        self.add_file_processing_settings(dialog);

        // Keyboard Shortcuts (from config if available)
        self.add_keyboard_shortcuts(dialog);
    }

    /// Load settings from dialog into config
    pub fn load_from_dialog(&mut self, dialog: &SettingsDialog) -> Result<()> {
        // Update connection settings - Moved to DeviceDB
        // self.update_connection_settings(dialog)?;

        // Update UI settings
        self.update_ui_settings(dialog)?;

        // Update file processing settings
        self.update_file_processing_settings(dialog)?;

        // Validate updated config
        self.config.validate()?;

        Ok(())
    }

    /// Get reference to config
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get mutable reference to config
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Validate settings
    pub fn validate(&self) -> Result<()> {
        self.config.validate()
    }

    /// Add UI settings to dialog
    fn add_ui_settings(&self, dialog: &mut SettingsDialog) {
        let ui = &self.config.ui;

        // Theme
        let themes = vec!["Dark".to_string(), "Light".to_string()];
        dialog.add_setting(
            Setting::new(
                "theme",
                "Theme",
                SettingValue::Enum(ui.theme.clone(), themes),
            )
            .with_description("Application color theme")
            .with_category(SettingsCategory::UserInterface),
        );

        // Window Width
        dialog.add_setting(
            Setting::new(
                "window_width",
                "Window Width",
                SettingValue::Integer(ui.window_width as i32),
            )
            .with_description("Default window width in pixels")
            .with_category(SettingsCategory::UserInterface),
        );

        // Window Height
        dialog.add_setting(
            Setting::new(
                "window_height",
                "Window Height",
                SettingValue::Integer(ui.window_height as i32),
            )
            .with_description("Default window height in pixels")
            .with_category(SettingsCategory::UserInterface),
        );

        // Show Toolbar
        dialog.add_setting(
            Setting::new(
                "show_toolbar",
                "Show Toolbar",
                SettingValue::Boolean(ui.panel_visibility.get("toolbar").copied().unwrap_or(true)),
            )
            .with_description("Show the main toolbar")
            .with_category(SettingsCategory::UserInterface),
        );

        // Show Status Bar
        dialog.add_setting(
            Setting::new(
                "show_status_bar",
                "Show Status Bar",
                SettingValue::Boolean(
                    ui.panel_visibility
                        .get("status_bar")
                        .copied()
                        .unwrap_or(true),
                ),
            )
            .with_description("Show the status bar at the bottom")
            .with_category(SettingsCategory::UserInterface),
        );
    }

    /// Add file processing settings to dialog
    fn add_file_processing_settings(&self, dialog: &mut SettingsDialog) {
        let file = &self.config.file_processing;

        // Preserve Comments (inverted logic: preserve = not remove)
        dialog.add_setting(
            Setting::new(
                "preserve_comments",
                "Preserve Comments",
                SettingValue::Boolean(file.preserve_comments),
            )
            .with_description("Keep G-code comments during file processing")
            .with_category(SettingsCategory::FileProcessing),
        );

        // Arc Segment Length
        dialog.add_setting(
            Setting::new(
                "arc_segment_length",
                "Arc Segment Length (mm)",
                SettingValue::String(file.arc_segment_length.to_string()),
            )
            .with_description("Length of arc segments for arc-to-line expansion")
            .with_category(SettingsCategory::FileProcessing),
        );

        // Max Line Length
        dialog.add_setting(
            Setting::new(
                "max_line_length",
                "Max Line Length",
                SettingValue::Integer(file.max_line_length as i32),
            )
            .with_description("Maximum characters per line in output files")
            .with_category(SettingsCategory::FileProcessing),
        );
    }

    /// Add keyboard shortcuts to dialog
    fn add_keyboard_shortcuts(&self, dialog: &mut SettingsDialog) {
        // Define default keyboard shortcuts
        let shortcuts = vec![
            KeyboardShortcut::new("file_open", "Open File", "Ctrl+O"),
            KeyboardShortcut::new("file_save", "Save File", "Ctrl+S"),
            KeyboardShortcut::new("file_exit", "Exit Application", "Ctrl+Q"),
            KeyboardShortcut::new("edit_undo", "Undo", "Ctrl+Z"),
            KeyboardShortcut::new("edit_redo", "Redo", "Ctrl+Y"),
            KeyboardShortcut::new("edit_cut", "Cut", "Ctrl+X"),
            KeyboardShortcut::new("edit_copy", "Copy", "Ctrl+C"),
            KeyboardShortcut::new("edit_paste", "Paste", "Ctrl+V"),
            KeyboardShortcut::new("edit_preferences", "Preferences", "Ctrl+,"),
            KeyboardShortcut::new("machine_connect", "Connect", "Alt+C"),
            KeyboardShortcut::new("machine_disconnect", "Disconnect", "Alt+D"),
            KeyboardShortcut::new("machine_home", "Home Machine", "Ctrl+H"),
            KeyboardShortcut::new("machine_reset", "Reset", "F5"),
        ];

        for shortcut in shortcuts {
            dialog.add_shortcut(shortcut);
        }
    }

    /// Update UI settings in config from dialog
    fn update_ui_settings(&mut self, dialog: &SettingsDialog) -> Result<()> {
        if let Some(setting) = dialog.get_setting("theme") {
            self.config.ui.theme = setting.value.as_str();
        }

        if let Some(setting) = dialog.get_setting("window_width") {
            if let Ok(width) = setting.value.as_str().parse::<u32>() {
                self.config.ui.window_width = width;
            }
        }

        if let Some(setting) = dialog.get_setting("window_height") {
            if let Ok(height) = setting.value.as_str().parse::<u32>() {
                self.config.ui.window_height = height;
            }
        }

        if let Some(setting) = dialog.get_setting("show_toolbar") {
            if let Ok(value) = setting.value.as_str().parse::<bool>() {
                self.config
                    .ui
                    .panel_visibility
                    .insert("toolbar".to_string(), value);
            }
        }

        if let Some(setting) = dialog.get_setting("show_status_bar") {
            if let Ok(value) = setting.value.as_str().parse::<bool>() {
                self.config
                    .ui
                    .panel_visibility
                    .insert("status_bar".to_string(), value);
            }
        }

        Ok(())
    }

    /// Update file processing settings in config from dialog
    fn update_file_processing_settings(&mut self, dialog: &SettingsDialog) -> Result<()> {
        if let Some(setting) = dialog.get_setting("preserve_comments") {
            if let Ok(value) = setting.value.as_str().parse::<bool>() {
                self.config.file_processing.preserve_comments = value;
            }
        }

        if let Some(setting) = dialog.get_setting("arc_segment_length") {
            if let Ok(value) = setting.value.as_str().parse::<f64>() {
                self.config.file_processing.arc_segment_length = value;
            }
        }

        if let Some(setting) = dialog.get_setting("max_line_length") {
            if let Ok(value) = setting.value.as_str().parse::<u32>() {
                self.config.file_processing.max_line_length = value;
            }
        }

        Ok(())
    }
}

impl Default for SettingsPersistence {
    fn default() -> Self {
        Self::new()
    }
}

