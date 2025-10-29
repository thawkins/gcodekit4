//! Tests for processing::advanced_features

use gcodekit4::processing::advanced_features::*;

#[test]
fn test_probing_system() {
    let mut probe = ProbingSystem::new();
    probe.add_result(ProbeResult::success(10.0));
    probe.add_result(ProbeResult::success(11.0));
    assert_eq!(probe.average_height(), Some(10.5));
}

#[test]
fn test_tool_library() {
    let mut lib = ToolLibrary::new();
    let tool = Tool::new(1, "End Mill");
    lib.add_tool(tool);
    assert!(lib.select_tool(1));
}

#[test]
fn test_work_coordinate_manager() {
    let mut wcs = WorkCoordinateManager::new();
    assert_eq!(wcs.get_gcode(1), "G54");
    wcs.select(2);
    assert_eq!(wcs.current_wcs, 2);
}

#[test]
fn test_soft_limits() {
    let limits = SoftLimits::new();
    assert!(limits.check(50.0, 50.0, -50.0));
    assert!(!limits.check(150.0, 50.0, -50.0));
}

#[test]
fn test_simulation_mode() {
    let mut sim = SimulationMode::new();
    sim.start();
    sim.execute_move(10.0, 10.0, 0.0);
    assert_eq!(sim.commands_executed, 1);
}

#[test]
fn test_command_history() {
    let mut history = CommandHistory::new(100);
    let entry = CommandHistoryEntry::new("G0 X10");
    history.add(entry);
    assert_eq!(history.get_history().len(), 1);
}
