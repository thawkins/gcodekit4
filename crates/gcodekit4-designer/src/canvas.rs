//! Canvas for drawing and manipulating shapes.

use super::pocket_operations::PocketStrategy;
use super::shapes::{
    Circle, Ellipse, Line, OperationType, PathShape, Point, Rectangle, Shape, ShapeType, TextShape,
};
use super::spatial_index::{Bounds, SpatialIndex};
use super::viewport::Viewport;

/// Snapshot of canvas state for undo/redo
#[derive(Clone)]
pub struct CanvasSnapshot {
    shapes: Vec<DrawingObject>,
    next_id: u64,
    spatial_index: SpatialIndex,
}

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
    pub group_id: Option<u64>,
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
            group_id: None,
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
            group_id: self.group_id,
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

#[derive(Debug, Clone, Copy)]
enum Alignment {
    Left,
    Right,
    CenterHorizontal,
    Top,
    Bottom,
    CenterVertical,
}

/// Canvas state managing shapes and drawing operations.
#[derive(Debug, Clone)]
pub struct Canvas {
    shapes: Vec<DrawingObject>,
    next_id: u64,
    mode: DrawingMode,
    viewport: Viewport,
    selected_id: Option<u64>,
    spatial_index: SpatialIndex,
}

impl Canvas {
    /// Creates a new canvas.
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            next_id: 1,
            mode: DrawingMode::Select,
            viewport: Viewport::new(1200.0, 600.0),
            selected_id: None,
            spatial_index: SpatialIndex::default(),
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
            spatial_index: SpatialIndex::default(),
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

    /// Returns the number of shapes on the canvas.
    pub fn shape_count(&self) -> usize {
        self.shapes.len()
    }

    /// Adds a rectangle to the canvas.
    pub fn add_rectangle(&mut self, x: f64, y: f64, width: f64, height: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let rect = Rectangle::new(x, y, width, height);
        let (min_x, min_y, max_x, max_y) = rect.bounding_box();
        self.shapes.push(DrawingObject::new(id, Box::new(rect)));
        self.spatial_index.insert(id, &Bounds::new(min_x, min_y, max_x, max_y));
        id
    }

    /// Adds a circle to the canvas.
    pub fn add_circle(&mut self, center: Point, radius: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let circle = Circle::new(center, radius);
        let (min_x, min_y, max_x, max_y) = circle.bounding_box();
        self.shapes.push(DrawingObject::new(id, Box::new(circle)));
        self.spatial_index.insert(id, &Bounds::new(min_x, min_y, max_x, max_y));
        id
    }

    /// Adds a generic shape to the canvas.
    pub fn add_shape(&mut self, shape: Box<dyn Shape>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let (min_x, min_y, max_x, max_y) = shape.bounding_box();
        self.shapes.push(DrawingObject::new(id, shape));
        self.spatial_index.insert(id, &Bounds::new(min_x, min_y, max_x, max_y));
        id
    }

    /// Adds a line to the canvas.
    pub fn add_line(&mut self, start: Point, end: Point) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let line = Line::new(start, end);
        let (min_x, min_y, max_x, max_y) = line.bounding_box();
        self.shapes.push(DrawingObject::new(id, Box::new(line)));
        self.spatial_index.insert(id, &Bounds::new(min_x, min_y, max_x, max_y));
        id
    }

    /// Adds an ellipse to the canvas.
    pub fn add_ellipse(&mut self, center: Point, rx: f64, ry: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let ellipse = Ellipse::new(center, rx, ry);
        let (min_x, min_y, max_x, max_y) = ellipse.bounding_box();
        self.shapes.push(DrawingObject::new(id, Box::new(ellipse)));
        self.spatial_index.insert(id, &Bounds::new(min_x, min_y, max_x, max_y));
        id
    }

    /// Adds a polyline to the canvas.
    pub fn add_polyline(&mut self, vertices: Vec<Point>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        // Create a closed PathShape from vertices
        let path_shape = PathShape::from_points(&vertices, true);
        let (min_x, min_y, max_x, max_y) = path_shape.bounding_box();
        self.shapes
            .push(DrawingObject::new(id, Box::new(path_shape)));
        self.spatial_index.insert(id, &Bounds::new(min_x, min_y, max_x, max_y));
        id
    }

    /// Adds a text shape to the canvas.
    pub fn add_text(&mut self, text: String, x: f64, y: f64, font_size: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let shape = TextShape::new(text, x, y, font_size);
        let (min_x, min_y, max_x, max_y) = shape.bounding_box();
        self.shapes.push(DrawingObject::new(id, Box::new(shape)));
        self.spatial_index.insert(id, &Bounds::new(min_x, min_y, max_x, max_y));
        id
    }

    /// Groups the selected shapes.
    pub fn group_selected(&mut self) {
        let selected_count = self.shapes.iter().filter(|o| o.selected).count();
        if selected_count < 2 {
            return;
        }

        let group_id = self.next_id;
        self.next_id += 1;

        for obj in self.shapes.iter_mut() {
            if obj.selected {
                obj.group_id = Some(group_id);
            }
        }
    }

    /// Ungroups the selected shapes.
    pub fn ungroup_selected(&mut self) {
        for obj in self.shapes.iter_mut() {
            if obj.selected {
                obj.group_id = None;
            }
        }
    }

    /// Selects a shape at the given point.
    /// If multi is true, toggles selection of the shape at point while keeping others.
    /// If multi is false, clears other selections and selects the shape at point.
    pub fn select_at(&mut self, point: &Point, multi: bool) -> Option<u64> {
        let mut found_id = None;
        let mut found_group_id = None;

        // Query spatial index for candidates
        let candidates = self.spatial_index.query_point(point.x, point.y);
        
        // Find the shape at the point (topmost first)
        // We iterate in reverse to find topmost, but only check candidates
        for obj in self.shapes.iter_mut().rev() {
            if candidates.contains(&obj.id) {
                if obj.shape.contains_point(point) {
                    found_id = Some(obj.id);
                    found_group_id = obj.group_id;
                    break;
                }
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
            // Determine which IDs to select (single shape or whole group)
            let ids_to_select: Vec<u64> = if let Some(gid) = found_group_id {
                self.shapes.iter().filter(|o| o.group_id == Some(gid)).map(|o| o.id).collect()
            } else {
                vec![id]
            };

            // If multi-select, check if we should toggle off (only if all are already selected)
            let all_selected = ids_to_select.iter().all(|&sid| {
                self.shapes.iter().find(|o| o.id == sid).map(|o| o.selected).unwrap_or(false)
            });

            let should_select = if multi { !all_selected } else { true };

            for sid in ids_to_select {
                if let Some(obj) = self.shapes.iter_mut().find(|o| o.id == sid) {
                    obj.selected = should_select;
                }
            }

            if should_select {
                self.selected_id = Some(id); // Set primary to the clicked one
            } else if self.selected_id == Some(id) {
                 self.selected_id = None;
                 // Try to find another selected shape
                 if let Some(other) = self.shapes.iter().find(|o| o.selected) {
                     self.selected_id = Some(other.id);
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
        for obj in self.shapes.iter().filter(|o| o.selected) {
            let (min_x, min_y, max_x, max_y) = obj.shape.bounding_box();
            self.spatial_index.remove(obj.id, &super::spatial_index::Bounds::new(min_x, min_y, max_x, max_y));
        }
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
            let obj = &self.shapes[pos];
            let (min_x, min_y, max_x, max_y) = obj.shape.bounding_box();
            self.spatial_index.remove(id, &super::spatial_index::Bounds::new(min_x, min_y, max_x, max_y));
            
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
        self.spatial_index.clear();
    }

    /// Moves the selected shape by (dx, dy).
    pub fn move_selected(&mut self, dx: f64, dy: f64) {
        let mut updates = Vec::new();
        
        for obj in self.shapes.iter_mut().filter(|o| o.selected) {
            let (old_x1, old_y1, old_x2, old_y2) = obj.shape.bounding_box();
            
            obj.shape = obj.shape.translate(dx, dy);
            
            let (new_x1, new_y1, new_x2, new_y2) = obj.shape.bounding_box();
            updates.push((obj.id, super::spatial_index::Bounds::new(old_x1, old_y1, old_x2, old_y2), super::spatial_index::Bounds::new(new_x1, new_y1, new_x2, new_y2)));
        }
        
        for (id, old_bounds, new_bounds) in updates {
            self.spatial_index.remove(id, &old_bounds);
            self.spatial_index.insert(id, &new_bounds);
        }
    }

    fn align_selected(&mut self, alignment: Alignment) -> bool {
        // 1. Calculate target value
        let target = match alignment {
            Alignment::Left => {
                self.shapes.iter().filter(|o| o.selected)
                    .map(|o| o.shape.bounding_box().0)
                    .fold(f64::INFINITY, f64::min)
            },
            Alignment::Right => {
                self.shapes.iter().filter(|o| o.selected)
                    .map(|o| o.shape.bounding_box().2)
                    .fold(f64::NEG_INFINITY, f64::max)
            },
            Alignment::CenterHorizontal => {
                let (min_x, max_x) = self.shapes.iter().filter(|o| o.selected)
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), o| {
                        let (x1, _, x2, _) = o.shape.bounding_box();
                        (min.min(x1), max.max(x2))
                    });
                if min_x.is_infinite() { f64::INFINITY } else { (min_x + max_x) / 2.0 }
            },
            Alignment::Top => {
                self.shapes.iter().filter(|o| o.selected)
                    .map(|o| o.shape.bounding_box().3)
                    .fold(f64::NEG_INFINITY, f64::max)
            },
            Alignment::Bottom => {
                self.shapes.iter().filter(|o| o.selected)
                    .map(|o| o.shape.bounding_box().1)
                    .fold(f64::INFINITY, f64::min)
            },
            Alignment::CenterVertical => {
                let (min_y, max_y) = self.shapes.iter().filter(|o| o.selected)
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), o| {
                        let (_, y1, _, y2) = o.shape.bounding_box();
                        (min.min(y1), max.max(y2))
                    });
                if min_y.is_infinite() { f64::INFINITY } else { (min_y + max_y) / 2.0 }
            },
        };

        if target.is_infinite() { return false; }

        let mut changed = false;
        let mut updates = Vec::new();

        for obj in self.shapes.iter_mut().filter(|o| o.selected) {
            let (x1, y1, x2, y2) = obj.shape.bounding_box();
            let (dx, dy) = match alignment {
                Alignment::Left => (target - x1, 0.0),
                Alignment::Right => (target - x2, 0.0),
                Alignment::CenterHorizontal => (target - (x1 + x2) / 2.0, 0.0),
                Alignment::Top => (0.0, target - y2),
                Alignment::Bottom => (0.0, target - y1),
                Alignment::CenterVertical => (0.0, target - (y1 + y2) / 2.0),
            };

            if dx.abs() > f64::EPSILON || dy.abs() > f64::EPSILON {
                obj.shape = obj.shape.translate(dx, dy);
                changed = true;
                
                let (new_x1, new_y1, new_x2, new_y2) = obj.shape.bounding_box();
                updates.push((obj.id, super::spatial_index::Bounds::new(x1, y1, x2, y2), super::spatial_index::Bounds::new(new_x1, new_y1, new_x2, new_y2)));
            }
        }

        for (id, old_bounds, new_bounds) in updates {
            self.spatial_index.remove(id, &old_bounds);
            self.spatial_index.insert(id, &new_bounds);
        }

        changed
    }

    /// Pastes objects onto the canvas with an offset.
    /// Returns the IDs of the new objects.
    pub fn paste_objects(&mut self, objects: &[DrawingObject], offset_x: f64, offset_y: f64) -> Vec<u64> {
        let mut new_ids = Vec::new();
        let mut group_map = std::collections::HashMap::new();

        // Deselect all existing shapes first
        for obj in self.shapes.iter_mut() {
            obj.selected = false;
        }
        self.selected_id = None;

        for obj in objects {
            let id = self.next_id;
            self.next_id += 1;

            let mut new_shape = obj.shape.clone_shape();
            new_shape = new_shape.translate(offset_x, offset_y);
            let (min_x, min_y, max_x, max_y) = new_shape.bounding_box();

            // Handle group ID mapping
            let new_group_id = if let Some(old_gid) = obj.group_id {
                if !group_map.contains_key(&old_gid) {
                    let new_gid = self.next_id;
                    self.next_id += 1;
                    group_map.insert(old_gid, new_gid);
                    Some(new_gid)
                } else {
                    Some(group_map[&old_gid])
                }
            } else {
                None
            };

            let new_obj = DrawingObject {
                id,
                group_id: new_group_id,
                shape: new_shape,
                selected: true, // Select the new object
                operation_type: obj.operation_type,
                pocket_depth: obj.pocket_depth,
                step_down: obj.step_down,
                step_in: obj.step_in,
                pocket_strategy: obj.pocket_strategy,
            };

            self.shapes.push(new_obj);
            self.spatial_index.insert(id, &Bounds::new(min_x, min_y, max_x, max_y));
            new_ids.push(id);
        }
        
        // Update selected_id to the last pasted object if any
        if let Some(last_id) = new_ids.last() {
            self.selected_id = Some(*last_id);
        }
        
        new_ids
    }

    pub fn align_selected_left(&mut self) -> bool {
        self.align_selected(Alignment::Left)
    }

    pub fn get_snapshot(&self) -> CanvasSnapshot {
        CanvasSnapshot {
            shapes: self.shapes.clone(),
            next_id: self.next_id,
            spatial_index: self.spatial_index.clone(),
        }
    }

    pub fn restore_snapshot(&mut self, snapshot: CanvasSnapshot) {
        self.shapes = snapshot.shapes;
        self.next_id = snapshot.next_id;
        self.spatial_index = snapshot.spatial_index;
        // Clear selection as it might refer to non-existent shapes or be confusing
        // Alternatively, we could try to preserve selection if IDs still exist
        // For now, let's clear it to be safe
        self.selected_id = None; 
    }

    pub fn align_selected_right(&mut self) -> bool {
        self.align_selected(Alignment::Right)
    }

    pub fn align_selected_center(&mut self) -> bool {
        self.align_selected(Alignment::CenterHorizontal)
    }

    pub fn align_selected_top(&mut self) -> bool {
        self.align_selected(Alignment::Top)
    }

    pub fn align_selected_bottom(&mut self) -> bool {
        self.align_selected(Alignment::Bottom)
    }

    pub fn align_selected_vertical_center(&mut self) -> bool {
        self.align_selected(Alignment::CenterVertical)
    }

    /// Resizes the selected shape. Handles: 0=TL, 1=TR, 2=BL, 3=BR, 4=Center (moves)
    pub fn resize_selected(&mut self, handle: usize, dx: f64, dy: f64) {
        // Calculate bounding box of ALL selected shapes
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut has_selected = false;

        for obj in self.shapes.iter().filter(|o| o.selected) {
            let (x1, y1, x2, y2) = obj.shape.bounding_box();
            min_x = min_x.min(x1);
            min_y = min_y.min(y1);
            max_x = max_x.max(x2);
            max_y = max_y.max(y2);
            has_selected = true;
        }

        if !has_selected {
            return;
        }

        // If handle is 4 (move), we just translate all selected shapes
        if handle == 4 {
            self.move_selected(dx, dy);
            return;
        }

        // Calculate new bounding box based on handle movement
        let (new_min_x, new_min_y, new_max_x, new_max_y) = match handle {
            0 => (min_x + dx, min_y + dy, max_x, max_y), // Top-left
            1 => (min_x, min_y + dy, max_x + dx, max_y), // Top-right
            2 => (min_x + dx, min_y, max_x, max_y + dy), // Bottom-left
            3 => (min_x, min_y, max_x + dx, max_y + dy), // Bottom-right
            _ => (min_x, min_y, max_x, max_y),
        };

        let old_width = max_x - min_x;
        let old_height = max_y - min_y;
        let new_width = (new_max_x - new_min_x).abs();
        let new_height = (new_max_y - new_min_y).abs();

        // Calculate scale factors
        let sx = if old_width.abs() > 1e-6 { new_width / old_width } else { 1.0 };
        let sy = if old_height.abs() > 1e-6 { new_height / old_height } else { 1.0 };

        // Center of scaling
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        
        let new_center_x = (new_min_x + new_max_x) / 2.0;
        let new_center_y = (new_min_y + new_max_y) / 2.0;
        
        let t_dx = new_center_x - center_x;
        let t_dy = new_center_y - center_y;

        let mut updates = Vec::new();

        for obj in self.shapes.iter_mut().filter(|o| o.selected) {
            let (old_x1, old_y1, old_x2, old_y2) = obj.shape.bounding_box();
            
            // Scale then translate
            let scaled = obj.shape.scale(sx, sy, Point::new(center_x, center_y));
            obj.shape = scaled.translate(t_dx, t_dy);
            
            let (new_x1, new_y1, new_x2, new_y2) = obj.shape.bounding_box();
            updates.push((obj.id, super::spatial_index::Bounds::new(old_x1, old_y1, old_x2, old_y2), super::spatial_index::Bounds::new(new_x1, new_y1, new_x2, new_y2)));
        }
        
        for (id, old_bounds, new_bounds) in updates {
            self.spatial_index.remove(id, &old_bounds);
            self.spatial_index.insert(id, &new_bounds);
        }
    }

    /// Snaps the selected shape's position to whole millimeters
    pub fn snap_selected_to_mm(&mut self) {
        let mut updates = Vec::new();
        
        for obj in self.shapes.iter_mut().filter(|o| o.selected) {
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
            
            let (new_x1, new_y1, new_x2, new_y2) = obj.shape.bounding_box();
            updates.push((obj.id, super::spatial_index::Bounds::new(x1, y1, x2, y2), super::spatial_index::Bounds::new(new_x1, new_y1, new_x2, new_y2)));
        }
        
        for (id, old_bounds, new_bounds) in updates {
            self.spatial_index.remove(id, &old_bounds);
            self.spatial_index.insert(id, &new_bounds);
        }
    }
    pub fn set_selected_position_and_size_with_flags(
        &mut self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        update_position: bool,
        update_size: bool,
    ) -> bool {
        use crate::shapes::*;

        let mut changed_any = false;
        let mut updates = Vec::new();
        
        for obj in self.shapes.iter_mut() {
            if !obj.selected {
                continue;
            }

            let (old_x, old_y, old_x2, old_y2) = obj.shape.bounding_box();
            let old_w = old_x2 - old_x;
            let old_h = old_y2 - old_y;

            let target_x = if update_position { x } else { old_x };
            let target_y = if update_position { y } else { old_y };
            let target_w = if update_size { w } else { old_w };
            let target_h = if update_size { h } else { old_h };

            match obj.shape.shape_type() {
                crate::ShapeType::Rectangle => {
                    obj.shape = Box::new(Rectangle::new(target_x, target_y, target_w, target_h));
                    changed_any = true;
                }
                crate::ShapeType::Circle => {
                    let radius = target_w.min(target_h) / 2.0;
                    obj.shape = Box::new(Circle::new(
                        Point::new(target_x + radius, target_y + radius),
                        radius,
                    ));
                    changed_any = true;
                }
                crate::ShapeType::Line => {
                    obj.shape = Box::new(Line::new(
                        Point::new(target_x, target_y),
                        Point::new(target_x + target_w, target_y + target_h),
                    ));
                    changed_any = true;
                }
                crate::ShapeType::Ellipse => {
                    let center = Point::new(target_x + target_w / 2.0, target_y + target_h / 2.0);
                    obj.shape = Box::new(Ellipse::new(center, target_w / 2.0, target_h / 2.0));
                    changed_any = true;
                }
                crate::ShapeType::Path => {
                    if let Some(path_shape) =
                        obj.shape.as_any().downcast_ref::<crate::shapes::PathShape>()
                    {
                        let (path_x1, path_y1, path_x2, path_y2) = path_shape.bounding_box();
                        let path_w = path_x2 - path_x1;
                        let path_h = path_y2 - path_y1;

                        let scale_x = if update_size && path_w.abs() > 1e-6 {
                            target_w / path_w
                        } else {
                            1.0
                        };
                        let scale_y = if update_size && path_h.abs() > 1e-6 {
                            target_h / path_h
                        } else {
                            1.0
                        };

                        let center_x = (path_x1 + path_x2) / 2.0;
                        let center_y = (path_y1 + path_y2) / 2.0;

                        let scaled =
                            path_shape.scale(scale_x, scale_y, Point::new(center_x, center_y));

                        let new_center_x = target_x + target_w / 2.0;
                        let new_center_y = target_y + target_h / 2.0;

                        let dx = new_center_x - center_x;
                        let dy = new_center_y - center_y;

                        obj.shape = Box::new(scaled.translate(dx, dy));
                        changed_any = true;
                    }
                }
                crate::ShapeType::Text => {
                    if let Some(text) = obj.shape.as_any().downcast_ref::<TextShape>() {
                        obj.shape = Box::new(TextShape::new(
                            text.text.clone(),
                            target_x,
                            target_y,
                            text.font_size,
                        ));
                        changed_any = true;
                    }
                }
            }
            
            if changed_any {
                let (new_x1, new_y1, new_x2, new_y2) = obj.shape.bounding_box();
                updates.push((obj.id, super::spatial_index::Bounds::new(old_x, old_y, old_x2, old_y2), super::spatial_index::Bounds::new(new_x1, new_y1, new_x2, new_y2)));
            }
        }
        
        for (id, old_bounds, new_bounds) in updates {
            self.spatial_index.remove(id, &old_bounds);
            self.spatial_index.insert(id, &new_bounds);
        }
        
        changed_any
    }
    pub fn set_selected_text_properties(&mut self, content: &str, font_size: f64) -> bool {
        let mut changed = false;
        let mut updates = Vec::new();
        
        for obj in self.shapes.iter_mut() {
            if !obj.selected {
                continue;
            }
            if let Some(text) = obj.shape.as_any().downcast_ref::<TextShape>() {
                let (old_x1, old_y1, old_x2, old_y2) = obj.shape.bounding_box();
                let (x, y) = (text.x, text.y);
                
                obj.shape = Box::new(TextShape::new(content.to_string(), x, y, font_size));
                changed = true;
                
                let (new_x1, new_y1, new_x2, new_y2) = obj.shape.bounding_box();
                updates.push((obj.id, super::spatial_index::Bounds::new(old_x1, old_y1, old_x2, old_y2), super::spatial_index::Bounds::new(new_x1, new_y1, new_x2, new_y2)));
            }
        }
        
        for (id, old_bounds, new_bounds) in updates {
            self.spatial_index.remove(id, &old_bounds);
            self.spatial_index.insert(id, &new_bounds);
        }
        
        changed
    }
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}
