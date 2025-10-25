//! Integration tests for Tasks 97-102
//!
//! Task 97: File Validation UI
//! Task 98: File Comparison
//! Task 99: Backup and Recovery
//! Task 100: File Templates
//! Task 101: Probing - Basic
//! Task 102: Probing - Advanced

use gcodekit4::utils::{
    AdvancedProber, BasicProber, BackupManager, FileComparison, GcodeTemplate, TemplateLibrary,
    TemplateVariable, ValidationIssue, ValidationResult, ValidationSeverity, ProbePoint,
};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// TASK 97: FILE VALIDATION UI TESTS
// ============================================================================

#[test]
fn test_task_97_validation_issue_creation() {
    let issue = ValidationIssue::new(1, ValidationSeverity::Error, "Invalid command");
    assert_eq!(issue.line_number, 1);
    assert_eq!(issue.severity, ValidationSeverity::Error);
    assert_eq!(issue.message, "Invalid command");
}

#[test]
fn test_task_97_validation_issue_with_suggestion() {
    let issue = ValidationIssue::new(5, ValidationSeverity::Warning, "Missing feedrate")
        .with_suggestion("Add F100 parameter");

    assert!(issue.suggestion.is_some());
    assert_eq!(issue.suggestion.unwrap(), "Add F100 parameter");
}

#[test]
fn test_task_97_validation_result_single_issue() {
    let mut result = ValidationResult::new();
    result.add_issue(ValidationIssue::new(1, ValidationSeverity::Error, "Error"));

    assert_eq!(result.error_count, 1);
    assert_eq!(result.warning_count, 0);
    assert!(!result.is_valid());
}

#[test]
fn test_task_97_validation_result_multiple_issues() {
    let mut result = ValidationResult::new();
    result.add_issue(ValidationIssue::new(1, ValidationSeverity::Error, "Error 1"));
    result.add_issue(ValidationIssue::new(2, ValidationSeverity::Error, "Error 2"));
    result.add_issue(ValidationIssue::new(3, ValidationSeverity::Warning, "Warning"));
    result.add_issue(ValidationIssue::new(4, ValidationSeverity::Info, "Info"));

    assert_eq!(result.error_count, 2);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.info_count, 1);
    assert_eq!(result.issues.len(), 4);
}

#[test]
fn test_task_97_validation_result_summary() {
    let mut result = ValidationResult::new();
    result.add_issue(ValidationIssue::new(1, ValidationSeverity::Error, "Error"));
    result.add_issue(ValidationIssue::new(2, ValidationSeverity::Warning, "Warning"));

    let summary = result.summary();
    assert!(summary.contains("Errors: 1"));
    assert!(summary.contains("Warnings: 1"));
}

#[test]
fn test_task_97_issues_at_line() {
    let mut result = ValidationResult::new();
    result.add_issue(ValidationIssue::new(5, ValidationSeverity::Error, "Error 1"));
    result.add_issue(ValidationIssue::new(5, ValidationSeverity::Warning, "Warning"));
    result.add_issue(ValidationIssue::new(6, ValidationSeverity::Error, "Error 2"));

    let issues_at_5 = result.issues_at_line(5);
    assert_eq!(issues_at_5.len(), 2);

    let issues_at_6 = result.issues_at_line(6);
    assert_eq!(issues_at_6.len(), 1);
}

#[test]
fn test_task_97_issues_by_severity() {
    let mut result = ValidationResult::new();
    result.add_issue(ValidationIssue::new(1, ValidationSeverity::Error, "Error"));
    result.add_issue(ValidationIssue::new(2, ValidationSeverity::Error, "Error"));
    result.add_issue(ValidationIssue::new(3, ValidationSeverity::Warning, "Warning"));

    let errors = result.issues_by_severity(ValidationSeverity::Error);
    assert_eq!(errors.len(), 2);

    let warnings = result.issues_by_severity(ValidationSeverity::Warning);
    assert_eq!(warnings.len(), 1);
}

// ============================================================================
// TASK 98: FILE COMPARISON TESTS
// ============================================================================

#[test]
fn test_task_98_basic_comparison() {
    let original = "G0 X10 Y20\nG1 Z5 F100\n";
    let processed = "G0 X10 Y20\nG1 Z5 F100\n";

    let comparison = FileComparison::new(original, processed);
    assert_eq!(comparison.added_count, 0);
    assert_eq!(comparison.removed_count, 0);
    assert_eq!(comparison.modified_count, 0);
}

#[test]
fn test_task_98_modified_lines() {
    let original = "G0 X10 Y20\nG1 Z5 F100\n";
    let processed = "G0 X10 Y20\nG1 Z5 F50\n";

    let comparison = FileComparison::new(original, processed);
    assert_eq!(comparison.modified_count, 1);
    assert_eq!(comparison.total_changes(), 1);
}

#[test]
fn test_task_98_added_lines() {
    let original = "G0 X10\n";
    let processed = "G0 X10\nG1 Y20\n";

    let comparison = FileComparison::new(original, processed);
    assert_eq!(comparison.added_count, 1);
}

#[test]
fn test_task_98_removed_lines() {
    let original = "G0 X10\nG1 Y20\n";
    let processed = "G0 X10\n";

    let comparison = FileComparison::new(original, processed);
    assert_eq!(comparison.removed_count, 1);
}

#[test]
fn test_task_98_comparison_summary() {
    let original = "G0 X10\nG1 Y20\nG1 Z5\n";
    let processed = "G0 X10\nG1 Y25\n";

    let comparison = FileComparison::new(original, processed);
    let summary = comparison.summary();

    assert!(summary.contains("Added: 0"));
    assert!(summary.contains("Modified: 1"));
    assert!(summary.contains("Removed: 1"));
}

#[test]
fn test_task_98_change_percentage() {
    let original = "G0 X10\nG1 Y20\nG1 Z5\n";
    let processed = "G0 X10\nG1 Y25\n"; // 1 modified, 1 removed

    let comparison = FileComparison::new(original, processed);
    let percentage = comparison.change_percentage();

    // Total changes = 2, max lines = 3, so 2/3 = 66.67%
    assert!(percentage > 60.0 && percentage < 70.0);
}

// ============================================================================
// TASK 99: BACKUP AND RECOVERY TESTS
// ============================================================================

#[test]
fn test_task_99_backup_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let source = temp_dir.path().join("source.nc");
    fs::write(&source, "G0 X10\nG1 Y20\n").expect("Failed to write source");

    let backup_dir = temp_dir.path().join("backups");
    let manager = BackupManager::new(&backup_dir);

    let backup = manager.backup(&source, "Test backup").expect("Backup failed");

    assert!(backup.backup_path.exists());
    assert_eq!(backup.description, "Test backup");
}

#[test]
fn test_task_99_backup_restore() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let source = temp_dir.path().join("source.nc");
    fs::write(&source, "G0 X10\nG1 Y20\n").expect("Failed to write source");

    let backup_dir = temp_dir.path().join("backups");
    let manager = BackupManager::new(&backup_dir);

    let backup = manager.backup(&source, "Test").expect("Backup failed");

    let restored = temp_dir.path().join("restored.nc");
    manager.restore(&backup, &restored).expect("Restore failed");

    assert!(restored.exists());
    let content = fs::read_to_string(&restored).expect("Failed to read restored");
    assert_eq!(content, "G0 X10\nG1 Y20\n");
}

#[test]
fn test_task_99_backup_listing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let source = temp_dir.path().join("source.nc");
    fs::write(&source, "test").expect("Failed to write");

    let backup_dir = temp_dir.path().join("backups");
    let manager = BackupManager::new(&backup_dir);

    manager.backup(&source, "Backup 1").expect("Backup 1 failed");
    std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
    manager.backup(&source, "Backup 2").expect("Backup 2 failed");

    let backups = manager.list_backups().expect("List failed");
    assert!(backups.len() >= 1); // At least one backup
}

#[test]
fn test_task_99_backup_cleanup() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let source = temp_dir.path().join("source.nc");
    fs::write(&source, "test").expect("Failed to write");

    let backup_dir = temp_dir.path().join("backups");
    let mut manager = BackupManager::new(&backup_dir);
    manager.set_max_backups(2);

    manager.backup(&source, "1").expect("Backup 1 failed");
    manager.backup(&source, "2").expect("Backup 2 failed");
    manager.backup(&source, "3").expect("Backup 3 failed");

    manager.cleanup().expect("Cleanup failed");

    let backups = manager.list_backups().expect("List failed");
    assert!(backups.len() <= 2);
}

// ============================================================================
// TASK 100: FILE TEMPLATES TESTS
// ============================================================================

#[test]
fn test_task_100_template_creation() {
    let template = GcodeTemplate::new("move", "Move Template", "G0 X10 Y20");

    assert_eq!(template.id, "move");
    assert_eq!(template.name, "Move Template");
    assert!(template.content.contains("G0"));
}

#[test]
fn test_task_100_template_variables() {
    let mut template = GcodeTemplate::new("move", "Move", "G0 X{{X}} Y{{Y}}");

    template.add_variable(TemplateVariable {
        name: "X".to_string(),
        description: "X position".to_string(),
        default_value: "0".to_string(),
    });

    assert_eq!(template.variables.len(), 1);
    assert_eq!(template.variables[0].name, "X");
}

#[test]
fn test_task_100_template_expansion() {
    let mut template = GcodeTemplate::new("move", "Move", "G0 X{{X}} Y{{Y}} Z{{Z}}");

    template.add_variable(TemplateVariable {
        name: "X".to_string(),
        description: "X".to_string(),
        default_value: "0".to_string(),
    });
    template.add_variable(TemplateVariable {
        name: "Y".to_string(),
        description: "Y".to_string(),
        default_value: "0".to_string(),
    });
    template.add_variable(TemplateVariable {
        name: "Z".to_string(),
        description: "Z".to_string(),
        default_value: "0".to_string(),
    });

    let mut values = HashMap::new();
    values.insert("X".to_string(), "10".to_string());
    values.insert("Y".to_string(), "20".to_string());
    values.insert("Z".to_string(), "5".to_string());

    let expanded = template.expand(&values);

    assert!(expanded.contains("X10"));
    assert!(expanded.contains("Y20"));
    assert!(expanded.contains("Z5"));
}

#[test]
fn test_task_100_template_defaults() {
    let mut template = GcodeTemplate::new("move", "Move", "G0 X{{X}} Y{{Y}}");

    template.add_variable(TemplateVariable {
        name: "X".to_string(),
        description: "X".to_string(),
        default_value: "5".to_string(),
    });
    template.add_variable(TemplateVariable {
        name: "Y".to_string(),
        description: "Y".to_string(),
        default_value: "10".to_string(),
    });

    let values = HashMap::new();
    let expanded = template.expand(&values);

    assert!(expanded.contains("X5"));
    assert!(expanded.contains("Y10"));
}

#[test]
fn test_task_100_template_library() {
    let mut library = TemplateLibrary::new();

    let template1 = GcodeTemplate::new("move", "Move", "G0 X10");
    let template2 = GcodeTemplate::new("feed", "Feed", "G1 X20 F100");

    library.add(template1);
    library.add(template2);

    assert_eq!(library.list().len(), 2);
    assert!(library.get("move").is_some());
    assert!(library.get("feed").is_some());
}

#[test]
fn test_task_100_template_removal() {
    let mut library = TemplateLibrary::new();
    let template = GcodeTemplate::new("move", "Move", "G0 X10");
    library.add(template);

    assert!(library.get("move").is_some());

    let removed = library.remove("move");
    assert!(removed.is_some());
    assert!(library.get("move").is_none());
}

// ============================================================================
// TASK 101: PROBING - BASIC TESTS
// ============================================================================

#[test]
fn test_task_101_basic_prober_creation() {
    let prober = BasicProber::new();

    assert_eq!(prober.current_z, 0.0);
    assert_eq!(prober.probe_offset, 0.0);
    assert_eq!(prober.probe_feed_rate, 50.0);
}

#[test]
fn test_task_101_probe_command_generation() {
    let prober = BasicProber::new();
    let command = prober.generate_probe_command(-10.0);

    assert!(command.contains("G38.2"));
    assert!(command.contains("Z-10"));
    assert!(command.contains("F50"));
}

#[test]
fn test_task_101_offset_calculation() {
    let prober = BasicProber {
        current_z: 0.0,
        probe_offset: 10.0,
        probe_feed_rate: 50.0,
    };

    let probed_z = 5.0;
    let offset = prober.calculate_offset(probed_z);

    assert_eq!(offset, -5.0);
}

#[test]
fn test_task_101_offset_command() {
    let prober = BasicProber::new();
    let command = prober.generate_offset_command(2.5);

    assert!(command.contains("G92"));
    assert!(command.contains("Z2.5"));
}

#[test]
fn test_task_101_probe_workflow() {
    let mut prober = BasicProber::new();
    prober.probe_offset = 10.0;

    let probe_cmd = prober.generate_probe_command(-50.0);
    assert!(probe_cmd.contains("G38.2"));

    let offset = prober.calculate_offset(15.0);
    let offset_cmd = prober.generate_offset_command(offset);
    assert!(offset_cmd.contains("G92"));
}

// ============================================================================
// TASK 102: PROBING - ADVANCED TESTS
// ============================================================================

#[test]
fn test_task_102_advanced_prober_creation() {
    let prober = AdvancedProber::new();
    assert_eq!(prober.probe_points().len(), 0);
}

#[test]
fn test_task_102_probe_point_creation() {
    let point = ProbePoint::new(10.0, 20.0, 5.0);

    assert_eq!(point.x, 10.0);
    assert_eq!(point.y, 20.0);
    assert_eq!(point.z, 5.0);
}

#[test]
fn test_task_102_add_probe_points() {
    let mut prober = AdvancedProber::new();

    prober.add_probe_point(0.0, 0.0);
    prober.add_probe_point(10.0, 10.0);
    prober.add_probe_point(20.0, 20.0);

    assert_eq!(prober.probe_points().len(), 3);
}

#[test]
fn test_task_102_probe_sequence_generation() {
    let mut prober = AdvancedProber::new();
    prober.add_probe_point(0.0, 0.0);
    prober.add_probe_point(10.0, 10.0);

    let sequence = prober.generate_probe_sequence();

    assert!(sequence.contains("Multi-point"));
    assert!(sequence.contains("G0 X0 Y0"));
    assert!(sequence.contains("G0 X10 Y10"));
}

#[test]
fn test_task_102_corner_finding() {
    let prober = AdvancedProber::new();

    let sequence = prober.generate_corner_finding((0.0, 0.0), (10.0, 10.0));

    assert!(sequence.contains("Corner finding"));
    assert!(sequence.contains("G0 X0 Y0"));
    assert!(sequence.contains("G0 X10 Y10"));
}

#[test]
fn test_task_102_center_finding() {
    let prober = AdvancedProber::new();

    let sequence = prober.generate_center_finding((10.0, 10.0), 5.0);

    assert!(sequence.contains("Center finding"));
    assert!(sequence.contains("circular"));
}

#[test]
fn test_task_102_clear_probe_points() {
    let mut prober = AdvancedProber::new();
    prober.add_probe_point(0.0, 0.0);
    prober.add_probe_point(10.0, 10.0);

    assert_eq!(prober.probe_points().len(), 2);

    prober.clear_probe_points();
    assert_eq!(prober.probe_points().len(), 0);
}

#[test]
fn test_task_102_advanced_workflow() {
    let mut prober = AdvancedProber::new();

    // Add probe points
    prober.add_probe_point(0.0, 0.0);
    prober.add_probe_point(10.0, 0.0);
    prober.add_probe_point(10.0, 10.0);
    prober.add_probe_point(0.0, 10.0);

    // Generate sequence
    let sequence = prober.generate_probe_sequence();
    assert!(sequence.contains("Multi-point"));

    // Generate corner finding
    let corners = prober.generate_corner_finding((0.0, 0.0), (10.0, 10.0));
    assert!(corners.contains("Corner"));

    // Clear for next operation
    prober.clear_probe_points();
    assert_eq!(prober.probe_points().len(), 0);
}
