//! 3D visualization module (wgpu-based)
//!
//! This module provides:
//! - 3D rendering engine (setup)
//! - Toolpath visualization (rendering)
//! - Interactive camera controls (controls)
//! - Grid and axis rendering

pub mod canvas_renderer;
pub mod controls;
pub mod features;
pub mod setup;
pub mod toolpath_cache;
pub mod toolpath_rendering;
pub mod viewport;
pub mod visualizer_2d;

pub use canvas_renderer::{
    render_grid_to_path, render_origin_to_path, render_rapid_moves_to_path, render_toolpath_to_path,
    render_g1_to_path, render_g2_to_path, render_g3_to_path, render_g4_to_path,
    render_intensity_overlay,
};
pub use controls::{CameraController, ViewPreset, VisualizerControls};
pub use features::{
    BoundingBox, GridConfig, MachineLimits, SceneFeatures, ToolMarker, WorkCoordinateSystem,
};
pub use setup::{Camera, CameraType, Color, Light, LightType, Renderer, Scene, Vector3};
pub use toolpath_cache::ToolpathCache;
pub use toolpath_rendering::{
    ArcSegment, LineSegment, MovementType, PathSegment, Toolpath, ToolpathStats,
};
pub use viewport::{Bounds, ViewportTransform};
pub use visualizer_2d::{GCodeCommand, Point2D, Visualizer2D};

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
