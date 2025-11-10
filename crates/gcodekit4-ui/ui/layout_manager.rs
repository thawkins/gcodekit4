//! Responsive layout and panel management system
//!
//! Handles resizable panels, show/hide toggling, and layout persistence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Panel identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelId {
    Connection,
    DRO,
    Jog,
    Editor,
    Console,
    Visualizer,
    Overrides,
    Macros,
    Settings,
}

impl std::fmt::Display for PanelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PanelId::Connection => write!(f, "Connection"),
            PanelId::DRO => write!(f, "DRO"),
            PanelId::Jog => write!(f, "Jog"),
            PanelId::Editor => write!(f, "G-Code Editor"),
            PanelId::Console => write!(f, "Console"),
            PanelId::Visualizer => write!(f, "Visualizer"),
            PanelId::Overrides => write!(f, "Overrides"),
            PanelId::Macros => write!(f, "Macros"),
            PanelId::Settings => write!(f, "Settings"),
        }
    }
}

/// Panel location in layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanelLocation {
    Left,
    Center,
    Right,
    Bottom,
}

/// Panel state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelState {
    /// Panel ID
    pub id: PanelId,
    /// Whether panel is visible
    pub visible: bool,
    /// Panel location
    pub location: PanelLocation,
    /// Panel width (in pixels or percentage)
    pub width: f32,
    /// Panel height (in pixels or percentage)
    pub height: f32,
    /// Is docked (vs floating)
    pub is_docked: bool,
    /// Z-order for floating panels
    pub z_order: u32,
}

impl PanelState {
    /// Create new panel state
    pub fn new(id: PanelId, location: PanelLocation) -> Self {
        Self {
            id,
            visible: true,
            location,
            width: 300.0,
            height: 400.0,
            is_docked: true,
            z_order: 0,
        }
    }

    /// Set panel width
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set panel height
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Toggle visibility
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Make panel floating
    pub fn make_floating(&mut self, z_order: u32) {
        self.is_docked = false;
        self.z_order = z_order;
    }

    /// Make panel docked
    pub fn make_docked(&mut self, location: PanelLocation) {
        self.is_docked = true;
        self.location = location;
        self.z_order = 0;
    }
}

/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    /// Name of layout
    pub name: String,
    /// Panel states
    pub panels: HashMap<PanelId, PanelState>,
    /// Layout description
    pub description: String,
}

impl Layout {
    /// Create new layout
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            panels: HashMap::new(),
            description: String::new(),
        }
    }

    /// Add panel to layout
    pub fn with_panel(mut self, panel: PanelState) -> Self {
        self.panels.insert(panel.id, panel);
        self
    }

    /// Set layout description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Get default layout (workbench style)
    pub fn default_workbench() -> Self {
        let mut layout = Self::new("Workbench");
        layout.description = "Optimized for machine operation".to_string();

        layout.panels.insert(
            PanelId::Connection,
            PanelState::new(PanelId::Connection, PanelLocation::Left)
                .with_width(300.0)
                .with_height(150.0),
        );

        layout.panels.insert(
            PanelId::DRO,
            PanelState::new(PanelId::DRO, PanelLocation::Left)
                .with_width(300.0)
                .with_height(200.0),
        );

        layout.panels.insert(
            PanelId::Jog,
            PanelState::new(PanelId::Jog, PanelLocation::Left)
                .with_width(300.0)
                .with_height(250.0),
        );

        layout.panels.insert(
            PanelId::Visualizer,
            PanelState::new(PanelId::Visualizer, PanelLocation::Center)
                .with_width(800.0)
                .with_height(600.0),
        );

        layout.panels.insert(
            PanelId::Overrides,
            PanelState::new(PanelId::Overrides, PanelLocation::Right)
                .with_width(250.0)
                .with_height(200.0),
        );

        layout.panels.insert(
            PanelId::Console,
            PanelState::new(PanelId::Console, PanelLocation::Bottom)
                .with_width(1024.0)
                .with_height(150.0),
        );

        layout
    }

    /// Get programming layout
    pub fn programming() -> Self {
        let mut layout = Self::new("Programming");
        layout.description = "Optimized for G-code editing".to_string();

        layout.panels.insert(
            PanelId::Editor,
            PanelState::new(PanelId::Editor, PanelLocation::Center)
                .with_width(800.0)
                .with_height(600.0),
        );

        layout.panels.insert(
            PanelId::Visualizer,
            PanelState::new(PanelId::Visualizer, PanelLocation::Right)
                .with_width(400.0)
                .with_height(600.0),
        );

        layout.panels.insert(
            PanelId::Console,
            PanelState::new(PanelId::Console, PanelLocation::Bottom)
                .with_width(1024.0)
                .with_height(150.0),
        );

        layout
    }

    /// Get monitoring layout
    pub fn monitoring() -> Self {
        let mut layout = Self::new("Monitoring");
        layout.description = "Optimized for machine status monitoring".to_string();

        layout.panels.insert(
            PanelId::DRO,
            PanelState::new(PanelId::DRO, PanelLocation::Left)
                .with_width(350.0)
                .with_height(250.0),
        );

        layout.panels.insert(
            PanelId::Visualizer,
            PanelState::new(PanelId::Visualizer, PanelLocation::Center)
                .with_width(800.0)
                .with_height(600.0),
        );

        layout
    }
}

/// Responsive layout manager
pub struct LayoutManager {
    current_layout: Layout,
    saved_layouts: HashMap<String, Layout>,
    next_z_order: u32,
}

impl LayoutManager {
    /// Create new layout manager
    pub fn new() -> Self {
        Self {
            current_layout: Layout::default_workbench(),
            saved_layouts: HashMap::new(),
            next_z_order: 1,
        }
    }

    /// Get current layout
    pub fn current(&self) -> &Layout {
        &self.current_layout
    }

    /// Get current layout (mutable)
    pub fn current_mut(&mut self) -> &mut Layout {
        &mut self.current_layout
    }

    /// Load layout by name
    pub fn load_layout(&mut self, name: &str) -> bool {
        if let Some(layout) = self.saved_layouts.get(name) {
            self.current_layout = layout.clone();
            true
        } else {
            false
        }
    }

    /// Save current layout
    pub fn save_current(&mut self, name: impl Into<String>) {
        let name_str = name.into();
        self.saved_layouts
            .insert(name_str, self.current_layout.clone());
    }

    /// Get saved layout names
    pub fn saved_names(&self) -> Vec<&str> {
        self.saved_layouts.keys().map(|s| s.as_str()).collect()
    }

    /// Toggle panel visibility
    pub fn toggle_panel(&mut self, panel_id: PanelId) {
        if let Some(panel) = self.current_layout.panels.get_mut(&panel_id) {
            panel.toggle_visibility();
        }
    }

    /// Set panel visibility
    pub fn set_panel_visible(&mut self, panel_id: PanelId, visible: bool) {
        if let Some(panel) = self.current_layout.panels.get_mut(&panel_id) {
            panel.set_visible(visible);
        }
    }

    /// Get panel state
    pub fn get_panel(&self, panel_id: PanelId) -> Option<&PanelState> {
        self.current_layout.panels.get(&panel_id)
    }

    /// Get panel state (mutable)
    pub fn get_panel_mut(&mut self, panel_id: PanelId) -> Option<&mut PanelState> {
        self.current_layout.panels.get_mut(&panel_id)
    }

    /// Move panel to location
    pub fn move_panel(&mut self, panel_id: PanelId, location: PanelLocation) {
        if let Some(panel) = self.current_layout.panels.get_mut(&panel_id) {
            panel.location = location;
            panel.is_docked = true;
        }
    }

    /// Make panel floating
    pub fn float_panel(&mut self, panel_id: PanelId) {
        if let Some(panel) = self.current_layout.panels.get_mut(&panel_id) {
            panel.make_floating(self.next_z_order);
            self.next_z_order += 1;
        }
    }

    /// Bring panel to front (floating)
    pub fn bring_to_front(&mut self, panel_id: PanelId) {
        if let Some(panel) = self.current_layout.panels.get_mut(&panel_id) {
            if !panel.is_docked {
                panel.z_order = self.next_z_order;
                self.next_z_order += 1;
            }
        }
    }

    /// Resize panel
    pub fn resize_panel(&mut self, panel_id: PanelId, width: f32, height: f32) {
        if let Some(panel) = self.current_layout.panels.get_mut(&panel_id) {
            panel.width = width;
            panel.height = height;
        }
    }

    /// Reset to default workbench layout
    pub fn reset_to_default(&mut self) {
        self.current_layout = Layout::default_workbench();
        self.next_z_order = 1;
    }

    /// Load preset layout
    pub fn load_preset(&mut self, preset: LayoutPreset) {
        self.current_layout = match preset {
            LayoutPreset::Workbench => Layout::default_workbench(),
            LayoutPreset::Programming => Layout::programming(),
            LayoutPreset::Monitoring => Layout::monitoring(),
        };
        self.next_z_order = 1;
    }

    /// Get all visible panels
    pub fn visible_panels(&self) -> Vec<&PanelState> {
        self.current_layout
            .panels
            .values()
            .filter(|p| p.visible)
            .collect()
    }

    /// Get panels by location
    pub fn panels_at_location(&self, location: PanelLocation) -> Vec<&PanelState> {
        self.current_layout
            .panels
            .values()
            .filter(|p| p.location == location && p.visible)
            .collect()
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Preset layouts
#[derive(Debug, Clone, Copy)]
pub enum LayoutPreset {
    Workbench,
    Programming,
    Monitoring,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_state_creation() {
        let panel = PanelState::new(PanelId::Connection, PanelLocation::Left);
        assert!(panel.visible);
        assert_eq!(panel.id, PanelId::Connection);
    }

    #[test]
    fn test_panel_visibility_toggle() {
        let mut panel = PanelState::new(PanelId::Connection, PanelLocation::Left);
        assert!(panel.visible);
        panel.toggle_visibility();
        assert!(!panel.visible);
    }

    #[test]
    fn test_layout_creation() {
        let layout = Layout::new("Test");
        assert_eq!(layout.name, "Test");
        assert!(layout.panels.is_empty());
    }

    #[test]
    fn test_default_workbench_layout() {
        let layout = Layout::default_workbench();
        assert!(!layout.panels.is_empty());
        assert!(layout.panels.contains_key(&PanelId::Connection));
    }

    #[test]
    fn test_layout_manager_creation() {
        let mgr = LayoutManager::new();
        assert!(!mgr.current().panels.is_empty());
    }

    #[test]
    fn test_toggle_panel_visibility() {
        let mut mgr = LayoutManager::new();
        mgr.toggle_panel(PanelId::Connection);
        assert!(!mgr.get_panel(PanelId::Connection).unwrap().visible);
    }

    #[test]
    fn test_save_and_load_layout() {
        let mut mgr = LayoutManager::new();
        let original_name = mgr.current().name.clone();
        mgr.save_current("my_custom_layout");

        mgr.load_preset(LayoutPreset::Programming);
        assert_ne!(mgr.current().name, original_name);

        mgr.load_layout("my_custom_layout");
        assert_eq!(mgr.current().name, original_name);
    }

    #[test]
    fn test_preset_layouts() {
        let mut mgr = LayoutManager::new();
        mgr.load_preset(LayoutPreset::Programming);
        assert!(mgr.get_panel(PanelId::Editor).is_some());
    }

    #[test]
    fn test_float_panel() {
        let mut mgr = LayoutManager::new();
        mgr.float_panel(PanelId::Connection);
        let panel = mgr.get_panel(PanelId::Connection).unwrap();
        assert!(!panel.is_docked);
        assert_eq!(panel.z_order, 1);
    }
}
