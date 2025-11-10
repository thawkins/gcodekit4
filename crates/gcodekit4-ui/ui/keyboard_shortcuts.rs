//! Keyboard shortcuts management and handling
//!
//! Centralized system for global keyboard shortcuts and actions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Keyboard modifier flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyModifiers {
    /// Create new modifiers
    pub fn new(ctrl: bool, shift: bool, alt: bool, meta: bool) -> Self {
        Self {
            ctrl,
            shift,
            alt,
            meta,
        }
    }

    /// No modifiers
    pub fn none() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        }
    }
}

/// Keyboard action types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyboardAction {
    // File operations
    OpenFile,
    SaveFile,
    ExitApplication,

    // Machine control
    HomeAll,
    SoftReset,
    KillAlarmLock,
    CheckMode,

    // Streaming control
    StartStream,
    PauseStream,
    StopStream,

    // Jogging
    JogXPositive,
    JogXNegative,
    JogYPositive,
    JogYNegative,
    JogZPositive,
    JogZNegative,

    // View controls
    FitView,
    TopView,
    FrontView,
    SideView,

    // UI
    TogglePanel,
    ShowSettings,
    ShowMacros,
    ShowHelp,

    // Custom action
    Custom(String),
}

/// Keyboard binding entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    /// Action to perform
    pub action: KeyboardAction,
    /// Key name (e.g., "a", "F1", "Return")
    pub key: String,
    /// Modifiers (Ctrl, Shift, Alt, Meta)
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    /// Create new key binding
    pub fn new(action: KeyboardAction, key: impl Into<String>, modifiers: KeyModifiers) -> Self {
        Self {
            action,
            key: key.into(),
            modifiers,
        }
    }

    /// Get display string for binding
    pub fn display(&self) -> String {
        let mut parts = Vec::new();

        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        if self.modifiers.meta {
            parts.push("Meta");
        }

        parts.push(&self.key);
        parts.join("+")
    }
}

/// Keyboard shortcuts manager
pub struct KeyboardManager {
    bindings: HashMap<String, KeyBinding>,
    custom_bindings: HashMap<String, KeyBinding>,
}

impl KeyboardManager {
    /// Create new keyboard manager with default bindings
    pub fn new() -> Self {
        let mut mgr = Self {
            bindings: HashMap::new(),
            custom_bindings: HashMap::new(),
        };
        mgr.setup_defaults();
        mgr
    }

    /// Setup default key bindings
    fn setup_defaults(&mut self) {
        // File operations
        self.add_default(
            KeyboardAction::OpenFile,
            "o",
            KeyModifiers::new(true, false, false, false),
        );
        self.add_default(
            KeyboardAction::SaveFile,
            "s",
            KeyModifiers::new(true, false, false, false),
        );
        self.add_default(
            KeyboardAction::ExitApplication,
            "q",
            KeyModifiers::new(true, false, false, false),
        );

        // Machine control
        self.add_default(
            KeyboardAction::HomeAll,
            "h",
            KeyModifiers::new(true, false, false, false),
        );
        self.add_default(
            KeyboardAction::SoftReset,
            "r",
            KeyModifiers::new(true, false, false, false),
        );

        // Streaming control
        self.add_default(KeyboardAction::PauseStream, "Space", KeyModifiers::none());
        self.add_default(KeyboardAction::StopStream, "Escape", KeyModifiers::none());

        // Jogging
        self.add_default(KeyboardAction::JogXPositive, "d", KeyModifiers::none());
        self.add_default(KeyboardAction::JogXNegative, "a", KeyModifiers::none());
        self.add_default(KeyboardAction::JogYPositive, "w", KeyModifiers::none());
        self.add_default(KeyboardAction::JogYNegative, "s", KeyModifiers::none());
        self.add_default(KeyboardAction::JogZPositive, "q", KeyModifiers::none());
        self.add_default(KeyboardAction::JogZNegative, "z", KeyModifiers::none());

        // View controls
        self.add_default(KeyboardAction::FitView, "0", KeyModifiers::none());
    }

    /// Add default binding
    fn add_default(&mut self, action: KeyboardAction, key: &str, modifiers: KeyModifiers) {
        let binding_key = format!("{}_{}", action_key(&action), key);
        self.bindings
            .insert(binding_key, KeyBinding::new(action, key, modifiers));
    }

    /// Add custom binding (overrides default)
    pub fn set_custom(&mut self, binding: KeyBinding) {
        let binding_key = format!("{}_{}", action_key(&binding.action), &binding.key);
        self.custom_bindings.insert(binding_key, binding);
    }

    /// Get binding for action
    pub fn get_binding(&self, action: &KeyboardAction) -> Option<KeyBinding> {
        // Check custom bindings first
        for binding in self.custom_bindings.values() {
            if &binding.action == action {
                return Some(binding.clone());
            }
        }

        // Fall back to defaults
        for binding in self.bindings.values() {
            if &binding.action == action {
                return Some(binding.clone());
            }
        }

        None
    }

    /// Get action for key combination
    pub fn get_action(&self, key: &str, modifiers: KeyModifiers) -> Option<KeyboardAction> {
        // Check custom bindings first
        for binding in self.custom_bindings.values() {
            if binding.key == key && binding.modifiers == modifiers {
                return Some(binding.action.clone());
            }
        }

        // Fall back to defaults
        for binding in self.bindings.values() {
            if binding.key == key && binding.modifiers == modifiers {
                return Some(binding.action.clone());
            }
        }

        None
    }

    /// Get all bindings
    pub fn get_all_bindings(&self) -> Vec<KeyBinding> {
        let mut bindings: Vec<_> = self.bindings.values().cloned().collect();
        for binding in self.custom_bindings.values() {
            // Remove default if overridden
            bindings.retain(|b| !(b.action == binding.action));
            bindings.push(binding.clone());
        }
        bindings.sort_by(|a, b| a.action.to_string().cmp(&b.action.to_string()));
        bindings
    }

    /// Reset to defaults
    pub fn reset_to_defaults(&mut self) {
        self.custom_bindings.clear();
    }

    /// Get custom bindings count
    pub fn custom_count(&self) -> usize {
        self.custom_bindings.len()
    }
}

impl Default for KeyboardManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to convert action to string key
fn action_key(action: &KeyboardAction) -> String {
    match action {
        KeyboardAction::OpenFile => "open_file".to_string(),
        KeyboardAction::SaveFile => "save_file".to_string(),
        KeyboardAction::ExitApplication => "exit_app".to_string(),
        KeyboardAction::HomeAll => "home_all".to_string(),
        KeyboardAction::SoftReset => "soft_reset".to_string(),
        KeyboardAction::KillAlarmLock => "kill_alarm".to_string(),
        KeyboardAction::CheckMode => "check_mode".to_string(),
        KeyboardAction::StartStream => "start_stream".to_string(),
        KeyboardAction::PauseStream => "pause_stream".to_string(),
        KeyboardAction::StopStream => "stop_stream".to_string(),
        KeyboardAction::JogXPositive => "jog_x_pos".to_string(),
        KeyboardAction::JogXNegative => "jog_x_neg".to_string(),
        KeyboardAction::JogYPositive => "jog_y_pos".to_string(),
        KeyboardAction::JogYNegative => "jog_y_neg".to_string(),
        KeyboardAction::JogZPositive => "jog_z_pos".to_string(),
        KeyboardAction::JogZNegative => "jog_z_neg".to_string(),
        KeyboardAction::FitView => "fit_view".to_string(),
        KeyboardAction::TopView => "top_view".to_string(),
        KeyboardAction::FrontView => "front_view".to_string(),
        KeyboardAction::SideView => "side_view".to_string(),
        KeyboardAction::TogglePanel => "toggle_panel".to_string(),
        KeyboardAction::ShowSettings => "show_settings".to_string(),
        KeyboardAction::ShowMacros => "show_macros".to_string(),
        KeyboardAction::ShowHelp => "show_help".to_string(),
        KeyboardAction::Custom(s) => format!("custom_{}", s),
    }
}

impl std::fmt::Display for KeyboardAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyboardAction::OpenFile => write!(f, "Open File"),
            KeyboardAction::SaveFile => write!(f, "Save File"),
            KeyboardAction::ExitApplication => write!(f, "Exit"),
            KeyboardAction::HomeAll => write!(f, "Home All Axes"),
            KeyboardAction::SoftReset => write!(f, "Soft Reset"),
            KeyboardAction::KillAlarmLock => write!(f, "Kill Alarm Lock"),
            KeyboardAction::CheckMode => write!(f, "Check Mode"),
            KeyboardAction::StartStream => write!(f, "Start Stream"),
            KeyboardAction::PauseStream => write!(f, "Pause/Resume Stream"),
            KeyboardAction::StopStream => write!(f, "Stop Stream"),
            KeyboardAction::JogXPositive => write!(f, "Jog +X"),
            KeyboardAction::JogXNegative => write!(f, "Jog -X"),
            KeyboardAction::JogYPositive => write!(f, "Jog +Y"),
            KeyboardAction::JogYNegative => write!(f, "Jog -Y"),
            KeyboardAction::JogZPositive => write!(f, "Jog +Z"),
            KeyboardAction::JogZNegative => write!(f, "Jog -Z"),
            KeyboardAction::FitView => write!(f, "Fit View"),
            KeyboardAction::TopView => write!(f, "Top View"),
            KeyboardAction::FrontView => write!(f, "Front View"),
            KeyboardAction::SideView => write!(f, "Side View"),
            KeyboardAction::TogglePanel => write!(f, "Toggle Panel"),
            KeyboardAction::ShowSettings => write!(f, "Show Settings"),
            KeyboardAction::ShowMacros => write!(f, "Show Macros"),
            KeyboardAction::ShowHelp => write!(f, "Show Help"),
            KeyboardAction::Custom(s) => write!(f, "Custom: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_modifiers() {
        let mods = KeyModifiers::new(true, false, false, false);
        assert!(mods.ctrl);
        assert!(!mods.shift);
    }

    #[test]
    fn test_key_binding_display() {
        let binding = KeyBinding::new(
            KeyboardAction::SaveFile,
            "s",
            KeyModifiers::new(true, false, false, false),
        );
        assert_eq!(binding.display(), "Ctrl+s");
    }

    #[test]
    fn test_keyboard_manager_defaults() {
        let mgr = KeyboardManager::new();
        let binding = mgr.get_binding(&KeyboardAction::SaveFile);
        assert!(binding.is_some());
    }

    #[test]
    fn test_get_action_by_key() {
        let mgr = KeyboardManager::new();
        let action = mgr.get_action("q", KeyModifiers::new(true, false, false, false));
        assert_eq!(action, Some(KeyboardAction::ExitApplication));
    }

    #[test]
    fn test_custom_binding() {
        let mut mgr = KeyboardManager::new();
        let binding = KeyBinding::new(
            KeyboardAction::SaveFile,
            "s",
            KeyModifiers::new(true, true, false, false),
        );
        mgr.set_custom(binding.clone());
        assert_eq!(mgr.custom_count(), 1);
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut mgr = KeyboardManager::new();
        mgr.set_custom(KeyBinding::new(
            KeyboardAction::SaveFile,
            "s",
            KeyModifiers::new(true, true, false, false),
        ));
        assert_eq!(mgr.custom_count(), 1);

        mgr.reset_to_defaults();
        assert_eq!(mgr.custom_count(), 0);
    }
}
