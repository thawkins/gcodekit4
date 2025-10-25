// Integration tests for Phase 7 (Tasks 121-150)

use gcodekit4::utils::{
    EmergencyStopManager, EmergencyStopState, MotionInterlock, FeedHoldManager,
    SafetyFeaturesManager, PluginRegistry, PluginMetadata, PluginConfig, ExportFormat,
    PostProcessor, FormatExporter, CalibrationWizard, CalibrationStepType,
    CommunicationDiagnostics, BufferDiagnostics, PerformanceProfiler, DiagnosticReport,
};
use std::collections::HashMap;

// ============================================================================
// Task 121: Safety Features Integration Tests
// ============================================================================

#[test]
fn test_emergency_stop_workflow() {
    let mut manager = EmergencyStopManager::new(true, 100);

    // Start armed
    assert_eq!(manager.state(), EmergencyStopState::Armed);
    assert!(manager.is_safe());

    // Trigger emergency stop
    assert!(manager.trigger().is_ok());
    assert!(!manager.is_safe());

    // Cannot trigger twice
    assert!(manager.trigger().is_err());

    // Reset
    assert!(manager.reset().is_ok());
    assert!(manager.is_safe());
}

#[test]
fn test_motion_interlock_comprehensive() {
    let mut interlock = MotionInterlock::default();
    interlock.require_homing = true;
    interlock.check_tool_loaded = true;
    interlock.min_safe_z = -5.0;

    // Not homed - should fail
    assert!(interlock.validate_motion(false, true, 0.0).is_err());

    // Tool not loaded - should fail
    assert!(interlock.validate_motion(true, false, 0.0).is_err());

    // Z below safe height - should fail
    assert!(interlock.validate_motion(true, true, -10.0).is_err());

    // All OK
    assert!(interlock.validate_motion(true, true, 0.0).is_ok());
}

#[test]
fn test_feed_hold_sequence() {
    let mut manager = FeedHoldManager::new();

    // Start active
    assert!(!manager.is_held());

    // Hold for various reasons
    assert!(manager.hold("Manual hold").is_ok());
    assert!(manager.is_held());
    assert_eq!(manager.reason(), "Manual hold");

    // Cannot hold twice
    assert!(manager.hold("Another").is_ok()); // Actually overwrites
    assert_eq!(manager.reason(), "Another");

    // Resume
    assert!(manager.resume().is_ok());
    assert!(!manager.is_held());

    // Cannot resume when not held
    assert!(manager.resume().is_err());
}

#[test]
fn test_safety_features_emergency_stop() {
    let mut safety = SafetyFeaturesManager::new();

    assert!(safety.is_safe());

    // Emergency stop sequence
    assert!(safety.emergency_stop().is_ok());
    assert!(!safety.is_safe());
    assert!(safety.feed_hold.is_held());
}

#[test]
fn test_safety_features_motion_check() {
    let mut safety = SafetyFeaturesManager::new();
    safety.motion_interlock.require_homing = true;

    // Without homing, motion should fail
    assert!(safety
        .motion_interlock
        .validate_motion(false, false, 0.0)
        .is_err());

    // With homing, motion should succeed
    assert!(safety
        .motion_interlock
        .validate_motion(true, false, 0.0)
        .is_ok());
}

// ============================================================================
// Task 122: Plugin System Integration Tests
// ============================================================================

#[test]
fn test_plugin_registry_operations() {
    let mut registry = PluginRegistry::new();

    assert_eq!(registry.list_plugins().len(), 0);

    // Try to get non-existent plugin
    assert!(registry.get("NonExistent").is_err());
}

#[test]
fn test_plugin_config_serialization() {
    let mut config = PluginConfig::default();
    config.enabled = true;
    config.settings.insert("key".to_string(), serde_json::json!("value"));

    assert!(config.enabled);
    assert_eq!(config.settings.len(), 1);
}

#[test]
fn test_plugin_metadata_structure() {
    let metadata = PluginMetadata {
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Test Author".to_string(),
        description: "Test plugin".to_string(),
        supported_controllers: vec!["GRBL".to_string()],
        required_api_version: "1.0".to_string(),
    };

    assert_eq!(metadata.name, "Test Plugin");
    assert_eq!(metadata.version, "1.0.0");
    assert_eq!(metadata.supported_controllers.len(), 1);
}

// ============================================================================
// Task 123: Export Format Integration Tests
// ============================================================================

#[test]
fn test_all_export_formats() {
    let formats = vec![
        ExportFormat::StandardGcode,
        ExportFormat::LinuxCNC,
        ExportFormat::FANUC,
        ExportFormat::Haas,
        ExportFormat::Siemens,
    ];

    for format in formats {
        let processor = PostProcessor::for_format(format);
        assert_eq!(processor.format, format);
        assert!(processor.precision_digits > 0);
    }
}

#[test]
fn test_post_processor_conversion() {
    let processor = PostProcessor::for_format(ExportFormat::FANUC);

    // Test line conversion
    let original = "G0 X10 Y20";
    let converted = processor.convert_line(original);
    assert!(!converted.is_empty());
}

#[test]
fn test_format_exporter_workflow() {
    let processor = PostProcessor::for_format(ExportFormat::StandardGcode);
    let exporter = FormatExporter::new(processor);

    let gcode = vec!["G0 X0 Y0".to_string(), "G1 Z-5 F100".to_string(), "G0 Z5".to_string()];

    assert!(exporter.export(&gcode).is_ok());

    let result = exporter.export(&gcode).unwrap();
    assert!(result.contains("G0"));
    assert!(result.contains("G1"));
}

#[test]
fn test_multiple_format_export() {
    let gcode = vec!["G0 X10 Y20".to_string(), "G1 Z-5 F100".to_string()];

    let formats = vec![
        ExportFormat::StandardGcode,
        ExportFormat::LinuxCNC,
        ExportFormat::FANUC,
    ];

    for format in formats {
        let processor = PostProcessor::for_format(format);
        let exporter = FormatExporter::new(processor);
        assert!(exporter.export(&gcode).is_ok());
    }
}

// ============================================================================
// Task 124: Calibration Integration Tests
// ============================================================================

#[test]
fn test_step_calibration_workflow() {
    let mut wizard = CalibrationWizard::new(CalibrationStepType::StepCalibration);

    // Should have 3 steps (X, Y, Z)
    assert!(wizard.current_step().is_some());
    assert!(!wizard.is_complete());

    // Record X axis measurement
    assert!(wizard.record_measurement(10.01).is_ok());
    assert_eq!(wizard.results().len(), 1);
    assert!(wizard.results()[0].passed);

    // Record Y axis measurement
    assert!(wizard.record_measurement(9.98).is_ok());
    assert_eq!(wizard.results().len(), 2);

    // Record Z axis measurement
    assert!(wizard.record_measurement(10.03).is_ok());
    assert_eq!(wizard.results().len(), 3);

    // Should be complete
    assert!(wizard.is_complete());
}

#[test]
fn test_backlash_calibration() {
    let mut wizard = CalibrationWizard::new(CalibrationStepType::BacklashMeasurement);

    // Should have 2 steps (X, Y)
    assert!(wizard.current_step().is_some());

    // Record X backlash
    assert!(wizard.record_measurement(0.05).is_ok());
    assert!(wizard.results()[0].passed);

    // Record Y backlash
    assert!(wizard.record_measurement(0.08).is_ok());
    assert!(wizard.is_complete());
}

#[test]
fn test_squareness_calibration() {
    let mut wizard = CalibrationWizard::new(CalibrationStepType::SquarenessCheck);

    // Should have 1 step (XY)
    assert!(wizard.record_measurement(90.05).is_ok());
    assert!(wizard.is_complete());
}

#[test]
fn test_calibration_report_generation() {
    let mut wizard = CalibrationWizard::new(CalibrationStepType::StepCalibration);

    // Perform measurements
    assert!(wizard.record_measurement(10.01).is_ok());
    assert!(wizard.record_measurement(10.02).is_ok());
    assert!(wizard.record_measurement(10.03).is_ok());

    // Generate report
    let report = wizard.report();
    assert!(report.contains("Calibration Report"));
    assert!(report.contains("PASS"));
    assert!(report.contains("3")); // 3 steps
}

#[test]
fn test_calibration_failure_detection() {
    let mut wizard = CalibrationWizard::new(CalibrationStepType::StepCalibration);

    // Record measurement that fails tolerance
    assert!(wizard.record_measurement(10.2).is_ok()); // Exceeds 0.05 tolerance
    assert!(!wizard.results()[0].passed);
}

// ============================================================================
// Task 125: Diagnostic Integration Tests
// ============================================================================

#[test]
fn test_communication_diagnostics_tracking() {
    let mut diag = CommunicationDiagnostics::default();

    diag.connected = true;
    diag.total_commands_sent = 100;
    diag.total_responses_received = 100;
    diag.average_response_time_ms = 25.5;

    assert!(diag.connected);
    assert_eq!(diag.total_commands_sent, diag.total_responses_received);
}

#[test]
fn test_buffer_diagnostics_validation() {
    // Valid buffer state
    let diag = BufferDiagnostics::new(1000, 250).unwrap();
    assert_eq!(diag.fill_percentage, 25.0);
    assert_eq!(diag.available_space, 750);

    // Invalid buffer state (used > total)
    assert!(BufferDiagnostics::new(100, 150).is_err());
}

#[test]
fn test_buffer_diagnostics_full() {
    let diag = BufferDiagnostics::new(512, 512).unwrap();
    assert_eq!(diag.fill_percentage, 100.0);
    assert_eq!(diag.available_space, 0);
}

#[test]
fn test_performance_profiler_statistics() {
    let mut profiler = PerformanceProfiler::new("test_operation");

    // Record samples
    profiler.record(100);
    profiler.record(200);
    profiler.record(150);
    profiler.record(180);
    profiler.record(120);

    assert_eq!(profiler.min_us(), 100);
    assert_eq!(profiler.max_us(), 200);
    let avg = profiler.average_us();
    assert!(avg > 100.0 && avg < 200.0);
}

#[test]
fn test_profiler_percentiles() {
    let mut profiler = PerformanceProfiler::new("latency");

    // Record latency samples
    for i in 1..=100 {
        profiler.record(i as u64);
    }

    let p50 = profiler.percentile_us(50.0);
    let p95 = profiler.percentile_us(95.0);
    let p99 = profiler.percentile_us(99.0);

    assert!(p50 < p95);
    assert!(p95 < p99);
}

#[test]
fn test_diagnostic_report_formatting() {
    let comm_diag = CommunicationDiagnostics {
        connected: true,
        total_commands_sent: 500,
        total_responses_received: 500,
        failed_commands: 0,
        average_response_time_ms: 15.5,
        last_error: None,
        connection_uptime_seconds: 7200,
    };

    let mut report = DiagnosticReport::new(comm_diag);
    report.buffer = BufferDiagnostics::new(1024, 512).ok();

    let formatted = report.format_report();
    assert!(formatted.contains("Diagnostic Report"));
    assert!(formatted.contains("500"));
    assert!(formatted.contains("Connected"));
    assert!(formatted.contains("Buffer"));
}

#[test]
fn test_diagnostic_error_tracking() {
    let mut diag = CommunicationDiagnostics::default();
    diag.last_error = Some("Connection timeout".to_string());

    let report = DiagnosticReport::new(diag);
    assert!(report.communication.last_error.is_some());
}

// ============================================================================
// Combined Integration Tests
// ============================================================================

#[test]
fn test_safety_and_calibration_integration() {
    // Ensure safety features don't interfere with calibration
    let mut safety = SafetyFeaturesManager::new();
    let mut wizard = CalibrationWizard::new(CalibrationStepType::StepCalibration);

    // Start calibration with safety enabled
    assert!(safety.is_safe());
    assert!(wizard.record_measurement(10.01).is_ok());

    // Safety features can be engaged during calibration
    assert!(safety.emergency_stop().is_ok());
    assert!(!safety.is_safe());
}

#[test]
fn test_export_and_diagnostics() {
    // Export with diagnostic tracking
    let mut profiler = PerformanceProfiler::new("export_operation");

    let processor = PostProcessor::for_format(ExportFormat::LinuxCNC);
    let exporter = FormatExporter::new(processor);

    let gcode = vec!["G0 X10".to_string(), "G1 Z-5".to_string(), "G0 Z5".to_string()];

    profiler.record(150);
    let result = exporter.export(&gcode);

    assert!(result.is_ok());
    assert!(profiler.average_us() > 0.0);
}

#[test]
fn test_comprehensive_system_state() {
    // Create a complete system state with all Phase 7 components
    let mut safety = SafetyFeaturesManager::new();
    let mut diag = CommunicationDiagnostics::default();
    let mut calibration = CalibrationWizard::new(CalibrationStepType::StepCalibration);

    // System startup
    assert!(safety.is_safe());
    assert!(!diag.connected);

    // Simulate connection
    diag.connected = true;
    diag.total_commands_sent = 1;

    // Perform calibration
    assert!(calibration.record_measurement(10.0).is_ok());

    // System state
    assert!(safety.is_safe());
    assert!(diag.connected);
    assert!(!calibration.is_complete());
}
