//! GTools: Quick G-code generators for common operations
//!
//! Provides a framework for creating parametric G-code generators (GTools)
//! that quickly generate G-code for standard operations like rectangles,
//! circles, lines, and specialized tools like image-to-laser and jigsaw patterns.

use std::collections::HashMap;
use std::fmt;

/// Parameter type for tool inputs
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterValue {
    /// Integer parameter (counts, discrete values)
    Integer(i32),
    /// Floating-point parameter (dimensions, speeds)
    Float(f64),
    /// Text/string parameter (names, modes)
    String(String),
    /// Boolean parameter (flags, toggles)
    Boolean(bool),
}

impl fmt::Display for ParameterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{:.2}", v),
            Self::String(v) => write!(f, "{}", v),
            Self::Boolean(v) => write!(f, "{}", v),
        }
    }
}

/// Parameter specification with validation rules
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Parameter name (unique within tool)
    pub name: String,
    /// Display label for UI
    pub label: String,
    /// Parameter description/help text
    pub description: String,
    /// Default value
    pub default: ParameterValue,
    /// Minimum value (for numeric parameters)
    pub min: Option<f64>,
    /// Maximum value (for numeric parameters)
    pub max: Option<f64>,
    /// Valid string choices (for enum-like parameters)
    pub choices: Option<Vec<String>>,
    /// Unit of measurement (mm, %, rpm, etc.)
    pub unit: Option<String>,
}

impl Parameter {
    /// Create new parameter
    pub fn new(name: impl Into<String>, label: impl Into<String>, default: ParameterValue) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            description: String::new(),
            default,
            min: None,
            max: None,
            choices: None,
            unit: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set min/max range
    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self
    }

    /// Set unit
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set valid choices
    pub fn with_choices(mut self, choices: Vec<String>) -> Self {
        self.choices = Some(choices);
        self
    }

    /// Validate parameter value
    pub fn validate(&self, value: &ParameterValue) -> Result<(), String> {
        match (&self.default, value) {
            (ParameterValue::Integer(_), ParameterValue::Integer(v)) => {
                if let (Some(min), Some(max)) = (self.min, self.max) {
                    let v_f = *v as f64;
                    if v_f < min || v_f > max {
                        return Err(format!(
                            "Value {} out of range [{}, {}]",
                            v, min as i32, max as i32
                        ));
                    }
                }
                Ok(())
            }
            (ParameterValue::Float(_), ParameterValue::Float(v)) => {
                if let (Some(min), Some(max)) = (self.min, self.max) {
                    if *v < min || *v > max {
                        return Err(format!(
                            "Value {:.2} out of range [{:.2}, {:.2}]",
                            v, min, max
                        ));
                    }
                }
                Ok(())
            }
            (ParameterValue::String(_), ParameterValue::String(v)) => {
                if let Some(ref choices) = self.choices {
                    if !choices.contains(v) {
                        return Err(format!(
                            "Invalid choice '{}'. Valid options: {}",
                            v,
                            choices.join(", ")
                        ));
                    }
                }
                Ok(())
            }
            (ParameterValue::Boolean(_), ParameterValue::Boolean(_)) => Ok(()),
            _ => Err(format!(
                "Type mismatch for parameter '{}'",
                self.name
            )),
        }
    }
}

/// Result of G-code generation
#[derive(Debug, Clone)]
pub struct GcodeOutput {
    /// Generated G-code
    pub gcode: String,
    /// Estimated machining time in seconds
    pub estimated_time: Option<f64>,
    /// Bounding box of the operation
    pub bounding_box: Option<BoundingBox>,
    /// Warnings or notes
    pub warnings: Vec<String>,
    /// Metadata about the operation
    pub metadata: HashMap<String, String>,
}

impl GcodeOutput {
    /// Create new G-code output
    pub fn new(gcode: impl Into<String>) -> Self {
        Self {
            gcode: gcode.into(),
            estimated_time: None,
            bounding_box: None,
            warnings: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add estimated time
    pub fn with_time(mut self, time: f64) -> Self {
        self.estimated_time = Some(time);
        self
    }

    /// Add bounding box
    pub fn with_bounding_box(mut self, bbox: BoundingBox) -> Self {
        self.bounding_box = Some(bbox);
        self
    }

    /// Add warning
    pub fn add_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Bounding box for G-code operation
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl BoundingBox {
    /// Create new bounding box
    pub fn new(min_x: f64, max_x: f64, min_y: f64, max_y: f64, min_z: f64, max_z: f64) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        }
    }

    /// Get width
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Get height
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Get depth
    pub fn depth(&self) -> f64 {
        self.max_z - self.min_z
    }
}

/// Trait for G-code generators (GTools)
pub trait GcodeGenerator: Send + Sync {
    /// Tool name
    fn name(&self) -> &str;

    /// Tool description
    fn description(&self) -> &str;

    /// Get tool parameters
    fn parameters(&self) -> Vec<Parameter>;

    /// Generate G-code from parameters
    fn generate(&self, params: &HashMap<String, ParameterValue>) -> Result<GcodeOutput, String>;

    /// Validate parameters before generation
    fn validate_parameters(&self, params: &HashMap<String, ParameterValue>) -> Result<(), String> {
        for param in self.parameters() {
            if let Some(value) = params.get(&param.name) {
                param.validate(value)?;
            } else if !matches!(param.default, ParameterValue::Boolean(_)) {
                // All non-boolean parameters are required
                return Err(format!("Missing required parameter: {}", param.name));
            }
        }
        Ok(())
    }
}

/// Registry of available GTools
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn GcodeGenerator>>,
}

impl ToolRegistry {
    /// Create new tool registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn GcodeGenerator>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get tool by name
    pub fn get(&self, name: &str) -> Option<&dyn GcodeGenerator> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// List all available tools
    pub fn list_tools(&self) -> Vec<&str> {
        self.tools.keys().map(|k| k.as_str()).collect()
    }

    /// Get tool count
    pub fn count(&self) -> usize {
        self.tools.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_integer() {
        let param = Parameter::new("width", "Width", ParameterValue::Integer(100))
            .with_range(10.0, 500.0)
            .with_unit("mm");

        assert_eq!(param.name, "width");
        assert_eq!(param.default, ParameterValue::Integer(100));
        assert_eq!(param.unit, Some("mm".to_string()));

        assert!(param.validate(&ParameterValue::Integer(50)).is_ok());
        assert!(param.validate(&ParameterValue::Integer(1000)).is_err());
    }

    #[test]
    fn test_parameter_float() {
        let param = Parameter::new("depth", "Cut Depth", ParameterValue::Float(3.0))
            .with_range(0.5, 10.0)
            .with_unit("mm");

        assert!(param.validate(&ParameterValue::Float(2.5)).is_ok());
        assert!(param.validate(&ParameterValue::Float(15.0)).is_err());
    }

    #[test]
    fn test_parameter_string_choices() {
        let param = Parameter::new("mode", "Mode", ParameterValue::String("contour".to_string()))
            .with_choices(vec![
                "contour".to_string(),
                "pocket".to_string(),
                "engrave".to_string(),
            ]);

        assert!(param.validate(&ParameterValue::String("pocket".to_string())).is_ok());
        assert!(param.validate(&ParameterValue::String("invalid".to_string())).is_err());
    }

    #[test]
    fn test_gcode_output() {
        let output = GcodeOutput::new("G0 X0 Y0\nG1 Z-5 F100")
            .with_time(10.5)
            .add_warning("Fast cutting")
            .with_metadata("operation", "pocket");

        assert_eq!(output.gcode, "G0 X0 Y0\nG1 Z-5 F100");
        assert_eq!(output.estimated_time, Some(10.5));
        assert_eq!(output.warnings.len(), 1);
        assert_eq!(output.metadata.get("operation"), Some(&"pocket".to_string()));
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(0.0, 100.0, 0.0, 80.0, 0.0, 5.0);

        assert_eq!(bbox.width(), 100.0);
        assert_eq!(bbox.height(), 80.0);
        assert_eq!(bbox.depth(), 5.0);
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();

        // Create a simple test tool
        struct TestTool;
        impl GcodeGenerator for TestTool {
            fn name(&self) -> &str {
                "test"
            }
            fn description(&self) -> &str {
                "Test tool"
            }
            fn parameters(&self) -> Vec<Parameter> {
                vec![]
            }
            fn generate(
                &self,
                _params: &HashMap<String, ParameterValue>,
            ) -> Result<GcodeOutput, String> {
                Ok(GcodeOutput::new("G0 X0 Y0"))
            }
        }

        registry.register(Box::new(TestTool));

        assert_eq!(registry.count(), 1);
        assert!(registry.get("test").is_some());
        assert_eq!(registry.list_tools(), vec!["test"]);
    }
}
