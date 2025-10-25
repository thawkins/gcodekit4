//! Integration tests for Tasks 103-120
//! Phase 6 Extended Features

use gcodekit4::utils::{
    Alarm, AlarmManager, AlarmType, AutoConnectConfig, BookmarkManager, CommandHistory,
    CustomAction, CustomMacro, DataLogger, HeightPoint, NetworkConfig, PendantConfig, ProbeMesh,
    Simulator, SoftLimits, Stepper, ToolInfo, ToolLibrary, ToolOffset, ToolOffsetManager,
    WorkCoordinateSystem, WorkOffset,
};

// ============================================================================
// TASK 103: AUTO-LEVELING PROBE MESH
// ============================================================================

#[test]
fn test_task_103_probe_mesh_creation() {
    let mesh = ProbeMesh::new(1.0, 1.0);
    assert_eq!(mesh.x_spacing, 1.0);
    assert_eq!(mesh.y_spacing, 1.0);
}

#[test]
fn test_task_103_probe_mesh_add_points() {
    let mut mesh = ProbeMesh::new(1.0, 1.0);
    mesh.add_point(HeightPoint {
        x: 0.0,
        y: 0.0,
        z: 0.5,
    });
    mesh.add_point(HeightPoint {
        x: 1.0,
        y: 1.0,
        z: 0.7,
    });
    mesh.add_point(HeightPoint {
        x: 2.0,
        y: 2.0,
        z: 0.6,
    });

    assert_eq!(mesh.points.len(), 3);
}

#[test]
fn test_task_103_probe_mesh_stats() {
    let mut mesh = ProbeMesh::new(1.0, 1.0);
    mesh.add_point(HeightPoint {
        x: 0.0,
        y: 0.0,
        z: 0.5,
    });
    mesh.add_point(HeightPoint {
        x: 1.0,
        y: 1.0,
        z: 0.8,
    });

    let (count, min, max) = mesh.stats();
    assert_eq!(count, 2);
    assert_eq!(min, 0.5);
    assert_eq!(max, 0.8);
}

#[test]
fn test_task_103_probe_mesh_z_offset() {
    let mut mesh = ProbeMesh::new(1.0, 1.0);
    mesh.add_point(HeightPoint {
        x: 0.0,
        y: 0.0,
        z: 0.5,
    });
    mesh.add_point(HeightPoint {
        x: 1.0,
        y: 0.0,
        z: 0.6,
    });
    mesh.add_point(HeightPoint {
        x: 0.0,
        y: 1.0,
        z: 0.7,
    });
    mesh.add_point(HeightPoint {
        x: 1.0,
        y: 1.0,
        z: 0.8,
    });

    let offset = mesh.get_z_offset(0.5, 0.5);
    assert!(offset.is_some());
}

// ============================================================================
// TASK 104: TOOL CHANGE MANAGEMENT
// ============================================================================

#[test]
fn test_task_104_tool_creation() {
    let tool = ToolInfo::new(1, "End Mill", 3.175);
    assert_eq!(tool.number, 1);
    assert_eq!(tool.name, "End Mill");
    assert_eq!(tool.diameter, 3.175);
}

#[test]
fn test_task_104_tool_library() {
    let mut library = ToolLibrary::new();
    let tool1 = ToolInfo::new(1, "End Mill", 3.175);
    let tool2 = ToolInfo::new(2, "Drill", 5.0);

    library.add_tool(tool1);
    library.add_tool(tool2);

    assert_eq!(library.list_tools().len(), 2);
}

#[test]
fn test_task_104_current_tool() {
    let mut library = ToolLibrary::new();
    let tool = ToolInfo::new(1, "End Mill", 3.175);
    library.add_tool(tool);
    library.set_current_tool(1);

    assert!(library.current_tool().is_some());
    assert_eq!(library.current_tool().unwrap().number, 1);
}

#[test]
fn test_task_104_tool_retrieval() {
    let mut library = ToolLibrary::new();
    let tool = ToolInfo::new(5, "Bit", 2.0);
    library.add_tool(tool);

    assert!(library.get_tool(5).is_some());
    assert!(library.get_tool(99).is_none());
}

// ============================================================================
// TASK 105: TOOL LENGTH OFFSET
// ============================================================================

#[test]
fn test_task_105_tool_offset_creation() {
    let offset = ToolOffset::new(1, 25.4);
    assert_eq!(offset.tool_number, 1);
    assert_eq!(offset.length_offset, 25.4);
}

#[test]
fn test_task_105_tool_offset_total() {
    let mut offset = ToolOffset::new(1, 25.4);
    offset.wear_offset = 0.1;

    assert_eq!(offset.total_offset(), 25.5);
}

#[test]
fn test_task_105_offset_manager() {
    let mut manager = ToolOffsetManager::new();
    let offset = ToolOffset::new(1, 25.4);
    manager.set_offset(offset);

    assert!(manager.get_offset(1).is_some());
    assert_eq!(manager.get_total_offset(1), 25.4);
}

#[test]
fn test_task_105_offset_adjustment() {
    let mut manager = ToolOffsetManager::new();
    let offset = ToolOffset::new(1, 25.4);
    manager.set_offset(offset);

    manager.adjust_wear(1, 0.05);
    assert_eq!(manager.get_total_offset(1), 25.45);
}

// ============================================================================
// TASK 106: WORK COORDINATE SYSTEMS
// ============================================================================

#[test]
fn test_task_106_work_offset_creation() {
    let offset = WorkOffset::new(10.0, 20.0, 30.0);
    assert_eq!(offset.x, 10.0);
    assert_eq!(offset.y, 20.0);
    assert_eq!(offset.z, 30.0);
}

#[test]
fn test_task_106_work_offset_zero() {
    let offset = WorkOffset::zero();
    assert_eq!(offset.x, 0.0);
    assert_eq!(offset.y, 0.0);
    assert_eq!(offset.z, 0.0);
}

#[test]
fn test_task_106_wcs_creation() {
    let wcs = WorkCoordinateSystem::new();
    assert_eq!(wcs.current_system(), 1);
}

#[test]
fn test_task_106_wcs_offset_setting() {
    let mut wcs = WorkCoordinateSystem::new();
    let offset = WorkOffset::new(10.0, 20.0, 30.0);
    wcs.set_offset(1, offset);

    let retrieved = wcs.get_offset(1).unwrap();
    assert_eq!(retrieved.x, 10.0);
}

#[test]
fn test_task_106_wcs_system_selection() {
    let mut wcs = WorkCoordinateSystem::new();
    assert!(wcs.select_system(3).is_ok());
    assert_eq!(wcs.current_system(), 3);

    assert!(wcs.select_system(0).is_err());
    assert!(wcs.select_system(10).is_err());
}

#[test]
fn test_task_106_wcs_current_offset() {
    let mut wcs = WorkCoordinateSystem::new();
    let offset = WorkOffset::new(5.0, 10.0, 15.0);
    wcs.set_offset(1, offset);

    let current = wcs.current_offset();
    assert_eq!(current.x, 5.0);
}

// ============================================================================
// TASK 107: SOFT LIMITS
// ============================================================================

#[test]
fn test_task_107_soft_limits_creation() {
    let limits = SoftLimits::new();
    assert!(limits.enabled);
    assert_eq!(limits.x_min, -100.0);
    assert_eq!(limits.x_max, 100.0);
}

#[test]
fn test_task_107_soft_limits_check_valid() {
    let limits = SoftLimits::new();
    assert!(limits.check(0.0, 0.0, 0.0));
    assert!(limits.check(50.0, 50.0, 50.0));
}

#[test]
fn test_task_107_soft_limits_check_invalid() {
    let limits = SoftLimits::new();
    assert!(!limits.check(200.0, 0.0, 0.0));
    assert!(!limits.check(0.0, 200.0, 0.0));
    assert!(!limits.check(0.0, 0.0, 200.0));
}

#[test]
fn test_task_107_soft_limits_violations() {
    let limits = SoftLimits::new();
    let violations = limits.get_violations(150.0, -150.0, 0.0);

    assert!(violations.len() > 0);
    assert!(violations.iter().any(|v| v.contains("X")));
    assert!(violations.iter().any(|v| v.contains("Y")));
}

#[test]
fn test_task_107_soft_limits_disabled() {
    let mut limits = SoftLimits::new();
    limits.enabled = false;

    assert!(limits.check(500.0, 500.0, 500.0));
}

// ============================================================================
// TASK 108: SIMULATION MODE
// ============================================================================

#[test]
fn test_task_108_simulator_creation() {
    let sim = Simulator::new();
    assert!(!sim.active);
}

#[test]
fn test_task_108_simulator_start_stop() {
    let mut sim = Simulator::new();
    sim.start();
    assert!(sim.active);

    sim.stop();
    assert!(!sim.active);
}

#[test]
fn test_task_108_simulator_move() {
    let mut sim = Simulator::new();
    sim.start();
    sim.move_to(10.0, 20.0, 30.0);

    assert_eq!(sim.position.x, 10.0);
    assert_eq!(sim.position.y, 20.0);
    assert_eq!(sim.position.z, 30.0);
}

#[test]
fn test_task_108_simulator_commands_executed() {
    let mut sim = Simulator::new();
    sim.start();

    sim.move_to(10.0, 0.0, 0.0);
    sim.move_to(20.0, 0.0, 0.0);
    sim.move_to(20.0, 10.0, 0.0);

    assert_eq!(sim.commands_executed, 3);
}

// ============================================================================
// TASK 109: STEP-THROUGH EXECUTION
// ============================================================================

#[test]
fn test_task_109_stepper_creation() {
    let stepper = Stepper::new(100);
    assert_eq!(stepper.current_line, 0);
    assert!(stepper.paused);
}

#[test]
fn test_task_109_stepper_forward() {
    let mut stepper = Stepper::new(100);
    assert!(stepper.step_forward());
    assert_eq!(stepper.current_line, 1);

    for _ in 0..99 {
        stepper.step_forward();
    }
    assert!(!stepper.step_forward()); // At end
}

#[test]
fn test_task_109_stepper_backward() {
    let mut stepper = Stepper::new(100);
    stepper.step_forward();
    stepper.step_forward();

    assert!(stepper.step_backward());
    assert_eq!(stepper.current_line, 1);
}

#[test]
fn test_task_109_stepper_pause_resume() {
    let mut stepper = Stepper::new(100);
    assert!(stepper.paused);

    stepper.resume();
    assert!(!stepper.paused);

    stepper.pause();
    assert!(stepper.paused);
}

// ============================================================================
// TASK 110: BOOKMARKS/BREAKPOINTS
// ============================================================================

#[test]
fn test_task_110_bookmark_add() {
    let mut manager = BookmarkManager::new();
    manager.add_bookmark(10, "Start cutting");

    assert!(manager.get_bookmark(10).is_some());
    assert_eq!(manager.list_bookmarks().len(), 1);
}

#[test]
fn test_task_110_breakpoint_add() {
    let mut manager = BookmarkManager::new();
    manager.add_breakpoint(50, "Check dimension");

    let bookmark = manager.get_bookmark(50).unwrap();
    assert!(bookmark.is_breakpoint);
}

#[test]
fn test_task_110_bookmark_removal() {
    let mut manager = BookmarkManager::new();
    manager.add_bookmark(10, "Test");

    assert!(manager.remove_bookmark(10));
    assert!(manager.get_bookmark(10).is_none());
}

#[test]
fn test_task_110_list_breakpoints() {
    let mut manager = BookmarkManager::new();
    manager.add_bookmark(10, "Bookmark");
    manager.add_breakpoint(20, "Breakpoint 1");
    manager.add_breakpoint(30, "Breakpoint 2");

    assert_eq!(manager.list_bookmarks().len(), 3);
    assert_eq!(manager.list_breakpoints().len(), 2);
}

// ============================================================================
// TASK 111: PROGRAM RESTART
// ============================================================================

// Covered via ProgramState tests in advanced.rs

// ============================================================================
// TASK 112: PERFORMANCE MONITORING
// ============================================================================

// Covered via PerformanceMetrics tests in advanced.rs

// ============================================================================
// TASK 113: COMMAND HISTORY
// ============================================================================

#[test]
fn test_task_113_history_add() {
    let mut history = CommandHistory::new(100);
    history.add("G0 X10", true);

    assert_eq!(history.get_history().len(), 1);
}

#[test]
fn test_task_113_history_multiple() {
    let mut history = CommandHistory::new(100);
    history.add("G0 X10", true);
    history.add("G1 Y20", false);
    history.add("G1 Z5", true);

    assert_eq!(history.get_history().len(), 3);
}

#[test]
fn test_task_113_history_last() {
    let mut history = CommandHistory::new(100);
    history.add("G0 X10", true);
    history.add("G1 Y20", false);

    let last = history.get_last().unwrap();
    assert_eq!(last.command, "G1 Y20");
    assert!(!last.success);
}

#[test]
fn test_task_113_history_limit() {
    let mut history = CommandHistory::new(5);
    for i in 0..10 {
        history.add(format!("G0 X{}", i), true);
    }

    assert_eq!(history.get_history().len(), 5);
}

#[test]
fn test_task_113_history_clear() {
    let mut history = CommandHistory::new(100);
    history.add("G0 X10", true);
    history.clear();

    assert_eq!(history.get_history().len(), 0);
}

// ============================================================================
// TASK 114: CUSTOM SCRIPTS/MACROS
// ============================================================================

#[test]
fn test_task_114_macro_creation() {
    let macro_obj = CustomMacro::new("move", "G0 X10 Y20");
    assert_eq!(macro_obj.name, "move");
}

#[test]
fn test_task_114_macro_variables() {
    let mut macro_obj = CustomMacro::new("move", "G0 X${X} Y${Y}");
    macro_obj.set_variable("X", "10");
    macro_obj.set_variable("Y", "20");

    assert_eq!(macro_obj.variables.len(), 2);
}

#[test]
fn test_task_114_macro_expansion() {
    let mut macro_obj = CustomMacro::new("move", "G0 X${X} Y${Y} Z${Z}");
    macro_obj.set_variable("X", "10");
    macro_obj.set_variable("Y", "20");
    macro_obj.set_variable("Z", "5");

    let expanded = macro_obj.expand();
    assert!(expanded.contains("X10"));
    assert!(expanded.contains("Y20"));
    assert!(expanded.contains("Z5"));
}

#[test]
fn test_task_114_macro_partial_expansion() {
    let mut macro_obj = CustomMacro::new("move", "G0 X${X} Y${Y} Z${Z}");
    macro_obj.set_variable("X", "10");

    let expanded = macro_obj.expand();
    assert!(expanded.contains("X10"));
    assert!(expanded.contains("${Y}"));
}

// ============================================================================
// TASK 115: PENDANT SUPPORT
// ============================================================================

#[test]
fn test_task_115_pendant_config() {
    let config = PendantConfig::new("/dev/ttyUSB0");
    assert!(!config.enabled);
}

// ============================================================================
// TASK 116: CUSTOM BUTTONS/ACTIONS
// ============================================================================

#[test]
fn test_task_116_custom_action() {
    let mut action = CustomAction::new("Drill");
    action.add_command("G0 Z5");
    action.add_command("G1 Z-5 F100");

    assert_eq!(action.commands.len(), 2);
    assert_eq!(action.commands[0], "G0 Z5");
}

// ============================================================================
// TASK 117: AUTO-CONNECT
// ============================================================================

#[test]
fn test_task_117_auto_connect_config() {
    let config = AutoConnectConfig::new();
    assert!(config.enabled);
    assert!(config.auto_detect_firmware);
}

// ============================================================================
// TASK 118: NETWORK/REMOTE ACCESS
// ============================================================================

#[test]
fn test_task_118_network_config() {
    let config = NetworkConfig::new();
    assert!(!config.websocket_enabled);
    assert!(!config.rest_enabled);
}

// ============================================================================
// TASK 119: DATA LOGGING
// ============================================================================

#[test]
fn test_task_119_data_logger() {
    let mut logger = DataLogger::new();
    logger.log("INFO", "Starting job");
    logger.log("ERROR", "Command failed");

    assert_eq!(logger.get_logs().len(), 2);
}

#[test]
fn test_task_119_data_logger_clear() {
    let mut logger = DataLogger::new();
    logger.log("INFO", "Test");
    logger.clear();

    assert_eq!(logger.get_logs().len(), 0);
}

// ============================================================================
// TASK 120: ALARMS AND NOTIFICATIONS
// ============================================================================

#[test]
fn test_task_120_alarm_creation() {
    let alarm = Alarm::new(AlarmType::Warning, "Low coolant");
    assert_eq!(alarm.alarm_type, AlarmType::Warning);
    assert!(!alarm.acknowledged);
}

#[test]
fn test_task_120_alarm_manager() {
    let mut manager = AlarmManager::new();
    let alarm = Alarm::new(AlarmType::Critical, "Emergency stop");
    manager.add_alarm(alarm);

    assert_eq!(manager.get_alarms().len(), 1);
}

#[test]
fn test_task_120_alarm_acknowledge() {
    let mut manager = AlarmManager::new();
    let alarm = Alarm::new(AlarmType::Error, "Test error");
    manager.add_alarm(alarm);

    assert_eq!(manager.unacknowledged().len(), 1);

    manager.acknowledge(0);
    assert_eq!(manager.unacknowledged().len(), 0);
}

#[test]
fn test_task_120_alarm_types() {
    let info = Alarm::new(AlarmType::Info, "Info message");
    let warning = Alarm::new(AlarmType::Warning, "Warning message");
    let error = Alarm::new(AlarmType::Error, "Error message");
    let critical = Alarm::new(AlarmType::Critical, "Critical message");

    assert_eq!(info.alarm_type, AlarmType::Info);
    assert_eq!(warning.alarm_type, AlarmType::Warning);
    assert_eq!(error.alarm_type, AlarmType::Error);
    assert_eq!(critical.alarm_type, AlarmType::Critical);
}

#[test]
fn test_task_120_alarm_clear() {
    let mut manager = AlarmManager::new();
    manager.add_alarm(Alarm::new(AlarmType::Error, "Error 1"));
    manager.add_alarm(Alarm::new(AlarmType::Error, "Error 2"));

    assert_eq!(manager.get_alarms().len(), 2);

    manager.clear();
    assert_eq!(manager.get_alarms().len(), 0);
}
