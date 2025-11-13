//! Tabbed Box Maker
//! 
//! Based on the superior algorithm from https://github.com/florianfesti/boxes
//! Uses finger/space multiples of thickness for automatic finger calculation

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxType {
    FullBox = 0,
    NoTop = 1,
}

impl From<i32> for BoxType {
    fn from(value: i32) -> Self {
        match value {
            0 => BoxType::FullBox,
            1 => BoxType::NoTop,
            _ => BoxType::FullBox,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FingerStyle {
    Rectangular = 0,
    Springs = 1,
    Barbs = 2,
    Snap = 3,
}

impl From<i32> for FingerStyle {
    fn from(value: i32) -> Self {
        match value {
            1 => FingerStyle::Springs,
            2 => FingerStyle::Barbs,
            3 => FingerStyle::Snap,
            _ => FingerStyle::Rectangular,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FingerJointSettings {
    /// Width of fingers in multiples of thickness
    pub finger: f32,
    /// Space between fingers in multiples of thickness
    pub space: f32,
    /// Space at start and end in multiples of normal spaces
    pub surrounding_spaces: f32,
    /// Extra space to allow fingers to move in/out (multiples of thickness)
    pub play: f32,
    /// Extra material for burn marks (multiples of thickness)
    pub extra_length: f32,
    /// Style of fingers
    pub style: FingerStyle,
}

impl Default for FingerJointSettings {
    fn default() -> Self {
        Self {
            finger: 2.0,
            space: 2.0,
            surrounding_spaces: 2.0,
            play: 0.0,
            extra_length: 0.0,
            style: FingerStyle::Rectangular,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoxParameters {
    pub x: f32,
    pub y: f32,
    pub h: f32,
    pub thickness: f32,
    pub outside: bool,
    pub box_type: BoxType,
    pub finger_joint: FingerJointSettings,
    pub burn: f32,
    pub laser_passes: i32,
    pub laser_power: i32,
    pub feed_rate: f32,
}

impl Default for BoxParameters {
    fn default() -> Self {
        Self {
            x: 100.0,
            y: 100.0,
            h: 100.0,
            thickness: 3.0,
            outside: false,
            box_type: BoxType::FullBox,
            finger_joint: FingerJointSettings::default(),
            burn: 0.1,
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
    x: f32,
    y: f32,
    h: f32,
    t: f32,
    paths: Vec<Vec<Point>>,
}

impl TabbedBoxMaker {
    pub fn new(params: BoxParameters) -> Result<Self, String> {
        Self::validate_parameters(&params)?;
        
        let mut x = params.x;
        let mut y = params.y;
        let mut h = params.h;
        
        let t = params.thickness;
        
        if params.outside {
            x = Self::adjust_size(x, t);
            y = Self::adjust_size(y, t);
            h = Self::adjust_size(h, t);
        }
        
        Ok(Self {
            params,
            x,
            y,
            h,
            t,
            paths: Vec::new(),
        })
    }

    fn validate_parameters(params: &BoxParameters) -> Result<(), String> {
        if params.x < 20.0 || params.y < 20.0 || params.h < 20.0 {
            return Err("All dimensions must be at least 20mm".to_string());
        }

        if params.thickness < 1.0 || params.thickness > 20.0 {
            return Err("Material thickness must be between 1mm and 20mm".to_string());
        }

        if (params.finger_joint.space + params.finger_joint.finger).abs() < 0.1 {
            return Err("Finger + space must not be close to zero".to_string());
        }

        Ok(())
    }

    fn adjust_size(size: f32, thickness: f32) -> f32 {
        size - 2.0 * thickness
    }

    /// Calculate number of fingers and leftover space for a given length
    fn calc_fingers(&self, length: f32) -> (usize, f32) {
        let settings = &self.params.finger_joint;
        let t = self.t;
        
        let space = settings.space * t;
        let finger = settings.finger * t;
        
        // Calculate number of fingers that fit
        let mut fingers = ((length - (settings.surrounding_spaces - 1.0) * space) / (space + finger)).floor() as usize;
        
        // Shrink surrounding space up to half thickness each side if needed
        if fingers == 0 && length > finger + 1.0 * t {
            fingers = 1;
        }
        
        if finger == 0.0 {
            fingers = 0;
        }
        
        // Calculate leftover space
        let leftover = if fingers > 0 {
            length - (fingers as f32) * (space + finger) + space
        } else {
            length
        };
        
        (fingers, leftover)
    }

    /// Draw finger joint edge
    fn draw_finger_edge(&self, length: f32, positive: bool) -> Vec<Point> {
        let mut path = Vec::new();
        let settings = &self.params.finger_joint;
        let t = self.t;
        
        let mut space = settings.space * t;
        let mut finger = settings.finger * t;
        let play = settings.play * t;
        
        let (fingers, mut leftover) = self.calc_fingers(length);
        
        // Adjust for play
        if !positive {
            finger += play;
            space -= play;
            leftover -= play;
        }
        
        let mut x = 0.0;
        let mut y = 0.0;
        
        // Start with leftover/2
        path.push(Point::new(x, y));
        x += leftover / 2.0;
        path.push(Point::new(x, y));
        
        // Draw fingers
        for _ in 0..fingers {
            if positive {
                // Finger protrudes
                y -= t;
                path.push(Point::new(x, y));
                x += finger;
                path.push(Point::new(x, y));
                y += t;
                path.push(Point::new(x, y));
            } else {
                // Notch for finger
                y += t;
                path.push(Point::new(x, y));
                x += finger;
                path.push(Point::new(x, y));
                y -= t;
                path.push(Point::new(x, y));
            }
            
            // Space between fingers
            x += space;
            path.push(Point::new(x, y));
        }
        
        // End with leftover/2
        x += leftover / 2.0;
        path.push(Point::new(x, y));
        
        path
    }

    /// Draw a rectangular wall with finger joints on specified edges
    /// edges: 4-char string, each char: 'f' = finger out, 'F' = finger in, 'e' = plain edge
    /// Edges: [0]=bottom, [1]=right, [2]=top, [3]=left
    fn draw_rectangular_wall(&self, width: f32, height: f32, edges: &str, start_x: f32, start_y: f32) -> Vec<Point> {
        let mut path = Vec::new();
        let edge_chars: Vec<char> = edges.chars().collect();
        
        // Bottom edge: left to right (0,0) → (width,0)
        // Direct mapping: p.x along edge, p.y perpendicular (down for fingers)
        if let Some(&c) = edge_chars.get(0) {
            if c == 'f' || c == 'F' {
                let base_path = self.draw_finger_edge(width, c == 'f');
                for p in &base_path {
                    path.push(Point::new(start_x + p.x, start_y + p.y));
                }
            } else {
                path.push(Point::new(start_x, start_y));
                path.push(Point::new(start_x + width, start_y));
            }
        }
        
        // Right edge: bottom to top (width,0) → (width,height)
        // Rotation: horizontal→vertical, p.x becomes distance up, p.y becomes distance left
        if let Some(&c) = edge_chars.get(1) {
            if c == 'f' || c == 'F' {
                let base_path = self.draw_finger_edge(height, c == 'f');
                // Add all points including first transformed correctly
                for p in &base_path {
                    let pt = Point::new(start_x + width - p.y, start_y + p.x);
                    // Skip if this is duplicate of last point in path
                    if let Some(last) = path.last() {
                        if (pt.x - last.x).abs() < 0.01 && (pt.y - last.y).abs() < 0.01 {
                            continue;
                        }
                    }
                    path.push(pt);
                }
            } else {
                path.push(Point::new(start_x + width, start_y + height));
            }
        }
        
        // Top edge: right to left (width,height) → (0,height)
        // Reversed horizontal: p.x from 0→width but we place from right→left
        if let Some(&c) = edge_chars.get(2) {
            if c == 'F' || c == 'f' {
                let base_path = self.draw_finger_edge(width, c == 'f');
                // Invert: distance from start becomes distance from end
                for p in &base_path {
                    let pt = Point::new(start_x + width - p.x, start_y + height - p.y);
                    // Skip if this is duplicate of last point in path
                    if let Some(last) = path.last() {
                        if (pt.x - last.x).abs() < 0.01 && (pt.y - last.y).abs() < 0.01 {
                            continue;
                        }
                    }
                    path.push(pt);
                }
            } else {
                path.push(Point::new(start_x, start_y + height));
            }
        }
        
        // Left edge: top to bottom (0,height) → (0,0)
        // Rotation + reverse: p.x becomes distance down from top, p.y becomes distance right
        if let Some(&c) = edge_chars.get(3) {
            if c == 'f' || c == 'F' {
                let base_path = self.draw_finger_edge(height, c == 'f');
                // Go down from top: height - p.x gives Y position
                for p in &base_path {
                    let pt = Point::new(start_x - p.y, start_y + height - p.x);
                    // Skip if this is duplicate of last point in path
                    if let Some(last) = path.last() {
                        if (pt.x - last.x).abs() < 0.01 && (pt.y - last.y).abs() < 0.01 {
                            continue;
                        }
                    }
                    path.push(pt);
                }
            }
        }
        
        path
    }

    pub fn generate(&mut self) -> Result<(), String> {
        self.paths.clear();
        
        let x = self.x;
        let y = self.y;
        let h = self.h;
        let _t = self.t;
        
        let mut x_offset = 0.0;
        let mut y_offset = 0.0;
        let spacing = 5.0;
        
        // Wall 1: x × h with all finger joints out
        self.paths.push(self.draw_rectangular_wall(x, h, "FFFF", x_offset, y_offset));
        x_offset += x + spacing;
        
        // Wall 2: y × h with alternating fingers (bottom/top out, sides in)
        self.paths.push(self.draw_rectangular_wall(y, h, "FfFf", x_offset, y_offset));
        y_offset += h + spacing;
        x_offset = 0.0;
        
        // Wall 4: y × h (copy of wall 2)
        self.paths.push(self.draw_rectangular_wall(y, h, "FfFf", x_offset, y_offset));
        x_offset += y + spacing;
        
        // Wall 3: x × h (copy of wall 1)
        self.paths.push(self.draw_rectangular_wall(x, h, "FFFF", x_offset, y_offset));
        x_offset += x + spacing;
        
        // Top: x × y with all fingers in
        self.paths.push(self.draw_rectangular_wall(x, y, "ffff", x_offset, y_offset));
        x_offset += x + spacing;
        
        // Bottom: x × y with all fingers in
        if self.params.box_type == BoxType::FullBox {
            self.paths.push(self.draw_rectangular_wall(x, y, "ffff", x_offset, y_offset));
        }
        
        Ok(())
    }

    pub fn to_gcode(&self) -> String {
        let mut gcode = String::new();
        
        gcode.push_str("; Tabbed Box Maker G-code\n");
        gcode.push_str("; Based on https://github.com/florianfesti/boxes\n");
        gcode.push_str(&format!("; Box: {}x{}x{} mm\n", self.params.x, self.params.y, self.params.h));
        gcode.push_str(&format!("; Material thickness: {} mm\n", self.params.thickness));
        gcode.push_str(&format!("; Finger width: {} * thickness = {} mm\n", 
            self.params.finger_joint.finger, self.params.finger_joint.finger * self.params.thickness));
        gcode.push_str(&format!("; Space width: {} * thickness = {} mm\n", 
            self.params.finger_joint.space, self.params.finger_joint.space * self.params.thickness));
        gcode.push_str(&format!("; Play: {} mm\n", self.params.finger_joint.play * self.params.thickness));
        gcode.push_str(&format!("; Laser passes: {}\n", self.params.laser_passes));
        gcode.push_str(&format!("; Laser power: S{}\n", self.params.laser_power));
        gcode.push_str(&format!("; Feed rate: {:.0} mm/min\n", self.params.feed_rate));
        gcode.push_str(";\n");
        
        gcode.push_str("; Initialization\n");
        gcode.push_str("G21 ; Set units to millimeters\n");
        gcode.push_str("G90 ; Absolute positioning\n");
        gcode.push_str("G17 ; XY plane selection\n");
        gcode.push_str("\n");
        
        gcode.push_str("; Home and set work coordinate system\n");
        gcode.push_str("$H ; Home all axes\n");
        gcode.push_str("G10 L2 P1 X0 Y0 Z0 ; Clear G54 offset\n");
        gcode.push_str("G54 ; Select work coordinate system 1\n");
        gcode.push_str("G0 X10.0 Y10.0 ; Move to work origin\n");
        gcode.push_str("G10 L20 P1 X0 Y0 Z0 ; Set current position as work zero\n");
        gcode.push_str(&format!("G0 Z{:.2} F{:.0} ; Move to safe height\n\n", 5.0, self.params.feed_rate));
        
        let panel_names = ["Wall 1", "Wall 2", "Wall 4", "Wall 3", "Top", "Bottom"];
        
        for (i, path) in self.paths.iter().enumerate() {
            gcode.push_str(&format!("; Panel {}: {}\n", i + 1, panel_names.get(i).unwrap_or(&"Unknown")));
            
            if let Some(first_point) = path.first() {
                gcode.push_str(&format!("G0 X{:.2} Y{:.2} ; Rapid to start\n", first_point.x, first_point.y));
                
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
                        gcode.push_str(&format!("G0 X{:.2} Y{:.2} ; Return to start\n", first_point.x, first_point.y));
                    }
                }
            }
            
            gcode.push_str("\n");
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
    fn test_default_box() {
        let params = BoxParameters::default();
        let mut maker = TabbedBoxMaker::new(params).unwrap();
        maker.generate().unwrap();
        let gcode = maker.to_gcode();
        assert!(gcode.contains("G21"));
        assert!(gcode.contains("M3"));
    }

    #[test]
    fn test_finger_calculation() {
        let params = BoxParameters::default();
        let maker = TabbedBoxMaker::new(params).unwrap();
        
        // For 100mm length with finger=2*t=6mm and space=2*t=6mm
        // fingers should be about 8-9
        let (fingers, leftover) = maker.calc_fingers(100.0);
        assert!(fingers >= 7 && fingers <= 10);
        assert!(leftover > 0.0);
    }
}
