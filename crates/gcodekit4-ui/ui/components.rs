//! UI Components - Slint component definitions
//!
//! Defines reusable UI components for the application

use crate::ui::architecture::ComponentType;

/// Base trait for all UI components
pub trait UiComponentTrait {
    /// Component type
    fn component_type(&self) -> ComponentType;

    /// Render component
    fn render(&self) -> String;

    /// Update component state
    fn update(&mut self, data: &str) -> anyhow::Result<()>;

    /// Get component name
    fn name(&self) -> &str;
}

/// Button component
#[derive(Debug, Clone)]
pub struct ButtonComponent {
    /// Button label
    pub label: String,
    /// Button is enabled
    pub enabled: bool,
    /// On click handler
    pub on_click: Option<String>,
}

impl ButtonComponent {
    /// Create new button
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
            on_click: None,
        }
    }

    /// Set click handler
    pub fn on_click(mut self, handler: impl Into<String>) -> Self {
        self.on_click = Some(handler.into());
        self
    }
}

/// Text input component
#[derive(Debug, Clone)]
pub struct TextInputComponent {
    /// Input value
    pub value: String,
    /// Input placeholder
    pub placeholder: String,
    /// Is editable
    pub editable: bool,
}

impl TextInputComponent {
    /// Create new text input
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            placeholder: placeholder.into(),
            editable: true,
        }
    }
}

/// Dropdown/Combo box component
#[derive(Debug, Clone)]
pub struct DropdownComponent {
    /// Available options
    pub options: Vec<String>,
    /// Selected option index
    pub selected_index: usize,
}

impl DropdownComponent {
    /// Create new dropdown
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            selected_index: 0,
        }
    }

    /// Get selected value
    pub fn selected_value(&self) -> Option<&str> {
        self.options.get(self.selected_index).map(|s| s.as_str())
    }
}

/// Label/Text component
#[derive(Debug, Clone)]
pub struct LabelComponent {
    /// Label text
    pub text: String,
    /// Text alignment: "left", "center", "right"
    pub alignment: String,
}

impl LabelComponent {
    /// Create new label
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            alignment: "left".to_string(),
        }
    }
}

/// Toggle/Checkbox component
#[derive(Debug, Clone)]
pub struct ToggleComponent {
    /// Toggle state
    pub checked: bool,
    /// Toggle label
    pub label: String,
}

impl ToggleComponent {
    /// Create new toggle
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            checked: false,
            label: label.into(),
        }
    }
}

/// Slider component
#[derive(Debug, Clone)]
pub struct SliderComponent {
    /// Current value
    pub value: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Step size
    pub step: f64,
}

impl SliderComponent {
    /// Create new slider
    pub fn new(min: f64, max: f64, step: f64) -> Self {
        Self {
            value: (min + max) / 2.0,
            min,
            max,
            step,
        }
    }
}

/// Gauge/Meter component
#[derive(Debug, Clone)]
pub struct GaugeComponent {
    /// Current value
    pub value: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Gauge label
    pub label: String,
}

impl GaugeComponent {
    /// Create new gauge
    pub fn new(min: f64, max: f64, label: impl Into<String>) -> Self {
        Self {
            value: (min + max) / 2.0,
            min,
            max,
            label: label.into(),
        }
    }
}

/// Status indicator component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusIndicator {
    /// Idle/disconnected status
    Idle,
    /// Running/active status
    Running,
    /// Error/warning status
    Error,
    /// Success/ok status
    Success,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let btn = ButtonComponent::new("Click Me");
        assert_eq!(btn.label, "Click Me");
        assert!(btn.enabled);
    }

    #[test]
    fn test_text_input() {
        let input = TextInputComponent::new("Enter text");
        assert_eq!(input.placeholder, "Enter text");
        assert!(input.editable);
    }

    #[test]
    fn test_dropdown() {
        let options = vec!["Option1".to_string(), "Option2".to_string()];
        let dropdown = DropdownComponent::new(options);
        assert_eq!(dropdown.selected_value(), Some("Option1"));
    }

    #[test]
    fn test_slider() {
        let slider = SliderComponent::new(0.0, 100.0, 1.0);
        assert_eq!(slider.value, 50.0);
        assert_eq!(slider.min, 0.0);
        assert_eq!(slider.max, 100.0);
    }
}
