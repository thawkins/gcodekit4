//! # Parametric Design Module
//!
//! Provides parametric design system for creating reusable design templates with variable parameters.
//!
//! Parametric design allows users to define templates that generate designs based on parameters
//! such as dimensions, angles, and counts. Templates can be saved, shared, and quickly regenerated
//! with new parameter values.
//!
//! Supports:
//! - Multiple parameter types (number, integer, angle, distance)
//! - Parameter constraints and validation
//! - Template library storage
//! - Generator functions for shape creation

use std::collections::HashMap;
use anyhow::Result;

/// Parameter types for parametric design
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParameterType {
    /// Real number parameter
    Number,
    /// Integer parameter
    Integer,
    /// Angle in degrees
    Angle,
    /// Distance/length parameter
    Distance,
    /// Boolean parameter
    Boolean,
}

/// Parameter constraint for validation
#[derive(Debug, Clone)]
pub struct ParameterConstraint {
    /// Minimum value allowed
    pub min: f64,
    /// Maximum value allowed
    pub max: f64,
    /// Default value
    pub default: f64,
    /// Step size for UI sliders
    pub step: f64,
}

impl ParameterConstraint {
    /// Create new constraint
    pub fn new(min: f64, max: f64, default: f64, step: f64) -> Self {
        Self {
            min,
            max,
            default,
            step,
        }
    }

    /// Validate a value against this constraint
    pub fn validate(&self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }

    /// Clamp a value to this constraint
    pub fn clamp(&self, value: f64) -> f64 {
        value.max(self.min).min(self.max)
    }

    /// Check if constraint is valid
    pub fn is_valid(&self) -> bool {
        self.min <= self.max && self.default >= self.min && self.default <= self.max && self.step > 0.0
    }
}

/// Single parameter definition
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Constraint for this parameter
    pub constraint: ParameterConstraint,
    /// Description of the parameter
    pub description: String,
}

impl Parameter {
    /// Create new parameter
    pub fn new(
        name: String,
        param_type: ParameterType,
        constraint: ParameterConstraint,
        description: String,
    ) -> Self {
        Self {
            name,
            param_type,
            constraint,
            description,
        }
    }

    /// Validate a parameter value
    pub fn validate(&self, value: f64) -> bool {
        self.constraint.validate(value)
    }

    /// Get default value
    pub fn default_value(&self) -> f64 {
        self.constraint.default
    }

    /// Get constraint for this parameter
    pub fn constraint(&self) -> &ParameterConstraint {
        &self.constraint
    }
}

/// Parameter set for a design instance
#[derive(Debug, Clone)]
pub struct ParameterSet {
    /// Map of parameter name to value
    values: HashMap<String, f64>,
    /// Template ID this set is for
    pub template_id: String,
}

impl ParameterSet {
    /// Create new parameter set
    pub fn new(template_id: String) -> Self {
        Self {
            values: HashMap::new(),
            template_id,
        }
    }

    /// Set a parameter value
    pub fn set(&mut self, name: &str, value: f64) -> Result<()> {
        self.values.insert(name.to_string(), value);
        Ok(())
    }

    /// Get a parameter value
    pub fn get(&self, name: &str) -> Option<f64> {
        self.values.get(name).copied()
    }

    /// Get all values
    pub fn all_values(&self) -> &HashMap<String, f64> {
        &self.values
    }

    /// Clear all values
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Parameter count
    pub fn param_count(&self) -> usize {
        self.values.len()
    }
}

/// Parametric template for generating designs
#[derive(Debug, Clone)]
pub struct ParametricTemplate {
    /// Template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Parameters for this template
    pub parameters: Vec<Parameter>,
    /// Version of the template
    pub version: String,
    /// Author of the template
    pub author: String,
}

impl ParametricTemplate {
    /// Create new parametric template
    pub fn new(id: String, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            parameters: Vec::new(),
            version: "1.0".to_string(),
            author: "Unknown".to_string(),
        }
    }

    /// Add a parameter to the template
    pub fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters.push(parameter);
    }

    /// Get parameter by name
    pub fn get_parameter(&self, name: &str) -> Option<&Parameter> {
        self.parameters.iter().find(|p| p.name == name)
    }

    /// Validate parameter set against this template
    pub fn validate_parameters(&self, params: &ParameterSet) -> bool {
        for param in &self.parameters {
            if let Some(value) = params.get(&param.name) {
                if !param.validate(value) {
                    return false;
                }
            }
        }
        true
    }

    /// Get parameter count
    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    /// Create default parameter set for this template
    pub fn create_default_parameters(&self) -> ParameterSet {
        let mut set = ParameterSet::new(self.id.clone());
        for param in &self.parameters {
            let _ = set.set(&param.name, param.default_value());
        }
        set
    }
}

/// Template library for managing parametric templates
#[derive(Debug, Clone)]
pub struct TemplateLibrary {
    /// Map of template ID to template
    templates: HashMap<String, ParametricTemplate>,
    /// Category for organizing templates
    pub category: String,
}

impl TemplateLibrary {
    /// Create new template library
    pub fn new(category: String) -> Self {
        Self {
            templates: HashMap::new(),
            category,
        }
    }

    /// Add template to library
    pub fn add_template(&mut self, template: ParametricTemplate) -> Result<()> {
        if self.templates.contains_key(&template.id) {
            return Err(anyhow::anyhow!("Template with ID {} already exists", template.id));
        }
        self.templates.insert(template.id.clone(), template);
        Ok(())
    }

    /// Get template by ID
    pub fn get_template(&self, id: &str) -> Option<&ParametricTemplate> {
        self.templates.get(id)
    }

    /// Get mutable template by ID
    pub fn get_template_mut(&mut self, id: &str) -> Option<&mut ParametricTemplate> {
        self.templates.get_mut(id)
    }

    /// Remove template from library
    pub fn remove_template(&mut self, id: &str) -> Option<ParametricTemplate> {
        self.templates.remove(id)
    }

    /// Get all template IDs
    pub fn template_ids(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }

    /// Get template count
    pub fn template_count(&self) -> usize {
        self.templates.len()
    }

    /// List all templates
    pub fn list_templates(&self) -> Vec<(&str, &str, &str)> {
        self.templates
            .values()
            .map(|t| (t.id.as_str(), t.name.as_str(), t.description.as_str()))
            .collect()
    }
}

/// Parametric design generator
pub struct ParametricGenerator;

impl ParametricGenerator {
    /// Validate all parameters against template
    pub fn validate_all(
        template: &ParametricTemplate,
        params: &ParameterSet,
    ) -> Result<()> {
        for param in &template.parameters {
            if let Some(value) = params.get(&param.name) {
                if !param.validate(value) {
                    return Err(anyhow::anyhow!(
                        "Parameter {} value {} is out of bounds [{}, {}]",
                        param.name,
                        value,
                        param.constraint.min,
                        param.constraint.max
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Required parameter {} not found",
                    param.name
                ));
            }
        }
        Ok(())
    }

    /// Calculate design metrics based on parameters
    pub fn estimate_complexity(params: &ParameterSet) -> usize {
        params.param_count() * 10
    }

    /// Check if parameters changed significantly
    pub fn parameters_changed(set1: &ParameterSet, set2: &ParameterSet) -> bool {
        set1.all_values() != set2.all_values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_constraint_creation() {
        let constraint = ParameterConstraint::new(0.0, 100.0, 50.0, 1.0);
        assert_eq!(constraint.min, 0.0);
        assert_eq!(constraint.max, 100.0);
        assert_eq!(constraint.default, 50.0);
    }

    #[test]
    fn test_parameter_constraint_validation() {
        let constraint = ParameterConstraint::new(0.0, 100.0, 50.0, 1.0);
        
        assert!(constraint.validate(50.0));
        assert!(constraint.validate(0.0));
        assert!(constraint.validate(100.0));
        assert!(!constraint.validate(101.0));
        assert!(!constraint.validate(-1.0));
    }

    #[test]
    fn test_parameter_constraint_clamping() {
        let constraint = ParameterConstraint::new(0.0, 100.0, 50.0, 1.0);
        
        assert_eq!(constraint.clamp(50.0), 50.0);
        assert_eq!(constraint.clamp(150.0), 100.0);
        assert_eq!(constraint.clamp(-50.0), 0.0);
    }

    #[test]
    fn test_parameter_creation() {
        let constraint = ParameterConstraint::new(0.0, 100.0, 50.0, 1.0);
        let param = Parameter::new(
            "width".to_string(),
            ParameterType::Distance,
            constraint,
            "Box width".to_string(),
        );

        assert_eq!(param.name, "width");
        assert_eq!(param.param_type, ParameterType::Distance);
        assert_eq!(param.default_value(), 50.0);
    }

    #[test]
    fn test_parameter_set_creation() {
        let set = ParameterSet::new("box_template".to_string());
        assert_eq!(set.template_id, "box_template");
        assert_eq!(set.param_count(), 0);
    }

    #[test]
    fn test_parameter_set_values() {
        let mut set = ParameterSet::new("template1".to_string());
        
        let _ = set.set("width", 100.0);
        let _ = set.set("height", 50.0);
        
        assert_eq!(set.get("width"), Some(100.0));
        assert_eq!(set.get("height"), Some(50.0));
        assert_eq!(set.param_count(), 2);
    }

    #[test]
    fn test_parametric_template_creation() {
        let template = ParametricTemplate::new(
            "box".to_string(),
            "Box Template".to_string(),
            "Create a simple box".to_string(),
        );

        assert_eq!(template.id, "box");
        assert_eq!(template.parameter_count(), 0);
    }

    #[test]
    fn test_parametric_template_add_parameter() {
        let mut template = ParametricTemplate::new(
            "box".to_string(),
            "Box Template".to_string(),
            "Create a simple box".to_string(),
        );

        let constraint = ParameterConstraint::new(1.0, 1000.0, 100.0, 1.0);
        let param = Parameter::new(
            "width".to_string(),
            ParameterType::Distance,
            constraint,
            "Box width".to_string(),
        );

        template.add_parameter(param);
        assert_eq!(template.parameter_count(), 1);
    }

    #[test]
    fn test_parametric_template_get_parameter() {
        let mut template = ParametricTemplate::new(
            "box".to_string(),
            "Box Template".to_string(),
            "Create a simple box".to_string(),
        );

        let constraint = ParameterConstraint::new(1.0, 1000.0, 100.0, 1.0);
        let param = Parameter::new(
            "width".to_string(),
            ParameterType::Distance,
            constraint,
            "Box width".to_string(),
        );

        template.add_parameter(param);
        let retrieved = template.get_parameter("width");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_template_library_creation() {
        let library = TemplateLibrary::new("design_templates".to_string());
        assert_eq!(library.template_count(), 0);
    }

    #[test]
    fn test_template_library_add_template() {
        let mut library = TemplateLibrary::new("templates".to_string());
        let template = ParametricTemplate::new(
            "box".to_string(),
            "Box".to_string(),
            "Box template".to_string(),
        );

        let result = library.add_template(template);
        assert!(result.is_ok());
        assert_eq!(library.template_count(), 1);
    }

    #[test]
    fn test_template_library_duplicate_prevention() {
        let mut library = TemplateLibrary::new("templates".to_string());
        let template1 = ParametricTemplate::new(
            "box".to_string(),
            "Box".to_string(),
            "Box template".to_string(),
        );
        let template2 = ParametricTemplate::new(
            "box".to_string(),
            "Box".to_string(),
            "Box template".to_string(),
        );

        let _ = library.add_template(template1);
        let result = library.add_template(template2);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_library_get_template() {
        let mut library = TemplateLibrary::new("templates".to_string());
        let template = ParametricTemplate::new(
            "circle".to_string(),
            "Circle".to_string(),
            "Circle template".to_string(),
        );

        let _ = library.add_template(template);
        let retrieved = library.get_template("circle");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Circle");
    }

    #[test]
    fn test_template_library_remove_template() {
        let mut library = TemplateLibrary::new("templates".to_string());
        let template = ParametricTemplate::new(
            "square".to_string(),
            "Square".to_string(),
            "Square template".to_string(),
        );

        let _ = library.add_template(template);
        assert_eq!(library.template_count(), 1);

        let removed = library.remove_template("square");
        assert!(removed.is_some());
        assert_eq!(library.template_count(), 0);
    }

    #[test]
    fn test_parametric_generator_validate() {
        let mut template = ParametricTemplate::new(
            "test".to_string(),
            "Test".to_string(),
            "Test template".to_string(),
        );

        let constraint = ParameterConstraint::new(0.0, 100.0, 50.0, 1.0);
        let param = Parameter::new(
            "value".to_string(),
            ParameterType::Number,
            constraint,
            "A value".to_string(),
        );
        template.add_parameter(param);

        let mut params = ParameterSet::new("test".to_string());
        let _ = params.set("value", 50.0);

        let result = ParametricGenerator::validate_all(&template, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parametric_template_default_parameters() {
        let mut template = ParametricTemplate::new(
            "box".to_string(),
            "Box".to_string(),
            "Box template".to_string(),
        );

        let constraint = ParameterConstraint::new(1.0, 100.0, 50.0, 1.0);
        let param = Parameter::new(
            "width".to_string(),
            ParameterType::Distance,
            constraint,
            "Width".to_string(),
        );
        template.add_parameter(param);

        let defaults = template.create_default_parameters();
        assert_eq!(defaults.get("width"), Some(50.0));
    }
}
