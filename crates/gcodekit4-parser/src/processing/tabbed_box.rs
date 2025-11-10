//! Tabbed Box Maker
//! 
//! Generates G-code toolpaths for laser/CNC cutting tabbed boxes with finger joints.
//! Based on the algorithm from https://github.com/paulh-rnd/TabbedBoxMaker

use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxType {
    FullBox = 1,
    NoTop = 2,
    NoTopFront = 3,
    NoTopFrontRight = 4,
    NoTopBottom = 5,
    NoTopFrontBackRight = 6,
}

impl From<i32> for BoxType {
    fn from(value: i32) -> Self {
        match value {
            1 => BoxType::FullBox,
            2 => BoxType::NoTop,
            3 => BoxType::NoTopFront,
            4 => BoxType::NoTopFrontRight,
            5 => BoxType::NoTopBottom,
            6 => BoxType::NoTopFrontBackRight,
            _ => BoxType::FullBox,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabType {
    Laser = 0,
    CNC = 1,
}

impl From<i32> for TabType {
    fn from(value: i32) -> Self {
        match value {
            1 => TabType::CNC,
            _ => TabType::Laser,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutStyle {
    Diagrammatic = 1,
    ThreePiece = 2,
    InlineCompact = 3,
}

impl From<i32> for LayoutStyle {
    fn from(value: i32) -> Self {
        match value {
            1 => LayoutStyle::Diagrammatic,
            2 => LayoutStyle::ThreePiece,
            3 => LayoutStyle::InlineCompact,
            _ => LayoutStyle::Diagrammatic,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoxParameters {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub thickness: f32,
    pub tab_width: f32,
    pub kerf: f32,
    pub spacing: f32,
    pub box_type: BoxType,
    pub tab_type: TabType,
    pub layout_style: LayoutStyle,
    pub inside_dimensions: bool,
    pub dividers_length: i32,
    pub dividers_width: i32,
    pub laser_passes: i32,
    pub laser_power: i32,
    pub feed_rate: f32,
}

impl Default for BoxParameters {
    fn default() -> Self {
        Self {
            length: 100.0,
            width: 100.0,
            height: 100.0,
            thickness: 3.0,
            tab_width: 25.0,
            kerf: 0.5,
            spacing: 5.0,
            box_type: BoxType::FullBox,
            tab_type: TabType::Laser,
            layout_style: LayoutStyle::Diagrammatic,
            inside_dimensions: false,
            dividers_length: 0,
            dividers_width: 0,
            laser_passes: 3,
            laser_power: 1000,
            feed_rate: 500.0,
        }
    }
}

#[derive(Debug, Clone)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

pub struct TabbedBoxMaker {
    params: BoxParameters,
    paths: Vec<Vec<Point>>,
}

impl TabbedBoxMaker {
    pub fn new(params: BoxParameters) -> Result<Self, String> {
        Self::validate_parameters(&params)?;
        Ok(Self {
            params,
            paths: Vec::new(),
        })
    }

    fn validate_parameters(params: &BoxParameters) -> Result<(), String> {
        if params.length < 20.0 || params.width < 20.0 || params.height < 20.0 {
            return Err("All dimensions must be at least 20mm".to_string());
        }

        if params.thickness < 1.0 || params.thickness > 20.0 {
            return Err("Material thickness must be between 1mm and 20mm".to_string());
        }

        if params.tab_width < 5.0 {
            return Err("Tab width must be at least 5mm".to_string());
        }

        let min_dimension = params.length.min(params.width).min(params.height);
        if params.tab_width > min_dimension / 2.5 {
            return Err(format!(
                "Tab width ({:.1}mm) is too large for smallest dimension ({:.1}mm). Maximum recommended: {:.1}mm",
                params.tab_width, min_dimension, min_dimension / 2.5
            ));
        }

        if params.kerf < 0.0 || params.kerf > params.thickness / 2.0 {
            return Err("Kerf must be between 0 and half the material thickness".to_string());
        }

        Ok(())
    }

    pub fn generate(&mut self) -> Result<(), String> {
        self.paths.clear();

        let mut length = self.params.length + self.params.kerf;
        let mut width = self.params.width + self.params.kerf;
        let mut height = self.params.height + self.params.kerf;

        if self.params.inside_dimensions {
            length += self.params.thickness * 2.0;
            width += self.params.thickness * 2.0;
            height += self.params.thickness * 2.0;
        }

        let has_parts = self.get_box_parts();

        self.generate_panels(length, width, height, &has_parts)?;

        Ok(())
    }

    fn get_box_parts(&self) -> [bool; 6] {
        let mut parts = [true; 6];
        
        match self.params.box_type {
            BoxType::FullBox => {},
            BoxType::NoTop => parts[0] = false,
            BoxType::NoTopFront => {
                parts[0] = false;
                parts[4] = false;
            },
            BoxType::NoTopFrontRight => {
                parts[0] = false;
                parts[4] = false;
                parts[3] = false;
            },
            BoxType::NoTopBottom => {
                parts[0] = false;
                parts[1] = false;
            },
            BoxType::NoTopFrontBackRight => {
                parts[0] = false;
                parts[4] = false;
                parts[5] = false;
                parts[3] = false;
            },
        }
        
        parts
    }

    fn generate_panels(&mut self, length: f32, width: f32, height: f32, has_parts: &[bool; 6]) -> Result<(), String> {
        let spacing = self.params.spacing;
        let thickness = self.params.thickness;
        
        let mut x_offset = 0.0;
        let mut y_offset = 0.0;

        // Bottom panel (length x width)
        // Parts: [0=top, 1=bottom, 2=left, 3=right, 4=front, 5=back]
        // Connects: Front(bottom), Back(bottom), Left(bottom), Right(bottom)
        if has_parts[1] {
            self.generate_side_with_tabs(x_offset, y_offset, length, width, 
                                        if has_parts[4] { 1 } else { 0 },  // bottom: tabs out if front exists
                                        if has_parts[3] { 1 } else { 0 },  // right: tabs out if right exists
                                        if has_parts[5] { 1 } else { 0 },  // top: tabs out if back exists
                                        if has_parts[2] { 1 } else { 0 }); // left: tabs out if left exists
            x_offset += length + spacing + thickness * 2.0;
        }

        // Top panel (length x width)
        // Connects: Front(top), Back(top), Left(top), Right(top)
        if has_parts[0] {
            self.generate_side_with_tabs(x_offset, y_offset, length, width, 
                                        if has_parts[4] { -1 } else { 0 }, // bottom: notches in if front exists
                                        if has_parts[3] { -1 } else { 0 }, // right: notches in if right exists
                                        if has_parts[5] { -1 } else { 0 }, // top: notches in if back exists
                                        if has_parts[2] { -1 } else { 0 });// left: notches in if left exists
            x_offset += length + spacing + thickness * 2.0;
        }

        x_offset = 0.0;
        y_offset += width + spacing + thickness * 2.0;

        // Left side (height x width)
        // Connects: Bottom(left), Top(left), Front(left), Back(left)
        if has_parts[2] {
            self.generate_side_with_tabs(x_offset, y_offset, height, width, 
                                        if has_parts[1] { -1 } else { 0 }, // bottom: notches in if bottom exists
                                        if has_parts[4] { 1 } else { 0 },  // right: tabs out if front exists
                                        if has_parts[0] { 1 } else { 0 },  // top: tabs out if top exists
                                        if has_parts[5] { 1 } else { 0 }); // left: tabs out if back exists
            x_offset += height + spacing + thickness * 2.0;
        }

        // Right side (height x width)
        // Connects: Bottom(right), Top(right), Front(right), Back(right)
        if has_parts[3] {
            self.generate_side_with_tabs(x_offset, y_offset, height, width, 
                                        if has_parts[1] { -1 } else { 0 }, // bottom: notches in if bottom exists
                                        if has_parts[5] { 1 } else { 0 },  // right: tabs out if back exists
                                        if has_parts[0] { 1 } else { 0 },  // top: tabs out if top exists
                                        if has_parts[4] { 1 } else { 0 }); // left: tabs out if front exists
            x_offset += height + spacing + thickness * 2.0;
        }

        x_offset = 0.0;
        y_offset += width + spacing + thickness * 2.0;

        // Front panel (length x height)
        // Connects: Bottom(bottom), Top(top), Left(right), Right(left)
        if has_parts[4] {
            self.generate_side_with_tabs(x_offset, y_offset, length, height, 
                                        if has_parts[1] { -1 } else { 0 }, // bottom: notches in if bottom exists
                                        if has_parts[2] { -1 } else { 0 }, // right: notches in if left exists
                                        if has_parts[0] { 1 } else { 0 },  // top: tabs out if top exists
                                        if has_parts[3] { -1 } else { 0 });// left: notches in if right exists
            x_offset += length + spacing + thickness * 2.0;
        }

        // Back panel (length x height)
        // Connects: Bottom(top), Top(top), Left(left), Right(right)
        if has_parts[5] {
            self.generate_side_with_tabs(x_offset, y_offset, length, height, 
                                        if has_parts[1] { -1 } else { 0 }, // bottom: notches in if bottom exists
                                        if has_parts[3] { -1 } else { 0 }, // right: notches in if right exists
                                        if has_parts[0] { 1 } else { 0 },  // top: tabs out if top exists
                                        if has_parts[2] { -1 } else { 0 });// left: notches in if left exists
        }

        Ok(())
    }

    fn generate_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let mut path = Vec::new();
        
        path.push(Point::new(x, y));
        path.push(Point::new(x + width, y));
        path.push(Point::new(x + width, y + height));
        path.push(Point::new(x, y + height));
        path.push(Point::new(x, y));
        
        self.paths.push(path);
    }

    fn generate_side_with_tabs(&mut self, x: f32, y: f32, width: f32, height: f32, 
                                tabs_bottom: i32, tabs_right: i32, tabs_top: i32, tabs_left: i32) {
        let mut path = Vec::new();
        let thickness = self.params.thickness;
        let tab_width = self.params.tab_width;
        
        // Calculate number of tabs per side
        let tabs_horizontal = ((width / tab_width).floor() as i32).max(1);
        let tabs_vertical = ((height / tab_width).floor() as i32).max(1);
        
        // Adjust tab width to fit evenly
        let actual_tab_h = width / tabs_horizontal as f32;
        let actual_tab_v = height / tabs_vertical as f32;
        
        // Start at bottom-left
        path.push(Point::new(x, y));
        
        // Bottom edge (moving right)
        // tabs_bottom: 0 = none, 1 = tabs out (start with out), -1 = tabs in (start with in)
        if tabs_bottom != 0 {
            let mut current_x = x;
            for i in 0..tabs_horizontal {
                let start_out = if tabs_bottom > 0 { i % 2 == 0 } else { i % 2 == 1 };
                if start_out {
                    // Tab sticks out
                    path.push(Point::new(current_x, y));
                    path.push(Point::new(current_x, y - thickness));
                    path.push(Point::new(current_x + actual_tab_h, y - thickness));
                    path.push(Point::new(current_x + actual_tab_h, y));
                } else {
                    // Tab notch in
                    path.push(Point::new(current_x, y));
                    path.push(Point::new(current_x, y + thickness));
                    path.push(Point::new(current_x + actual_tab_h, y + thickness));
                    path.push(Point::new(current_x + actual_tab_h, y));
                }
                current_x += actual_tab_h;
            }
        } else {
            path.push(Point::new(x + width, y));
        }
        
        // Right edge (moving up)
        if tabs_right != 0 {
            let mut current_y = y;
            for i in 0..tabs_vertical {
                let start_out = if tabs_right > 0 { i % 2 == 0 } else { i % 2 == 1 };
                if start_out {
                    path.push(Point::new(x + width, current_y));
                    path.push(Point::new(x + width + thickness, current_y));
                    path.push(Point::new(x + width + thickness, current_y + actual_tab_v));
                    path.push(Point::new(x + width, current_y + actual_tab_v));
                } else {
                    path.push(Point::new(x + width, current_y));
                    path.push(Point::new(x + width - thickness, current_y));
                    path.push(Point::new(x + width - thickness, current_y + actual_tab_v));
                    path.push(Point::new(x + width, current_y + actual_tab_v));
                }
                current_y += actual_tab_v;
            }
        } else {
            path.push(Point::new(x + width, y + height));
        }
        
        // Top edge (moving left)
        if tabs_top != 0 {
            let mut current_x = x + width;
            for i in 0..tabs_horizontal {
                let start_out = if tabs_top > 0 { i % 2 == 0 } else { i % 2 == 1 };
                if start_out {
                    path.push(Point::new(current_x, y + height));
                    path.push(Point::new(current_x, y + height + thickness));
                    path.push(Point::new(current_x - actual_tab_h, y + height + thickness));
                    path.push(Point::new(current_x - actual_tab_h, y + height));
                } else {
                    path.push(Point::new(current_x, y + height));
                    path.push(Point::new(current_x, y + height - thickness));
                    path.push(Point::new(current_x - actual_tab_h, y + height - thickness));
                    path.push(Point::new(current_x - actual_tab_h, y + height));
                }
                current_x -= actual_tab_h;
            }
        } else {
            path.push(Point::new(x, y + height));
        }
        
        // Left edge (moving down)
        if tabs_left != 0 {
            let mut current_y = y + height;
            for i in 0..tabs_vertical {
                let start_out = if tabs_left > 0 { i % 2 == 0 } else { i % 2 == 1 };
                if start_out {
                    path.push(Point::new(x, current_y));
                    path.push(Point::new(x - thickness, current_y));
                    path.push(Point::new(x - thickness, current_y - actual_tab_v));
                    path.push(Point::new(x, current_y - actual_tab_v));
                } else {
                    path.push(Point::new(x, current_y));
                    path.push(Point::new(x + thickness, current_y));
                    path.push(Point::new(x + thickness, current_y - actual_tab_v));
                    path.push(Point::new(x, current_y - actual_tab_v));
                }
                current_y -= actual_tab_v;
            }
        } else {
            path.push(Point::new(x, y));
        }
        
        // Close path
        path.push(Point::new(x, y));
        
        self.paths.push(path);
    }

    pub fn to_gcode(&self, feed_rate: f32, plunge_rate: f32, cut_depth: f32) -> String {
        let mut gcode = String::new();
        
        gcode.push_str("; Tabbed Box Maker G-code\n");
        gcode.push_str("; https://github.com/paulh-rnd/TabbedBoxMaker\n");
        gcode.push_str(&format!("; Box: {}x{}x{} mm\n", self.params.length, self.params.width, self.params.height));
        gcode.push_str(&format!("; Material thickness: {} mm\n", self.params.thickness));
        gcode.push_str(&format!("; Tab width: {} mm\n", self.params.tab_width));
        gcode.push_str(&format!("; Tab depth (protrusion): {} mm\n", self.params.thickness));
        gcode.push_str(&format!("; Kerf: {} mm\n", self.params.kerf));
        gcode.push_str(&format!("; Laser passes: {}\n", self.params.laser_passes));
        gcode.push_str(&format!("; Laser power: S{}\n", self.params.laser_power));
        gcode.push_str(&format!("; Feed rate: {:.0} mm/min\n", self.params.feed_rate));
        gcode.push_str(";\n");
        
        let has_parts = self.get_box_parts();
        gcode.push_str("; Box Layout:\n");
        gcode.push_str(&format!(";   Type: {:?}\n", self.params.box_type));
        gcode.push_str(";   Panels included:\n");
        if has_parts[0] { gcode.push_str(";     - Top (length x width)\n"); }
        if has_parts[1] { gcode.push_str(";     - Bottom (length x width)\n"); }
        if has_parts[2] { gcode.push_str(";     - Left side (height x width)\n"); }
        if has_parts[3] { gcode.push_str(";     - Right side (height x width)\n"); }
        if has_parts[4] { gcode.push_str(";     - Front (length x height)\n"); }
        if has_parts[5] { gcode.push_str(";     - Back (length x height)\n"); }
        gcode.push_str(";\n");
        gcode.push_str(";   Assembly view (unfolded):\n");
        gcode.push_str(";        +-------+\n");
        gcode.push_str(&format!(";        |  {}  |\n", if has_parts[0] { "TOP" } else { "   " }));
        gcode.push_str(";   +----+-------+----+\n");
        gcode.push_str(&format!(";   | {} | FRONT | {}  |\n", 
            if has_parts[2] { "L" } else { " " },
            if has_parts[3] { "R" } else { " " }));
        gcode.push_str(";   +----+-------+----+\n");
        gcode.push_str(&format!(";        | {}  |\n", if has_parts[1] { "BOT" } else { "   " }));
        gcode.push_str(";        +-------+\n");
        gcode.push_str(&format!(";        | {}  |\n", if has_parts[5] { "BACK" } else { "    " }));
        gcode.push_str(";        +-------+\n");
        gcode.push_str("\n");
        
        gcode.push_str("; Initialization sequence\n");
        gcode.push_str("G21 ; Set units to millimeters\n");
        gcode.push_str("G90 ; Absolute positioning\n");
        gcode.push_str("G17 ; XY plane selection\n");
        gcode.push_str("\n");
        
        gcode.push_str("; Home and set work coordinate system\n");
        gcode.push_str("$H ; Home all axes (bottom-left corner)\n");
        gcode.push_str("G10 L2 P1 X0 Y0 Z0 ; Clear G54 offset\n");
        gcode.push_str("G54 ; Select work coordinate system 1\n");
        gcode.push_str("G0 X10.0 Y10.0 ; Move to work origin (10mm from corner)\n");
        gcode.push_str("G10 L20 P1 X0 Y0 Z0 ; Set current position as work zero\n");
        gcode.push_str(&format!("G0 Z{:.2} F{:.0} ; Move to safe height\n", 5.0, self.params.feed_rate));
        
        for (i, path) in self.paths.iter().enumerate() {
            gcode.push_str(&format!("; Path {} (panel {})\n", i + 1, i + 1));
            
            if let Some(first_point) = path.first() {
                gcode.push_str(&format!("G0 X{:.2} Y{:.2} ; Rapid to start\n", first_point.x, first_point.y));
                gcode.push_str(&format!("G1 Z{:.2} F{:.0} ; Plunge\n", -cut_depth, plunge_rate));
                
                for pass_num in 1..=self.params.laser_passes {
                    gcode.push_str(&format!("; Pass {}/{}\n", pass_num, self.params.laser_passes));
                    gcode.push_str(&format!("M3 S{} ; Laser on\n", self.params.laser_power));
                    
                    for (idx, point) in path.iter().skip(1).enumerate() {
                        if idx == 0 {
                            gcode.push_str(&format!("G1 X{:.2} Y{:.2} F{:.0}\n", point.x, point.y, self.params.feed_rate));
                        } else {
                            gcode.push_str(&format!("G1 X{:.2} Y{:.2}\n", point.x, point.y));
                        }
                    }
                    
                    gcode.push_str("M5 ; Laser off\n");
                    
                    if pass_num < self.params.laser_passes {
                        gcode.push_str(&format!("G0 X{:.2} Y{:.2} ; Return to start for next pass\n", first_point.x, first_point.y));
                    }
                }
            }
            
            gcode.push_str(&format!("G0 Z{:.2} ; Retract\n\n", 5.0));
        }
        
        gcode.push_str("M5 ; Ensure laser off\n");
        gcode.push_str("G0 Z10.0 ; Move to safe height\n");
        gcode.push_str("G0 X0 Y0 ; Return to origin\n");
        gcode.push_str("M2 ; Program end\n");
        
        gcode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_parameters() {
        let params = BoxParameters::default();
        assert_eq!(params.length, 100.0);
        assert_eq!(params.width, 100.0);
        assert_eq!(params.height, 100.0);
    }

    #[test]
    fn test_parameter_validation() {
        let mut params = BoxParameters::default();
        params.length = 10.0;
        
        let result = TabbedBoxMaker::new(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_simple_box() {
        let params = BoxParameters::default();
        let mut maker = TabbedBoxMaker::new(params).unwrap();
        
        let result = maker.generate();
        assert!(result.is_ok());
        assert!(!maker.paths.is_empty());
    }

    #[test]
    fn test_gcode_generation() {
        let params = BoxParameters::default();
        let mut maker = TabbedBoxMaker::new(params).unwrap();
        maker.generate().unwrap();
        
        let gcode = maker.to_gcode(1000.0, 300.0, 3.0);
        assert!(gcode.contains("G21"));
        assert!(gcode.contains("G90"));
        assert!(gcode.contains("M2"));
    }
}
