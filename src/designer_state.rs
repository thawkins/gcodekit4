//! Designer state manager for UI integration.
//! Manages the designer canvas state and handles UI callbacks.

use crate::data::Units;
use crate::designer::{
    Canvas, Circle, DrawingMode, Line, Point, Polygon, Rectangle, ToolpathGenerator,
    ToolpathToGcode,
};

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
            canvas: Canvas::with_size(800.0, 600.0),
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
            4 => DrawingMode::Ellipse,
            5 => DrawingMode::Polygon,
            6 => DrawingMode::RoundRectangle,
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
                    let radius =
                        ((shape.shape.bounding_box().2 - shape.shape.bounding_box().0) / 2.0).abs();
                    let circle = Circle::new(Point::new(cx, cy), radius);
                    self.toolpath_generator.generate_circle_contour(&circle)
                }
                crate::designer::ShapeType::Line => {
                    let (x1, y1, x2, y2) = shape.shape.bounding_box();
                    let line = Line::new(Point::new(x1, y1), Point::new(x2, y2));
                    self.toolpath_generator.generate_line_contour(&line)
                }
                crate::designer::ShapeType::Ellipse => {
                    // For ellipses, generate circle contour approximation
                    let (x1, y1, x2, y2) = shape.shape.bounding_box();
                    let cx = (x1 + x2) / 2.0;
                    let cy = (y1 + y2) / 2.0;
                    let radius = ((x2 - x1).abs().max((y2 - y1).abs())) / 2.0;
                    let circle = Circle::new(Point::new(cx, cy), radius);
                    self.toolpath_generator.generate_circle_contour(&circle)
                }
                crate::designer::ShapeType::Polygon => {
                    // For polygons, generate rectangle contour as approximation
                    let (x1, y1, x2, y2) = shape.shape.bounding_box();
                    let rect = Rectangle::new(x1, y1, x2 - x1, y2 - y1);
                    self.toolpath_generator.generate_rectangle_contour(&rect)
                }
                crate::designer::ShapeType::RoundRectangle => {
                    // For round rectangles, generate rectangle contour (ignoring corner radius for now)
                    let (x1, y1, x2, y2) = shape.shape.bounding_box();
                    let rect = Rectangle::new(x1, y1, x2 - x1, y2 - y1);
                    self.toolpath_generator.generate_rectangle_contour(&rect)
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

    /// Adds a test rectangle to the canvas.
    pub fn add_test_rectangle(&mut self) {
        self.canvas.add_rectangle(10.0, 10.0, 50.0, 40.0);
    }

    /// Adds a test circle to the canvas.
    pub fn add_test_circle(&mut self) {
        self.canvas.add_circle(Point::new(75.0, 75.0), 20.0);
    }

    /// Adds a test line to the canvas.
    pub fn add_test_line(&mut self) {
        self.canvas
            .add_line(Point::new(10.0, 10.0), Point::new(100.0, 100.0));
    }

    /// Adds a shape to the canvas at the specified position based on current mode.
    pub fn add_shape_at(&mut self, x: f64, y: f64) {
        match self.canvas.mode() {
            DrawingMode::Select => {
                // Select mode - just select shape at position
                self.canvas.select_at(&Point::new(x, y));
            }
            DrawingMode::Rectangle => {
                // Draw 60x40 rectangle starting at click point
                self.canvas.add_rectangle(x, y, 60.0, 40.0);
            }
            DrawingMode::Circle => {
                // Draw circle with radius 25 centered at click point
                self.canvas.add_circle(Point::new(x, y), 25.0);
            }
            DrawingMode::Line => {
                // Draw 50 unit line from click point
                self.canvas
                    .add_line(Point::new(x, y), Point::new(x + 50.0, y));
            }
            DrawingMode::Ellipse => {
                // Draw ellipse with rx=40, ry=25 centered at click point
                self.canvas.add_ellipse(Point::new(x, y), 40.0, 25.0);
            }
            DrawingMode::Polygon => {
                // Draw regular hexagon with radius 30 centered at click point
                self.canvas
                    .add_polygon(Polygon::regular(Point::new(x, y), 30.0, 6).vertices);
            }
            DrawingMode::RoundRectangle => {
                // Draw 60x40 rounded rectangle with default 5% radius
                let height = 40.0_f64;
                let radius = (height * 0.20).max(1.0);
                self.canvas.add_round_rectangle(x, y, 60.0, height, radius);
            }
        }
    }

    /// Moves the selected shape by (dx, dy).
    pub fn move_selected(&mut self, dx: f64, dy: f64) {
        self.canvas.move_selected(dx, dy);
    }

    /// Resizes the selected shape via handle drag.
    /// handle: 0=TL, 1=TR, 2=BL, 3=BR, 4=Center (move)
    pub fn resize_selected(&mut self, handle: usize, dx: f64, dy: f64) {
        self.canvas.resize_selected(handle, dx, dy);
    }

    /// Snaps the selected shape to whole millimeters
    pub fn snap_selected_to_mm(&mut self) {
        self.canvas.snap_selected_to_mm();
    }

    /// Deselects all shapes.
    pub fn deselect_all(&mut self) {
        self.canvas.deselect_all();
    }
    
    /// Updates the corner radius of the selected shape (if it's a RoundRectangle)
    pub fn set_selected_corner_radius(&mut self, radius: f64) {
        use crate::designer::shapes::RoundRectangle;
        
        if let Some(id) = self.canvas.selected_id() {
            if let Some(obj) = self.canvas.shapes_mut().iter_mut().find(|o| o.id == id) {
                // Check if it's a RoundRectangle
                if obj.shape.shape_type() == crate::designer::ShapeType::RoundRectangle {
                    let (x, y, x2, y2) = obj.shape.bounding_box();
                    let width = (x2 - x).abs();
                    let height = (y2 - y).abs();
                    
                    // Create new RoundRectangle with updated radius
                    obj.shape = Box::new(RoundRectangle::new(
                        x.min(x2),
                        y.min(y2),
                        width,
                        height,
                        radius,
                    ));
                }
            }
        }
    }

    pub fn set_selected_position_and_size(&mut self, x: f64, y: f64, w: f64, h: f64) {
        use crate::designer::shapes::*;
        
        if let Some(id) = self.canvas.selected_id() {
            if let Some(obj) = self.canvas.shapes_mut().iter_mut().find(|o| o.id == id) {
                let (old_x, old_y, old_x2, old_y2) = obj.shape.bounding_box();
                let old_w = old_x2 - old_x;
                let old_h = old_y2 - old_y;
                
                match obj.shape.shape_type() {
                    crate::designer::ShapeType::Rectangle => {
                        obj.shape = Box::new(Rectangle::new(x, y, w, h));
                    }
                    crate::designer::ShapeType::Circle => {
                        let radius = w.min(h) / 2.0;
                        obj.shape = Box::new(Circle::new(Point::new(x + radius, y + radius), radius));
                    }
                    crate::designer::ShapeType::Line => {
                        obj.shape = Box::new(Line::new(Point::new(x, y), Point::new(x + w, y + h)));
                    }
                    crate::designer::ShapeType::Ellipse => {
                        let center = Point::new(x + w / 2.0, y + h / 2.0);
                        obj.shape = Box::new(Ellipse::new(center, w / 2.0, h / 2.0));
                    }
                    crate::designer::ShapeType::Polygon => {
                        // For polygon, we recreate a regular hexagon at the new position/size
                        obj.shape = Box::new(Polygon::regular(Point::new(x + w / 2.0, y + h / 2.0), w.min(h) / 2.0, 6));
                    }
                    crate::designer::ShapeType::RoundRectangle => {
                        // Get current radius from bounding box, preserve it
                        let current_radius = if old_w > 0.0 && old_h > 0.0 {
                            5.0 // Default radius, will be overridden in set_selected_corner_radius
                        } else {
                            5.0
                        };
                        obj.shape = Box::new(RoundRectangle::new(x, y, w, h, current_radius));
                    }
                }
            }
        }
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
