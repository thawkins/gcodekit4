//! UI Architecture Setup - Task 66
//!
//! Defines the Slint component hierarchy, main window layout,
//! component communication patterns, and UI state management.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    /// Main application window
    MainWindow,
    /// Connection management panel
    ConnectionPanel,
    /// Controller state display (DRO)
    ControllerStatePanel,
    /// Manual jogging control
    JogControlPanel,
    /// File operations panel
    FileOperationsPanel,
    /// G-Code editor/viewer
    GCodeViewerPanel,
    /// Machine monitoring panel
    MachineMonitorPanel,
    /// Settings and configuration panel
    SettingsPanel,
    /// Macro management panel
    MacroPanel,
    /// Simulation mode panel
    SimulationPanel,
    /// 3D visualizer panel
    VisualizerPanel,
}

/// Component communication channel
#[derive(Debug, Clone)]
pub struct ComponentChannel {
    /// Source component
    pub source: ComponentType,
    /// Target component
    pub target: ComponentType,
    /// Channel name
    pub name: String,
}

/// UI Component Definition
#[derive(Debug, Clone)]
pub struct UiComponent {
    /// Component type
    pub component_type: ComponentType,
    /// Component name
    pub name: String,
    /// Is component visible
    pub visible: bool,
    /// Is component enabled
    pub enabled: bool,
    /// Child components
    pub children: Vec<String>,
}

impl UiComponent {
    /// Create a new UI component
    pub fn new(component_type: ComponentType, name: impl Into<String>) -> Self {
        Self {
            component_type,
            name: name.into(),
            visible: true,
            enabled: true,
            children: Vec::new(),
        }
    }

    /// Show component
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide component
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Enable component
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable component
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

/// UI Architecture - manages the complete component hierarchy
#[derive(Debug)]
pub struct UiArchitecture {
    /// Root component (main window)
    root: UiComponent,
    /// All components in hierarchy
    components: std::collections::HashMap<String, UiComponent>,
    /// Component communication channels
    channels: Vec<ComponentChannel>,
}

impl UiArchitecture {
    /// Create new UI architecture
    pub fn new() -> Self {
        let mut arch = Self {
            root: UiComponent::new(ComponentType::MainWindow, "MainWindow"),
            components: std::collections::HashMap::new(),
            channels: Vec::new(),
        };

        // Initialize default component hierarchy
        arch.initialize_hierarchy();

        arch
    }

    /// Initialize component hierarchy
    fn initialize_hierarchy(&mut self) {
        // Main window is root
        self.components
            .insert("MainWindow".to_string(), self.root.clone());

        // Add main panels
        let panels = vec![
            ("ConnectionPanel", ComponentType::ConnectionPanel),
            ("ControllerStatePanel", ComponentType::ControllerStatePanel),
            ("JogControlPanel", ComponentType::JogControlPanel),
            ("FileOperationsPanel", ComponentType::FileOperationsPanel),
            ("GCodeViewerPanel", ComponentType::GCodeViewerPanel),
            ("MachineMonitorPanel", ComponentType::MachineMonitorPanel),
            ("SettingsPanel", ComponentType::SettingsPanel),
            ("MacroPanel", ComponentType::MacroPanel),
            ("SimulationPanel", ComponentType::SimulationPanel),
            ("VisualizerPanel", ComponentType::VisualizerPanel),
        ];

        for (name, comp_type) in panels {
            let component = UiComponent::new(comp_type, name);
            self.components.insert(name.to_string(), component);
            self.root.children.push(name.to_string());
        }
    }

    /// Register a communication channel between components
    pub fn register_channel(
        &mut self,
        source: ComponentType,
        target: ComponentType,
        name: impl Into<String>,
    ) {
        let channel = ComponentChannel {
            source,
            target,
            name: name.into(),
        };
        self.channels.push(channel);
    }

    /// Get component by type
    pub fn get_component(&self, comp_type: ComponentType) -> Option<&UiComponent> {
        self.components
            .values()
            .find(|c| c.component_type == comp_type)
    }

    /// Get mutable component by type
    pub fn get_component_mut(&mut self, comp_type: ComponentType) -> Option<&mut UiComponent> {
        self.components
            .values_mut()
            .find(|c| c.component_type == comp_type)
    }

    /// Get all components
    pub fn components(&self) -> &std::collections::HashMap<String, UiComponent> {
        &self.components
    }

    /// Get communication channels
    pub fn channels(&self) -> &[ComponentChannel] {
        &self.channels
    }

    /// Get root component
    pub fn root(&self) -> &UiComponent {
        &self.root
    }

    /// Layout configuration
    pub fn layout_config(&self) -> LayoutConfig {
        LayoutConfig {
            main_window_width: 1280,
            main_window_height: 960,
            panel_layout: PanelLayout::GridLayout,
            theme: UiTheme::Dark,
        }
    }
}

impl Default for UiArchitecture {
    fn default() -> Self {
        Self::new()
    }
}

/// Layout configuration for the UI
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Main window width in pixels
    pub main_window_width: u32,
    /// Main window height in pixels
    pub main_window_height: u32,
    /// Panel layout style
    pub panel_layout: PanelLayout,
    /// UI theme
    pub theme: UiTheme,
}

/// Panel layout style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelLayout {
    /// Grid-based layout
    GridLayout,
    /// Tabbed layout
    TabbedLayout,
    /// Docking layout
    DockingLayout,
}

/// UI theme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTheme {
    /// Dark theme
    Dark,
    /// Light theme
    Light,
    /// High contrast theme
    HighContrast,
}

/// Component communication pattern
#[derive(Debug, Clone)]
pub enum CommunicationPattern {
    /// Parent to child communication
    ParentToChild,
    /// Child to parent communication
    ChildToParent,
    /// Sibling to sibling communication
    SiblingToSibling,
    /// Broadcast to all components
    Broadcast,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_architecture_creation() {
        let arch = UiArchitecture::new();
        assert_eq!(arch.root().component_type, ComponentType::MainWindow);
    }

    #[test]
    fn test_component_initialization() {
        let arch = UiArchitecture::new();
        assert!(arch.root().children.len() >= 10);
    }

    #[test]
    fn test_component_visibility() {
        let mut component = UiComponent::new(ComponentType::ConnectionPanel, "Test");
        assert!(component.visible);
        component.hide();
        assert!(!component.visible);
        component.show();
        assert!(component.visible);
    }

    #[test]
    fn test_channel_registration() {
        let mut arch = UiArchitecture::new();
        arch.register_channel(
            ComponentType::ConnectionPanel,
            ComponentType::ControllerStatePanel,
            "status_update",
        );
        assert_eq!(arch.channels().len(), 1);
    }

    #[test]
    fn test_layout_config() {
        let arch = UiArchitecture::new();
        let config = arch.layout_config();
        assert_eq!(config.main_window_width, 1280);
        assert_eq!(config.main_window_height, 960);
    }
}
