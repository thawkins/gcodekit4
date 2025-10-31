//! Designer to G-code Editor integration module
//!
//! Handles seamless integration between the Designer tool and the G-code Editor,
//! including G-code export, tab switching, and design tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a design export to the G-code Editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignExport {
    /// Unique ID for this export
    pub id: String,
    /// Design name
    pub name: String,
    /// Generated G-code
    pub gcode: String,
    /// Timestamp when exported
    pub timestamp: String,
    /// Design parameters used for generation
    pub parameters: ExportParameters,
    /// Source design ID (for tracking)
    pub source_design_id: Option<String>,
}

impl DesignExport {
    /// Create new design export
    pub fn new(
        name: String,
        gcode: String,
        parameters: ExportParameters,
    ) -> Self {
        let timestamp = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            gcode,
            timestamp,
            parameters,
            source_design_id: None,
        }
    }

    /// Get G-code length
    pub fn gcode_lines(&self) -> usize {
        self.gcode.lines().count()
    }

    /// Get G-code size in bytes
    pub fn gcode_size(&self) -> usize {
        self.gcode.len()
    }
}

/// Parameters used for G-code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportParameters {
    /// Tool diameter in mm
    pub tool_diameter: f64,
    /// Cut depth in mm
    pub cut_depth: f64,
    /// Feed rate in mm/min
    pub feed_rate: f64,
    /// Spindle speed in RPM
    pub spindle_speed: u32,
    /// Safe Z height in mm
    pub safe_z: f64,
}

impl Default for ExportParameters {
    fn default() -> Self {
        Self {
            tool_diameter: 3.0,
            cut_depth: 5.0,
            feed_rate: 500.0,
            spindle_speed: 12000,
            safe_z: 10.0,
        }
    }
}

/// Tracks the relationship between designs and exported G-code
pub struct DesignEditorIntegration {
    /// Exports from Designer to Editor
    exports: HashMap<String, DesignExport>,
    /// Map from design ID to export ID for tracking
    design_to_export: HashMap<String, String>,
    /// Recently exported designs (for quick access)
    recent_exports: Vec<String>,
    /// Maximum number of recent exports to track
    max_recent: usize,
}

impl DesignEditorIntegration {
    /// Create new integration manager
    pub fn new() -> Self {
        Self {
            exports: HashMap::new(),
            design_to_export: HashMap::new(),
            recent_exports: Vec::new(),
            max_recent: 10,
        }
    }

    /// Register a design export
    pub fn export_design(&mut self, design_id: Option<String>, export: DesignExport) -> String {
        let export_id = export.id.clone();

        if let Some(did) = &design_id {
            self.design_to_export.insert(did.clone(), export_id.clone());
        }

        self.exports.insert(export_id.clone(), export);

        // Track recent exports
        if self.recent_exports.len() >= self.max_recent {
            self.recent_exports.remove(0);
        }
        self.recent_exports.push(export_id.clone());

        export_id
    }

    /// Get export by ID
    pub fn get_export(&self, id: &str) -> Option<&DesignExport> {
        self.exports.get(id)
    }

    /// Get exports for a design
    pub fn get_design_exports(&self, design_id: &str) -> Vec<&DesignExport> {
        if let Some(export_id) = self.design_to_export.get(design_id) {
            if let Some(export) = self.exports.get(export_id) {
                return vec![export];
            }
        }
        Vec::new()
    }

    /// Get recent exports
    pub fn get_recent_exports(&self) -> Vec<&DesignExport> {
        self.recent_exports
            .iter()
            .rev()
            .filter_map(|id| self.exports.get(id))
            .collect()
    }

    /// Delete an export
    pub fn delete_export(&mut self, id: &str) -> bool {
        if self.exports.remove(id).is_some() {
            self.recent_exports.retain(|x| x != id);
            true
        } else {
            false
        }
    }

    /// Clear old exports
    pub fn clear_old_exports(&mut self, keep_count: usize) {
        let to_remove: Vec<_> = self
            .recent_exports
            .iter()
            .take(self.recent_exports.len().saturating_sub(keep_count))
            .cloned()
            .collect();

        for id in to_remove {
            self.delete_export(&id);
        }
    }

    /// Get statistics
    pub fn stats(&self) -> IntegrationStats {
        IntegrationStats {
            total_exports: self.exports.len(),
            recent_exports: self.recent_exports.len(),
            total_gcode_lines: self.exports.values().map(|e| e.gcode_lines()).sum(),
        }
    }
}

impl Default for DesignEditorIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for integration
#[derive(Debug, Clone)]
pub struct IntegrationStats {
    pub total_exports: usize,
    pub recent_exports: usize,
    pub total_gcode_lines: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_design_export_creation() {
        let params = ExportParameters::default();
        let export = DesignExport::new(
            "Test Design".to_string(),
            "G00 X0 Y0\n".to_string(),
            params,
        );

        assert_eq!(export.name, "Test Design");
        assert_eq!(export.gcode_lines(), 1);
    }

    #[test]
    fn test_integration_export_design() {
        let mut integration = DesignEditorIntegration::new();
        let params = ExportParameters::default();
        let export = DesignExport::new(
            "Test".to_string(),
            "G-code here".to_string(),
            params,
        );

        let export_id = integration.export_design(None, export);
        assert!(!export_id.is_empty());
        assert!(integration.get_export(&export_id).is_some());
    }

    #[test]
    fn test_integration_recent_exports() {
        let mut integration = DesignEditorIntegration::new();
        let params = ExportParameters::default();

        for i in 0..5 {
            let export = DesignExport::new(
                format!("Design {}", i),
                "G-code".to_string(),
                params.clone(),
            );
            integration.export_design(None, export);
        }

        let recent = integration.get_recent_exports();
        assert_eq!(recent.len(), 5);
    }

    #[test]
    fn test_integration_max_recent() {
        let mut integration = DesignEditorIntegration::new();
        integration.max_recent = 3;

        let params = ExportParameters::default();

        for i in 0..5 {
            let export = DesignExport::new(
                format!("Design {}", i),
                "G-code".to_string(),
                params.clone(),
            );
            integration.export_design(None, export);
        }

        let recent = integration.get_recent_exports();
        assert!(recent.len() <= 3);
    }

    #[test]
    fn test_integration_delete_export() {
        let mut integration = DesignEditorIntegration::new();
        let params = ExportParameters::default();
        let export = DesignExport::new(
            "Test".to_string(),
            "G-code".to_string(),
            params,
        );

        let export_id = integration.export_design(None, export);
        assert!(integration.get_export(&export_id).is_some());

        assert!(integration.delete_export(&export_id));
        assert!(integration.get_export(&export_id).is_none());
    }

    #[test]
    fn test_integration_stats() {
        let mut integration = DesignEditorIntegration::new();
        let params = ExportParameters::default();

        let export1 = DesignExport::new(
            "Design 1".to_string(),
            "G00 X0 Y0\nG01 X10 Y10\n".to_string(),
            params.clone(),
        );
        let export2 = DesignExport::new(
            "Design 2".to_string(),
            "G00 X0 Y0\n".to_string(),
            params,
        );

        integration.export_design(None, export1);
        integration.export_design(None, export2);

        let stats = integration.stats();
        assert_eq!(stats.total_exports, 2);
        assert_eq!(stats.total_gcode_lines, 3);
    }

    #[test]
    fn test_integration_design_tracking() {
        let mut integration = DesignEditorIntegration::new();
        let design_id = Some("design_123".to_string());
        let params = ExportParameters::default();
        let export = DesignExport::new(
            "Test".to_string(),
            "G-code".to_string(),
            params,
        );

        integration.export_design(design_id.clone(), export);

        let exports = integration.get_design_exports(design_id.as_ref().unwrap());
        assert_eq!(exports.len(), 1);
    }

    #[test]
    fn test_export_parameters_default() {
        let params = ExportParameters::default();
        assert_eq!(params.tool_diameter, 3.0);
        assert_eq!(params.cut_depth, 5.0);
        assert_eq!(params.feed_rate, 500.0);
        assert_eq!(params.spindle_speed, 12000);
        assert_eq!(params.safe_z, 10.0);
    }

    #[test]
    fn test_gcode_size() {
        let params = ExportParameters::default();
        let gcode = "G00 X0 Y0\nG01 X10 Y10\nM30\n".to_string();
        let export = DesignExport::new(
            "Test".to_string(),
            gcode.clone(),
            params,
        );

        assert_eq!(export.gcode_size(), gcode.len());
    }

    #[test]
    fn test_clear_old_exports() {
        let mut integration = DesignEditorIntegration::new();
        let params = ExportParameters::default();

        for i in 0..10 {
            let export = DesignExport::new(
                format!("Design {}", i),
                "G-code".to_string(),
                params.clone(),
            );
            integration.export_design(None, export);
        }

        assert_eq!(integration.stats().total_exports, 10);

        integration.clear_old_exports(5);
        assert_eq!(integration.stats().total_exports, 5);
    }
}
