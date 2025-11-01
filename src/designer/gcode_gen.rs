//! G-code generation from toolpaths.

use super::toolpath::{Toolpath, ToolpathSegmentType};
use crate::data::Units;

/// G-code generator for converting toolpaths to G-code commands.
pub struct ToolpathToGcode {
    units: Units,
    safe_z: f64,
    line_numbers_enabled: bool,
}

impl ToolpathToGcode {
    /// Creates a new G-code generator.
    pub fn new(units: Units, safe_z: f64) -> Self {
        Self {
            units,
            safe_z,
            line_numbers_enabled: false,
        }
    }

    /// Creates a new G-code generator with line numbers enabled.
    pub fn with_line_numbers(units: Units, safe_z: f64, enabled: bool) -> Self {
        Self {
            units,
            safe_z,
            line_numbers_enabled: enabled,
        }
    }

    /// Generates G-code from a toolpath.
    pub fn generate(&self, toolpath: &Toolpath) -> String {
        let mut gcode = String::new();

        // Header
        gcode.push_str("; Generated G-code from Designer tool\n");
        gcode.push_str(&format!(
            "; Tool diameter: {:.3}mm\n",
            toolpath.tool_diameter
        ));
        gcode.push_str(&format!("; Cut depth: {:.3}mm\n", toolpath.depth));
        gcode.push_str(&format!(
            "; Total path length: {:.3}mm\n",
            toolpath.total_length()
        ));
        gcode.push('\n');

        // Setup
        gcode.push_str("G90         ; Absolute positioning\n");
        gcode.push_str("G21         ; Millimeter units\n");
        gcode.push_str("G17         ; XY plane\n");
        gcode.push_str("M3          ; Spindle on\n");
        gcode.push('\n');

        // Generate moves for each segment
        let mut line_number = 10;
        let mut current_z = self.safe_z;

        for segment in &toolpath.segments {
            match segment.segment_type {
                ToolpathSegmentType::RapidMove => {
                    // Rapid move (G00)
                    let line_prefix = if self.line_numbers_enabled {
                        format!("N{} ", line_number)
                    } else {
                        String::new()
                    };
                    gcode.push_str(&format!(
                        "{}G00 X{:.3} Y{:.3} Z{:.3}\n",
                        line_prefix, segment.end.x, segment.end.y, self.safe_z
                    ));
                    current_z = self.safe_z;
                }
                ToolpathSegmentType::LinearMove => {
                    // First plunge if needed
                    if (current_z - self.safe_z).abs() > 0.01 {
                        let line_prefix = if self.line_numbers_enabled {
                            format!("N{} ", line_number)
                        } else {
                            String::new()
                        };
                        gcode.push_str(&format!(
                            "{}G01 Z{:.3} F{:.0}\n",
                            line_prefix, toolpath.depth, segment.feed_rate
                        ));
                        line_number += 10;
                        current_z = toolpath.depth;
                    } else if (current_z - self.safe_z).abs() < 0.01 {
                        // Plunge before first move
                        let line_prefix = if self.line_numbers_enabled {
                            format!("N{} ", line_number)
                        } else {
                            String::new()
                        };
                        gcode.push_str(&format!(
                            "{}G01 Z{:.3} F{:.0}\n",
                            line_prefix, toolpath.depth, segment.feed_rate
                        ));
                        line_number += 10;
                        current_z = toolpath.depth;
                    }

                    // Linear move (G01)
                    let line_prefix = if self.line_numbers_enabled {
                        format!("N{} ", line_number)
                    } else {
                        String::new()
                    };
                    gcode.push_str(&format!(
                        "{}G01 X{:.3} Y{:.3} F{:.0}\n",
                        line_prefix, segment.end.x, segment.end.y, segment.feed_rate
                    ));
                }
                ToolpathSegmentType::ArcMove => {
                    // Arc move (G02/G03) - for future use
                    let line_prefix = if self.line_numbers_enabled {
                        format!("N{} ", line_number)
                    } else {
                        String::new()
                    };
                    gcode.push_str(&format!(
                        "{}G01 X{:.3} Y{:.3} F{:.0}\n",
                        line_prefix, segment.end.x, segment.end.y, segment.feed_rate
                    ));
                }
            }

            line_number += 10;
        }

        // Cleanup
        gcode.push('\n');
        gcode.push_str("M5          ; Spindle off\n");
        gcode.push_str("G00 Z10     ; Raise tool\n");
        gcode.push_str("G00 X0 Y0   ; Return to origin\n");
        gcode.push_str("M30         ; End program\n");

        gcode
    }
}

impl Default for ToolpathToGcode {
    fn default() -> Self {
        Self::new(Units::MM, 10.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::designer::shapes::Rectangle;
    use crate::designer::toolpath::ToolpathGenerator;

    #[test]
    fn test_gcode_generation() {
        let gen = ToolpathGenerator::new();
        let rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        let toolpath = gen.generate_rectangle_contour(&rect);

        let gcode_gen = ToolpathToGcode::new(Units::MM, 10.0);
        let gcode = gcode_gen.generate(&toolpath);

        assert!(gcode.contains("G90"));
        assert!(gcode.contains("G21"));
        assert!(gcode.contains("G00"));
        assert!(gcode.contains("G01"));
        assert!(gcode.contains("M30"));
    }

    #[test]
    fn test_gcode_header() {
        let toolpath = Toolpath::new(3.175, -5.0);
        let gcode_gen = ToolpathToGcode::new(Units::MM, 10.0);
        let gcode = gcode_gen.generate(&toolpath);

        assert!(gcode.contains("Generated G-code from Designer tool"));
        assert!(gcode.contains("Tool diameter"));
        assert!(gcode.contains("Cut depth"));
    }
}
