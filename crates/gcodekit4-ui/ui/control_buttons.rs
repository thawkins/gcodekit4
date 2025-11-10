//! Control Buttons Panel - Task 74
//!
//! Machine control buttons: Start, Pause, Stop, Home, Reset, Unlock, Alarm Clear

/// Button type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonType {
    /// Start/Run execution
    Start,
    /// Pause execution
    Pause,
    /// Stop execution
    Stop,
    /// Home machine (go to origin)
    Home,
    /// Reset machine
    Reset,
    /// Unlock machine
    Unlock,
    /// Clear alarm
    ClearAlarm,
}

impl ButtonType {
    /// Get button label
    pub fn label(&self) -> &str {
        match self {
            Self::Start => "START",
            Self::Pause => "PAUSE",
            Self::Stop => "STOP",
            Self::Home => "HOME",
            Self::Reset => "RESET",
            Self::Unlock => "UNLOCK",
            Self::ClearAlarm => "ALARM CLR",
        }
    }

    /// Get button description
    pub fn description(&self) -> &str {
        match self {
            Self::Start => "Start program execution",
            Self::Pause => "Pause program execution",
            Self::Stop => "Stop program execution",
            Self::Home => "Go to home position (G28)",
            Self::Reset => "Reset machine state",
            Self::Unlock => "Unlock machine for jogging",
            Self::ClearAlarm => "Clear alarm/error state",
        }
    }

    /// Get keyboard shortcut
    pub fn shortcut(&self) -> &str {
        match self {
            Self::Start => "Space",
            Self::Pause => "P",
            Self::Stop => "Esc",
            Self::Home => "H",
            Self::Reset => "R",
            Self::Unlock => "U",
            Self::ClearAlarm => "A",
        }
    }

    /// Get all button types
    pub fn all() -> Vec<Self> {
        vec![
            Self::Start,
            Self::Pause,
            Self::Stop,
            Self::Home,
            Self::Reset,
            Self::Unlock,
            Self::ClearAlarm,
        ]
    }
}

/// Button state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// Button is enabled
    Enabled,
    /// Button is disabled
    Disabled,
    /// Button is pressed/active
    Active,
    /// Button is in progress/loading
    Loading,
}

impl std::fmt::Display for ButtonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Enabled => write!(f, "enabled"),
            Self::Disabled => write!(f, "disabled"),
            Self::Active => write!(f, "active"),
            Self::Loading => write!(f, "loading"),
        }
    }
}

/// Control button definition
#[derive(Debug, Clone)]
pub struct ControlButton {
    /// Button type
    pub button_type: ButtonType,
    /// Current state
    pub state: ButtonState,
    /// Is pressed/held
    pub pressed: bool,
    /// Action command to send
    pub command: String,
}

impl ControlButton {
    /// Create new control button
    pub fn new(button_type: ButtonType) -> Self {
        let command = match button_type {
            ButtonType::Start => "CYCLE_START",
            ButtonType::Pause => "CYCLE_PAUSE",
            ButtonType::Stop => "CYCLE_STOP",
            ButtonType::Home => "$H",
            ButtonType::Reset => "CTRL+Z",
            ButtonType::Unlock => "$X",
            ButtonType::ClearAlarm => "$X",
        };

        Self {
            button_type,
            state: ButtonState::Enabled,
            pressed: false,
            command: command.to_string(),
        }
    }

    /// Get button label
    pub fn label(&self) -> &str {
        self.button_type.label()
    }

    /// Get button description
    pub fn description(&self) -> &str {
        self.button_type.description()
    }

    /// Get keyboard shortcut
    pub fn shortcut(&self) -> &str {
        self.button_type.shortcut()
    }

    /// Press the button
    pub fn press(&mut self) {
        if self.state == ButtonState::Enabled {
            self.pressed = true;
        }
    }

    /// Release the button
    pub fn release(&mut self) {
        self.pressed = false;
    }

    /// Enable button
    pub fn enable(&mut self) {
        if self.state != ButtonState::Active {
            self.state = ButtonState::Enabled;
        }
    }

    /// Disable button
    pub fn disable(&mut self) {
        self.state = ButtonState::Disabled;
    }

    /// Set button as active/loading
    pub fn set_loading(&mut self) {
        self.state = ButtonState::Loading;
    }

    /// Set button as active
    pub fn set_active(&mut self) {
        self.state = ButtonState::Active;
    }

    /// Is button enabled
    pub fn is_enabled(&self) -> bool {
        self.state == ButtonState::Enabled
    }

    /// Can button be clicked
    pub fn can_click(&self) -> bool {
        self.state == ButtonState::Enabled || self.state == ButtonState::Active
    }
}

/// Control buttons panel
#[derive(Debug)]
pub struct ControlButtonsPanel {
    /// Control buttons
    pub buttons: std::collections::HashMap<ButtonType, ControlButton>,
    /// Last pressed button
    pub last_pressed: Option<ButtonType>,
    /// Pending action commands
    pub pending_actions: Vec<String>,
}

impl ControlButtonsPanel {
    /// Create new control buttons panel
    pub fn new() -> Self {
        let mut buttons = std::collections::HashMap::new();

        for button_type in ButtonType::all() {
            buttons.insert(button_type, ControlButton::new(button_type));
        }

        Self {
            buttons,
            last_pressed: None,
            pending_actions: Vec::new(),
        }
    }

    /// Click button
    pub fn click_button(&mut self, button_type: ButtonType) -> Option<String> {
        if let Some(button) = self.buttons.get_mut(&button_type) {
            if button.can_click() {
                button.press();
                self.last_pressed = Some(button_type);
                let command = button.command.clone();
                self.pending_actions.push(command.clone());
                return Some(command);
            }
        }
        None
    }

    /// Release button
    pub fn release_button(&mut self, button_type: ButtonType) {
        if let Some(button) = self.buttons.get_mut(&button_type) {
            button.release();
        }
    }

    /// Handle keyboard input
    pub fn keyboard_input(&mut self, key: char) -> Option<String> {
        for button_type in ButtonType::all() {
            if let Some(button) = self.buttons.get(&button_type) {
                if button
                    .shortcut()
                    .to_lowercase()
                    .starts_with(key.to_lowercase().to_string().as_str())
                {
                    return self.click_button(button_type);
                }
            }
        }
        None
    }

    /// Get button by type
    pub fn get_button(&self, button_type: ButtonType) -> Option<&ControlButton> {
        self.buttons.get(&button_type)
    }

    /// Get mutable button by type
    pub fn get_button_mut(&mut self, button_type: ButtonType) -> Option<&mut ControlButton> {
        self.buttons.get_mut(&button_type)
    }

    /// Enable button
    pub fn enable_button(&mut self, button_type: ButtonType) {
        if let Some(button) = self.buttons.get_mut(&button_type) {
            button.enable();
        }
    }

    /// Disable button
    pub fn disable_button(&mut self, button_type: ButtonType) {
        if let Some(button) = self.buttons.get_mut(&button_type) {
            button.disable();
        }
    }

    /// Enable all buttons
    pub fn enable_all(&mut self) {
        for button in self.buttons.values_mut() {
            button.enable();
        }
    }

    /// Disable all buttons
    pub fn disable_all(&mut self) {
        for button in self.buttons.values_mut() {
            button.disable();
        }
    }

    /// Enable run controls (Start, Pause, Stop)
    pub fn enable_run_controls(&mut self) {
        self.enable_button(ButtonType::Start);
        self.enable_button(ButtonType::Pause);
        self.enable_button(ButtonType::Stop);
    }

    /// Disable run controls
    pub fn disable_run_controls(&mut self) {
        self.disable_button(ButtonType::Start);
        self.disable_button(ButtonType::Pause);
        self.disable_button(ButtonType::Stop);
    }

    /// Get next pending action
    pub fn next_action(&mut self) -> Option<String> {
        if self.pending_actions.is_empty() {
            None
        } else {
            Some(self.pending_actions.remove(0))
        }
    }

    /// Clear pending actions
    pub fn clear_pending(&mut self) {
        self.pending_actions.clear();
    }

    /// Get all button states
    pub fn get_button_states(&self) -> Vec<(ButtonType, ButtonState)> {
        self.buttons
            .iter()
            .map(|(bt, button)| (*bt, button.state))
            .collect()
    }

    /// Set button as loading/executing
    pub fn set_button_loading(&mut self, button_type: ButtonType) {
        if let Some(button) = self.buttons.get_mut(&button_type) {
            button.set_loading();
        }
    }

    /// Complete button action (set back to enabled)
    pub fn complete_button_action(&mut self, button_type: ButtonType) {
        if let Some(button) = self.buttons.get_mut(&button_type) {
            button.enable();
        }
    }

    /// Get all buttons as vector
    pub fn get_all_buttons(&self) -> Vec<&ControlButton> {
        ButtonType::all()
            .iter()
            .filter_map(|bt| self.buttons.get(bt))
            .collect()
    }
}

impl Default for ControlButtonsPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_type_label() {
        assert_eq!(ButtonType::Start.label(), "START");
        assert_eq!(ButtonType::Home.label(), "HOME");
    }

    #[test]
    fn test_button_type_shortcut() {
        assert_eq!(ButtonType::Start.shortcut(), "Space");
        assert_eq!(ButtonType::Stop.shortcut(), "Esc");
    }

    #[test]
    fn test_button_type_all() {
        let all = ButtonType::all();
        assert_eq!(all.len(), 7);
    }

    #[test]
    fn test_control_button_creation() {
        let button = ControlButton::new(ButtonType::Start);
        assert_eq!(button.label(), "START");
        assert!(button.is_enabled());
    }

    #[test]
    fn test_control_button_press() {
        let mut button = ControlButton::new(ButtonType::Start);
        button.press();
        assert!(button.pressed);
    }

    #[test]
    fn test_control_button_enable_disable() {
        let mut button = ControlButton::new(ButtonType::Start);
        button.disable();
        assert!(!button.is_enabled());
        button.enable();
        assert!(button.is_enabled());
    }

    #[test]
    fn test_control_button_press_when_disabled() {
        let mut button = ControlButton::new(ButtonType::Start);
        button.disable();
        button.press();
        assert!(!button.pressed);
    }

    #[test]
    fn test_panel_creation() {
        let panel = ControlButtonsPanel::new();
        assert_eq!(panel.buttons.len(), 7);
    }

    #[test]
    fn test_panel_click_button() {
        let mut panel = ControlButtonsPanel::new();
        let result = panel.click_button(ButtonType::Start);
        assert!(result.is_some());
    }

    #[test]
    fn test_panel_disabled_click() {
        let mut panel = ControlButtonsPanel::new();
        panel.disable_button(ButtonType::Start);
        let result = panel.click_button(ButtonType::Start);
        assert!(result.is_none());
    }

    #[test]
    fn test_panel_pending_actions() {
        let mut panel = ControlButtonsPanel::new();
        panel.click_button(ButtonType::Start);
        panel.click_button(ButtonType::Home);
        assert_eq!(panel.pending_actions.len(), 2);
    }

    #[test]
    fn test_panel_next_action() {
        let mut panel = ControlButtonsPanel::new();
        panel.click_button(ButtonType::Start);
        let action = panel.next_action();
        assert!(action.is_some());
        assert_eq!(panel.pending_actions.len(), 0);
    }

    #[test]
    fn test_panel_enable_all() {
        let mut panel = ControlButtonsPanel::new();
        panel.disable_all();
        panel.enable_all();
        assert!(panel.get_button(ButtonType::Start).unwrap().is_enabled());
    }

    #[test]
    fn test_panel_run_controls() {
        let mut panel = ControlButtonsPanel::new();
        panel.disable_all();
        panel.enable_run_controls();
        assert!(panel.get_button(ButtonType::Start).unwrap().is_enabled());
        assert!(panel.get_button(ButtonType::Pause).unwrap().is_enabled());
        assert!(panel.get_button(ButtonType::Stop).unwrap().is_enabled());
        assert!(!panel.get_button(ButtonType::Home).unwrap().is_enabled());
    }

    #[test]
    fn test_button_states() {
        let panel = ControlButtonsPanel::new();
        let states = panel.get_button_states();
        assert_eq!(states.len(), 7);
        assert!(states.iter().all(|(_, s)| *s == ButtonState::Enabled));
    }

    #[test]
    fn test_button_loading() {
        let mut panel = ControlButtonsPanel::new();
        panel.set_button_loading(ButtonType::Start);
        assert_eq!(
            panel.get_button(ButtonType::Start).unwrap().state,
            ButtonState::Loading
        );
    }
}
