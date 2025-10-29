//! Canvas for drawing and manipulating shapes.

use super::shapes::{Circle, Line, Point, Rectangle, Shape};

/// Canvas coordinates for drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasPoint {
    pub x: f64,
    pub y: f64,
}

impl CanvasPoint {
    /// Creates a new canvas point.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Converts to a design point.
    pub fn to_point(&self) -> Point {
        Point::new(self.x, self.y)
    }
}

impl From<Point> for CanvasPoint {
    fn from(p: Point) -> Self {
        Self::new(p.x, p.y)
    }
}

/// Drawing modes for the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingMode {
    Select,
    Rectangle,
    Circle,
    Line,
}

/// Drawing object on the canvas that can be selected and manipulated.
#[derive(Debug)]
pub struct DrawingObject {
    pub id: u64,
    pub shape: Box<dyn Shape>,
    pub selected: bool,
}

impl DrawingObject {
    /// Creates a new drawing object.
    pub fn new(id: u64, shape: Box<dyn Shape>) -> Self {
        Self {
            id,
            shape,
            selected: false,
        }
    }
}

impl Clone for DrawingObject {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            shape: self.shape.clone_shape(),
            selected: self.selected,
        }
    }
}

/// Canvas state managing shapes and drawing operations.
#[derive(Debug)]
pub struct Canvas {
    shapes: Vec<DrawingObject>,
    next_id: u64,
    mode: DrawingMode,
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
    selected_id: Option<u64>,
}

impl Canvas {
    /// Creates a new canvas.
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            next_id: 1,
            mode: DrawingMode::Select,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            selected_id: None,
        }
    }

    /// Sets the drawing mode.
    pub fn set_mode(&mut self, mode: DrawingMode) {
        self.mode = mode;
    }

    /// Gets the current drawing mode.
    pub fn mode(&self) -> DrawingMode {
        self.mode
    }

    /// Adds a rectangle to the canvas.
    pub fn add_rectangle(&mut self, x: f64, y: f64, width: f64, height: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let rect = Rectangle::new(x, y, width, height);
        self.shapes.push(DrawingObject::new(id, Box::new(rect)));
        id
    }

    /// Adds a circle to the canvas.
    pub fn add_circle(&mut self, center: Point, radius: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let circle = Circle::new(center, radius);
        self.shapes.push(DrawingObject::new(id, Box::new(circle)));
        id
    }

    /// Adds a line to the canvas.
    pub fn add_line(&mut self, start: Point, end: Point) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let line = Line::new(start, end);
        self.shapes.push(DrawingObject::new(id, Box::new(line)));
        id
    }

    /// Selects a shape at the given point, or deselects if no shape at that point.
    pub fn select_at(&mut self, point: &Point) -> Option<u64> {
        for obj in self.shapes.iter_mut().rev() {
            if obj.shape.contains_point(point) {
                obj.selected = true;
                self.selected_id = Some(obj.id);
                return Some(obj.id);
            }
        }

        for obj in self.shapes.iter_mut() {
            obj.selected = false;
        }
        self.selected_id = None;
        None
    }

    /// Gets all shapes on the canvas.
    pub fn shapes(&self) -> &[DrawingObject] {
        &self.shapes
    }

    /// Gets the selected shape ID.
    pub fn selected_id(&self) -> Option<u64> {
        self.selected_id
    }

    /// Removes a shape by ID.
    pub fn remove_shape(&mut self, id: u64) -> bool {
        if let Some(pos) = self.shapes.iter().position(|obj| obj.id == id) {
            self.shapes.remove(pos);
            if self.selected_id == Some(id) {
                self.selected_id = None;
            }
            true
        } else {
            false
        }
    }

    /// Sets zoom level (1.0 = 100%).
    pub fn set_zoom(&mut self, zoom: f64) {
        if zoom > 0.1 && zoom < 10.0 {
            self.zoom = zoom;
        }
    }

    /// Gets current zoom level.
    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    /// Pans the canvas.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    /// Gets the pan offset.
    pub fn pan_offset(&self) -> (f64, f64) {
        (self.pan_x, self.pan_y)
    }

    /// Clears all shapes from the canvas.
    pub fn clear(&mut self) {
        self.shapes.clear();
        self.selected_id = None;
        self.next_id = 1;
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_add_shapes() {
        let mut canvas = Canvas::new();
        let rect_id = canvas.add_rectangle(0.0, 0.0, 10.0, 10.0);
        let circle_id = canvas.add_circle(Point::new(20.0, 20.0), 5.0);

        assert_eq!(canvas.shapes().len(), 2);
        assert_ne!(rect_id, circle_id);
    }

    #[test]
    fn test_canvas_select() {
        let mut canvas = Canvas::new();
        canvas.add_rectangle(0.0, 0.0, 10.0, 10.0);

        let p = Point::new(5.0, 5.0);
        let selected = canvas.select_at(&p);

        assert!(selected.is_some());
        assert_eq!(canvas.selected_id(), selected);
    }

    #[test]
    fn test_canvas_zoom() {
        let mut canvas = Canvas::new();
        canvas.set_zoom(2.0);
        assert_eq!(canvas.zoom(), 2.0);

        canvas.set_zoom(0.05); // Out of range, should stay at 2.0
        assert_eq!(canvas.zoom(), 2.0);

        canvas.set_zoom(0.5); // Valid zoom
        assert_eq!(canvas.zoom(), 0.5);
    }

    #[test]
    fn test_canvas_clear() {
        let mut canvas = Canvas::new();
        canvas.add_rectangle(0.0, 0.0, 10.0, 10.0);
        canvas.clear();

        assert_eq!(canvas.shapes().len(), 0);
        assert_eq!(canvas.selected_id(), None);
    }
}
