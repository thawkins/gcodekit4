//! Toolpath generation from design shapes.

use super::shapes::{Circle, Line, Point, Rectangle};
use std::f64::consts::PI;

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
pub struct ToolpathGenerator {
    feed_rate: f64,
    spindle_speed: u32,
    tool_diameter: f64,
    cut_depth: f64,
}

impl ToolpathGenerator {
    /// Creates a new toolpath generator with default parameters.
    pub fn new() -> Self {
        Self {
            feed_rate: 100.0,
            spindle_speed: 1000,
            tool_diameter: 3.175, // 1/8 inch
            cut_depth: -5.0,      // 5mm deep
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
