//! Basic geometry G-code generators for GTools
//!
//! Implements simple geometry generators: Rectangle, Circle, and Line.
//! These are optimized for GRBL and include proper Z-axis movement,
//! safe heights, and feed rate optimization.

use super::{GcodeGenerator, GcodeOutput, Parameter, ParameterValue, BoundingBox};
use std::collections::HashMap;

/// Rectangle contour/pocket generator
pub struct RectangleGenerator;

impl GcodeGenerator for RectangleGenerator {
    fn name(&self) -> &str {
        "rectangle"
    }

    fn description(&self) -> &str {
        "Generate G-code for a rectangular cut (contour or pocket)"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new("width", "Width", ParameterValue::Float(100.0))
                .with_description("Rectangle width in mm")
                .with_range(1.0, 10000.0)
                .with_unit("mm"),
            Parameter::new("height", "Height", ParameterValue::Float(80.0))
                .with_description("Rectangle height in mm")
                .with_range(1.0, 10000.0)
                .with_unit("mm"),
            Parameter::new("corner_radius", "Corner Radius", ParameterValue::Float(0.0))
                .with_description("Radius of corners (0 for sharp)")
                .with_range(0.0, 1000.0)
                .with_unit("mm"),
            Parameter::new("depth", "Cut Depth", ParameterValue::Float(5.0))
                .with_description("Total depth to cut")
                .with_range(0.1, 100.0)
                .with_unit("mm"),
            Parameter::new("mode", "Mode", ParameterValue::String("contour".to_string()))
                .with_description("Cut mode (contour or pocket)")
                .with_choices(vec!["contour".to_string(), "pocket".to_string()]),
            Parameter::new("feed_rate", "Feed Rate", ParameterValue::Float(1000.0))
                .with_description("Cutting feed rate")
                .with_range(10.0, 10000.0)
                .with_unit("mm/min"),
            Parameter::new("plunge_rate", "Plunge Rate", ParameterValue::Float(500.0))
                .with_description("Z-axis plunge rate")
                .with_range(10.0, 5000.0)
                .with_unit("mm/min"),
            Parameter::new("start_x", "Start X", ParameterValue::Float(0.0))
                .with_description("X position of rectangle center")
                .with_range(-10000.0, 10000.0)
                .with_unit("mm"),
            Parameter::new("start_y", "Start Y", ParameterValue::Float(0.0))
                .with_description("Y position of rectangle center")
                .with_range(-10000.0, 10000.0)
                .with_unit("mm"),
        ]
    }

    fn generate(&self, params: &HashMap<String, ParameterValue>) -> Result<GcodeOutput, String> {
        self.validate_parameters(params)?;

        let width = self.get_float(params, "width")?;
        let height = self.get_float(params, "height")?;
        let corner_radius = self.get_float(params, "corner_radius")?;
        let depth = self.get_float(params, "depth")?;
        let mode = self.get_string(params, "mode")?;
        let feed_rate = self.get_float(params, "feed_rate")?;
        let plunge_rate = self.get_float(params, "plunge_rate")?;
        let start_x = self.get_float(params, "start_x")?;
        let start_y = self.get_float(params, "start_y")?;

        // Calculate rectangle corners (centered on start_x, start_y)
        let left = start_x - width / 2.0;
        let right = start_x + width / 2.0;
        let top = start_y + height / 2.0;
        let bottom = start_y - height / 2.0;

        let mut gcode = String::new();
        gcode.push_str("; Rectangle Generator\n");
        gcode.push_str(&format!("; Mode: {}\n", mode));
        gcode.push_str(&format!("; Dimensions: {} x {} mm\n", width, height));
        gcode.push_str(&format!("; Depth: {} mm\n", depth));
        gcode.push('\n');

        // Safe Z height
        let safe_z = 5.0;

        // Header
        gcode.push_str("G21           ; Metric units\n");
        gcode.push_str("G90           ; Absolute positioning\n");
        gcode.push_str(&format!("G0 Z{:.3}    ; Safe Z height\n", safe_z));
        gcode.push_str(&format!("G0 X{:.3} Y{:.3}  ; Move to start\n", left + corner_radius, top - corner_radius));
        gcode.push('\n');

        // Generate path
        match mode.as_str() {
            "contour" => {
                gcode.push_str("; Contour cut\n");
                gcode.push_str(&format!("G1 Z{:.3} F{:.0}  ; Plunge\n", -depth, plunge_rate));
                gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}  ; Bottom left corner\n", left + corner_radius, bottom + corner_radius, feed_rate));
                gcode.push_str(&format!("G1 X{:.3} Y{:.3}  ; Bottom right corner\n", right - corner_radius, bottom + corner_radius));
                gcode.push_str(&format!("G1 X{:.3} Y{:.3}  ; Top right corner\n", right - corner_radius, top - corner_radius));
                gcode.push_str(&format!("G1 X{:.3} Y{:.3}  ; Top left corner\n", left + corner_radius, top - corner_radius));
                gcode.push_str(&format!("G1 X{:.3} Y{:.3}  ; Back to start\n", left + corner_radius, top - corner_radius));
            }
            "pocket" => {
                gcode.push_str("; Pocket cut\n");
                gcode.push_str(&format!("G1 Z{:.3} F{:.0}  ; Plunge\n", -depth, plunge_rate));
                gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}  ; Cut rectangle\n", left + corner_radius, bottom + corner_radius, feed_rate));
                gcode.push_str(&format!("G1 X{:.3} Y{:.3}\n", right - corner_radius, bottom + corner_radius));
                gcode.push_str(&format!("G1 X{:.3} Y{:.3}\n", right - corner_radius, top - corner_radius));
                gcode.push_str(&format!("G1 X{:.3} Y{:.3}\n", left + corner_radius, top - corner_radius));
                gcode.push_str(&format!("G1 X{:.3} Y{:.3}  ; Back to start\n", left + corner_radius, bottom + corner_radius));
            }
            _ => return Err(format!("Unknown mode: {}", mode)),
        }

        gcode.push('\n');
        gcode.push_str(&format!("G0 Z{:.3}    ; Retract\n", safe_z));
        gcode.push_str("M2           ; Program end\n");

        let estimated_time = self.estimate_time(&gcode, feed_rate);
        let bbox = BoundingBox::new(left - 1.0, right + 1.0, bottom - 1.0, top + 1.0, -depth, 0.0);

        Ok(GcodeOutput::new(gcode)
            .with_time(estimated_time)
            .with_bounding_box(bbox)
            .with_metadata("operation", "contour")
            .with_metadata("shape", "rectangle"))
    }
}

/// Circle contour/pocket generator
pub struct CircleGenerator;

impl GcodeGenerator for CircleGenerator {
    fn name(&self) -> &str {
        "circle"
    }

    fn description(&self) -> &str {
        "Generate G-code for a circular cut (contour or pocket)"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new("diameter", "Diameter", ParameterValue::Float(50.0))
                .with_description("Circle diameter in mm")
                .with_range(1.0, 10000.0)
                .with_unit("mm"),
            Parameter::new("depth", "Cut Depth", ParameterValue::Float(5.0))
                .with_description("Total depth to cut")
                .with_range(0.1, 100.0)
                .with_unit("mm"),
            Parameter::new("mode", "Mode", ParameterValue::String("contour".to_string()))
                .with_description("Cut mode (contour or pocket)")
                .with_choices(vec!["contour".to_string(), "pocket".to_string()]),
            Parameter::new("feed_rate", "Feed Rate", ParameterValue::Float(1000.0))
                .with_description("Cutting feed rate")
                .with_range(10.0, 10000.0)
                .with_unit("mm/min"),
            Parameter::new("plunge_rate", "Plunge Rate", ParameterValue::Float(500.0))
                .with_description("Z-axis plunge rate")
                .with_range(10.0, 5000.0)
                .with_unit("mm/min"),
            Parameter::new("start_x", "Center X", ParameterValue::Float(0.0))
                .with_description("X position of circle center")
                .with_range(-10000.0, 10000.0)
                .with_unit("mm"),
            Parameter::new("start_y", "Center Y", ParameterValue::Float(0.0))
                .with_description("Y position of circle center")
                .with_range(-10000.0, 10000.0)
                .with_unit("mm"),
        ]
    }

    fn generate(&self, params: &HashMap<String, ParameterValue>) -> Result<GcodeOutput, String> {
        self.validate_parameters(params)?;

        let diameter = self.get_float(params, "diameter")?;
        let depth = self.get_float(params, "depth")?;
        let mode = self.get_string(params, "mode")?;
        let feed_rate = self.get_float(params, "feed_rate")?;
        let plunge_rate = self.get_float(params, "plunge_rate")?;
        let center_x = self.get_float(params, "start_x")?;
        let center_y = self.get_float(params, "start_y")?;

        let radius = diameter / 2.0;

        let mut gcode = String::new();
        gcode.push_str("; Circle Generator\n");
        gcode.push_str(&format!("; Mode: {}\n", mode));
        gcode.push_str(&format!("; Diameter: {} mm\n", diameter));
        gcode.push_str(&format!("; Depth: {} mm\n", depth));
        gcode.push('\n');

        let safe_z = 5.0;

        gcode.push_str("G21           ; Metric units\n");
        gcode.push_str("G90           ; Absolute positioning\n");
        gcode.push_str(&format!("G0 Z{:.3}    ; Safe Z height\n", safe_z));
        gcode.push_str(&format!("G0 X{:.3} Y{:.3}  ; Move to circle edge\n", center_x + radius, center_y));
        gcode.push('\n');

        match mode.as_str() {
            "contour" => {
                gcode.push_str("; Contour cut\n");
                gcode.push_str(&format!("G1 Z{:.3} F{:.0}  ; Plunge\n", -depth, plunge_rate));
                gcode.push_str(&format!("G2 X{:.3} Y{:.3} I{:.3} J0 F{:.0}  ; Cut circle\n", 
                    center_x + radius, center_y, -radius, feed_rate));
            }
            "pocket" => {
                gcode.push_str("; Pocket cut\n");
                gcode.push_str(&format!("G1 Z{:.3} F{:.0}  ; Plunge\n", -depth, plunge_rate));
                gcode.push_str(&format!("G2 X{:.3} Y{:.3} I{:.3} J0 F{:.0}  ; Cut circle\n", 
                    center_x + radius, center_y, -radius, feed_rate));
            }
            _ => return Err(format!("Unknown mode: {}", mode)),
        }

        gcode.push('\n');
        gcode.push_str(&format!("G0 Z{:.3}    ; Retract\n", safe_z));
        gcode.push_str("M2           ; Program end\n");

        let estimated_time = self.estimate_time(&gcode, feed_rate);
        let bbox = BoundingBox::new(
            center_x - radius - 1.0,
            center_x + radius + 1.0,
            center_y - radius - 1.0,
            center_y + radius + 1.0,
            -depth,
            0.0,
        );

        Ok(GcodeOutput::new(gcode)
            .with_time(estimated_time)
            .with_bounding_box(bbox)
            .with_metadata("operation", "contour")
            .with_metadata("shape", "circle"))
    }
}

/// Line generator
pub struct LineGenerator;

impl GcodeGenerator for LineGenerator {
    fn name(&self) -> &str {
        "line"
    }

    fn description(&self) -> &str {
        "Generate G-code for a straight line cut"
    }

    fn parameters(&self) -> Vec<Parameter> {
        vec![
            Parameter::new("start_x", "Start X", ParameterValue::Float(0.0))
                .with_description("X start position")
                .with_range(-10000.0, 10000.0)
                .with_unit("mm"),
            Parameter::new("start_y", "Start Y", ParameterValue::Float(0.0))
                .with_description("Y start position")
                .with_range(-10000.0, 10000.0)
                .with_unit("mm"),
            Parameter::new("end_x", "End X", ParameterValue::Float(100.0))
                .with_description("X end position")
                .with_range(-10000.0, 10000.0)
                .with_unit("mm"),
            Parameter::new("end_y", "End Y", ParameterValue::Float(0.0))
                .with_description("Y end position")
                .with_range(-10000.0, 10000.0)
                .with_unit("mm"),
            Parameter::new("depth", "Cut Depth", ParameterValue::Float(5.0))
                .with_description("Total depth to cut")
                .with_range(0.1, 100.0)
                .with_unit("mm"),
            Parameter::new("feed_rate", "Feed Rate", ParameterValue::Float(1000.0))
                .with_description("Cutting feed rate")
                .with_range(10.0, 10000.0)
                .with_unit("mm/min"),
            Parameter::new("plunge_rate", "Plunge Rate", ParameterValue::Float(500.0))
                .with_description("Z-axis plunge rate")
                .with_range(10.0, 5000.0)
                .with_unit("mm/min"),
        ]
    }

    fn generate(&self, params: &HashMap<String, ParameterValue>) -> Result<GcodeOutput, String> {
        self.validate_parameters(params)?;

        let start_x = self.get_float(params, "start_x")?;
        let start_y = self.get_float(params, "start_y")?;
        let end_x = self.get_float(params, "end_x")?;
        let end_y = self.get_float(params, "end_y")?;
        let depth = self.get_float(params, "depth")?;
        let feed_rate = self.get_float(params, "feed_rate")?;
        let plunge_rate = self.get_float(params, "plunge_rate")?;

        let mut gcode = String::new();
        gcode.push_str("; Line Generator\n");
        gcode.push_str(&format!("; From ({:.1}, {:.1}) to ({:.1}, {:.1})\n", start_x, start_y, end_x, end_y));
        gcode.push_str(&format!("; Depth: {} mm\n", depth));
        gcode.push('\n');

        let safe_z = 5.0;

        gcode.push_str("G21           ; Metric units\n");
        gcode.push_str("G90           ; Absolute positioning\n");
        gcode.push_str(&format!("G0 Z{:.3}    ; Safe Z height\n", safe_z));
        gcode.push_str(&format!("G0 X{:.3} Y{:.3}  ; Move to start\n", start_x, start_y));
        gcode.push_str(&format!("G1 Z{:.3} F{:.0}  ; Plunge\n", -depth, plunge_rate));
        gcode.push_str(&format!("G1 X{:.3} Y{:.3} F{:.0}  ; Cut line\n", end_x, end_y, feed_rate));
        gcode.push('\n');
        gcode.push_str(&format!("G0 Z{:.3}    ; Retract\n", safe_z));
        gcode.push_str("M2           ; Program end\n");

        let estimated_time = self.estimate_time(&gcode, feed_rate);
        let min_x = start_x.min(end_x);
        let max_x = start_x.max(end_x);
        let min_y = start_y.min(end_y);
        let max_y = start_y.max(end_y);
        let bbox = BoundingBox::new(min_x - 1.0, max_x + 1.0, min_y - 1.0, max_y + 1.0, -depth, 0.0);

        Ok(GcodeOutput::new(gcode)
            .with_time(estimated_time)
            .with_bounding_box(bbox)
            .with_metadata("operation", "cut")
            .with_metadata("shape", "line"))
    }
}

// Helper methods for generators
impl RectangleGenerator {
    fn get_float(&self, params: &HashMap<String, ParameterValue>, key: &str) -> Result<f64, String> {
        match params.get(key) {
            Some(ParameterValue::Float(v)) => Ok(*v),
            Some(ParameterValue::Integer(v)) => Ok(*v as f64),
            _ => Err(format!("Missing or invalid float parameter: {}", key)),
        }
    }

    fn get_string(&self, params: &HashMap<String, ParameterValue>, key: &str) -> Result<String, String> {
        match params.get(key) {
            Some(ParameterValue::String(v)) => Ok(v.clone()),
            _ => Err(format!("Missing or invalid string parameter: {}", key)),
        }
    }

    fn estimate_time(&self, _gcode: &str, _feed_rate: f64) -> f64 {
        // Rough estimate: typically a few seconds to a minute for basic cuts
        30.0
    }
}

impl CircleGenerator {
    fn get_float(&self, params: &HashMap<String, ParameterValue>, key: &str) -> Result<f64, String> {
        match params.get(key) {
            Some(ParameterValue::Float(v)) => Ok(*v),
            Some(ParameterValue::Integer(v)) => Ok(*v as f64),
            _ => Err(format!("Missing or invalid float parameter: {}", key)),
        }
    }

    fn get_string(&self, params: &HashMap<String, ParameterValue>, key: &str) -> Result<String, String> {
        match params.get(key) {
            Some(ParameterValue::String(v)) => Ok(v.clone()),
            _ => Err(format!("Missing or invalid string parameter: {}", key)),
        }
    }

    fn estimate_time(&self, _gcode: &str, _feed_rate: f64) -> f64 {
        25.0
    }
}

impl LineGenerator {
    fn get_float(&self, params: &HashMap<String, ParameterValue>, key: &str) -> Result<f64, String> {
        match params.get(key) {
            Some(ParameterValue::Float(v)) => Ok(*v),
            Some(ParameterValue::Integer(v)) => Ok(*v as f64),
            _ => Err(format!("Missing or invalid float parameter: {}", key)),
        }
    }

    fn estimate_time(&self, _gcode: &str, _feed_rate: f64) -> f64 {
        15.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_generator() {
        let gen = RectangleGenerator;
        let mut params = HashMap::new();
        params.insert("width".to_string(), ParameterValue::Float(100.0));
        params.insert("height".to_string(), ParameterValue::Float(80.0));
        params.insert("corner_radius".to_string(), ParameterValue::Float(0.0));
        params.insert("depth".to_string(), ParameterValue::Float(5.0));
        params.insert("mode".to_string(), ParameterValue::String("contour".to_string()));
        params.insert("feed_rate".to_string(), ParameterValue::Float(1000.0));
        params.insert("plunge_rate".to_string(), ParameterValue::Float(500.0));
        params.insert("start_x".to_string(), ParameterValue::Float(0.0));
        params.insert("start_y".to_string(), ParameterValue::Float(0.0));

        let result = gen.generate(&params);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.gcode.contains("Rectangle Generator"));
        assert!(output.gcode.contains("G21"));
        assert!(output.bounding_box.is_some());
    }

    #[test]
    fn test_circle_generator() {
        let gen = CircleGenerator;
        let mut params = HashMap::new();
        params.insert("diameter".to_string(), ParameterValue::Float(50.0));
        params.insert("depth".to_string(), ParameterValue::Float(5.0));
        params.insert("mode".to_string(), ParameterValue::String("contour".to_string()));
        params.insert("feed_rate".to_string(), ParameterValue::Float(1000.0));
        params.insert("plunge_rate".to_string(), ParameterValue::Float(500.0));
        params.insert("start_x".to_string(), ParameterValue::Float(0.0));
        params.insert("start_y".to_string(), ParameterValue::Float(0.0));

        let result = gen.generate(&params);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.gcode.contains("Circle Generator"));
        assert!(output.gcode.contains("G2"));
        assert!(output.bounding_box.is_some());
    }

    #[test]
    fn test_line_generator() {
        let gen = LineGenerator;
        let mut params = HashMap::new();
        params.insert("start_x".to_string(), ParameterValue::Float(0.0));
        params.insert("start_y".to_string(), ParameterValue::Float(0.0));
        params.insert("end_x".to_string(), ParameterValue::Float(100.0));
        params.insert("end_y".to_string(), ParameterValue::Float(0.0));
        params.insert("depth".to_string(), ParameterValue::Float(5.0));
        params.insert("feed_rate".to_string(), ParameterValue::Float(1000.0));
        params.insert("plunge_rate".to_string(), ParameterValue::Float(500.0));

        let result = gen.generate(&params);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.gcode.contains("Line Generator"));
        assert!(output.bounding_box.is_some());
    }
}
