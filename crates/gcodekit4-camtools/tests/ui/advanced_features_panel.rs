use gcodekit4_camtools::advanced_features_panel::{AdvancedFeaturesPanel, SoftLimits, WorkCoordinateSystem, Tool, SimulationState};

#[test]
fn test_soft_limits_new() {
    let limits = SoftLimits::default();
    assert!(limits.enabled);
    assert_eq!(limits.x_min, -100.0);
}

#[test]
fn test_wcs_new() {
    let wcs = WorkCoordinateSystem::new(0);
    assert_eq!(wcs.number, 0);
    assert_eq!(wcs.name(), "G54");
}

#[test]
fn test_tool_new() {
    let tool = Tool::new(1);
    assert_eq!(tool.number, 1);
    assert!(tool.description.contains("Tool 1"));
}

#[test]
fn test_advanced_features_panel_new() {
    let panel = AdvancedFeaturesPanel::new();
    assert_eq!(panel.coordinate_systems.len(), 6);
    assert_eq!(panel.tools.len(), 0);
    assert_eq!(panel.simulation_state, SimulationState::Idle);
}

#[test]
fn test_add_tool() {
    let mut panel = AdvancedFeaturesPanel::new();
    let tool = Tool::new(1);
    panel.add_tool(tool);
    assert_eq!(panel.tools.len(), 1);
}

#[test]
fn test_set_current_tool() {
    let mut panel = AdvancedFeaturesPanel::new();
    let tool = Tool::new(1);
    panel.add_tool(tool);
    assert!(panel.set_current_tool(1).is_ok());
    assert!(panel.current_tool.is_some());
}

#[test]
fn test_start_simulation() {
    let mut panel = AdvancedFeaturesPanel::new();
    panel.start_simulation();
    assert_eq!(panel.simulation_state, SimulationState::Running);
}

#[test]
fn test_check_soft_limits() {
    let panel = AdvancedFeaturesPanel::new();
    let violations = panel.check_soft_limits(150.0, 0.0, 0.0);
    assert!(!violations.is_empty());
    assert!(violations[0].contains("X exceeds maximum"));
}

#[test]
fn test_add_bookmark() {
    let mut panel = AdvancedFeaturesPanel::new();
    panel.add_bookmark(10);
    panel.add_bookmark(20);
    assert_eq!(panel.bookmarks.len(), 2);
    assert_eq!(panel.bookmarks[0], 10);
}

#[test]
fn test_probing() {
    let mut panel = AdvancedFeaturesPanel::new();
    panel.start_probing();
    assert!(panel.probing_active);
    panel.set_probe_result(5.5);
    assert!(!panel.probing_active);
    assert_eq!(panel.probe_result, Some(5.5));
}

#[test]
fn test_performance_metrics() {
    let mut panel = AdvancedFeaturesPanel::new();
    panel.update_performance_metrics(100.0, 50.0, 1000, 10, 5.5);
    assert_eq!(panel.performance_metrics.cmd_per_sec, 100.0);
    assert_eq!(panel.performance_metrics.buffer_usage, 50.0);
}
