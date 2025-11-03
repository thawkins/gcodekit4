//! Canvas for drawing and manipulating shapes.

use super::shapes::{
    Circle, Ellipse, Line, Point, Polygon, Rectangle, RoundRectangle, Shape, ShapeType,
};
use super::viewport::Viewport;

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
    Ellipse,
    Polygon,
    RoundRectangle,
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
    viewport: Viewport,
    selected_id: Option<u64>,
}

impl Canvas {
    /// Creates a new canvas.
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            next_id: 1,
            mode: DrawingMode::Select,
            viewport: Viewport::new(800.0, 600.0),
            selected_id: None,
        }
    }

    /// Creates a canvas with specified dimensions.
    pub fn with_size(width: f64, height: f64) -> Self {
        Self {
            shapes: Vec::new(),
            next_id: 1,
            mode: DrawingMode::Select,
            viewport: Viewport::new(width, height),
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

    /// Adds a generic shape to the canvas.
    pub fn add_shape(&mut self, shape: Box<dyn Shape>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.shapes.push(DrawingObject::new(id, shape));
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

    /// Adds an ellipse to the canvas.
    pub fn add_ellipse(&mut self, center: Point, rx: f64, ry: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let ellipse = Ellipse::new(center, rx, ry);
        self.shapes.push(DrawingObject::new(id, Box::new(ellipse)));
        id
    }

    /// Adds a polygon to the canvas.
    pub fn add_polygon(&mut self, vertices: Vec<Point>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let polygon = Polygon::new(vertices);
        self.shapes.push(DrawingObject::new(id, Box::new(polygon)));
        id
    }

    /// Adds a round rectangle to the canvas.
    pub fn add_round_rectangle(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        radius: f64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let round_rect = RoundRectangle::new(x, y, width, height, radius);
        self.shapes
            .push(DrawingObject::new(id, Box::new(round_rect)));
        id
    }

    /// Selects a shape at the given point, or deselects if no shape at that point.
    pub fn select_at(&mut self, point: &Point) -> Option<u64> {
        // Deselect all shapes first
        for obj in self.shapes.iter_mut() {
            obj.selected = false;
        }
        self.selected_id = None;

        // Then try to select the shape at the given point
        for obj in self.shapes.iter_mut().rev() {
            if obj.shape.contains_point(point) {
                obj.selected = true;
                self.selected_id = Some(obj.id);
                return Some(obj.id);
            }
        }

        None
    }

    /// Gets all shapes on the canvas.
    pub fn shapes(&self) -> &[DrawingObject] {
        &self.shapes
    }
    
    /// Returns a mutable reference to the shapes array.
    pub fn shapes_mut(&mut self) -> &mut [DrawingObject] {
        &mut self.shapes
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

    /// Checks if point is inside currently selected shape
    pub fn is_point_in_selected(&self, point: &Point) -> bool {
        if let Some(id) = self.selected_id {
            if let Some(obj) = self.shapes.iter().find(|o| o.id == id) {
                return obj.shape.contains_point(point);
            }
        }
        false
    }

    /// Sets zoom level (1.0 = 100%).
    pub fn set_zoom(&mut self, zoom: f64) {
        self.viewport.set_zoom(zoom);
    }

    /// Gets current zoom level.
    pub fn zoom(&self) -> f64 {
        self.viewport.zoom()
    }

    /// Zooms in.
    pub fn zoom_in(&mut self) {
        self.viewport.zoom_in();
    }

    /// Zooms out.
    pub fn zoom_out(&mut self) {
        self.viewport.zoom_out();
    }

    /// Resets zoom to 100%.
    pub fn reset_zoom(&mut self) {
        self.viewport.reset_zoom();
    }

    /// Sets pan offset.
    pub fn set_pan(&mut self, x: f64, y: f64) {
        self.viewport.set_pan(x, y);
    }

    /// Gets pan X offset.
    pub fn pan_x(&self) -> f64 {
        self.viewport.pan_x()
    }

    /// Gets pan Y offset.
    pub fn pan_y(&self) -> f64 {
        self.viewport.pan_y()
    }

    /// Pans by a delta amount.
    pub fn pan_by(&mut self, dx: f64, dy: f64) {
        self.viewport.pan_by(dx, dy);
    }

    /// Resets pan to origin.
    pub fn reset_pan(&mut self) {
        self.viewport.reset_pan();
    }

    /// Gets a reference to the viewport for coordinate transformations.
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Gets a mutable reference to the viewport.
    pub fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    /// Converts pixel coordinates to world coordinates.
    pub fn pixel_to_world(&self, pixel_x: f64, pixel_y: f64) -> Point {
        self.viewport.pixel_to_world(pixel_x, pixel_y)
    }

    /// Converts world coordinates to pixel coordinates.
    pub fn world_to_pixel(&self, world_x: f64, world_y: f64) -> (f64, f64) {
        self.viewport.world_to_pixel(world_x, world_y)
    }

    /// Fits the canvas to show all shapes with padding.
    pub fn fit_all_shapes(&mut self) {
        if self.shapes.is_empty() {
            self.viewport.reset();
            return;
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for obj in &self.shapes {
            let (x1, y1, x2, y2) = obj.shape.bounding_box();
            min_x = min_x.min(x1);
            min_y = min_y.min(y1);
            max_x = max_x.max(x2);
            max_y = max_y.max(y2);
        }

        self.viewport.fit_to_view(min_x, min_y, max_x, max_y);
    }

    /// Zooms to a point with optional zoom level.
    pub fn zoom_to_point(&mut self, world_point: &Point, zoom: f64) {
        self.viewport.zoom_to_point(world_point, zoom);
    }

    /// Centers the canvas on a point.
    pub fn center_on(&mut self, point: &Point) {
        self.viewport.center_on_point(point);
    }

    /// Resets viewport to default state.
    pub fn reset_view(&mut self) {
        self.viewport.reset();
    }

    /// Gets the pan offset (compatibility method).
    pub fn pan_offset(&self) -> (f64, f64) {
        (self.viewport.pan_x(), self.viewport.pan_y())
    }

    /// Pans the canvas (compatibility method).
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.viewport.pan_by(dx, dy);
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
                        Box::new(Circle::new(
                            Point::new(center_x + dx, center_y + dy),
                            radius,
                        ))
                    }
                    ShapeType::Line => Box::new(Line::new(
                        Point::new(x1 + dx, y1 + dy),
                        Point::new(x2 + dx, y2 + dy),
                    )),
                    ShapeType::Ellipse => {
                        let center_x = (x1 + x2) / 2.0;
                        let center_y = (y1 + y2) / 2.0;
                        let rx = (x2 - x1) / 2.0;
                        let ry = (y2 - y1) / 2.0;
                        Box::new(Ellipse::new(
                            Point::new(center_x + dx, center_y + dy),
                            rx,
                            ry,
                        ))
                    }
                    ShapeType::Polygon => {
                        // For polygon, move all vertices
                        obj.shape.clone_shape()
                    }
                    ShapeType::RoundRectangle => Box::new(RoundRectangle::new(
                        x1 + dx,
                        y1 + dy,
                        x2 - x1,
                        y2 - y1,
                        ((y2 - y1) * 0.20).max(1.0),
                    )),
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
                        Box::new(Rectangle::new(
                            new_x1.min(new_x2),
                            new_y1.min(new_y2),
                            width,
                            height,
                        ))
                    }
                    ShapeType::Circle => {
                        let center_x = (x1 + x2) / 2.0;
                        let center_y = (y1 + y2) / 2.0;
                        let radius = (x2 - x1) / 2.0;

                        let (new_cx, new_cy, new_r) = match handle {
                            0 => {
                                // Top-left: adjust radius by the average of dx and dy movement
                                // Moving handle away from center increases radius
                                let delta = ((-dx) + (-dy)) / 2.0;
                                let new_r = (radius + delta).max(5.0);
                                (center_x, center_y, new_r)
                            }
                            1 => {
                                // Top-right: adjust radius by the average of dx and dy movement
                                let delta = (dx + (-dy)) / 2.0;
                                let new_r = (radius + delta).max(5.0);
                                (center_x, center_y, new_r)
                            }
                            2 => {
                                // Bottom-left: adjust radius by the average of dx and dy movement
                                let delta = ((-dx) + dy) / 2.0;
                                let new_r = (radius + delta).max(5.0);
                                (center_x, center_y, new_r)
                            }
                            3 => {
                                // Bottom-right: adjust radius by the average of dx and dy movement
                                let delta = (dx + dy) / 2.0;
                                let new_r = (radius + delta).max(5.0);
                                (center_x, center_y, new_r)
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
                        Box::new(Line::new(
                            Point::new(new_x1, new_y1),
                            Point::new(new_x2, new_y2),
                        ))
                    }
                    ShapeType::Ellipse => {
                        let center_x = (x1 + x2) / 2.0;
                        let center_y = (y1 + y2) / 2.0;
                        let rx = (x2 - x1) / 2.0;
                        let ry = (y2 - y1) / 2.0;

                        let (new_cx, new_cy, new_rx, new_ry) = match handle {
                            0 => {
                                // Top-left: resize
                                let new_rx = ((center_x - (x1 + dx)) / 1.0).abs().max(5.0);
                                let new_ry = ((center_y - (y1 + dy)) / 1.0).abs().max(5.0);
                                (center_x, center_y, new_rx, new_ry)
                            }
                            1 => {
                                // Top-right: resize
                                let new_rx = ((center_x - (x2 + dx)) / 1.0).abs().max(5.0);
                                let new_ry = ((center_y - (y1 + dy)) / 1.0).abs().max(5.0);
                                (center_x, center_y, new_rx, new_ry)
                            }
                            2 => {
                                // Bottom-left: resize
                                let new_rx = ((center_x - (x1 + dx)) / 1.0).abs().max(5.0);
                                let new_ry = ((center_y - (y2 + dy)) / 1.0).abs().max(5.0);
                                (center_x, center_y, new_rx, new_ry)
                            }
                            3 => {
                                // Bottom-right: resize
                                let new_rx = ((center_x - (x2 + dx)) / 1.0).abs().max(5.0);
                                let new_ry = ((center_y - (y2 + dy)) / 1.0).abs().max(5.0);
                                (center_x, center_y, new_rx, new_ry)
                            }
                            4 => (center_x + dx, center_y + dy, rx, ry), // Center (move)
                            _ => (center_x, center_y, rx, ry),
                        };
                        Box::new(Ellipse::new(Point::new(new_cx, new_cy), new_rx, new_ry))
                    }
                    ShapeType::Polygon => {
                        // For polygon, apply move only
                        if handle == 4 {
                            // Center move: move all vertices
                            obj.shape.clone_shape()
                        } else {
                            obj.shape.clone_shape()
                        }
                    }
                    ShapeType::RoundRectangle => {
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
                        let radius = (height * 0.20).max(1.0);
                        Box::new(RoundRectangle::new(
                            new_x1.min(new_x2),
                            new_y1.min(new_y2),
                            width,
                            height,
                            radius,
                        ))
                    }
                };
                obj.shape = new_shape;
            }
        }
    }

    /// Snaps the selected shape's position to whole millimeters
    pub fn snap_selected_to_mm(&mut self) {
        if let Some(id) = self.selected_id {
            if let Some(obj) = self.shapes.iter_mut().find(|o| o.id == id) {
                let (x1, y1, x2, y2) = obj.shape.bounding_box();
                let width = x2 - x1;
                let height = y2 - y1;

                // Snap the top-left corner and dimensions to whole mm
                let snapped_x1 = (x1 + 0.5).floor();
                let snapped_y1 = (y1 + 0.5).floor();
                let snapped_width = (width + 0.5).floor();
                let snapped_height = (height + 0.5).floor();

                // Replace the shape with snapped position and dimensions
                let shape = &*obj.shape;
                let new_shape: Box<dyn Shape> = match shape.shape_type() {
                    ShapeType::Rectangle => Box::new(Rectangle::new(
                        snapped_x1,
                        snapped_y1,
                        snapped_width,
                        snapped_height,
                    )),
                    ShapeType::Circle => {
                        let radius = snapped_width / 2.0;
                        Box::new(Circle::new(
                            Point::new(snapped_x1 + radius, snapped_y1 + radius),
                            radius,
                        ))
                    }
                    ShapeType::Line => Box::new(Line::new(
                        Point::new(snapped_x1, snapped_y1),
                        Point::new(snapped_x1 + snapped_width, snapped_y1 + snapped_height),
                    )),
                    ShapeType::Ellipse => {
                        let rx = snapped_width / 2.0;
                        let ry = snapped_height / 2.0;
                        Box::new(Ellipse::new(
                            Point::new(snapped_x1 + rx, snapped_y1 + ry),
                            rx,
                            ry,
                        ))
                    }
                    ShapeType::Polygon => obj.shape.clone_shape(),
                    ShapeType::RoundRectangle => {
                        let radius = (snapped_height * 0.20).max(1.0);
                        Box::new(RoundRectangle::new(
                            snapped_x1,
                            snapped_y1,
                            snapped_width,
                            snapped_height,
                            radius,
                        ))
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

    #[test]
    fn test_resize_handle_sequence() {
        let mut canvas = Canvas::with_size(800.0, 600.0);
        canvas.add_rectangle(0.0, 0.0, 100.0, 100.0);
        canvas.select_at(&Point::new(50.0, 50.0));

        // Verify initial state
        let shape = &canvas.shapes()[0];
        let (x1, y1, x2, y2) = shape.shape.bounding_box();
        assert_eq!((x1, y1, x2, y2), (0.0, 0.0, 100.0, 100.0));

        // Drag bottom-left handle down by 20
        canvas.resize_selected(2, 0.0, 20.0);
        let shape = &canvas.shapes()[0];
        let (x1, y1, x2, y2) = shape.shape.bounding_box();
        assert_eq!((x1, y1, x2, y2), (0.0, 0.0, 100.0, 120.0));

        // Drag center handle by (10, 10)
        canvas.resize_selected(4, 10.0, 10.0);
        let shape = &canvas.shapes()[0];
        let (x1, y1, x2, y2) = shape.shape.bounding_box();
        // Expected: center was at (50, 60), moving by (10, 10) should give (60, 70)
        // Which means rect should be at (10, 10, 110, 130)
        assert_eq!((x1, y1, x2, y2), (10.0, 10.0, 110.0, 130.0));
    }

    #[test]
    fn test_deselect_by_clicking_empty_space() {
        let mut canvas = Canvas::new();
        let rect_id = canvas.add_rectangle(0.0, 0.0, 10.0, 10.0);

        // Select the rectangle
        let p = Point::new(5.0, 5.0);
        let selected = canvas.select_at(&p);
        assert_eq!(selected, Some(rect_id));
        assert_eq!(canvas.selected_id(), Some(rect_id));

        // Click on empty space (far away from rectangle)
        let empty_point = Point::new(100.0, 100.0);
        let result = canvas.select_at(&empty_point);

        // Should return None (no shape at that point)
        assert_eq!(result, None);
        // And selected_id should be None (deselected)
        assert_eq!(canvas.selected_id(), None);
    }
}
