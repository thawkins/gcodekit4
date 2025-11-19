//! 3D Visualizer - Features - Task 83
//!
//! Show work coordinate system, display machine limits, add grid,
//! show tool position marker, implement bounding box

use crate::visualizer::setup::{Color, Vector3};

/// Grid configuration
#[derive(Debug, Clone)]
pub struct GridConfig {
    /// Grid size in world units
    pub size: f32,
    /// Number of grid divisions
    pub divisions: u32,
    /// Grid color
    pub color: Color,
    /// Whether grid is visible
    pub visible: bool,
}

impl GridConfig {
    /// Create new grid configuration
    pub fn new(size: f32, divisions: u32) -> Self {
        Self {
            size,
            divisions,
            color: Color::gray(),
            visible: true,
        }
    }

    /// Set grid color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Default for GridConfig {
    fn default() -> Self {
        Self::new(100.0, 10)
    }
}

/// Work coordinate system (WCS)
#[derive(Debug, Clone)]
pub struct WorkCoordinateSystem {
    /// WCS number (1-6 for G54-G59)
    pub number: u32,
    /// Origin offset
    pub origin: Vector3,
    /// WCS color for visualization
    pub color: Color,
}

impl WorkCoordinateSystem {
    /// Create new WCS
    pub fn new(number: u32, origin: Vector3) -> Self {
        let color = match number {
            1 => Color::red(),
            2 => Color::green(),
            3 => Color::blue(),
            4 => Color::yellow(),
            5 => Color::magenta(),
            6 => Color::cyan(),
            _ => Color::gray(),
        };

        Self {
            number,
            origin,
            color,
        }
    }

    /// Get G-code designation
    pub fn g_code(&self) -> String {
        match self.number {
            1..=6 => format!("G{}", 53 + self.number),
            _ => "Unknown".to_string(),
        }
    }
}

/// Machine limits (soft limits)
#[derive(Debug, Clone)]
pub struct MachineLimits {
    /// Minimum point in machine space
    pub min: Vector3,
    /// Maximum point in machine space
    pub max: Vector3,
    /// Limits color for visualization
    pub color: Color,
    /// Whether limits are enforced
    pub enforced: bool,
}

impl MachineLimits {
    /// Create new machine limits
    pub fn new(min: Vector3, max: Vector3) -> Self {
        Self {
            min,
            max,
            color: Color::orange(),
            enforced: true,
        }
    }

    /// Check if point is within limits
    pub fn contains(&self, point: Vector3) -> bool {
        if !self.enforced {
            return true;
        }
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Get size of limits
    pub fn size(&self) -> Vector3 {
        Vector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    /// Get center of limits
    pub fn center(&self) -> Vector3 {
        Vector3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }
}

/// Bounding box for visualization
#[derive(Debug, Clone)]
pub struct BoundingBox {
    /// Minimum corner
    pub min: Vector3,
    /// Maximum corner
    pub max: Vector3,
    /// Box color
    pub color: Color,
    /// Whether to show edges
    pub show_edges: bool,
    /// Whether to show filled faces
    pub show_faces: bool,
}

impl BoundingBox {
    /// Create new bounding box
    pub fn new(min: Vector3, max: Vector3) -> Self {
        Self {
            min,
            max,
            color: Color::white(),
            show_edges: true,
            show_faces: false,
        }
    }

    /// Set colors
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Get 8 corner vertices
    pub fn vertices(&self) -> Vec<Vector3> {
        vec![
            Vector3::new(self.min.x, self.min.y, self.min.z),
            Vector3::new(self.max.x, self.min.y, self.min.z),
            Vector3::new(self.max.x, self.max.y, self.min.z),
            Vector3::new(self.min.x, self.max.y, self.min.z),
            Vector3::new(self.min.x, self.min.y, self.max.z),
            Vector3::new(self.max.x, self.min.y, self.max.z),
            Vector3::new(self.max.x, self.max.y, self.max.z),
            Vector3::new(self.min.x, self.max.y, self.max.z),
        ]
    }

    /// Get center
    pub fn center(&self) -> Vector3 {
        Vector3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    /// Get size
    pub fn size(&self) -> Vector3 {
        Vector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }
}

/// Tool position marker
#[derive(Debug, Clone)]
pub struct ToolMarker {
    /// Current tool position
    pub position: Vector3,
    /// Marker size (sphere radius)
    pub size: f32,
    /// Marker color
    pub color: Color,
    /// Whether marker is visible
    pub visible: bool,
}

impl ToolMarker {
    /// Create new tool marker
    pub fn new(position: Vector3) -> Self {
        Self {
            position,
            size: 2.0,
            color: Color::red(),
            visible: true,
        }
    }

    /// Update position
    pub fn set_position(&mut self, position: Vector3) {
        self.position = position;
    }

    /// Set marker size
    pub fn set_size(&mut self, size: f32) {
        self.size = size.max(0.1);
    }
}

/// 3D scene features
#[derive(Debug, Clone)]
pub struct SceneFeatures {
    /// Grid configuration
    pub grid: GridConfig,
    /// Work coordinate systems
    pub coordinate_systems: Vec<WorkCoordinateSystem>,
    /// Machine limits
    pub limits: Option<MachineLimits>,
    /// Bounding box
    pub bounding_box: Option<BoundingBox>,
    /// Tool position marker
    pub tool_marker: ToolMarker,
}

impl SceneFeatures {
    /// Create new scene features
    pub fn new() -> Self {
        Self {
            grid: GridConfig::default(),
            coordinate_systems: vec![WorkCoordinateSystem::new(1, Vector3::zero())],
            limits: None,
            bounding_box: None,
            tool_marker: ToolMarker::new(Vector3::zero()),
        }
    }

    /// Add WCS
    pub fn add_wcs(&mut self, wcs: WorkCoordinateSystem) {
        self.coordinate_systems.push(wcs);
    }

    /// Set machine limits
    pub fn set_limits(&mut self, limits: MachineLimits) {
        self.limits = Some(limits);
    }

    /// Set bounding box
    pub fn set_bounding_box(&mut self, bbox: BoundingBox) {
        self.bounding_box = Some(bbox);
    }

    /// Toggle grid visibility
    pub fn toggle_grid(&mut self) {
        self.grid.visible = !self.grid.visible;
    }

    /// Toggle tool marker visibility
    pub fn toggle_tool_marker(&mut self) {
        self.tool_marker.visible = !self.tool_marker.visible;
    }
}

impl Default for SceneFeatures {
    fn default() -> Self {
        Self::new()
    }
}
