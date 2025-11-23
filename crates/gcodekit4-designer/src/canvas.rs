//! Canvas for drawing and manipulating shapes.

use super::pocket_operations::PocketStrategy;
use super::shapes::{
    Circle, Ellipse, Line, OperationType, PathShape, Point, Rectangle, Shape, ShapeType, TextShape,
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
    Polyline,
    Text,
}

/// Drawing object on the canvas that can be selected and manipulated.
#[derive(Debug)]
pub struct DrawingObject {
    pub id: u64,
    pub shape: Box<dyn Shape>,
    pub selected: bool,
    pub operation_type: OperationType,
    pub pocket_depth: f64,
    pub step_down: f32,
    pub step_in: f32,
    pub pocket_strategy: PocketStrategy,
}

impl DrawingObject {
    /// Creates a new drawing object.
    pub fn new(id: u64, shape: Box<dyn Shape>) -> Self {
        Self {
            id,
            shape,
            selected: false,
            operation_type: OperationType::default(),
            pocket_depth: 0.0,
            step_down: 0.0,
            step_in: 0.0,
            pocket_strategy: PocketStrategy::ContourParallel,
        }
    }
}

impl Clone for DrawingObject {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            shape: self.shape.clone_shape(),
            selected: self.selected,
            operation_type: self.operation_type,
            pocket_depth: self.pocket_depth,
            step_down: self.step_down,
            step_in: self.step_in,
            pocket_strategy: self.pocket_strategy,
        }
    }
}

/// Canvas state managing shapes and drawing operations.
#[derive(Debug, Clone)]
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

    /// Adds a polyline to the canvas.
    pub fn add_polyline(&mut self, vertices: Vec<Point>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        // Create a closed PathShape from vertices
        let path_shape = PathShape::from_points(&vertices, true);
        self.shapes
            .push(DrawingObject::new(id, Box::new(path_shape)));
        id
    }

    /// Adds a text shape to the canvas.
    pub fn add_text(&mut self, text: String, x: f64, y: f64, font_size: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let shape = TextShape::new(text, x, y, font_size);
        self.shapes.push(DrawingObject::new(id, Box::new(shape)));
        id
    }

    /// Selects a shape at the given point.
    /// If multi is true, toggles selection of the shape at point while keeping others.
    /// If multi is false, clears other selections and selects the shape at point.
    pub fn select_at(&mut self, point: &Point, multi: bool) -> Option<u64> {
        let mut found_id = None;

        // Find the shape at the point (topmost first)
        for obj in self.shapes.iter_mut().rev() {
            if obj.shape.contains_point(point) {
                found_id = Some(obj.id);
                break;
            }
        }

        if !multi {
            // Deselect all shapes first if not in multi-select mode
            for obj in self.shapes.iter_mut() {
                obj.selected = false;
            }
            self.selected_id = None;
        }

        if let Some(id) = found_id {
            if let Some(obj) = self.shapes.iter_mut().find(|o| o.id == id) {
                if multi {
                    // Toggle selection
                    obj.selected = !obj.selected;
                    if obj.selected {
                        self.selected_id = Some(id); // Update primary selection to most recent
                    } else if self.selected_id == Some(id) {
                        self.selected_id = None; // Deselected primary
                                                 // Try to find another selected shape to be primary
                        if let Some(other) = self.shapes.iter().find(|o| o.selected) {
                            self.selected_id = Some(other.id);
                        }
                    }
                } else {
                    // Single select
                    obj.selected = true;
                    self.selected_id = Some(id);
                }
            }
        } else if !multi {
            // Clicked on empty space without shift -> deselect all
            self.selected_id = None;
        }

        self.selected_id
    }

    /// Gets the number of selected shapes.
    pub fn selected_count(&self) -> usize {
        self.shapes.iter().filter(|o| o.selected).count()
    }

    /// Removes all selected shapes.
    pub fn remove_selected(&mut self) {
        self.shapes.retain(|obj| !obj.selected);
        self.selected_id = None;
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
                obj.shape = Canvas::translate_shape(&*obj.shape, dx, dy);
            }
        }
    }

    fn translate_shape(shape: &dyn Shape, dx: f64, dy: f64) -> Box<dyn Shape> {
        let (x1, y1, x2, y2) = shape.bounding_box();
        match shape.shape_type() {
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
            ShapeType::Path => {
                if let Some(path_shape) = shape.as_any().downcast_ref::<PathShape>() {
                    Box::new(path_shape.translate(dx, dy))
                } else {
                    shape.clone_shape()
                }
            }
            ShapeType::Text => {
                if let Some(text) = shape.as_any().downcast_ref::<TextShape>() {
                    Box::new(TextShape::new(
                        text.text.clone(),
                        text.x + dx,
                        text.y + dy,
                        text.font_size,
                    ))
                } else {
                    shape.clone_shape()
                }
            }
        }
    }

    pub fn align_selected_left(&mut self) -> bool {
        let mut min_x = f64::INFINITY;
        for obj in self.shapes.iter().filter(|o| o.selected) {
            let (x1, _, _, _) = obj.shape.bounding_box();
            if x1 < min_x {
                min_x = x1;
            }
        }
        if !min_x.is_finite() {
            return false;
        }
        let mut changed = false;
        for obj in self.shapes.iter_mut().filter(|o| o.selected) {
            let (x1, _, _, _) = obj.shape.bounding_box();
            let dx = min_x - x1;
            if dx.abs() > f64::EPSILON {
                obj.shape = Canvas::translate_shape(&*obj.shape, dx, 0.0);
                changed = true;
            }
        }
        changed
    }

    pub fn align_selected_right(&mut self) -> bool {
        let mut max_x = f64::NEG_INFINITY;
        for obj in self.shapes.iter().filter(|o| o.selected) {
            let (_, _, x2, _) = obj.shape.bounding_box();
            if x2 > max_x {
                max_x = x2;
            }
        }
        if !max_x.is_finite() {
            return false;
        }
        let mut changed = false;
        for obj in self.shapes.iter_mut().filter(|o| o.selected) {
            let (_, _, x2, _) = obj.shape.bounding_box();
            let dx = max_x - x2;
            if dx.abs() > f64::EPSILON {
                obj.shape = Canvas::translate_shape(&*obj.shape, dx, 0.0);
                changed = true;
            }
        }
        changed
    }

    pub fn align_selected_center(&mut self) -> bool {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        for obj in self.shapes.iter().filter(|o| o.selected) {
            let (x1, _, x2, _) = obj.shape.bounding_box();
            if x1 < min_x {
                min_x = x1;
            }
            if x2 > max_x {
                max_x = x2;
            }
        }
        if !min_x.is_finite() || !max_x.is_finite() {
            return false;
        }
        let target_center = (min_x + max_x) / 2.0;
        let mut changed = false;
        for obj in self.shapes.iter_mut().filter(|o| o.selected) {
            let (x1, _, x2, _) = obj.shape.bounding_box();
            let shape_center = (x1 + x2) / 2.0;
            let dx = target_center - shape_center;
            if dx.abs() > f64::EPSILON {
                obj.shape = Canvas::translate_shape(&*obj.shape, dx, 0.0);
                changed = true;
            }
        }
        changed
    }

    pub fn align_selected_top(&mut self) -> bool {
        let mut max_y = f64::NEG_INFINITY;
        for obj in self.shapes.iter().filter(|o| o.selected) {
            let (_, _, _, y2) = obj.shape.bounding_box();
            if y2 > max_y {
                max_y = y2;
            }
        }
        if !max_y.is_finite() {
            return false;
        }
        let mut changed = false;
        for obj in self.shapes.iter_mut().filter(|o| o.selected) {
            let (_, _, _, y2) = obj.shape.bounding_box();
            let dy = max_y - y2;
            if dy.abs() > f64::EPSILON {
                obj.shape = Canvas::translate_shape(&*obj.shape, 0.0, dy);
                changed = true;
            }
        }
        changed
    }

    pub fn align_selected_bottom(&mut self) -> bool {
        let mut min_y = f64::INFINITY;
        for obj in self.shapes.iter().filter(|o| o.selected) {
            let (_, y1, _, _) = obj.shape.bounding_box();
            if y1 < min_y {
                min_y = y1;
            }
        }
        if !min_y.is_finite() {
            return false;
        }
        let mut changed = false;
        for obj in self.shapes.iter_mut().filter(|o| o.selected) {
            let (_, y1, _, _) = obj.shape.bounding_box();
            let dy = min_y - y1;
            if dy.abs() > f64::EPSILON {
                obj.shape = Canvas::translate_shape(&*obj.shape, 0.0, dy);
                changed = true;
            }
        }
        changed
    }

    pub fn align_selected_vertical_center(&mut self) -> bool {
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for obj in self.shapes.iter().filter(|o| o.selected) {
            let (_, y1, _, y2) = obj.shape.bounding_box();
            if y1 < min_y {
                min_y = y1;
            }
            if y2 > max_y {
                max_y = y2;
            }
        }
        if !min_y.is_finite() || !max_y.is_finite() {
            return false;
        }
        let target_center = (min_y + max_y) / 2.0;
        let mut changed = false;
        for obj in self.shapes.iter_mut().filter(|o| o.selected) {
            let (_, y1, _, y2) = obj.shape.bounding_box();
            let shape_center = (y1 + y2) / 2.0;
            let dy = target_center - shape_center;
            if dy.abs() > f64::EPSILON {
                obj.shape = Canvas::translate_shape(&*obj.shape, 0.0, dy);
                changed = true;
            }
        }
        changed
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
                    ShapeType::Path => {
                        if let Some(path_shape) = shape.as_any().downcast_ref::<PathShape>() {
                            if handle == 4 {
                                Box::new(path_shape.translate(dx, dy))
                            } else {
                                let (new_x1, new_y1, new_x2, new_y2) = match handle {
                                    0 => (x1 + dx, y1 + dy, x2, y2), // Top-left
                                    1 => (x1, y1 + dy, x2 + dx, y2), // Top-right
                                    2 => (x1 + dx, y1, x2, y2 + dy), // Bottom-left
                                    3 => (x1, y1, x2 + dx, y2 + dy), // Bottom-right
                                    _ => (x1, y1, x2, y2),
                                };
                                let width = x2 - x1;
                                let height = y2 - y1;
                                let new_width = (new_x2 - new_x1).abs();
                                let new_height = (new_y2 - new_y1).abs();

                                let sx = if width.abs() > 1e-6 {
                                    new_width / width
                                } else {
                                    1.0
                                };
                                let sy = if height.abs() > 1e-6 {
                                    new_height / height
                                } else {
                                    1.0
                                };

                                let center_x = (x1 + x2) / 2.0;
                                let center_y = (y1 + y2) / 2.0;

                                let scaled =
                                    path_shape.scale(sx, sy, Point::new(center_x, center_y));

                                let new_center_x = (new_x1 + new_x2) / 2.0;
                                let new_center_y = (new_y1 + new_y2) / 2.0;
                                let t_dx = new_center_x - center_x;
                                let t_dy = new_center_y - center_y;

                                Box::new(scaled.translate(t_dx, t_dy))
                            }
                        } else {
                            shape.clone_shape()
                        }
                    }
                    ShapeType::Text => {
                        if handle == 4 {
                            if let Some(text) = shape.as_any().downcast_ref::<TextShape>() {
                                Box::new(TextShape::new(
                                    text.text.clone(),
                                    text.x + dx,
                                    text.y + dy,
                                    text.font_size,
                                ))
                            } else {
                                shape.clone_shape()
                            }
                        } else {
                            shape.clone_shape()
                        }
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
                    ShapeType::Path => {
                        if let Some(path_shape) = shape.as_any().downcast_ref::<PathShape>() {
                            let sx = if width.abs() > 1e-6 {
                                snapped_width / width
                            } else {
                                1.0
                            };
                            let sy = if height.abs() > 1e-6 {
                                snapped_height / height
                            } else {
                                1.0
                            };

                            let center_x = (x1 + x2) / 2.0;
                            let center_y = (y1 + y2) / 2.0;

                            let scaled = path_shape.scale(sx, sy, Point::new(center_x, center_y));

                            let new_center_x = snapped_x1 + snapped_width / 2.0;
                            let new_center_y = snapped_y1 + snapped_height / 2.0;
                            let t_dx = new_center_x - center_x;
                            let t_dy = new_center_y - center_y;

                            Box::new(scaled.translate(t_dx, t_dy))
                        } else {
                            shape.clone_shape()
                        }
                    }
                    ShapeType::Text => {
                        if let Some(text) = shape.as_any().downcast_ref::<TextShape>() {
                            Box::new(TextShape::new(
                                text.text.clone(),
                                snapped_x1,
                                snapped_y1,
                                text.font_size,
                            ))
                        } else {
                            shape.clone_shape()
                        }
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
