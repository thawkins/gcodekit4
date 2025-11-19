//! 3D Visualizer - Controls - Task 82
//!
//! Camera controls with rotation, zoom, pan, view presets,
//! and reset functionality

use crate::visualizer::setup::{Camera, Vector3};

/// View preset angles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewPreset {
    /// Top view (looking down Z axis)
    Top,
    /// Bottom view (looking up Z axis)
    Bottom,
    /// Front view (looking along Y axis)
    Front,
    /// Back view (looking along negative Y axis)
    Back,
    /// Right view (looking along X axis)
    Right,
    /// Left view (looking along negative X axis)
    Left,
    /// Isometric view
    Isometric,
    /// Custom view
    Custom,
}

impl std::fmt::Display for ViewPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Top => write!(f, "Top"),
            Self::Bottom => write!(f, "Bottom"),
            Self::Front => write!(f, "Front"),
            Self::Back => write!(f, "Back"),
            Self::Right => write!(f, "Right"),
            Self::Left => write!(f, "Left"),
            Self::Isometric => write!(f, "Isometric"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// Camera controller for 3D visualizer
#[derive(Debug, Clone)]
pub struct CameraController {
    /// Camera reference
    pub camera: Camera,
    /// Current view preset
    pub current_view: ViewPreset,
    /// Last mouse X position
    pub last_mouse_x: f32,
    /// Last mouse Y position
    pub last_mouse_y: f32,
    /// Mouse sensitivity for rotation
    pub rotation_sensitivity: f32,
    /// Mouse sensitivity for zoom
    pub zoom_sensitivity: f32,
    /// Pan speed
    pub pan_speed: f32,
    /// Minimum zoom distance
    pub min_zoom: f32,
    /// Maximum zoom distance
    pub max_zoom: f32,
    /// Current rotation angles (pitch, yaw, roll)
    pub rotation: (f32, f32, f32),
}

impl CameraController {
    /// Create new camera controller
    pub fn new(camera: Camera) -> Self {
        Self {
            camera,
            current_view: ViewPreset::Isometric,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            rotation_sensitivity: 0.005,
            zoom_sensitivity: 0.1,
            pan_speed: 1.0,
            min_zoom: 1.0,
            max_zoom: 1000.0,
            rotation: (0.0, 0.0, 0.0),
        }
    }

    /// Handle mouse movement for rotation (drag)
    pub fn on_mouse_drag(&mut self, current_x: f32, current_y: f32) {
        let dx = current_x - self.last_mouse_x;
        let dy = current_y - self.last_mouse_y;

        let pitch = dy * self.rotation_sensitivity;
        let yaw = dx * self.rotation_sensitivity;

        self.camera.rotate(pitch, yaw);
        self.rotation.0 += pitch;
        self.rotation.1 += yaw;

        self.last_mouse_x = current_x;
        self.last_mouse_y = current_y;
        self.current_view = ViewPreset::Custom;
    }

    /// Handle mouse wheel for zoom
    pub fn on_mouse_wheel(&mut self, delta: f32) {
        let factor = 1.0 - (delta * self.zoom_sensitivity);
        let direction = self.camera.position - self.camera.target;
        let distance = direction.magnitude();
        let new_distance = (distance * factor).clamp(self.min_zoom, self.max_zoom);
        let normalized = direction.normalize();

        self.camera.position = self.camera.target + (normalized * new_distance);

        self.current_view = ViewPreset::Custom;
    }

    /// Handle middle mouse button for panning
    pub fn on_pan(&mut self, dx: f32, dy: f32) {
        let right = self.camera.get_right();
        let up = self.camera.up;

        let pan_delta = (right * (dx * self.pan_speed)) + (up * (dy * self.pan_speed));
        self.camera.move_camera(pan_delta);
        self.current_view = ViewPreset::Custom;
    }

    /// Reset view to default
    pub fn reset_view(&mut self) {
        self.set_view_preset(ViewPreset::Isometric);
    }

    /// Set view to preset
    pub fn set_view_preset(&mut self, preset: ViewPreset) {
        if preset == ViewPreset::Custom {
            return;
        }

        let distance = (self.camera.position - self.camera.target).magnitude();
        let target = self.camera.target;
        let half_dist = distance / 2.0;
        let iso_dist = distance / std::f32::consts::SQRT_2;

        self.camera.position = target
            + match preset {
                ViewPreset::Top => Vector3::new(0.0, 0.0, distance),
                ViewPreset::Bottom => Vector3::new(0.0, 0.0, -distance),
                ViewPreset::Front => Vector3::new(0.0, -distance, half_dist),
                ViewPreset::Back => Vector3::new(0.0, distance, half_dist),
                ViewPreset::Right => Vector3::new(distance, 0.0, half_dist),
                ViewPreset::Left => Vector3::new(-distance, 0.0, half_dist),
                ViewPreset::Isometric => Vector3::new(iso_dist, iso_dist, iso_dist),
                ViewPreset::Custom => unreachable!(),
            };

        self.rotation = (0.0, 0.0, 0.0);
        self.current_view = preset;
    }

    /// Set camera target and adjust position
    pub fn set_target(&mut self, target: Vector3) {
        let direction = self.camera.position - self.camera.target;
        self.camera.target = target;
        self.camera.position = target + direction;
    }

    /// Fit all content in view
    pub fn fit_all(&mut self, bounding_box: Option<(Vector3, Vector3)>) {
        if let Some((min, max)) = bounding_box {
            let center = Vector3::new(
                (min.x + max.x) / 2.0,
                (min.y + max.y) / 2.0,
                (min.z + max.z) / 2.0,
            );

            let size = max - min;
            let max_dim = size.x.max(size.y).max(size.z);
            let distance = max_dim / (2.0 * (45.0_f32.to_radians() / 2.0).tan());
            let iso_dist = distance / std::f32::consts::SQRT_2;

            self.set_target(center);
            self.camera.position = center + Vector3::new(iso_dist, iso_dist, iso_dist);
        }
    }

    /// Set mouse position for next drag operation
    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.last_mouse_x = x;
        self.last_mouse_y = y;
    }

    /// Set rotation sensitivity
    pub fn set_rotation_sensitivity(&mut self, sensitivity: f32) {
        self.rotation_sensitivity = sensitivity.max(0.0001);
    }

    /// Set zoom sensitivity
    pub fn set_zoom_sensitivity(&mut self, sensitivity: f32) {
        self.zoom_sensitivity = sensitivity.max(0.01);
    }

    /// Set pan speed
    pub fn set_pan_speed(&mut self, speed: f32) {
        self.pan_speed = speed.max(0.1);
    }
}

/// Visualizer controls state
#[derive(Debug, Clone)]
pub struct VisualizerControls {
    /// Camera controller
    pub camera_controller: CameraController,
    /// Show grid
    pub show_grid: bool,
    /// Show work coordinate system
    pub show_wcs: bool,
    /// Show machine limits
    pub show_limits: bool,
    /// Show bounding box
    pub show_bounding_box: bool,
    /// Show current position marker
    pub show_position_marker: bool,
    /// Grid size
    pub grid_size: f32,
    /// Show rapid moves
    pub show_rapid_moves: bool,
    /// Show feed moves
    pub show_feed_moves: bool,
    /// Show arcs
    pub show_arcs: bool,
    /// Transparency of toolpath
    pub toolpath_alpha: f32,
}

impl VisualizerControls {
    /// Create new visualizer controls
    pub fn new(camera: Camera) -> Self {
        Self {
            camera_controller: CameraController::new(camera),
            show_grid: true,
            show_wcs: true,
            show_limits: true,
            show_bounding_box: false,
            show_position_marker: true,
            grid_size: 10.0,
            show_rapid_moves: true,
            show_feed_moves: true,
            show_arcs: true,
            toolpath_alpha: 1.0,
        }
    }

    /// Toggle grid display
    pub fn toggle_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    /// Toggle WCS display
    pub fn toggle_wcs(&mut self) {
        self.show_wcs = !self.show_wcs;
    }

    /// Toggle limits display
    pub fn toggle_limits(&mut self) {
        self.show_limits = !self.show_limits;
    }

    /// Toggle bounding box
    pub fn toggle_bounding_box(&mut self) {
        self.show_bounding_box = !self.show_bounding_box;
    }

    /// Set toolpath transparency
    pub fn set_toolpath_alpha(&mut self, alpha: f32) {
        self.toolpath_alpha = alpha.clamp(0.0, 1.0);
    }
}
