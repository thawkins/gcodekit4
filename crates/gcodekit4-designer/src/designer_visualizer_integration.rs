//! Designer to Visualizer integration module
//!
//! Handles seamless integration between the Designer tool and the G-code Visualizer,
//! including toolpath visualization, real-time updates, and simulation preview.

use serde::{Deserialize, Serialize};

/// Represents a design view in the visualizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignVisualization {
    /// Unique ID
    pub id: String,
    /// Design name
    pub name: String,
    /// Design bounds
    pub bounds: VisualizationBounds,
    /// Material preview settings
    pub material_settings: MaterialSettings,
    /// Toolpath visibility
    pub show_toolpath: bool,
    /// Design shapes visibility
    pub show_shapes: bool,
    /// Real-time update enabled
    pub real_time_updates: bool,
}

impl DesignVisualization {
    /// Create new design visualization
    pub fn new(name: String, bounds: VisualizationBounds) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            bounds,
            material_settings: MaterialSettings::default(),
            show_toolpath: true,
            show_shapes: true,
            real_time_updates: false,
        }
    }
}

/// Bounds for visualization
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VisualizationBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl VisualizationBounds {
    /// Create new bounds
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            min_z: min_z.min(max_z),
            max_x: min_x.max(max_x),
            max_y: min_y.max(max_y),
            max_z: min_z.max(max_z),
        }
    }

    /// Get center point
    pub fn center(&self) -> (f64, f64, f64) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
            (self.min_z + self.max_z) / 2.0,
        )
    }

    /// Get dimensions
    pub fn dimensions(&self) -> (f64, f64, f64) {
        (
            self.max_x - self.min_x,
            self.max_y - self.min_y,
            self.max_z - self.min_z,
        )
    }
}

impl Default for VisualizationBounds {
    fn default() -> Self {
        Self::new(-100.0, -100.0, -10.0, 100.0, 100.0, 10.0)
    }
}

/// Material preview settings for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialSettings {
    /// Material removal animation enabled
    pub show_material_removal: bool,
    /// Material color (RGB)
    pub material_color: (f32, f32, f32),
    /// Opacity (0.0-1.0)
    pub opacity: f32,
    /// Show material as solid or wireframe
    pub solid_view: bool,
    /// Animation speed (1.0 = normal)
    pub animation_speed: f32,
}

impl Default for MaterialSettings {
    fn default() -> Self {
        Self {
            show_material_removal: true,
            material_color: (0.8, 0.7, 0.5), // Wood color
            opacity: 0.9,
            solid_view: true,
            animation_speed: 1.0,
        }
    }
}

/// Toolpath visualization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolpathViewSettings {
    /// Show toolpath line
    pub show_toolpath: bool,
    /// Toolpath color (RGB)
    pub toolpath_color: (f32, f32, f32),
    /// Toolpath thickness
    pub line_thickness: f32,
    /// Show tool marker
    pub show_tool_marker: bool,
    /// Show cutting forces
    pub show_forces: bool,
    /// Show estimated time
    pub show_time_estimate: bool,
}

impl Default for ToolpathViewSettings {
    fn default() -> Self {
        Self {
            show_toolpath: true,
            toolpath_color: (1.0, 0.0, 0.0), // Red
            line_thickness: 2.0,
            show_tool_marker: true,
            show_forces: false,
            show_time_estimate: true,
        }
    }
}

/// Manages Designer to Visualizer integration
pub struct DesignerVisualizerIntegration {
    /// Current design visualization
    pub current_view: Option<DesignVisualization>,
    /// Toolpath view settings
    pub toolpath_settings: ToolpathViewSettings,
    /// Simulation state
    pub simulation_state: SimulationState,
    /// Update callbacks enabled
    pub updates_enabled: bool,
}

impl DesignerVisualizerIntegration {
    /// Create new integration manager
    pub fn new() -> Self {
        Self {
            current_view: None,
            toolpath_settings: ToolpathViewSettings::default(),
            simulation_state: SimulationState::Idle,
            updates_enabled: true,
        }
    }

    /// Load design for visualization
    pub fn load_design(&mut self, design: DesignVisualization) {
        self.current_view = Some(design);
    }

    /// Get current visualization
    pub fn current_visualization(&self) -> Option<&DesignVisualization> {
        self.current_view.as_ref()
    }

    /// Get current visualization (mutable)
    pub fn current_visualization_mut(&mut self) -> Option<&mut DesignVisualization> {
        self.current_view.as_mut()
    }

    /// Clear current visualization
    pub fn clear(&mut self) {
        self.current_view = None;
        self.simulation_state = SimulationState::Idle;
    }

    /// Start simulation
    pub fn start_simulation(&mut self) -> bool {
        if self.current_view.is_some() {
            self.simulation_state = SimulationState::Running;
            true
        } else {
            false
        }
    }

    /// Pause simulation
    pub fn pause_simulation(&mut self) -> bool {
        if self.simulation_state == SimulationState::Running {
            self.simulation_state = SimulationState::Paused;
            true
        } else {
            false
        }
    }

    /// Resume simulation
    pub fn resume_simulation(&mut self) -> bool {
        if self.simulation_state == SimulationState::Paused {
            self.simulation_state = SimulationState::Running;
            true
        } else {
            false
        }
    }

    /// Stop simulation
    pub fn stop_simulation(&mut self) {
        self.simulation_state = SimulationState::Idle;
    }

    /// Enable real-time updates
    pub fn enable_realtime_updates(&mut self, enabled: bool) {
        self.updates_enabled = enabled;
        if let Some(view) = &mut self.current_view {
            view.real_time_updates = enabled;
        }
    }

    /// Toggle toolpath visibility
    pub fn toggle_toolpath(&mut self) -> bool {
        if let Some(view) = &mut self.current_view {
            view.show_toolpath = !view.show_toolpath;
            return view.show_toolpath;
        }
        false
    }

    /// Toggle shapes visibility
    pub fn toggle_shapes(&mut self) -> bool {
        if let Some(view) = &mut self.current_view {
            view.show_shapes = !view.show_shapes;
            return view.show_shapes;
        }
        false
    }

    /// Get integration statistics
    pub fn stats(&self) -> IntegrationStats {
        IntegrationStats {
            has_active_design: self.current_view.is_some(),
            is_simulating: self.simulation_state == SimulationState::Running,
            real_time_enabled: self.updates_enabled,
        }
    }
}

impl Default for DesignerVisualizerIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Simulation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulationState {
    /// Idle (not running)
    Idle,
    /// Simulation running
    Running,
    /// Simulation paused
    Paused,
    /// Simulation completed
    Completed,
}

/// Statistics for integration
#[derive(Debug, Clone)]
pub struct IntegrationStats {
    pub has_active_design: bool,
    pub is_simulating: bool,
    pub real_time_enabled: bool,
}

