//! Designer state manager for UI integration.
//! Manages the designer canvas state and handles UI callbacks.

use crate::designer::{Canvas, DrawingMode, Point, Rectangle, Circle, Line, ToolpathGenerator, ToolpathToGcode};
use crate::data::Units;

/// Designer state for UI integration
pub struct DesignerState {
    pub canvas: Canvas,
    pub toolpath_generator: ToolpathGenerator,
    pub generated_gcode: String,
    pub gcode_generated: bool,
}

impl DesignerState {
    /// Creates a new designer state.
    pub fn new() -> Self {
        Self {
            canvas: Canvas::new(),
            toolpath_generator: ToolpathGenerator::new(),
            generated_gcode: String::new(),
            gcode_generated: false,
        }
    }

    /// Sets the drawing mode.
    pub fn set_mode(&mut self, mode: i32) {
        let drawing_mode = match mode {
            0 => DrawingMode::Select,
            1 => DrawingMode::Rectangle,
            2 => DrawingMode::Circle,
            3 => DrawingMode::Line,
            _ => DrawingMode::Select,
        };
        self.canvas.set_mode(drawing_mode);
    }

    /// Zooms in on the canvas.
    pub fn zoom_in(&mut self) {
        let current = self.canvas.zoom();
        let new_zoom = (current * 1.2).min(10.0);
        self.canvas.set_zoom(new_zoom);
    }

    /// Zooms out on the canvas.
    pub fn zoom_out(&mut self) {
        let current = self.canvas.zoom();
        let new_zoom = (current / 1.2).max(0.1);
        self.canvas.set_zoom(new_zoom);
    }

    /// Zoom to fit all shapes.
    pub fn zoom_fit(&mut self) {
        if !self.canvas.shapes().is_empty() {
            self.canvas.set_zoom(1.0);
        }
    }

    /// Deletes the selected shape.
    pub fn delete_selected(&mut self) {
        if let Some(id) = self.canvas.selected_id() {
            self.canvas.remove_shape(id);
        }
    }

    /// Clears all shapes from the canvas.
    pub fn clear_canvas(&mut self) {
        self.canvas.clear();
        self.gcode_generated = false;
    }

    /// Generates G-code from the current design.
    pub fn generate_gcode(&mut self) -> String {
        let mut gcode = String::new();

        // Generate toolpath for each shape
        for shape in self.canvas.shapes() {
            let toolpath = match shape.shape.shape_type() {
                crate::designer::ShapeType::Rectangle => {
                    // Get rectangle bounds from the shape
                    let (x1, y1, x2, y2) = shape.shape.bounding_box();
                    let rect = Rectangle::new(x1, y1, x2 - x1, y2 - y1);
                    self.toolpath_generator.generate_rectangle_contour(&rect)
                }
                crate::designer::ShapeType::Circle => {
                    // Get circle from shape
                    let (cx, cy, _, _) = shape.shape.bounding_box();
                    // For circles, we need radius - estimate from bounds
                    let radius = ((shape.shape.bounding_box().2 - shape.shape.bounding_box().0) / 2.0).abs();
                    let circle = Circle::new(Point::new(cx, cy), radius);
                    self.toolpath_generator.generate_circle_contour(&circle)
                }
                crate::designer::ShapeType::Line => {
                    let (x1, y1, x2, y2) = shape.shape.bounding_box();
                    let line = Line::new(Point::new(x1, y1), Point::new(x2, y2));
                    self.toolpath_generator.generate_line_contour(&line)
                }
            };

            // Generate G-code for this toolpath
            let gcode_gen = ToolpathToGcode::new(Units::MM, 10.0);
            let shape_gcode = gcode_gen.generate(&toolpath);
            gcode.push_str(&shape_gcode);
            gcode.push('\n');
        }

        self.generated_gcode = gcode.clone();
        self.gcode_generated = !self.canvas.shapes().is_empty();
        gcode
    }

    /// Sets feed rate for toolpath generation.
    pub fn set_feed_rate(&mut self, rate: f64) {
        self.toolpath_generator.set_feed_rate(rate);
    }

    /// Sets spindle speed for toolpath generation.
    pub fn set_spindle_speed(&mut self, speed: u32) {
        self.toolpath_generator.set_spindle_speed(speed);
    }

    /// Sets tool diameter for toolpath generation.
    pub fn set_tool_diameter(&mut self, diameter: f64) {
        self.toolpath_generator.set_tool_diameter(diameter);
    }

    /// Sets cut depth for toolpath generation.
    pub fn set_cut_depth(&mut self, depth: f64) {
        self.toolpath_generator.set_cut_depth(depth);
    }
}

impl Default for DesignerState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_designer_state_new() {
        let state = DesignerState::new();
        assert_eq!(state.canvas.shapes().len(), 0);
        assert!(!state.gcode_generated);
    }

    #[test]
    fn test_set_mode() {
        let mut state = DesignerState::new();
        state.set_mode(1);
        assert_eq!(state.canvas.mode(), DrawingMode::Rectangle);
        
        state.set_mode(2);
        assert_eq!(state.canvas.mode(), DrawingMode::Circle);
    }

    #[test]
    fn test_zoom() {
        let mut state = DesignerState::new();
        let initial = state.canvas.zoom();
        
        state.zoom_in();
        assert!(state.canvas.zoom() > initial);
        
        state.zoom_out();
        assert!(state.canvas.zoom() <= initial * 1.1);
    }

    #[test]
    fn test_generate_gcode() {
        let mut state = DesignerState::new();
        state.canvas.add_rectangle(0.0, 0.0, 10.0, 10.0);
        
        let gcode = state.generate_gcode();
        assert!(!gcode.is_empty());
        assert!(state.gcode_generated);
        assert!(gcode.contains("G90"));
    }
}
