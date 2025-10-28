//! Features module integration tests

use gcodekit4::visualizer::{
    BoundingBox, GridConfig, MachineLimits, SceneFeatures, ToolMarker, Vector3,
    WorkCoordinateSystem,
};

#[test]
fn test_grid_config_creation() {
    let grid = GridConfig::new(100.0, 10);
    assert_eq!(grid.size, 100.0);
    assert_eq!(grid.divisions, 10);
    assert!(grid.visible);
}

#[test]
fn test_grid_config_default() {
    let grid = GridConfig::default();
    assert!(grid.size > 0.0);
    assert!(grid.divisions > 0);
}

#[test]
fn test_work_coordinate_system_creation() {
    let wcs1 = WorkCoordinateSystem::new(1, Vector3::zero());
    assert_eq!(wcs1.number, 1);
    assert_eq!(wcs1.g_code(), "G54");

    let wcs2 = WorkCoordinateSystem::new(2, Vector3::new(10.0, 20.0, 30.0));
    assert_eq!(wcs2.g_code(), "G55");
    assert_eq!(wcs2.origin, Vector3::new(10.0, 20.0, 30.0));

    let wcs6 = WorkCoordinateSystem::new(6, Vector3::zero());
    assert_eq!(wcs6.g_code(), "G59");
}

#[test]
fn test_machine_limits_creation() {
    let limits = MachineLimits::new(Vector3::zero(), Vector3::new(100.0, 100.0, 100.0));
    assert_eq!(limits.min, Vector3::zero());
    assert_eq!(limits.max, Vector3::new(100.0, 100.0, 100.0));
    assert!(limits.enforced);
}

#[test]
fn test_machine_limits_contains() {
    let limits = MachineLimits::new(Vector3::zero(), Vector3::new(100.0, 100.0, 100.0));

    assert!(limits.contains(Vector3::new(50.0, 50.0, 50.0)));
    assert!(limits.contains(Vector3::zero()));
    assert!(limits.contains(Vector3::new(100.0, 100.0, 100.0)));

    assert!(!limits.contains(Vector3::new(150.0, 50.0, 50.0)));
    assert!(!limits.contains(Vector3::new(-10.0, 50.0, 50.0)));
}

#[test]
fn test_machine_limits_size_and_center() {
    let limits = MachineLimits::new(Vector3::zero(), Vector3::new(100.0, 200.0, 300.0));

    let size = limits.size();
    assert_eq!(size, Vector3::new(100.0, 200.0, 300.0));

    let center = limits.center();
    assert_eq!(center, Vector3::new(50.0, 100.0, 150.0));
}

#[test]
fn test_machine_limits_enforcement() {
    let mut limits = MachineLimits::new(Vector3::zero(), Vector3::new(100.0, 100.0, 100.0));
    
    limits.enforced = false;
    assert!(limits.contains(Vector3::new(500.0, 500.0, 500.0)));
}

#[test]
fn test_bounding_box_creation() {
    let bbox = BoundingBox::new(Vector3::zero(), Vector3::new(10.0, 10.0, 10.0));
    assert_eq!(bbox.min, Vector3::zero());
    assert_eq!(bbox.max, Vector3::new(10.0, 10.0, 10.0));
    assert!(bbox.show_edges);
    assert!(!bbox.show_faces);
}

#[test]
fn test_bounding_box_vertices() {
    let bbox = BoundingBox::new(Vector3::zero(), Vector3::new(10.0, 10.0, 10.0));
    let vertices = bbox.vertices();
    
    assert_eq!(vertices.len(), 8);
    assert!(vertices.contains(&Vector3::zero()));
    assert!(vertices.contains(&Vector3::new(10.0, 10.0, 10.0)));
}

#[test]
fn test_bounding_box_center_and_size() {
    let bbox = BoundingBox::new(Vector3::zero(), Vector3::new(20.0, 30.0, 40.0));

    let center = bbox.center();
    assert_eq!(center, Vector3::new(10.0, 15.0, 20.0));

    let size = bbox.size();
    assert_eq!(size, Vector3::new(20.0, 30.0, 40.0));
}

#[test]
fn test_tool_marker_creation() {
    let marker = ToolMarker::new(Vector3::new(5.0, 10.0, 15.0));
    assert_eq!(marker.position, Vector3::new(5.0, 10.0, 15.0));
    assert!(marker.visible);
    assert!(marker.size > 0.0);
}

#[test]
fn test_tool_marker_position_update() {
    let mut marker = ToolMarker::new(Vector3::zero());
    marker.set_position(Vector3::new(10.0, 10.0, 10.0));
    assert_eq!(marker.position, Vector3::new(10.0, 10.0, 10.0));
}

#[test]
fn test_tool_marker_size() {
    let mut marker = ToolMarker::new(Vector3::zero());
    marker.set_size(5.0);
    assert_eq!(marker.size, 5.0);

    marker.set_size(0.05);
    assert_eq!(marker.size, 0.1);
}

#[test]
fn test_scene_features_creation() {
    let features = SceneFeatures::new();
    assert!(features.grid.visible);
    assert!(!features.coordinate_systems.is_empty());
    assert!(features.limits.is_none());
    assert!(features.bounding_box.is_none());
}

#[test]
fn test_scene_features_default() {
    let features = SceneFeatures::default();
    assert!(features.grid.visible);
}

#[test]
fn test_scene_features_add_wcs() {
    let mut features = SceneFeatures::new();
    let initial_count = features.coordinate_systems.len();

    features.add_wcs(WorkCoordinateSystem::new(2, Vector3::new(10.0, 0.0, 0.0)));
    assert_eq!(features.coordinate_systems.len(), initial_count + 1);
}

#[test]
fn test_scene_features_set_limits() {
    let mut features = SceneFeatures::new();
    assert!(features.limits.is_none());

    features.set_limits(MachineLimits::new(Vector3::zero(), Vector3::new(100.0, 100.0, 100.0)));
    assert!(features.limits.is_some());
}

#[test]
fn test_scene_features_set_bounding_box() {
    let mut features = SceneFeatures::new();
    assert!(features.bounding_box.is_none());

    features.set_bounding_box(BoundingBox::new(Vector3::zero(), Vector3::new(50.0, 50.0, 50.0)));
    assert!(features.bounding_box.is_some());
}

#[test]
fn test_scene_features_toggle_grid() {
    let mut features = SceneFeatures::new();
    let initial = features.grid.visible;

    features.toggle_grid();
    assert_eq!(features.grid.visible, !initial);

    features.toggle_grid();
    assert_eq!(features.grid.visible, initial);
}

#[test]
fn test_scene_features_toggle_tool_marker() {
    let mut features = SceneFeatures::new();
    let initial = features.tool_marker.visible;

    features.toggle_tool_marker();
    assert_eq!(features.tool_marker.visible, !initial);

    features.toggle_tool_marker();
    assert_eq!(features.tool_marker.visible, initial);
}
