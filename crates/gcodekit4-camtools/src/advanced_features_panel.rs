//! Advanced Features UI Panel - Phase 6/7 Integration
//!
//! Displays and controls advanced features including:
//! - Task 103-111: Probing, tool management, coordinates, simulation
//! - Task 112-120: Performance monitoring, logging, alarms
//! - Task 121-125: Safety, plugins, export, calibration, diagnostics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool change mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolChangeMode {
    /// No tool change
    None,
    /// Manual tool change
    Manual,
    /// Automatic tool change
    Automatic,
}

/// Simulation state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulationState {
    /// Not simulating
    Idle,
    /// Simulation in progress
    Running,
    /// Simulation paused
    Paused,
    /// Simulation completed
    Completed,
}

/// Soft limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftLimits {
    /// X-axis min limit (mm)
    pub x_min: f64,
    /// X-axis max limit (mm)
    pub x_max: f64,
    /// Y-axis min limit (mm)
    pub y_min: f64,
    /// Y-axis max limit (mm)
    pub y_max: f64,
    /// Z-axis min limit (mm)
    pub z_min: f64,
    /// Z-axis max limit (mm)
    pub z_max: f64,
    /// Whether soft limits are enabled
    pub enabled: bool,
}

impl Default for SoftLimits {
    fn default() -> Self {
        Self {
            x_min: -100.0,
            x_max: 100.0,
            y_min: -100.0,
            y_max: 100.0,
            z_min: -100.0,
            z_max: 100.0,
            enabled: true,
        }
    }
}

/// Work coordinate system (WCS)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkCoordinateSystem {
    /// System number (G54=0, G55=1, ..., G59=5)
    pub number: u8,
    /// X offset
    pub x_offset: f64,
    /// Y offset
    pub y_offset: f64,
    /// Z offset
    pub z_offset: f64,
    /// A offset (if available)
    pub a_offset: Option<f64>,
    /// B offset (if available)
    pub b_offset: Option<f64>,
    /// C offset (if available)
    pub c_offset: Option<f64>,
}

impl WorkCoordinateSystem {
    /// Create new WCS
    pub fn new(number: u8) -> Self {
        Self {
            number,
            x_offset: 0.0,
            y_offset: 0.0,
            z_offset: 0.0,
            a_offset: None,
            b_offset: None,
            c_offset: None,
        }
    }

    /// Get WCS name (G54-G59)
    pub fn name(&self) -> String {
        match self.number {
            0 => "G54".to_string(),
            1 => "G55".to_string(),
            2 => "G56".to_string(),
            3 => "G57".to_string(),
            4 => "G58".to_string(),
            5 => "G59".to_string(),
            _ => format!("WCS{}", self.number),
        }
    }
}

/// Tool in tool library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool number
    pub number: u32,
    /// Tool description
    pub description: String,
    /// Tool diameter (mm)
    pub diameter: f64,
    /// Tool length offset (mm)
    pub length_offset: f64,
    /// Tool flute count
    pub flute_count: u32,
    /// Maximum spindle speed (RPM)
    pub max_speed: u32,
}

impl Tool {
    /// Create new tool
    pub fn new(number: u32) -> Self {
        Self {
            number,
            description: format!("Tool {}", number),
            diameter: 0.0,
            length_offset: 0.0,
            flute_count: 1,
            max_speed: 10000,
        }
    }
}

/// Advanced features panel state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedFeaturesPanel {
    /// Tool change mode
    pub tool_change_mode: ToolChangeMode,
    /// Current tool
    pub current_tool: Option<Tool>,
    /// Tool library
    pub tools: HashMap<u32, Tool>,
    /// Work coordinate systems
    pub coordinate_systems: Vec<WorkCoordinateSystem>,
    /// Active WCS number
    pub active_wcs: u8,
    /// Soft limits configuration
    pub soft_limits: SoftLimits,
    /// Simulation state
    pub simulation_state: SimulationState,
    /// Simulated position X
    pub sim_pos_x: f64,
    /// Simulated position Y
    pub sim_pos_y: f64,
    /// Simulated position Z
    pub sim_pos_z: f64,
    /// Whether probing is active
    pub probing_active: bool,
    /// Probe result (Z height)
    pub probe_result: Option<f64>,
    /// Step-through mode enabled
    pub step_through_enabled: bool,
    /// Current step number
    pub current_step: usize,
    /// Total steps
    pub total_steps: usize,
    /// Bookmarks (line numbers)
    pub bookmarks: Vec<usize>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Commands per second
    pub cmd_per_sec: f64,
    /// Buffer usage percentage (0-100)
    pub buffer_usage: f64,
    /// Total commands sent
    pub total_commands: usize,
    /// Commands in queue
    pub queued_commands: usize,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cmd_per_sec: 0.0,
            buffer_usage: 0.0,
            total_commands: 0,
            queued_commands: 0,
            avg_latency_ms: 0.0,
        }
    }
}

impl AdvancedFeaturesPanel {
    /// Create new advanced features panel
    pub fn new() -> Self {
        let mut coordinate_systems = Vec::new();
        for i in 0..6 {
            coordinate_systems.push(WorkCoordinateSystem::new(i));
        }

        Self {
            tool_change_mode: ToolChangeMode::None,
            current_tool: None,
            tools: HashMap::new(),
            coordinate_systems,
            active_wcs: 0,
            soft_limits: SoftLimits::default(),
            simulation_state: SimulationState::Idle,
            sim_pos_x: 0.0,
            sim_pos_y: 0.0,
            sim_pos_z: 0.0,
            probing_active: false,
            probe_result: None,
            step_through_enabled: false,
            current_step: 0,
            total_steps: 0,
            bookmarks: Vec::new(),
            performance_metrics: PerformanceMetrics::default(),
        }
    }

    /// Add tool to library
    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.insert(tool.number, tool);
    }

    /// Set current tool
    pub fn set_current_tool(&mut self, tool_number: u32) -> anyhow::Result<()> {
        if let Some(tool) = self.tools.get(&tool_number) {
            self.current_tool = Some(tool.clone());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Tool {} not found", tool_number))
        }
    }

    /// Start simulation
    pub fn start_simulation(&mut self) {
        self.simulation_state = SimulationState::Running;
        self.sim_pos_x = 0.0;
        self.sim_pos_y = 0.0;
        self.sim_pos_z = 0.0;
    }

    /// Pause simulation
    pub fn pause_simulation(&mut self) {
        self.simulation_state = SimulationState::Paused;
    }

    /// Resume simulation
    pub fn resume_simulation(&mut self) {
        self.simulation_state = SimulationState::Running;
    }

    /// Stop simulation
    pub fn stop_simulation(&mut self) {
        self.simulation_state = SimulationState::Idle;
    }

    /// Update simulated position
    pub fn update_sim_position(&mut self, x: f64, y: f64, z: f64) {
        self.sim_pos_x = x;
        self.sim_pos_y = y;
        self.sim_pos_z = z;
    }

    /// Start probing
    pub fn start_probing(&mut self) {
        self.probing_active = true;
        self.probe_result = None;
    }

    /// Set probe result
    pub fn set_probe_result(&mut self, z_height: f64) {
        self.probe_result = Some(z_height);
        self.probing_active = false;
    }

    /// Add bookmark
    pub fn add_bookmark(&mut self, line: usize) {
        if !self.bookmarks.contains(&line) {
            self.bookmarks.push(line);
            self.bookmarks.sort();
        }
    }

    /// Remove bookmark
    pub fn remove_bookmark(&mut self, line: usize) {
        self.bookmarks.retain(|&x| x != line);
    }

    /// Check if position violates soft limits
    pub fn check_soft_limits(&self, x: f64, y: f64, z: f64) -> Vec<String> {
        let mut violations = Vec::new();

        if !self.soft_limits.enabled {
            return violations;
        }

        if x < self.soft_limits.x_min {
            violations.push(format!(
                "X below minimum: {} < {}",
                x, self.soft_limits.x_min
            ));
        }
        if x > self.soft_limits.x_max {
            violations.push(format!(
                "X exceeds maximum: {} > {}",
                x, self.soft_limits.x_max
            ));
        }
        if y < self.soft_limits.y_min {
            violations.push(format!(
                "Y below minimum: {} < {}",
                y, self.soft_limits.y_min
            ));
        }
        if y > self.soft_limits.y_max {
            violations.push(format!(
                "Y exceeds maximum: {} > {}",
                y, self.soft_limits.y_max
            ));
        }
        if z < self.soft_limits.z_min {
            violations.push(format!(
                "Z below minimum: {} < {}",
                z, self.soft_limits.z_min
            ));
        }
        if z > self.soft_limits.z_max {
            violations.push(format!(
                "Z exceeds maximum: {} > {}",
                z, self.soft_limits.z_max
            ));
        }

        violations
    }

    /// Update performance metrics
    pub fn update_performance_metrics(
        &mut self,
        cmd_per_sec: f64,
        buffer_usage: f64,
        total_commands: usize,
        queued_commands: usize,
        avg_latency_ms: f64,
    ) {
        self.performance_metrics = PerformanceMetrics {
            cmd_per_sec,
            buffer_usage,
            total_commands,
            queued_commands,
            avg_latency_ms,
        };
    }
}

impl Default for AdvancedFeaturesPanel {
    fn default() -> Self {
        Self::new()
    }
}


