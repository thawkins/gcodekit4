//! Designer state manager for UI integration.
//! Manages the designer canvas state and handles UI callbacks.

use crate::{
    shapes::{OperationType, TextShape},
    Canvas, Circle, DrawingMode, Line, Point, Rectangle, ToolpathGenerator, ToolpathToGcode,
};
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
        }
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
        self.canvas.remove_selected();
    }

    /// Get number of selected shapes
    pub fn selected_count(&self) -> usize {
        self.canvas.selected_count()
    }

    /// Align selected shapes by their left edges
    pub fn align_selected_horizontal_left(&mut self) {
        if self.canvas.align_selected_left() {
            self.is_modified = true;
            self.gcode_generated = false;
        }
    }

    /// Align selected shapes by their horizontal centers
    pub fn align_selected_horizontal_center(&mut self) {
        if self.canvas.align_selected_center() {
            self.is_modified = true;
            self.gcode_generated = false;
        }
    }

    /// Align selected shapes by their right edges
    pub fn align_selected_horizontal_right(&mut self) {
        if self.canvas.align_selected_right() {
            self.is_modified = true;
            self.gcode_generated = false;
        }
    }

    /// Align selected shapes by their top edges
    pub fn align_selected_vertical_top(&mut self) {
        if self.canvas.align_selected_top() {
            self.is_modified = true;
            self.gcode_generated = false;
        }
    }

    /// Align selected shapes by their vertical centers
    pub fn align_selected_vertical_center(&mut self) {
        if self.canvas.align_selected_vertical_center() {
            self.is_modified = true;
            self.gcode_generated = false;
        }
    }

    /// Align selected shapes by their bottom edges
    pub fn align_selected_vertical_bottom(&mut self) {
        if self.canvas.align_selected_bottom() {
            self.is_modified = true;
            self.gcode_generated = false;
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
    pub fn add_shape_at(&mut self, x: f64, y: f64, multi_select: bool) {
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
        use crate::shapes::*;

        let mut changed_any = false;
        for obj in self.canvas.shapes_mut().iter_mut() {
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
        }

        if changed_any {
            self.is_modified = true;
            self.gcode_generated = false;
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
        }
    }

    pub fn set_selected_step_down(&mut self, step_down: f64) {
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
        }
    }

    pub fn set_selected_step_in(&mut self, step_in: f64) {
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
        }
    }

    pub fn set_selected_text_properties(&mut self, content: &str, font_size: f64) {
        let mut changed = false;
        for obj in self.canvas.shapes_mut().iter_mut() {
            if !obj.selected {
                continue;
            }
            if obj.shape.as_any().downcast_ref::<TextShape>().is_some() {
                let (x, y) = {
                    let (x1, y1, _, _) = obj.shape.bounding_box();
                    (x1, y1)
                };
                obj.shape = Box::new(TextShape::new(content.to_string(), x, y, font_size));
                changed = true;
            }
        }
        if changed {
            self.is_modified = true;
            self.gcode_generated = false;
        }
    }

    pub fn set_selected_pocket_strategy(
        &mut self,
        strategy: crate::pocket_operations::PocketStrategy,
    ) {
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
        }
    }
}

impl Default for DesignerState {
    fn default() -> Self {
        Self::new()
    }
}
