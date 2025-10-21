//! 3D Visualizer - Setup - Task 80
//!
//! Initialize 3D rendering context, camera system, basic scene, and lighting

/// 3D point/vector
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
    /// Z coordinate
    pub z: f32,
}

impl Vector3 {
    /// Create new vector
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Zero vector
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    /// Unit X vector
    pub fn unit_x() -> Self {
        Self { x: 1.0, y: 0.0, z: 0.0 }
    }

    /// Unit Y vector
    pub fn unit_y() -> Self {
        Self { x: 0.0, y: 1.0, z: 0.0 }
    }

    /// Unit Z vector
    pub fn unit_z() -> Self {
        Self { x: 0.0, y: 0.0, z: 1.0 }
    }

    /// Calculate magnitude
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalize vector
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        } else {
            Self::zero()
        }
    }

    /// Add vectors
    pub fn add(&self, other: Vector3) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    /// Subtract vectors
    pub fn subtract(&self, other: Vector3) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    /// Dot product
    pub fn dot(&self, other: Vector3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product
    pub fn cross(&self, other: Vector3) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

/// RGB Color
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component (0.0-1.0)
    pub r: f32,
    /// Green component (0.0-1.0)
    pub g: f32,
    /// Blue component (0.0-1.0)
    pub b: f32,
    /// Alpha component (0.0-1.0)
    pub a: f32,
}

impl Color {
    /// Create new color
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create color with alpha
    pub fn with_alpha(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// White color
    pub fn white() -> Self {
        Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }

    /// Black color
    pub fn black() -> Self {
        Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }
    }

    /// Red color
    pub fn red() -> Self {
        Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }
    }

    /// Green color
    pub fn green() -> Self {
        Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 }
    }

    /// Blue color
    pub fn blue() -> Self {
        Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 }
    }

    /// Gray color
    pub fn gray() -> Self {
        Self { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }
    }
}

/// Camera type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraType {
    /// Orthographic camera (no perspective distortion)
    Orthographic,
    /// Perspective camera
    Perspective,
}

/// 3D Camera
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera position
    pub position: Vector3,
    /// Point camera is looking at
    pub target: Vector3,
    /// Up direction
    pub up: Vector3,
    /// Camera type
    pub camera_type: CameraType,
    /// Field of view (for perspective camera, in degrees)
    pub fov: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
    /// Aspect ratio (width/height)
    pub aspect_ratio: f32,
}

impl Camera {
    /// Create new camera
    pub fn new(position: Vector3, target: Vector3) -> Self {
        Self {
            position,
            target,
            up: Vector3::unit_z(),
            camera_type: CameraType::Perspective,
            fov: 45.0,
            near: 0.1,
            far: 1000.0,
            aspect_ratio: 16.0 / 9.0,
        }
    }

    /// Create orthographic camera
    pub fn orthographic(position: Vector3, target: Vector3) -> Self {
        Self {
            position,
            target,
            up: Vector3::unit_z(),
            camera_type: CameraType::Orthographic,
            fov: 45.0,
            near: 0.1,
            far: 1000.0,
            aspect_ratio: 16.0 / 9.0,
        }
    }

    /// Set aspect ratio
    pub fn set_aspect_ratio(&mut self, width: f32, height: f32) {
        if height > 0.0 {
            self.aspect_ratio = width / height;
        }
    }

    /// Move camera
    pub fn move_camera(&mut self, delta: Vector3) {
        self.position = self.position.add(delta);
        self.target = self.target.add(delta);
    }

    /// Rotate camera around target
    pub fn rotate(&mut self, pitch: f32, yaw: f32) {
        let mut direction = self.position.subtract(self.target);
        
        let cos_yaw = yaw.cos();
        let sin_yaw = yaw.sin();
        direction = Vector3::new(
            direction.x * cos_yaw - direction.y * sin_yaw,
            direction.x * sin_yaw + direction.y * cos_yaw,
            direction.z,
        );

        let cos_pitch = pitch.cos();
        let sin_pitch = pitch.sin();
        direction = Vector3::new(
            direction.x,
            direction.y * cos_pitch - direction.z * sin_pitch,
            direction.y * sin_pitch + direction.z * cos_pitch,
        );

        self.position = self.target.add(direction);
    }

    /// Zoom camera
    pub fn zoom(&mut self, factor: f32) {
        let direction = self.position.subtract(self.target);
        let distance = direction.magnitude();
        let new_distance = (distance * factor).max(0.1);
        let normalized = direction.normalize();
        
        self.position = self.target.add(Vector3::new(
            normalized.x * new_distance,
            normalized.y * new_distance,
            normalized.z * new_distance,
        ));
    }

    /// Get view direction
    pub fn get_view_direction(&self) -> Vector3 {
        self.target.subtract(self.position).normalize()
    }

    /// Get right vector
    pub fn get_right(&self) -> Vector3 {
        self.get_view_direction().cross(self.up).normalize()
    }
}

/// Light type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    /// Directional light (like sun)
    Directional,
    /// Point light
    Point,
    /// Spot light
    Spot,
}

/// 3D Light
#[derive(Debug, Clone)]
pub struct Light {
    /// Light position
    pub position: Vector3,
    /// Light direction (for directional/spot lights)
    pub direction: Vector3,
    /// Light color
    pub color: Color,
    /// Light intensity
    pub intensity: f32,
    /// Light type
    pub light_type: LightType,
}

impl Light {
    /// Create new light
    pub fn new(light_type: LightType, color: Color) -> Self {
        Self {
            position: Vector3::zero(),
            direction: Vector3::new(0.0, 0.0, -1.0),
            color,
            intensity: 1.0,
            light_type,
        }
    }

    /// Create directional light
    pub fn directional(direction: Vector3, color: Color) -> Self {
        Self {
            position: Vector3::zero(),
            direction: direction.normalize(),
            color,
            intensity: 1.0,
            light_type: LightType::Directional,
        }
    }

    /// Create point light
    pub fn point(position: Vector3, color: Color) -> Self {
        Self {
            position,
            direction: Vector3::zero(),
            color,
            intensity: 1.0,
            light_type: LightType::Point,
        }
    }

    /// Set intensity
    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity.max(0.0);
    }
}

/// 3D Scene
#[derive(Debug, Clone)]
pub struct Scene {
    /// Scene background color
    pub background_color: Color,
    /// Lights in scene
    pub lights: Vec<Light>,
    /// Ambient light intensity
    pub ambient_intensity: f32,
}

impl Scene {
    /// Create new scene
    pub fn new() -> Self {
        Self {
            background_color: Color::new(0.2, 0.2, 0.2),
            lights: Vec::new(),
            ambient_intensity: 0.3,
        }
    }

    /// Add light to scene
    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    /// Create default lighting
    pub fn setup_default_lights(&mut self) {
        self.lights.clear();
        self.add_light(Light::directional(
            Vector3::new(1.0, 1.0, 1.0),
            Color::white(),
        ));
        self.add_light(Light::directional(
            Vector3::new(-1.0, -1.0, -0.5),
            Color::gray(),
        ));
    }
}

impl Default for Scene {
    fn default() -> Self {
        let mut scene = Self::new();
        scene.setup_default_lights();
        scene
    }
}

/// 3D Renderer context
#[derive(Debug)]
pub struct Renderer {
    /// Rendering canvas width
    pub width: u32,
    /// Rendering canvas height
    pub height: u32,
    /// Active camera
    pub camera: Camera,
    /// Scene being rendered
    pub scene: Scene,
    /// Whether renderer is initialized
    pub initialized: bool,
}

impl Renderer {
    /// Create new renderer
    pub fn new(width: u32, height: u32) -> Self {
        let mut camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
        camera.set_aspect_ratio(width as f32, height as f32);

        Self {
            width,
            height,
            camera,
            scene: Scene::default(),
            initialized: false,
        }
    }

    /// Initialize renderer
    pub fn initialize(&mut self) -> Result<(), String> {
        self.initialized = true;
        Ok(())
    }

    /// Resize renderer
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.camera.set_aspect_ratio(width as f32, height as f32);
    }

    /// Clear screen with background color
    pub fn clear(&self) -> String {
        format!(
            "Clear {{r: {}, g: {}, b: {}}}",
            self.scene.background_color.r,
            self.scene.background_color.g,
            self.scene.background_color.b
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3_creation() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_vector3_operations() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        
        let sum = v1.add(v2);
        assert_eq!(sum, Vector3::new(5.0, 7.0, 9.0));

        let diff = v1.subtract(v2);
        assert_eq!(diff, Vector3::new(-3.0, -3.0, -3.0));

        let dot = v1.dot(v2);
        assert_eq!(dot, 32.0);
    }

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new(Vector3::new(10.0, 10.0, 10.0), Vector3::zero());
        assert_eq!(camera.position, Vector3::new(10.0, 10.0, 10.0));
        assert_eq!(camera.camera_type, CameraType::Perspective);
    }

    #[test]
    fn test_color_creation() {
        let color = Color::new(1.0, 0.5, 0.0);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_scene_creation() {
        let scene = Scene::default();
        assert!(!scene.lights.is_empty());
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new(800, 600);
        assert_eq!(renderer.width, 800);
        assert_eq!(renderer.height, 600);
    }
}
