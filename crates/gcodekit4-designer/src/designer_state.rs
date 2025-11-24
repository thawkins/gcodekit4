//! Designer state manager for UI integration.
//! Manages the designer canvas state and handles UI callbacks.

use crate::{
    shapes::{OperationType, TextShape},
    Canvas, Circle, DrawingMode, Line, Point, Rectangle, ToolpathGenerator, ToolpathToGcode,
};
use crate::canvas::CanvasSnapshot;
use gcodekit4_core::Units;

/// Designer state for UI integration
#[derive(Clone)]
pub struct DesignerState {
    pub canvas: Canvas,
    pub toolpath_generator: ToolpathGenerator,
    pub generated_gcode: String,
    pub gcode_generated: bool,
    pub current_file_path: Option<std::path::PathBuf>,
    pub is_modified: bool,
    pub design_name: String,
    pub show_grid: bool,
    pub clipboard: Vec<crate::canvas::DrawingObject>,
    undo_stack: Vec<CanvasSnapshot>,
    redo_stack: Vec<CanvasSnapshot>,
}

impl DesignerState {
    /// Creates a new designer state.
    pub fn new() -> Self {
        Self {
            canvas: Canvas::with_size(800.0, 600.0),
            toolpath_generator: ToolpathGenerator::new(),
            generated_gcode: String::new(),
            gcode_generated: false,
            current_file_path: None,
            is_modified: false,
            design_name: "Untitled".to_string(),
            show_grid: true,
            clipboard: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Saves current state to history
    pub fn save_history(&mut self) {
        self.undo_stack.push(self.canvas.get_snapshot());
        self.redo_stack.clear();
        // Limit stack size to 50
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
    }

    /// Undo last change
    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.redo_stack.push(self.canvas.get_snapshot());
            self.canvas.restore_snapshot(snapshot);
            self.gcode_generated = false;
            self.is_modified = true;
        }
    }

    /// Redo last undo
    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack.push(self.canvas.get_snapshot());
            self.canvas.restore_snapshot(snapshot);
            self.gcode_generated = false;
            self.is_modified = true;
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Check if grouping is possible (at least 2 items selected, and at least one is not already in a group)
    pub fn can_group(&self) -> bool {
        let selected: Vec<_> = self.canvas.shapes().iter().filter(|s| s.selected).collect();
        if selected.len() < 2 {
            return false;
        }
        // "activate if there are selected items that do not have groupids"
        // Interpreted as: at least one selected item is not in a group.
        // If all are already grouped, maybe we shouldn't group them again?
        // Or maybe we can merge groups?
        // For now, let's follow the prompt's implication:
        selected.iter().any(|s| s.group_id.is_none())
    }

    /// Check if ungrouping is possible (any selected item is in a group)
    pub fn can_ungroup(&self) -> bool {
        self.canvas.shapes().iter().filter(|s| s.selected).any(|s| s.group_id.is_some())
    }

    /// Clear history stacks
    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Toggle grid visibility
    pub fn toggle_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    /// Sets the drawing mode.
    pub fn set_mode(&mut self, mode: i32) {
        let drawing_mode = match mode {
            0 => DrawingMode::Select,
            1 => DrawingMode::Rectangle,
            2 => DrawingMode::Circle,
            3 => DrawingMode::Line,
            4 => DrawingMode::Ellipse,
            5 => DrawingMode::Polyline,
            6 => DrawingMode::Text,
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
        self.canvas.fit_all_shapes();
    }

    /// Reset view to default (origin at bottom-left with padding)
    pub fn reset_view(&mut self) {
        // Reset zoom to 100%
        self.canvas.set_zoom(1.0);

        // Reset pan to place origin at bottom-left with 5px padding
        // We need to access the viewport to set this up correctly
        // Since we don't have direct access to viewport dimensions here easily without passing them,
        // we'll rely on the viewport's internal size which should be updated by update_designer_ui
        let _height = self.canvas.viewport().canvas_height();

        // In screen coordinates, (0, height) is bottom-left.
        // We want world (0,0) to be at screen (5, height-5).
        // world_to_screen(0,0) = (pan_x, pan_y) usually (depending on implementation)
        // Let's assume standard: screen_x = world_x * zoom + pan_x
        // screen_y = height - (world_y * zoom + pan_y)  <-- typical for Y-up world, Y-down screen
        // If we want screen_x = 5, screen_y = height - 5 for world(0,0):
        // 5 = 0 * 1.0 + pan_x  => pan_x = 5
        // height - 5 = height - (0 * 1.0 + pan_y) => 5 = pan_y

        // So we set pan to (5, 5)
        self.canvas.set_pan(5.0, 5.0);
    }

    /// Deletes the selected shape(s).
    pub fn delete_selected(&mut self) {
        if self.canvas.selected_count() > 0 {
            self.save_history();
            self.canvas.remove_selected();
        }
    }

    /// Get number of selected shapes
    pub fn selected_count(&self) -> usize {
        self.canvas.selected_count()
    }

    /// Copies selected shapes to clipboard
    pub fn copy_selected(&mut self) {
        self.clipboard = self.canvas.shapes().iter()
            .filter(|s| s.selected)
            .cloned()
            .collect();
    }

    /// Pastes shapes from clipboard at specified location
    pub fn paste_at_location(&mut self, x: f64, y: f64) {
        if self.clipboard.is_empty() {
            return;
        }
        self.save_history();

        // Calculate bounding box of clipboard items
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for obj in &self.clipboard {
            let (x1, y1, x2, y2) = obj.shape.bounding_box();
            min_x = min_x.min(x1);
            min_y = min_y.min(y1);
            max_x = max_x.max(x2);
            max_y = max_y.max(y2);
        }

        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;

        // Calculate offset to move center to (x, y)
        let dx = x - center_x;
        let dy = y - center_y;

        self.canvas.paste_objects(&self.clipboard, dx, dy);
        
        self.is_modified = true;
        self.gcode_generated = false;
    }

    /// Align selected shapes by their left edges
    pub fn align_selected_horizontal_left(&mut self) {
        self.save_history();
        if self.canvas.align_selected_left() {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            // If no change, pop the history state we just pushed
            self.undo_stack.pop();
        }
    }

    /// Align selected shapes by their horizontal centers
    pub fn align_selected_horizontal_center(&mut self) {
        self.save_history();
        if self.canvas.align_selected_center() {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            self.undo_stack.pop();
        }
    }

    /// Align selected shapes by their right edges
    pub fn align_selected_horizontal_right(&mut self) {
        self.save_history();
        if self.canvas.align_selected_right() {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            self.undo_stack.pop();
        }
    }

    /// Align selected shapes by their top edges
    pub fn align_selected_vertical_top(&mut self) {
        self.save_history();
        if self.canvas.align_selected_top() {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            self.undo_stack.pop();
        }
    }

    /// Align selected shapes by their vertical centers
    pub fn align_selected_vertical_center(&mut self) {
        self.save_history();
        if self.canvas.align_selected_vertical_center() {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            self.undo_stack.pop();
        }
    }

    /// Align selected shapes by their bottom edges
    pub fn align_selected_vertical_bottom(&mut self) {
        self.save_history();
        if self.canvas.align_selected_bottom() {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            self.undo_stack.pop();
        }
    }

    /// Clears all shapes from the canvas.
    pub fn clear_canvas(&mut self) {
        if !self.canvas.shapes().is_empty() {
            self.save_history();
            self.canvas.clear();
            self.gcode_generated = false;
        }
    }

    /// Generates G-code from the current design.
    pub fn generate_gcode(&mut self) -> String {
        let mut gcode = String::new();
        let gcode_gen = ToolpathToGcode::new(Units::MM, 10.0);
        let mut toolpaths = Vec::new();

        for shape in self.canvas.shapes() {
            // Set strategy for this shape
            self.toolpath_generator
                .set_pocket_strategy(shape.pocket_strategy);

            let shape_toolpaths = match shape.shape.shape_type() {
                crate::ShapeType::Rectangle => {
                    let (x1, y1, x2, y2) = shape.shape.bounding_box();
                    let rect = Rectangle::new(x1, y1, x2 - x1, y2 - y1);
                    if shape.operation_type == OperationType::Pocket {
                        self.toolpath_generator.generate_rectangle_pocket(
                            &rect,
                            shape.pocket_depth,
                            shape.step_down as f64,
                            shape.step_in as f64,
                        )
                    } else {
                        vec![self.toolpath_generator.generate_rectangle_contour(&rect)]
                    }
                }
                crate::ShapeType::Circle => {
                    let (cx, cy, _, _) = shape.shape.bounding_box();
                    let radius =
                        ((shape.shape.bounding_box().2 - shape.shape.bounding_box().0) / 2.0).abs();
                    let circle = Circle::new(Point::new(cx, cy), radius);
                    if shape.operation_type == OperationType::Pocket {
                        self.toolpath_generator.generate_circle_pocket(
                            &circle,
                            shape.pocket_depth,
                            shape.step_down as f64,
                            shape.step_in as f64,
                        )
                    } else {
                        vec![self.toolpath_generator.generate_circle_contour(&circle)]
                    }
                }
                crate::ShapeType::Line => {
                    let (x1, y1, x2, y2) = shape.shape.bounding_box();
                    let line = Line::new(Point::new(x1, y1), Point::new(x2, y2));
                    vec![self.toolpath_generator.generate_line_contour(&line)]
                }
                crate::ShapeType::Ellipse => {
                    let (x1, y1, x2, y2) = shape.shape.bounding_box();
                    let cx = (x1 + x2) / 2.0;
                    let cy = (y1 + y2) / 2.0;
                    let radius = ((x2 - x1).abs().max((y2 - y1).abs())) / 2.0;
                    let circle = Circle::new(Point::new(cx, cy), radius);
                    vec![self.toolpath_generator.generate_circle_contour(&circle)]
                }
                crate::ShapeType::Path => {
                    if let Some(path_shape) = shape
                        .shape
                        .as_any()
                        .downcast_ref::<crate::shapes::PathShape>()
                    {
                        if shape.operation_type == OperationType::Pocket {
                            self.toolpath_generator.generate_path_pocket(
                                path_shape,
                                shape.pocket_depth,
                                shape.step_down as f64,
                                shape.step_in as f64,
                            )
                        } else {
                            vec![self.toolpath_generator.generate_path_contour(path_shape)]
                        }
                    } else {
                        vec![self.toolpath_generator.empty_toolpath()]
                    }
                }
                crate::ShapeType::Text => {
                    if let Some(text) = shape.shape.as_any().downcast_ref::<TextShape>() {
                        vec![self.toolpath_generator.generate_text_toolpath(text)]
                    } else {
                        vec![self.toolpath_generator.empty_toolpath()]
                    }
                }
            };
            toolpaths.extend(shape_toolpaths);
        }

        let total_length: f64 = toolpaths.iter().map(|tp| tp.total_length()).sum();

        // Use settings from first toolpath if available, or defaults
        let (header_speed, header_feed, header_diam, header_depth) =
            if let Some(first) = toolpaths.first() {
                let s = first
                    .segments
                    .first()
                    .map(|seg| seg.spindle_speed)
                    .unwrap_or(3000);
                let f = first
                    .segments
                    .first()
                    .map(|seg| seg.feed_rate)
                    .unwrap_or(100.0);
                (s, f, first.tool_diameter, first.depth)
            } else {
                (3000, 100.0, 3.175, -5.0)
            };

        gcode.push_str(&gcode_gen.generate_header(
            header_speed,
            header_feed,
            header_diam,
            header_depth,
            total_length,
        ));

        let mut line_number = 10;
        for toolpath in toolpaths {
            gcode.push_str(&gcode_gen.generate_body(&toolpath, line_number));
            // Estimate line count increment (rough)
            line_number += (toolpath.segments.len() as u32) * 10;
        }

        gcode.push_str(&gcode_gen.generate_footer());

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
        self.save_history();
        self.canvas.add_rectangle(10.0, 10.0, 50.0, 40.0);
    }

    /// Adds a test circle to the canvas.
    pub fn add_test_circle(&mut self) {
        self.save_history();
        self.canvas.add_circle(Point::new(75.0, 75.0), 20.0);
    }

    /// Adds a test line to the canvas.
    pub fn add_test_line(&mut self) {
        self.save_history();
        self.canvas
            .add_line(Point::new(10.0, 10.0), Point::new(100.0, 100.0));
    }

    /// Groups the selected shapes.
    pub fn group_selected(&mut self) {
        self.save_history();
        self.canvas.group_selected();
        self.is_modified = true;
        self.gcode_generated = false;
    }

    /// Ungroups the selected shapes.
    pub fn ungroup_selected(&mut self) {
        self.save_history();
        self.canvas.ungroup_selected();
        self.is_modified = true;
        self.gcode_generated = false;
    }

    /// Adds a shape to the canvas at the specified position based on current mode.
    pub fn add_shape_at(&mut self, x: f64, y: f64, multi_select: bool) {
        if self.canvas.mode() != DrawingMode::Select {
            self.save_history();
        }
        match self.canvas.mode() {
            DrawingMode::Select => {
                // Select mode - just select shape at position
                self.canvas.select_at(&Point::new(x, y), multi_select);
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
            DrawingMode::Polyline => {
                // Draw regular hexagon with radius 30 centered at click point
                let center = Point::new(x, y);
                let radius = 30.0;
                let sides = 6;
                let mut vertices = Vec::with_capacity(sides);
                for i in 0..sides {
                    let angle = 2.0 * std::f64::consts::PI * (i as f64) / (sides as f64);
                    let vx = center.x + radius * angle.cos();
                    let vy = center.y + radius * angle.sin();
                    vertices.push(Point::new(vx, vy));
                }
                self.canvas.add_polyline(vertices);
            }
            DrawingMode::Text => {
                // Add default text
                self.canvas.add_text("Text".to_string(), x, y, 20.0);
            }
        }
    }

    /// Selects shapes within the given rectangle.
    pub fn select_in_rect(&mut self, x: f64, y: f64, width: f64, height: f64, multi_select: bool) {
        if self.canvas.mode() == DrawingMode::Select {
            self.canvas.select_in_rect(x, y, width, height, multi_select);
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
        self.save_history();
        self.canvas.snap_selected_to_mm();
    }

    /// Deselects all shapes.
    pub fn deselect_all(&mut self) {
        self.canvas.deselect_all();
    }

    /// Selects all shapes.
    pub fn select_all(&mut self) {
        self.canvas.select_all();
    }

    pub fn set_selected_position_and_size(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.set_selected_position_and_size_with_flags(x, y, w, h, true, true);
    }

    pub fn set_selected_position_and_size_with_flags(
        &mut self,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        update_position: bool,
        update_size: bool,
    ) {
        self.save_history();
        if self.canvas.set_selected_position_and_size_with_flags(x, y, w, h, update_position, update_size) {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            self.undo_stack.pop();
        }
    }

    /// Save design to file
    pub fn save_to_file(&mut self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        use crate::serialization::DesignFile;

        let mut design = DesignFile::new(&self.design_name);

        // Save viewport state
        design.viewport.zoom = self.canvas.zoom();
        design.viewport.pan_x = self.canvas.pan_x();
        design.viewport.pan_y = self.canvas.pan_y();

        // Save all shapes
        for obj in self.canvas.shapes() {
            design.shapes.push(DesignFile::from_drawing_object(obj));
        }

        // Save to file
        design.save_to_file(&path)?;

        // Update state
        self.current_file_path = Some(path.as_ref().to_path_buf());
        self.is_modified = false;

        Ok(())
    }

    /// Load design from file
    pub fn load_from_file(&mut self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        use crate::serialization::DesignFile;

        let design = DesignFile::load_from_file(&path)?;

        // Clear existing shapes
        self.canvas.clear();

        // Restore viewport
        self.canvas.set_zoom(design.viewport.zoom);
        self.canvas
            .set_pan(design.viewport.pan_x, design.viewport.pan_y);

        // Restore shapes
        let mut next_id = 1;
        for shape_data in &design.shapes {
            if let Ok(obj) = DesignFile::to_drawing_object(shape_data, next_id) {
                self.canvas.add_shape(obj.shape);
                next_id += 1;
            }
        }

        // Update state
        self.design_name = design.metadata.name.clone();
        self.current_file_path = Some(path.as_ref().to_path_buf());
        self.is_modified = false;
        self.clear_history();

        Ok(())
    }

    /// Create new design (clear all)
    pub fn new_design(&mut self) {
        self.canvas.clear();
        self.generated_gcode.clear();
        self.gcode_generated = false;
        self.current_file_path = None;
        self.is_modified = false;
        self.design_name = "Untitled".to_string();
        self.clear_history();
    }

    /// Mark design as modified
    pub fn mark_modified(&mut self) {
        self.is_modified = true;
    }

    /// Get display name for the design
    pub fn display_name(&self) -> String {
        let name = if let Some(path) = &self.current_file_path {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&self.design_name)
        } else {
            &self.design_name
        };

        if self.is_modified {
            format!("{}*", name)
        } else {
            name.to_string()
        }
    }

    pub fn set_selected_pocket_properties(&mut self, is_pocket: bool, depth: f64) {
        self.save_history();
        let mut changed = false;
        for obj in self.canvas.shapes_mut().iter_mut() {
            if !obj.selected {
                continue;
            }
            let new_type = if is_pocket {
                OperationType::Pocket
            } else {
                OperationType::Profile
            };
            if obj.operation_type != new_type || (obj.pocket_depth - depth).abs() > f64::EPSILON {
                obj.operation_type = new_type;
                obj.pocket_depth = depth;
                changed = true;
            }
        }
        if changed {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            self.undo_stack.pop();
        }
    }

    pub fn set_selected_step_down(&mut self, step_down: f64) {
        self.save_history();
        let mut changed = false;
        for obj in self.canvas.shapes_mut().iter_mut() {
            if !obj.selected {
                continue;
            }
            if (obj.step_down as f64 - step_down).abs() > f64::EPSILON {
                obj.step_down = step_down as f32;
                changed = true;
            }
        }
        if changed {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            self.undo_stack.pop();
        }
    }

    pub fn set_selected_step_in(&mut self, step_in: f64) {
        self.save_history();
        let mut changed = false;
        for obj in self.canvas.shapes_mut().iter_mut() {
            if !obj.selected {
                continue;
            }
            if (obj.step_in as f64 - step_in).abs() > f64::EPSILON {
                obj.step_in = step_in as f32;
                changed = true;
            }
        }
        if changed {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            self.undo_stack.pop();
        }
    }

    pub fn set_selected_text_properties(&mut self, content: &str, font_size: f64) {
        self.save_history();
        if self.canvas.set_selected_text_properties(content, font_size) {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            self.undo_stack.pop();
        }
    }

    pub fn set_selected_pocket_strategy(
        &mut self,
        strategy: crate::pocket_operations::PocketStrategy,
    ) {
        self.save_history();
        let mut changed = false;
        for obj in self.canvas.shapes_mut().iter_mut() {
            if !obj.selected {
                continue;
            }
            if obj.pocket_strategy != strategy {
                obj.pocket_strategy = strategy;
                changed = true;
            }
        }
        if changed {
            self.is_modified = true;
            self.gcode_generated = false;
        } else {
            self.undo_stack.pop();
        }
    }
}

impl Default for DesignerState {
    fn default() -> Self {
        Self::new()
    }
}
