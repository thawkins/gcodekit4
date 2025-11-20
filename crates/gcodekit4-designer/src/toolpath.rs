//! Toolpath generation from design shapes.

use super::shapes::{Circle, Line, Point, Rectangle};
use super::pocket_operations::{PocketGenerator, PocketOperation};
use std::f64::consts::PI;
use rusttype::{OutlineBuilder, Scale, point as rt_point};
use crate::font_manager;

/// Types of toolpath segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolpathSegmentType {
    RapidMove,
    LinearMove,
    ArcMove,
}

/// A single segment of a toolpath.
#[derive(Debug, Clone)]
pub struct ToolpathSegment {
    pub segment_type: ToolpathSegmentType,
    pub start: Point,
    pub end: Point,
    pub feed_rate: f64,
    pub spindle_speed: u32,
}

impl ToolpathSegment {
    /// Creates a new toolpath segment.
    pub fn new(
        segment_type: ToolpathSegmentType,
        start: Point,
        end: Point,
        feed_rate: f64,
        spindle_speed: u32,
    ) -> Self {
        Self {
            segment_type,
            start,
            end,
            feed_rate,
            spindle_speed,
        }
    }
}

/// A complete toolpath made up of multiple segments.
#[derive(Debug, Clone)]
pub struct Toolpath {
    pub segments: Vec<ToolpathSegment>,
    pub tool_diameter: f64,
    pub depth: f64,
}

impl Toolpath {
    /// Creates a new empty toolpath.
    pub fn new(tool_diameter: f64, depth: f64) -> Self {
        Self {
            segments: Vec::new(),
            tool_diameter,
            depth,
        }
    }

    /// Adds a segment to the toolpath.
    pub fn add_segment(&mut self, segment: ToolpathSegment) {
        self.segments.push(segment);
    }

    /// Gets the total length of the toolpath.
    pub fn total_length(&self) -> f64 {
        self.segments
            .iter()
            .map(|seg| seg.start.distance_to(&seg.end))
            .sum()
    }
}

/// Generates toolpaths from design shapes.
#[derive(Debug, Clone)]
pub struct ToolpathGenerator {
    feed_rate: f64,
    spindle_speed: u32,
    tool_diameter: f64,
    cut_depth: f64,
    step_in: f64,
}

impl ToolpathGenerator {
    /// Creates a new toolpath generator with default parameters.
    pub fn new() -> Self {
        Self {
            feed_rate: 100.0,
            spindle_speed: 1000,
            tool_diameter: 3.175, // 1/8 inch
            cut_depth: -5.0,      // 5mm deep
            step_in: 1.0,
        }
    }

    /// Sets the feed rate in mm/min.
    pub fn set_feed_rate(&mut self, feed_rate: f64) {
        self.feed_rate = feed_rate;
    }

    /// Sets the spindle speed in RPM.
    pub fn set_spindle_speed(&mut self, speed: u32) {
        self.spindle_speed = speed;
    }

    /// Sets the tool diameter in mm.
    pub fn set_tool_diameter(&mut self, diameter: f64) {
        self.tool_diameter = diameter;
    }

    /// Sets the cut depth in mm (negative for downward).
    pub fn set_cut_depth(&mut self, depth: f64) {
        self.cut_depth = depth;
    }

    /// Sets the step in (step over) in mm.
    pub fn set_step_in(&mut self, step_in: f64) {
        self.step_in = step_in;
    }

    /// Creates an empty toolpath with current settings.
    pub fn empty_toolpath(&self) -> Toolpath {
        Toolpath::new(self.tool_diameter, self.cut_depth)
    }

    /// Generates a contour toolpath for a rectangle.
    pub fn generate_rectangle_contour(&self, rect: &Rectangle) -> Toolpath {
        let mut toolpath = Toolpath::new(self.tool_diameter, self.cut_depth);

        let corners = rect.corners();

        // Start at first corner with rapid move
        let first_move = ToolpathSegment::new(
            ToolpathSegmentType::RapidMove,
            Point::new(0.0, 0.0),
            corners[0],
            self.feed_rate,
            self.spindle_speed,
        );
        toolpath.add_segment(first_move);

        // Move around the rectangle
        for i in 0..4 {
            let next_i = (i + 1) % 4;
            let segment = ToolpathSegment::new(
                ToolpathSegmentType::LinearMove,
                corners[i],
                corners[next_i],
                self.feed_rate,
                self.spindle_speed,
            );
            toolpath.add_segment(segment);
        }

        // Return to origin with rapid move
        let return_move = ToolpathSegment::new(
            ToolpathSegmentType::RapidMove,
            corners[0],
            Point::new(0.0, 0.0),
            self.feed_rate,
            self.spindle_speed,
        );
        toolpath.add_segment(return_move);

        toolpath
    }

    /// Generates a contour toolpath for a circle.
    pub fn generate_circle_contour(&self, circle: &Circle) -> Toolpath {
        let mut toolpath = Toolpath::new(self.tool_diameter, self.cut_depth);

        // Start at rightmost point of circle with rapid move
        let start_point = Point::new(circle.center.x + circle.radius, circle.center.y);
        let first_move = ToolpathSegment::new(
            ToolpathSegmentType::RapidMove,
            Point::new(0.0, 0.0),
            start_point,
            self.feed_rate,
            self.spindle_speed,
        );
        toolpath.add_segment(first_move);

        // Generate arc segments around the circle (8 segments for smooth motion)
        let num_segments = 8;
        for i in 0..num_segments {
            let angle1 = (i as f64) * 2.0 * PI / (num_segments as f64);
            let angle2 = ((i + 1) as f64) * 2.0 * PI / (num_segments as f64);

            let start = Point::new(
                circle.center.x + circle.radius * angle1.cos(),
                circle.center.y + circle.radius * angle1.sin(),
            );
            let end = Point::new(
                circle.center.x + circle.radius * angle2.cos(),
                circle.center.y + circle.radius * angle2.sin(),
            );

            let segment = ToolpathSegment::new(
                ToolpathSegmentType::LinearMove,
                start,
                end,
                self.feed_rate,
                self.spindle_speed,
            );
            toolpath.add_segment(segment);
        }

        // Return to origin with rapid move
        let return_move = ToolpathSegment::new(
            ToolpathSegmentType::RapidMove,
            start_point,
            Point::new(0.0, 0.0),
            self.feed_rate,
            self.spindle_speed,
        );
        toolpath.add_segment(return_move);

        toolpath
    }

    /// Generates a contour toolpath for a line.
    pub fn generate_line_contour(&self, line: &Line) -> Toolpath {
        let mut toolpath = Toolpath::new(self.tool_diameter, self.cut_depth);

        // Rapid move to start
        let first_move = ToolpathSegment::new(
            ToolpathSegmentType::RapidMove,
            Point::new(0.0, 0.0),
            line.start,
            self.feed_rate,
            self.spindle_speed,
        );
        toolpath.add_segment(first_move);

        // Linear move along the line
        let line_move = ToolpathSegment::new(
            ToolpathSegmentType::LinearMove,
            line.start,
            line.end,
            self.feed_rate,
            self.spindle_speed,
        );
        toolpath.add_segment(line_move);

        // Return to origin
        let return_move = ToolpathSegment::new(
            ToolpathSegmentType::RapidMove,
            line.end,
            Point::new(0.0, 0.0),
            self.feed_rate,
            self.spindle_speed,
        );
        toolpath.add_segment(return_move);

        toolpath
    }

    /// Generates a pocket toolpath for a rectangle.
    pub fn generate_rectangle_pocket(&self, rect: &Rectangle, pocket_depth: f64) -> Toolpath {
        let op = PocketOperation::new("rect_pocket".to_string(), pocket_depth, self.tool_diameter);
        let mut gen = PocketGenerator::new(op);
        gen.operation.set_parameters(self.step_in, self.feed_rate, self.spindle_speed);
        gen.generate_rectangular_pocket(rect)
    }

    /// Generates a pocket toolpath for a circle.
    pub fn generate_circle_pocket(&self, circle: &Circle, pocket_depth: f64) -> Toolpath {
        let op = PocketOperation::new("circle_pocket".to_string(), pocket_depth, self.tool_diameter);
        let mut gen = PocketGenerator::new(op);
        gen.operation.set_parameters(self.step_in, self.feed_rate, self.spindle_speed);
        gen.generate_circular_pocket(circle)
    }

    /// Generates a toolpath for text.
    pub fn generate_text_toolpath(&self, text_shape: &crate::shapes::TextShape) -> Toolpath {
        let mut toolpath = Toolpath::new(self.tool_diameter, self.cut_depth);
        let font = font_manager::get_font();
        let scale = Scale::uniform(text_shape.font_size as f32);
        let v_metrics = font.v_metrics(scale);
        let start = rt_point(text_shape.x as f32, text_shape.y as f32 + v_metrics.ascent);
        
        for glyph in font.layout(&text_shape.text, scale, start) {
             let mut builder = ToolpathBuilder::new(self.feed_rate, self.spindle_speed);
             glyph.build_outline(&mut builder);
             toolpath.segments.extend(builder.segments);
        }
        
        toolpath
    }
}

struct ToolpathBuilder {
    segments: Vec<ToolpathSegment>,
    current_point: Point,
    start_point: Point,
    feed_rate: f64,
    spindle_speed: u32,
}

impl ToolpathBuilder {
    fn new(feed_rate: f64, spindle_speed: u32) -> Self {
        Self {
            segments: Vec::new(),
            current_point: Point::new(0.0, 0.0),
            start_point: Point::new(0.0, 0.0),
            feed_rate,
            spindle_speed,
        }
    }
}

impl OutlineBuilder for ToolpathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = Point::new(x as f64, y as f64);
        // Rapid move to start of contour (assumed safe height handling in G-code gen)
        self.segments.push(ToolpathSegment::new(
            ToolpathSegmentType::RapidMove,
            self.current_point, 
            p,
            self.feed_rate,
            self.spindle_speed,
        ));
        self.current_point = p;
        self.start_point = p;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = Point::new(x as f64, y as f64);
        self.segments.push(ToolpathSegment::new(
            ToolpathSegmentType::LinearMove,
            self.current_point,
            p,
            self.feed_rate,
            self.spindle_speed,
        ));
        self.current_point = p;
    }

    fn quad_to(&mut self, _x1: f32, _y1: f32, x: f32, y: f32) {
        // Approximate quadratic bezier with line for now
        self.line_to(x, y);
    }

    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, x: f32, y: f32) {
        // Approximate cubic bezier with line for now
        self.line_to(x, y);
    }

    fn close(&mut self) {
        self.segments.push(ToolpathSegment::new(
            ToolpathSegmentType::LinearMove,
            self.current_point,
            self.start_point,
            self.feed_rate,
            self.spindle_speed,
        ));
        self.current_point = self.start_point;
    }
}

impl Default for ToolpathGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolpath_generator_rectangle() {
        let gen = ToolpathGenerator::new();
        let rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        let toolpath = gen.generate_rectangle_contour(&rect);

        assert!(toolpath.segments.len() > 0);
        assert_eq!(toolpath.tool_diameter, 3.175);
        assert_eq!(toolpath.depth, -5.0);
    }

    #[test]
    fn test_toolpath_total_length() {
        let gen = ToolpathGenerator::new();
        let rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        let toolpath = gen.generate_rectangle_contour(&rect);

        let length = toolpath.total_length();
        assert!(length > 0.0);
    }
}
