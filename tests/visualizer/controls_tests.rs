//! Controls module integration tests

use gcodekit4::visualizer::{
    Camera, CameraController, Vector3, ViewPreset, VisualizerControls,
};

#[test]
fn test_view_preset_display() {
    assert_eq!(format!("{}", ViewPreset::Top), "Top");
    assert_eq!(format!("{}", ViewPreset::Bottom), "Bottom");
    assert_eq!(format!("{}", ViewPreset::Front), "Front");
    assert_eq!(format!("{}", ViewPreset::Back), "Back");
    assert_eq!(format!("{}", ViewPreset::Right), "Right");
    assert_eq!(format!("{}", ViewPreset::Left), "Left");
    assert_eq!(format!("{}", ViewPreset::Isometric), "Isometric");
    assert_eq!(format!("{}", ViewPreset::Custom), "Custom");
}

#[test]
fn test_camera_controller_creation() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let controller = CameraController::new(camera);
    
    assert_eq!(controller.current_view, ViewPreset::Isometric);
    assert!(controller.rotation_sensitivity > 0.0);
    assert!(controller.zoom_sensitivity > 0.0);
}

#[test]
fn test_camera_controller_zoom() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controller = CameraController::new(camera);
    let initial_distance = controller.camera.position.magnitude();

    controller.on_mouse_wheel(1.0);
    let new_distance = controller.camera.position.magnitude();

    assert!(new_distance < initial_distance);
    assert_eq!(controller.current_view, ViewPreset::Custom);
}

#[test]
fn test_camera_controller_zoom_limits() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controller = CameraController::new(camera);

    for _ in 0..100 {
        controller.on_mouse_wheel(10.0);
    }
    let distance = controller.camera.position.magnitude();
    assert!(distance >= controller.min_zoom);

    let camera2 = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controller2 = CameraController::new(camera2);
    for _ in 0..100 {
        controller2.on_mouse_wheel(-10.0);
    }
    let distance2 = controller2.camera.position.magnitude();
    assert!(distance2 <= controller2.max_zoom);
}

#[test]
fn test_camera_controller_pan() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controller = CameraController::new(camera);
    let initial_target = controller.camera.target;

    controller.on_pan(10.0, 10.0);

    assert_ne!(controller.camera.target, initial_target);
    assert_eq!(controller.current_view, ViewPreset::Custom);
}

#[test]
fn test_camera_controller_drag() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controller = CameraController::new(camera);
    
    controller.set_mouse_position(0.0, 0.0);
    let initial_rotation = controller.rotation;

    controller.on_mouse_drag(100.0, 100.0);

    assert_ne!(controller.rotation, initial_rotation);
    assert_eq!(controller.current_view, ViewPreset::Custom);
}

#[test]
fn test_camera_controller_view_presets() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controller = CameraController::new(camera);
    let distance = controller.camera.position.magnitude();

    controller.set_view_preset(ViewPreset::Top);
    assert_eq!(controller.current_view, ViewPreset::Top);
    assert!(controller.camera.position.z > 0.0);
    assert!((controller.camera.position.magnitude() - distance).abs() < 0.1);

    controller.set_view_preset(ViewPreset::Front);
    assert_eq!(controller.current_view, ViewPreset::Front);

    controller.set_view_preset(ViewPreset::Right);
    assert_eq!(controller.current_view, ViewPreset::Right);
}

#[test]
fn test_camera_controller_reset_view() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controller = CameraController::new(camera);

    controller.on_pan(10.0, 10.0);
    assert_eq!(controller.current_view, ViewPreset::Custom);

    controller.reset_view();
    assert_eq!(controller.current_view, ViewPreset::Isometric);
}

#[test]
fn test_camera_controller_set_target() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controller = CameraController::new(camera);
    let initial_distance = controller.camera.position.magnitude();

    let new_target = Vector3::new(10.0, 10.0, 10.0);
    controller.set_target(new_target);

    assert_eq!(controller.camera.target, new_target);
    let new_distance = (controller.camera.position - new_target).magnitude();
    assert!((new_distance - initial_distance).abs() < 0.1);
}

#[test]
fn test_camera_controller_fit_all() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controller = CameraController::new(camera);

    let bbox = (Vector3::zero(), Vector3::new(50.0, 50.0, 50.0));
    controller.fit_all(Some(bbox));

    let expected_center = Vector3::new(25.0, 25.0, 25.0);
    assert_eq!(controller.camera.target, expected_center);
}

#[test]
fn test_camera_controller_sensitivity_settings() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controller = CameraController::new(camera);

    controller.set_rotation_sensitivity(0.01);
    assert_eq!(controller.rotation_sensitivity, 0.01);

    controller.set_zoom_sensitivity(0.2);
    assert_eq!(controller.zoom_sensitivity, 0.2);

    controller.set_pan_speed(2.0);
    assert_eq!(controller.pan_speed, 2.0);
}

#[test]
fn test_visualizer_controls_creation() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let controls = VisualizerControls::new(camera);

    assert!(controls.show_grid);
    assert!(controls.show_wcs);
    assert!(controls.show_limits);
    assert!(controls.show_position_marker);
    assert_eq!(controls.toolpath_alpha, 1.0);
}

#[test]
fn test_visualizer_controls_toggles() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controls = VisualizerControls::new(camera);

    assert!(controls.show_grid);
    controls.toggle_grid();
    assert!(!controls.show_grid);

    assert!(controls.show_wcs);
    controls.toggle_wcs();
    assert!(!controls.show_wcs);

    assert!(controls.show_limits);
    controls.toggle_limits();
    assert!(!controls.show_limits);

    assert!(!controls.show_bounding_box);
    controls.toggle_bounding_box();
    assert!(controls.show_bounding_box);
}

#[test]
fn test_visualizer_controls_toolpath_alpha() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let mut controls = VisualizerControls::new(camera);

    controls.set_toolpath_alpha(0.5);
    assert_eq!(controls.toolpath_alpha, 0.5);

    controls.set_toolpath_alpha(1.5);
    assert_eq!(controls.toolpath_alpha, 1.0);

    controls.set_toolpath_alpha(-0.5);
    assert_eq!(controls.toolpath_alpha, 0.0);
}

#[test]
fn test_visualizer_controls_show_flags() {
    let camera = Camera::new(Vector3::new(100.0, 100.0, 100.0), Vector3::zero());
    let controls = VisualizerControls::new(camera);

    assert!(controls.show_rapid_moves);
    assert!(controls.show_feed_moves);
    assert!(controls.show_arcs);
}
