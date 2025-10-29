//! Tests for testing::project_completion

use gcodekit4::testing::project_completion::*;

#[test]
fn test_test_suite() {
    let mut suite = TestSuite::new();
    suite.add_result(TestResult::pass("test1"));
    suite.add_result(TestResult::fail("test2", "Error"));
    assert_eq!(suite.passed, 1);
    assert_eq!(suite.failed, 1);
}

#[test]
fn test_documentation_section() {
    let section = DocumentationSection::new("Title", "Content");
    assert!(section.to_markdown(0).contains("Title"));
}

#[test]
fn test_code_quality_metrics() {
    let metrics = CodeQualityMetrics::new();
    assert!(metrics.quality_score() >= 0.0);
}

#[test]
fn test_project_milestone() {
    let mut milestone = ProjectMilestone::new("Phase 1", 10);
    milestone.complete_task();
    assert_eq!(milestone.completed, 1);
}

#[test]
fn test_release_checklist() {
    let mut checklist = ReleaseChecklist::new();
    checklist.add_item("Item 1");
    checklist.mark_complete("Item 1");
    assert!(checklist.all_complete());
}
