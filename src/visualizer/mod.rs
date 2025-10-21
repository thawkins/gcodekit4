//! 3D visualization module (wgpu-based)
//!
//! This module provides:
//! - 3D rendering engine (setup)
//! - Toolpath visualization (rendering)
//! - Interactive camera controls (controls)
//! - Grid and axis rendering

pub mod setup;
pub mod toolpath_rendering;
pub mod controls;

pub use setup::{Vector3, Color, Camera, CameraType, Light, LightType, Scene, Renderer};
pub use toolpath_rendering::{MovementType, LineSegment, ArcSegment, Toolpath, ToolpathStats};
pub use controls::{ViewPreset, CameraController, VisualizerControls};

/// 3D Visualizer - Task 80-82
pub struct Visualizer {
    /// Rendering context
    pub renderer: Renderer,
    /// Toolpath data
    pub toolpath: Toolpath,
    /// Controls
    pub controls: VisualizerControls,
}

impl Visualizer {
    /// Create a new visualizer
    pub fn new(width: u32, height: u32) -> Self {
        let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
        let renderer = Renderer::new(width, height);
        let controls = VisualizerControls::new(camera.clone());
        
        Self {
            renderer,
            toolpath: Toolpath::default(),
            controls,
        }
    }

    /// Resize visualizer
    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.controls.camera_controller.camera = self.renderer.camera.clone();
    }

    /// Get toolpath statistics
    pub fn get_toolpath_stats(&self) -> toolpath_rendering::ToolpathStats {
        self.toolpath.get_statistics()
    }
}

impl Default for Visualizer {
    fn default() -> Self {
        Self::new(800, 600)
    }
}
