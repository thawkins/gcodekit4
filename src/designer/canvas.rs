//! Canvas for drawing and manipulating shapes.

use super::shapes::{Circle, Line, Point, Rectangle, Shape, ShapeType};

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

    /// Deselects all shapes.
    pub fn deselect_all(&mut self) {
        for obj in self.shapes.iter_mut() {
            obj.selected = false;
        }
        self.selected_id = None;
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

    /// Moves the selected shape by (dx, dy).
    pub fn move_selected(&mut self, dx: f64, dy: f64) {
        if let Some(id) = self.selected_id {
            if let Some(obj) = self.shapes.iter_mut().find(|o| o.id == id) {
                // Get current bounding box
                let (x1, y1, x2, y2) = obj.shape.bounding_box();
                
                // Replace the shape with a moved version
                let shape = &*obj.shape;
                let new_shape: Box<dyn Shape> = match shape.shape_type() {
                    ShapeType::Rectangle => {
                        let width = x2 - x1;
                        let height = y2 - y1;
                        Box::new(Rectangle::new(x1 + dx, y1 + dy, width, height))
                    }
                    ShapeType::Circle => {
                        let center_x = (x1 + x2) / 2.0;
                        let center_y = (y1 + y2) / 2.0;
                        let radius = (x2 - x1) / 2.0;
                        Box::new(Circle::new(Point::new(center_x + dx, center_y + dy), radius))
                    }
                    ShapeType::Line => {
                        Box::new(Line::new(
                            Point::new(x1 + dx, y1 + dy),
                            Point::new(x2 + dx, y2 + dy),
                        ))
                    }
                };
                obj.shape = new_shape;
            }
        }
    }

    /// Resizes the selected shape. Handles: 0=TL, 1=TR, 2=BL, 3=BR, 4=Center (moves)
    pub fn resize_selected(&mut self, handle: usize, dx: f64, dy: f64) {
        if let Some(id) = self.selected_id {
            if let Some(obj) = self.shapes.iter_mut().find(|o| o.id == id) {
                let (x1, y1, x2, y2) = obj.shape.bounding_box();
                let shape = &*obj.shape;
                
                let new_shape: Box<dyn Shape> = match shape.shape_type() {
                    ShapeType::Rectangle => {
                        let (new_x1, new_y1, new_x2, new_y2) = match handle {
                            0 => (x1 + dx, y1 + dy, x2, y2),           // Top-left
                            1 => (x1, y1 + dy, x2 + dx, y2),           // Top-right
                            2 => (x1 + dx, y1, x2, y2 + dy),           // Bottom-left
                            3 => (x1, y1, x2 + dx, y2 + dy),           // Bottom-right
                            4 => (x1 + dx, y1 + dy, x2 + dx, y2 + dy), // Center (move)
                            _ => (x1, y1, x2, y2),
                        };
                        
                        let width = (new_x2 - new_x1).abs();
                        let height = (new_y2 - new_y1).abs();
                        Box::new(Rectangle::new(new_x1.min(new_x2), new_y1.min(new_y2), width, height))
                    }
                    ShapeType::Circle => {
                        let center_x = (x1 + x2) / 2.0;
                        let center_y = (y1 + y2) / 2.0;
                        let radius = (x2 - x1) / 2.0;
                        
                        let (new_cx, new_cy, new_r) = match handle {
                            0 => {
                                // Top-left: resize from opposite corner (bottom-right)
                                let dist = ((dx * dx + dy * dy).sqrt()) / 1.414;
                                (center_x + dx / 2.0, center_y + dy / 2.0, (radius - dist).max(5.0))
                            }
                            1 => {
                                // Top-right: resize from opposite corner (bottom-left)
                                let dist = ((dx * dx + dy * dy).sqrt()) / 1.414;
                                (center_x + dx / 2.0, center_y + dy / 2.0, (radius - dist).max(5.0))
                            }
                            2 => {
                                // Bottom-left: resize from opposite corner (top-right)
                                let dist = ((dx * dx + dy * dy).sqrt()) / 1.414;
                                (center_x + dx / 2.0, center_y + dy / 2.0, (radius - dist).max(5.0))
                            }
                            3 => {
                                // Bottom-right: resize from opposite corner (top-left)
                                let dist = ((dx * dx + dy * dy).sqrt()) / 1.414;
                                (center_x + dx / 2.0, center_y + dy / 2.0, (radius - dist).max(5.0))
                            }
                            4 => (center_x + dx, center_y + dy, radius), // Center (move)
                            _ => (center_x, center_y, radius),
                        };
                        Box::new(Circle::new(Point::new(new_cx, new_cy), new_r))
                    }
                    ShapeType::Line => {
                        let (new_x1, new_y1, new_x2, new_y2) = match handle {
                            0 => (x1 + dx, y1 + dy, x2, y2),           // Move start
                            1 => (x1, y1, x2 + dx, y2 + dy),           // Move end
                            4 => (x1 + dx, y1 + dy, x2 + dx, y2 + dy), // Move both
                            _ => (x1, y1, x2, y2),
                        };
                        Box::new(Line::new(Point::new(new_x1, new_y1), Point::new(new_x2, new_y2)))
                    }
                };
                obj.shape = new_shape;
            }
        }
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
