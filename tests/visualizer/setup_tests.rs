//! Setup module integration tests

use gcodekit4::visualizer::{
    Camera, CameraType, Color, Light, LightType, Renderer, Scene, Vector3,
};

#[test]
fn test_vector3_creation_and_operations() {
    let v1 = Vector3::new(1.0, 2.0, 3.0);
    let v2 = Vector3::new(4.0, 5.0, 6.0);

    assert_eq!(v1.x, 1.0);
    assert_eq!(v1.y, 2.0);
    assert_eq!(v1.z, 3.0);

    let sum = v1.add(v2);
    assert_eq!(sum, Vector3::new(5.0, 7.0, 9.0));

    let diff = v1.subtract(v2);
    assert_eq!(diff, Vector3::new(-3.0, -3.0, -3.0));

    let dot = v1.dot(v2);
    assert_eq!(dot, 32.0);
}

#[test]
fn test_vector3_magnitude_and_normalize() {
    let v = Vector3::new(3.0, 4.0, 0.0);
    assert_eq!(v.magnitude(), 5.0);

    let normalized = v.normalize();
    assert!((normalized.magnitude() - 1.0).abs() < 0.001);
}

#[test]
fn test_vector3_cross_product() {
    let v1 = Vector3::unit_x();
    let v2 = Vector3::unit_y();
    let cross = v1.cross(v2);
    assert_eq!(cross, Vector3::unit_z());
}

#[test]
fn test_vector3_operators() {
    let v1 = Vector3::new(1.0, 2.0, 3.0);
    let v2 = Vector3::new(2.0, 3.0, 4.0);

    let sum = v1 + v2;
    assert_eq!(sum, Vector3::new(3.0, 5.0, 7.0));

    let diff = v2 - v1;
    assert_eq!(diff, Vector3::new(1.0, 1.0, 1.0));

    let scaled = v1 * 2.0;
    assert_eq!(scaled, Vector3::new(2.0, 4.0, 6.0));
}

#[test]
fn test_color_creation() {
    let color = Color::new(1.0, 0.5, 0.0);
    assert_eq!(color.r, 1.0);
    assert_eq!(color.g, 0.5);
    assert_eq!(color.b, 0.0);
    assert_eq!(color.a, 1.0);

    let color_with_alpha = Color::with_alpha(0.2, 0.3, 0.4, 0.5);
    assert_eq!(color_with_alpha.a, 0.5);
}

#[test]
fn test_color_presets() {
    assert_eq!(Color::white(), Color::new(1.0, 1.0, 1.0));
    assert_eq!(Color::black(), Color::new(0.0, 0.0, 0.0));
    assert_eq!(Color::red(), Color::new(1.0, 0.0, 0.0));
    assert_eq!(Color::green(), Color::new(0.0, 1.0, 0.0));
    assert_eq!(Color::blue(), Color::new(0.0, 0.0, 1.0));
}

#[test]
fn test_camera_creation_and_types() {
    let perspective_camera = Camera::new(Vector3::new(10.0, 10.0, 10.0), Vector3::zero());
    assert_eq!(perspective_camera.position, Vector3::new(10.0, 10.0, 10.0));
    assert_eq!(perspective_camera.target, Vector3::zero());
    assert_eq!(perspective_camera.camera_type, CameraType::Perspective);

    let ortho_camera = Camera::orthographic(Vector3::new(5.0, 5.0, 5.0), Vector3::zero());
    assert_eq!(ortho_camera.camera_type, CameraType::Orthographic);
}

#[test]
fn test_camera_movement() {
    let mut camera = Camera::new(Vector3::new(10.0, 10.0, 10.0), Vector3::zero());
    let delta = Vector3::new(1.0, 2.0, 3.0);
    camera.move_camera(delta);

    assert_eq!(camera.position, Vector3::new(11.0, 12.0, 13.0));
    assert_eq!(camera.target, Vector3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_camera_zoom() {
    let mut camera = Camera::new(Vector3::new(10.0, 10.0, 10.0), Vector3::zero());
    let initial_distance = camera.position.magnitude();

    camera.zoom(0.5);
    let new_distance = camera.position.magnitude();

    assert!(new_distance < initial_distance);
    assert!((new_distance / initial_distance - 0.5).abs() < 0.01);
}

#[test]
fn test_camera_aspect_ratio() {
    let mut camera = Camera::new(Vector3::new(10.0, 10.0, 10.0), Vector3::zero());
    camera.set_aspect_ratio(1920.0, 1080.0);

    assert!((camera.aspect_ratio - (1920.0 / 1080.0)).abs() < 0.01);
}

#[test]
fn test_camera_view_direction() {
    let camera = Camera::new(Vector3::new(0.0, 0.0, 10.0), Vector3::zero());
    let view_dir = camera.get_view_direction();

    assert_eq!(view_dir, Vector3::new(0.0, 0.0, -1.0));
}

#[test]
fn test_light_creation() {
    let directional = Light::directional(Vector3::new(1.0, 1.0, 1.0), Color::white());
    assert_eq!(directional.light_type, LightType::Directional);

    let point = Light::point(Vector3::new(5.0, 5.0, 5.0), Color::yellow());
    assert_eq!(point.light_type, LightType::Point);
    assert_eq!(point.position, Vector3::new(5.0, 5.0, 5.0));
}

#[test]
fn test_light_intensity() {
    let mut light = Light::new(LightType::Point, Color::white());
    assert_eq!(light.intensity, 1.0);

    light.set_intensity(0.5);
    assert_eq!(light.intensity, 0.5);

    light.set_intensity(-1.0);
    assert_eq!(light.intensity, 0.0);
}

#[test]
fn test_scene_creation_and_lights() {
    let mut scene = Scene::new();
    assert_eq!(scene.lights.len(), 0);

    scene.add_light(Light::directional(Vector3::unit_z(), Color::white()));
    assert_eq!(scene.lights.len(), 1);

    scene.setup_default_lights();
    assert!(!scene.lights.is_empty());
}

#[test]
fn test_scene_default() {
    let scene = Scene::default();
    assert!(!scene.lights.is_empty());
    assert!(scene.ambient_intensity > 0.0);
}

#[test]
fn test_renderer_creation_and_resize() {
    let mut renderer = Renderer::new(800, 600);
    assert_eq!(renderer.width, 800);
    assert_eq!(renderer.height, 600);
    assert!(!renderer.initialized);

    renderer.resize(1024, 768);
    assert_eq!(renderer.width, 1024);
    assert_eq!(renderer.height, 768);
}

#[test]
fn test_renderer_initialization() {
    let mut renderer = Renderer::new(800, 600);
    assert!(!renderer.initialized);

    let result = renderer.initialize();
    assert!(result.is_ok());
    assert!(renderer.initialized);
}

#[test]
fn test_renderer_clear() {
    let renderer = Renderer::new(800, 600);
    let clear_str = renderer.clear();
    assert!(clear_str.contains("Clear"));
}
