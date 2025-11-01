//! File Validation UI Panel - Task 97
//!
//! Displays file validation results, errors, warnings, and suggestions.
//! Integrates with utils::file_io::FileValidation to show validation status.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Info message
    Info,
    /// Warning - possible issue
    Warning,
    /// Error - file may not work
    Error,
    /// Critical - file won't work
    Critical,
}

impl std::fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationSeverity::Info => write!(f, "INFO"),
            ValidationSeverity::Warning => write!(f, "WARNING"),
            ValidationSeverity::Error => write!(f, "ERROR"),
            ValidationSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Validation issue with location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue severity
    pub severity: ValidationSeverity,
    /// Issue message
    pub message: String,
    /// Line number (0 for file-level issues)
    pub line_number: usize,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// File validation panel state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileValidationPanel {
    /// All validation issues
    pub issues: Vec<ValidationIssue>,
    /// Total lines validated
    pub total_lines: usize,
    /// Issues by severity
    pub severity_counts: HashMap<String, usize>,
    /// Whether validation passed (no critical/error issues)
    pub validation_passed: bool,
    /// Validation summary message
    pub summary: String,
}

impl FileValidationPanel {
    /// Create new validation panel
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            total_lines: 0,
            severity_counts: HashMap::new(),
            validation_passed: true,
            summary: "No file loaded".to_string(),
        }
    }

    /// Add validation issue
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        let severity_key = format!("{}", issue.severity);
        *self.severity_counts.entry(severity_key).or_insert(0) += 1;

        // Check if validation still passes
        if issue.severity == ValidationSeverity::Error
            || issue.severity == ValidationSeverity::Critical
        {
            self.validation_passed = false;
        }

        self.issues.push(issue);
    }

    /// Update from file validation results
    pub fn update_from_validation(
        &mut self,
        total_lines: usize,
        motion_commands: usize,
        has_spindle: bool,
        has_tool_change: bool,
        errors: Vec<String>,
    ) {
        self.total_lines = total_lines;
        self.issues.clear();
        self.severity_counts.clear();
        self.validation_passed = true;

        // Add info messages
        self.add_issue(ValidationIssue {
            severity: ValidationSeverity::Info,
            message: format!("File contains {} lines", total_lines),
            line_number: 0,
            suggestion: None,
        });

        self.add_issue(ValidationIssue {
            severity: ValidationSeverity::Info,
            message: format!("Motion commands: {}", motion_commands),
            line_number: 0,
            suggestion: None,
        });

        if has_spindle {
            self.add_issue(ValidationIssue {
                severity: ValidationSeverity::Info,
                message: "File uses spindle (M3/M4)".to_string(),
                line_number: 0,
                suggestion: None,
            });
        }

        if has_tool_change {
            self.add_issue(ValidationIssue {
                severity: ValidationSeverity::Warning,
                message: "File contains tool changes (M6)".to_string(),
                line_number: 0,
                suggestion: Some("Verify all tools are available in your tool library".to_string()),
            });
        }

        // Add error messages
        for (idx, error) in errors.iter().enumerate() {
            self.add_issue(ValidationIssue {
                severity: ValidationSeverity::Error,
                message: error.clone(),
                line_number: idx + 1,
                suggestion: Some("Check G-code syntax and parameters".to_string()),
            });
        }

        // Update summary
        self.update_summary();
    }

    /// Generate validation summary
    fn update_summary(&mut self) {
        if self.validation_passed {
            self.summary = format!(
                "✓ File validated: {} lines, {} issues",
                self.total_lines,
                self.issues.len()
            );
        } else {
            self.summary = format!(
                "✗ Validation failed: {} errors found",
                self.severity_counts.get("ERROR").unwrap_or(&0)
                    + self.severity_counts.get("CRITICAL").unwrap_or(&0)
            );
        }
    }

    /// Get issues filtered by severity
    pub fn get_issues_by_severity(&self, severity: ValidationSeverity) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|issue| issue.severity == severity)
            .collect()
    }

    /// Get critical issues
    pub fn get_critical_issues(&self) -> Vec<&ValidationIssue> {
        self.get_issues_by_severity(ValidationSeverity::Critical)
    }

    /// Get error issues
    pub fn get_error_issues(&self) -> Vec<&ValidationIssue> {
        self.get_issues_by_severity(ValidationSeverity::Error)
    }

    /// Get warning issues
    pub fn get_warning_issues(&self) -> Vec<&ValidationIssue> {
        self.get_issues_by_severity(ValidationSeverity::Warning)
    }

    /// Clear all issues
    pub fn clear(&mut self) {
        self.issues.clear();
        self.severity_counts.clear();
        self.validation_passed = true;
        self.summary = "No file loaded".to_string();
        self.total_lines = 0;
    }

    /// Export issues as formatted text
    pub fn export_as_text(&self) -> String {
        let mut output = String::new();
        output.push_str(&"File Validation Report\n".to_string());
        output.push_str(&"======================\n".to_string());
        output.push_str(&format!("Status: {}\n", self.summary));
        output.push_str(&format!("Total Lines: {}\n\n", self.total_lines));

        for issue in &self.issues {
            output.push_str(&format!(
                "[{}] Line {}: {}\n",
                issue.severity, issue.line_number, issue.message
            ));
            if let Some(ref suggestion) = issue.suggestion {
                output.push_str(&format!("    Suggestion: {}\n", suggestion));
            }
        }

        output
    }
}

impl Default for FileValidationPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_panel_new() {
        let panel = FileValidationPanel::new();
        assert_eq!(panel.issues.len(), 0);
        assert!(panel.validation_passed);
    }

    #[test]
    fn test_add_issue() {
        let mut panel = FileValidationPanel::new();
        panel.add_issue(ValidationIssue {
            severity: ValidationSeverity::Warning,
            message: "Test warning".to_string(),
            line_number: 10,
            suggestion: None,
        });

        assert_eq!(panel.issues.len(), 1);
        assert_eq!(panel.severity_counts.get("WARNING").unwrap(), &1);
    }

    #[test]
    fn test_error_fails_validation() {
        let mut panel = FileValidationPanel::new();
        panel.add_issue(ValidationIssue {
            severity: ValidationSeverity::Error,
            message: "Test error".to_string(),
            line_number: 5,
            suggestion: None,
        });

        assert!(!panel.validation_passed);
    }

    #[test]
    fn test_get_issues_by_severity() {
        let mut panel = FileValidationPanel::new();
        panel.add_issue(ValidationIssue {
            severity: ValidationSeverity::Warning,
            message: "Warning 1".to_string(),
            line_number: 1,
            suggestion: None,
        });
        panel.add_issue(ValidationIssue {
            severity: ValidationSeverity::Error,
            message: "Error 1".to_string(),
            line_number: 2,
            suggestion: None,
        });
        panel.add_issue(ValidationIssue {
            severity: ValidationSeverity::Warning,
            message: "Warning 2".to_string(),
            line_number: 3,
            suggestion: None,
        });

        let warnings = panel.get_warning_issues();
        assert_eq!(warnings.len(), 2);

        let errors = panel.get_error_issues();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_update_from_validation() {
        let mut panel = FileValidationPanel::new();
        panel.update_from_validation(
            100,
            50,
            true,
            false,
            vec!["Invalid G-code on line 42".to_string()],
        );

        assert_eq!(panel.total_lines, 100);
        assert!(panel.issues.len() > 0);
        assert!(!panel.validation_passed);
    }

    #[test]
    fn test_export_as_text() {
        let mut panel = FileValidationPanel::new();
        panel.add_issue(ValidationIssue {
            severity: ValidationSeverity::Info,
            message: "Test message".to_string(),
            line_number: 5,
            suggestion: Some("Do this".to_string()),
        });

        let text = panel.export_as_text();
        assert!(text.contains("Test message"));
        assert!(text.contains("Do this"));
    }

    #[test]
    fn test_clear() {
        let mut panel = FileValidationPanel::new();
        panel.add_issue(ValidationIssue {
            severity: ValidationSeverity::Warning,
            message: "Test".to_string(),
            line_number: 1,
            suggestion: None,
        });

        panel.clear();
        assert_eq!(panel.issues.len(), 0);
        assert!(panel.validation_passed);
    }
}
