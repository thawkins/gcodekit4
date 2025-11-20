//! Tabbed Box Maker
//!
//! Based on the superior algorithm from https://github.com/florianfesti/boxes
//! Uses finger/space multiples of thickness for automatic finger calculation

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxType {
    FullBox = 0,
    NoTop = 1,
    NoBottom = 2,
    NoSides = 3,
    NoFrontBack = 4,
    NoLeftRight = 5,
}

impl From<i32> for BoxType {
    fn from(value: i32) -> Self {
        match value {
            0 => BoxType::FullBox,
            1 => BoxType::NoTop,
            2 => BoxType::NoBottom,
            3 => BoxType::NoSides,
            4 => BoxType::NoFrontBack,
            5 => BoxType::NoLeftRight,
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
    Dogbone = 4,
}

impl From<i32> for FingerStyle {
    fn from(value: i32) -> Self {
        match value {
            1 => FingerStyle::Springs,
            2 => FingerStyle::Barbs,
            3 => FingerStyle::Snap,
            4 => FingerStyle::Dogbone,
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
    /// Height of dimple (friction fit bump)
    pub dimple_height: f32,
    /// Length of dimple
    pub dimple_length: f32,
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
            dimple_height: 0.0,
            dimple_length: 0.0,
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
    pub offset_x: f32,
    pub offset_y: f32,
    pub dividers_x: u32,
    pub dividers_y: u32,
    pub optimize_layout: bool,
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
            offset_x: 10.0,
            offset_y: 10.0,
            dividers_x: 0,
            dividers_y: 0,
            optimize_layout: false,
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
        let mut fingers = ((length - (settings.surrounding_spaces - 1.0) * space)
            / (space + finger))
            .floor() as usize;

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
        let extra = settings.extra_length * t;
        let kerf = self.params.burn;
        let half_kerf = kerf / 2.0;
        let dogbone = settings.style == FingerStyle::Dogbone;
        let dimple_h = settings.dimple_height;
        let dimple_l = settings.dimple_length;
        // Overcut for dogbone: usually tool radius. Assuming burn is tool diameter.
        let overcut = half_kerf; 

        let (fingers, mut leftover) = self.calc_fingers(length);

        // Adjust for play
        if !positive {
            finger += play;
            space -= play;
            leftover -= play;
        }

        let mut x = 0.0;
        
        let finger_draw;
        let space_draw;
        let leftover_draw;
        let base_y = -half_kerf;
        let tip_y;

        if positive {
            // Fingers out
            // Finger width increases by kerf
            finger_draw = finger + kerf;
            // Space width decreases by kerf
            space_draw = space - kerf;
            // Leftover decreases by kerf (split between two ends)
            leftover_draw = leftover - kerf;
            
            // Tip Y
            tip_y = -t - extra - half_kerf;
        } else {
            // Notches in
            // Notch width decreases by kerf
            finger_draw = finger - kerf;
            // Space width increases by kerf
            space_draw = space + kerf;
            // Leftover increases by kerf
            leftover_draw = leftover + kerf;
            
            // Notch Depth Y
            tip_y = t - half_kerf;
        }

        // Helper for dimpled side
        // Draws a side from (x, y1) to (x, y2) with a dimple if configured
        let draw_side = |path: &mut Vec<Point>, x: f32, y1: f32, y2: f32| {
            if dimple_h > 0.0 && dimple_l > 0.0 && (y2 - y1).abs() > dimple_l {
                let mid_y = (y1 + y2) / 2.0;
                let half_l = dimple_l / 2.0;
                let dir = if y2 > y1 { 1.0 } else { -1.0 };
                
                // Start of side
                path.push(Point::new(x, y1));
                
                // Start of dimple
                let d_start_y = mid_y - half_l * dir;
                path.push(Point::new(x, d_start_y));
                
                // Dimple peak
                // Dimple sticks out from the finger side.
                // If positive (finger), side goes out (tip_y) to in (base_y) or vice versa.
                // We want the dimple to bulge OUT of the finger material.
                // If positive:
                //   Left side: base -> tip. Bulge is -x direction? No, +x is along edge.
                //   Wait, x is along the edge length. y is depth.
                //   The side is vertical (constant x).
                //   We want the dimple to change x.
                
                // Let's assume dimple bulges OUTWARDS from the finger center.
                // But here we are drawing the outline.
                // If positive (finger), the material is "inside" the finger.
                // Left side of finger: x increases. Side is at x. Material is at x+? No.
                // We are drawing the path.
                // Finger: (x, base) -> (x, tip) -> (x+w, tip) -> (x+w, base).
                // Left side: (x, base) -> (x, tip). Material is to the right (x increasing).
                // So bulge should be to the left (-x).
                // Right side: (x+w, tip) -> (x+w, base). Material is to the left.
                // So bulge should be to the right (+x).
                
                // If negative (notch):
                // (x, base) -> (x, tip) -> (x+w, tip) -> (x+w, base).
                // Left side: (x, base) -> (x, tip). Material is to the left (it's a hole).
                // So bulge should be to the right (+x) (into the hole, making the hole smaller/friction).
                
                // Wait, for friction fit:
                // Finger: dimple sticks OUT (wider finger).
                // Notch: dimple sticks IN (narrower hole).
                
                // So:
                // Left side (base->tip):
                //   Positive: Bulge Left (-x).
                //   Negative: Bulge Right (+x).
                // Right side (tip->base):
                //   Positive: Bulge Right (+x).
                //   Negative: Bulge Left (-x).
                
                let bulge_dir = if y2 < y1 { 
                    // tip -> base (Right side)
                    if positive { 1.0 } else { -1.0 }
                } else {
                    // base -> tip (Left side)
                    if positive { -1.0 } else { 1.0 }
                };

                path.push(Point::new(x + dimple_h * bulge_dir, mid_y));
                
                // End of dimple
                let d_end_y = mid_y + half_l * dir;
                path.push(Point::new(x, d_end_y));
                
                // End of side
                path.push(Point::new(x, y2));
            } else {
                path.push(Point::new(x, y1));
                path.push(Point::new(x, y2));
            }
        };

        // Start point
        path.push(Point::new(x, base_y));
        x += leftover_draw / 2.0;
        path.push(Point::new(x, base_y));

        // Draw fingers
        for i in 0..fingers {
            if positive {
                // Finger protrudes
                // Left side: base -> tip
                draw_side(&mut path, x, base_y, tip_y);
                
                x += finger_draw;
                path.push(Point::new(x, tip_y));
                
                // Right side: tip -> base
                draw_side(&mut path, x, tip_y, base_y);
            } else {
                // Notch for finger
                // Left side: base -> tip
                // path.push(Point::new(x, tip_y)); // Replaced by draw_side
                
                if dogbone {
                    // Dogbone overcut at first corner
                    path.push(Point::new(x, base_y)); // Ensure we start at base
                    path.push(Point::new(x, tip_y + overcut)); // Go past tip
                    path.push(Point::new(x - overcut, tip_y + overcut)); // Dogbone out
                    path.push(Point::new(x, tip_y)); // Back to corner
                } else {
                    draw_side(&mut path, x, base_y, tip_y);
                }

                x += finger_draw;
                path.push(Point::new(x, tip_y));

                if dogbone {
                    // Dogbone overcut at second corner
                    path.push(Point::new(x + overcut, tip_y + overcut));
                    path.push(Point::new(x, tip_y + overcut));
                    path.push(Point::new(x, base_y)); // Back to base
                } else {
                    draw_side(&mut path, x, tip_y, base_y);
                }
            }

            // Space between fingers
            if i < fingers - 1 {
                x += space_draw;
                path.push(Point::new(x, base_y));
            }
        }

        // End with leftover/2
        x += leftover_draw / 2.0;
        path.push(Point::new(x, base_y));

        path
    }

    /// Draw a rectangular wall with finger joints on specified edges
    /// edges: 4-char string, each char: 'f' = finger out, 'F' = finger in, 'e' = plain edge
    /// Edges: [0]=bottom, [1]=right, [2]=top, [3]=left
    fn draw_rectangular_wall(
        &self,
        width: f32,
        height: f32,
        edges: &str,
        start_x: f32,
        start_y: f32,
    ) -> Vec<Point> {
        let mut path: Vec<Point> = Vec::new();
        let edge_chars: Vec<char> = edges.chars().collect();
        let kerf = self.params.burn;
        let half_kerf = kerf / 2.0;

        // Helper to add point with offset check
        fn add_point_to(path: &mut Vec<Point>, p: Point) {
             if let Some(last) = path.last() {
                if (p.x - last.x).abs() < 0.01 && (p.y - last.y).abs() < 0.01 {
                    return;
                }
            }
            path.push(p);
        }

        // Bottom edge: left to right (0,0) → (width,0)
        if let Some(&c) = edge_chars.get(0) {
            if c == 'f' || c == 'F' {
                let base_path = self.draw_finger_edge(width, c == 'f');
                for p in &base_path {
                    add_point_to(&mut path, Point::new(start_x + p.x, start_y + p.y));
                }
            } else {
                // Plain edge. Offset outwards (down)
                add_point_to(&mut path, Point::new(start_x - half_kerf, start_y - half_kerf));
                add_point_to(&mut path, Point::new(start_x + width + half_kerf, start_y - half_kerf));
            }
        }
        // Ensure corner 1 is closed
        add_point_to(&mut path, Point::new(start_x + width + half_kerf, start_y - half_kerf));

        // Right edge: bottom to top (width,0) → (width,height)
        if let Some(&c) = edge_chars.get(1) {
            if c == 'f' || c == 'F' {
                let base_path = self.draw_finger_edge(height, c == 'f');
                for p in &base_path {
                    add_point_to(&mut path, Point::new(start_x + width - p.y, start_y + p.x));
                }
            } else {
                // Plain edge. Offset outwards (right)
                add_point_to(&mut path, Point::new(start_x + width + half_kerf, start_y - half_kerf));
                add_point_to(&mut path, Point::new(start_x + width + half_kerf, start_y + height + half_kerf));
            }
        }
        // Ensure corner 2 is closed
        add_point_to(&mut path, Point::new(start_x + width + half_kerf, start_y + height + half_kerf));

        // Top edge: right to left (width,height) → (0,height)
        if let Some(&c) = edge_chars.get(2) {
            if c == 'F' || c == 'f' {
                let base_path = self.draw_finger_edge(width, c == 'f');
                for p in &base_path {
                    add_point_to(&mut path, Point::new(start_x + width - p.x, start_y + height - p.y));
                }
            } else {
                // Plain edge. Offset outwards (up)
                add_point_to(&mut path, Point::new(start_x + width + half_kerf, start_y + height + half_kerf));
                add_point_to(&mut path, Point::new(start_x - half_kerf, start_y + height + half_kerf));
            }
        }
        // Ensure corner 3 is closed
        add_point_to(&mut path, Point::new(start_x - half_kerf, start_y + height + half_kerf));

        // Left edge: top to bottom (0,height) → (0,0)
        if let Some(&c) = edge_chars.get(3) {
            if c == 'f' || c == 'F' {
                let base_path = self.draw_finger_edge(height, c == 'f');
                for p in &base_path {
                    add_point_to(&mut path, Point::new(start_x + p.y, start_y + height - p.x));
                }
            } else {
                 // Plain edge. Offset outwards (left)
                add_point_to(&mut path, Point::new(start_x - half_kerf, start_y + height + half_kerf));
                add_point_to(&mut path, Point::new(start_x - half_kerf, start_y - half_kerf));
            }
        }
        // Ensure corner 4 is closed (back to start)
        add_point_to(&mut path, Point::new(start_x - half_kerf, start_y - half_kerf));

        // Ensure closed loop by connecting back to the very first point
        if let Some(first) = path.first().cloned() {
            add_point_to(&mut path, first);
        }

        path
    }

    fn pack_paths(&mut self) {
        if self.paths.is_empty() {
            return;
        }

        struct Item {
            path_index: usize,
            width: f32,
            height: f32,
            original_min_x: f32,
            original_min_y: f32,
        }

        let mut items = Vec::new();
        let spacing = 5.0; // Gap between parts

        for (i, path) in self.paths.iter().enumerate() {
            if path.is_empty() { continue; }
            let min_x = path.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
            let max_x = path.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
            let min_y = path.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
            let max_y = path.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
            
            items.push(Item {
                path_index: i,
                width: max_x - min_x,
                height: max_y - min_y,
                original_min_x: min_x,
                original_min_y: min_y,
            });
        }

        // Sort by height descending
        items.sort_by(|a, b| b.height.partial_cmp(&a.height).unwrap_or(std::cmp::Ordering::Equal));

        // Estimate target width as sqrt of total area * 1.5 (aspect ratio preference)
        // Or just ensure it's at least as wide as the widest item
        let total_area: f32 = items.iter().map(|i| i.width * i.height).sum();
        let max_item_width = items.iter().map(|i| i.width).fold(0.0, f32::max);
        let target_width = (total_area.sqrt() * 1.5).max(max_item_width);
        
        let mut current_x = 0.0;
        let mut current_y = 0.0;
        let mut row_height: f32 = 0.0;
        
        // Store new positions: index -> (x, y)
        let mut new_positions = vec![(0.0, 0.0); self.paths.len()];

        for item in &items {
            if current_x > 0.0 && current_x + item.width > target_width {
                // New row
                current_x = 0.0;
                current_y += row_height + spacing;
                row_height = 0.0;
            }

            new_positions[item.path_index] = (current_x, current_y);
            
            row_height = row_height.max(item.height);
            current_x += item.width + spacing;
        }

        // Apply new positions
        for item in items {
            let (new_x, new_y) = new_positions[item.path_index];
            let dx = new_x - item.original_min_x;
            let dy = new_y - item.original_min_y;
            
            for p in &mut self.paths[item.path_index] {
                p.x += dx;
                p.y += dy;
            }
        }
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

        let (has_top, has_bottom, has_front, has_back, has_left, has_right) = match self.params.box_type {
            BoxType::FullBox => (true, true, true, true, true, true),
            BoxType::NoTop => (false, true, true, true, true, true),
            BoxType::NoBottom => (true, false, true, true, true, true),
            BoxType::NoSides | BoxType::NoLeftRight => (true, true, true, true, false, false),
            BoxType::NoFrontBack => (true, true, false, false, true, true),
        };

        // Helper to format edge string
        let edges = |b, r, t, l| {
            let c = |cond, ch| if cond { ch } else { 'e' };
            format!("{}{}{}{}", c(b, 'F'), c(r, 'F'), c(t, 'F'), c(l, 'F'))
        };
        
        // Wall 2/4 edges need 'f' for side connections
        let edges_side = |b, r, t, l| {
            let c = |cond, ch| if cond { ch } else { 'e' };
            format!("{}{}{}{}", c(b, 'F'), c(r, 'f'), c(t, 'F'), c(l, 'f'))
        };

        // Top/Bottom edges need 'f' for all connections
        let edges_tb = |b, r, t, l| {
            let c = |cond, ch| if cond { ch } else { 'e' };
            format!("{}{}{}{}", c(b, 'f'), c(r, 'f'), c(t, 'f'), c(l, 'f'))
        };

        // Wall 1: Front (x × h)
        if has_front {
            let e = edges(has_bottom, has_right, has_top, has_left);
            self.paths.push(self.draw_rectangular_wall(x, h, &e, x_offset, y_offset));
            x_offset += x + spacing;
        }

        // Wall 2: Right (y × h)
        if has_right {
            let e = edges_side(has_bottom, has_back, has_top, has_front);
            self.paths.push(self.draw_rectangular_wall(y, h, &e, x_offset, y_offset));
            y_offset += h + spacing;
            x_offset = 0.0;
        }

        // Wall 4: Left (y × h)
        if has_left {
            let e = edges_side(has_bottom, has_front, has_top, has_back);
            self.paths.push(self.draw_rectangular_wall(y, h, &e, x_offset, y_offset));
            x_offset += y + spacing;
        }

        // Wall 3: Back (x × h)
        if has_back {
            let e = edges(has_bottom, has_left, has_top, has_right);
            self.paths.push(self.draw_rectangular_wall(x, h, &e, x_offset, y_offset));
            x_offset += x + spacing;
        }

        // Top: x × y
        if has_top {
            let e = edges_tb(has_front, has_right, has_back, has_left);
            self.paths.push(self.draw_rectangular_wall(x, y, &e, x_offset, y_offset));
            x_offset += x + spacing;
        }

        // Bottom: x × y
        if has_bottom {
            let e = edges_tb(has_front, has_right, has_back, has_left);
            self.paths.push(self.draw_rectangular_wall(x, y, &e, x_offset, y_offset));
        }

        // Dividers (Placeholder for now, just generating panels if requested)
        if self.params.dividers_x > 0 {
            // Dividers along X axis (spanning Y)
            // Dimensions: y x h
            // Edges: f e f e (fingers on ends, plain top/bottom? No, usually fingers on bottom too)
            // Let's assume fingers on Left/Right/Bottom. Top plain.
            let div_edges = "FfeF"; // Bottom=F, Right=f, Top=e, Left=F
            for _ in 0..self.params.dividers_x {
                 x_offset += y + spacing;
                 self.paths.push(self.draw_rectangular_wall(y, h, div_edges, x_offset, y_offset));
            }
        }

        if self.params.dividers_y > 0 {
            // Dividers along Y axis (spanning X)
            // Dimensions: x x h
            let div_edges = "FfeF";
            for _ in 0..self.params.dividers_y {
                 x_offset += x + spacing;
                 self.paths.push(self.draw_rectangular_wall(x, h, div_edges, x_offset, y_offset));
            }
        }

        if self.params.optimize_layout {
            self.pack_paths();
        }

        Ok(())
    }

    pub fn to_gcode(&self) -> String {
        let mut gcode = String::new();

        gcode.push_str("; Tabbed Box Maker G-code\n");
        gcode.push_str("; Based on https://github.com/florianfesti/boxes\n");
        gcode.push_str(&format!(
            "; Box: {}x{}x{} mm\n",
            self.params.x, self.params.y, self.params.h
        ));
        gcode.push_str(&format!(
            "; Material thickness: {} mm\n",
            self.params.thickness
        ));
        gcode.push_str(&format!(
            "; Finger width: {} * thickness = {} mm\n",
            self.params.finger_joint.finger,
            self.params.finger_joint.finger * self.params.thickness
        ));
        gcode.push_str(&format!(
            "; Space width: {} * thickness = {} mm\n",
            self.params.finger_joint.space,
            self.params.finger_joint.space * self.params.thickness
        ));
        gcode.push_str(&format!(
            "; Play: {} mm\n",
            self.params.finger_joint.play * self.params.thickness
        ));
        gcode.push_str(&format!("; Laser passes: {}\n", self.params.laser_passes));
        gcode.push_str(&format!("; Laser power: S{}\n", self.params.laser_power));
        gcode.push_str(&format!(
            "; Feed rate: {:.0} mm/min\n",
            self.params.feed_rate
        ));
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
        gcode.push_str(&format!(
            "G0 X{:.1} Y{:.1} ; Move to work origin\n",
            self.params.offset_x, self.params.offset_y
        ));
        gcode.push_str("G10 L20 P1 X0 Y0 Z0 ; Set current position as work zero\n");
        gcode.push_str(&format!(
            "G0 Z{:.2} F{:.0} ; Move to safe height\n\n",
            5.0, self.params.feed_rate
        ));

        let panel_names = ["Wall 1", "Wall 2", "Wall 4", "Wall 3", "Top", "Bottom"];

        for (i, path) in self.paths.iter().enumerate() {
            gcode.push_str(&format!(
                "; Panel {}: {}\n",
                i + 1,
                panel_names.get(i).unwrap_or(&"Unknown")
            ));

            if let Some(first_point) = path.first() {
                gcode.push_str(&format!(
                    "G0 X{:.2} Y{:.2} ; Rapid to start\n",
                    first_point.x, first_point.y
                ));

                for pass_num in 1..=self.params.laser_passes {
                    gcode.push_str(&format!(
                        "; Pass {}/{}\n",
                        pass_num, self.params.laser_passes
                    ));
                    gcode.push_str(&format!("M3 S{} ; Laser on\n", self.params.laser_power));

                    for (idx, point) in path.iter().skip(1).enumerate() {
                        if idx == 0 {
                            gcode.push_str(&format!(
                                "G1 X{:.2} Y{:.2} F{:.0}\n",
                                point.x, point.y, self.params.feed_rate
                            ));
                        } else {
                            gcode.push_str(&format!("G1 X{:.2} Y{:.2}\n", point.x, point.y));
                        }
                    }

                    gcode.push_str("M5 ; Laser off\n");

                    if pass_num < self.params.laser_passes {
                        gcode.push_str(&format!(
                            "G0 X{:.2} Y{:.2} ; Return to start\n",
                            first_point.x, first_point.y
                        ));
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
