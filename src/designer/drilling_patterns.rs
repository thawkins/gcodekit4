//! Drilling pattern generation for CAM operations.
//!
//! Generates drilling toolpaths for various hole patterns: linear, circular, and grid.
//! Supports custom hole definitions and automatic pattern generation.

use super::shapes::Point;
use super::toolpath::{Toolpath, ToolpathSegment, ToolpathSegmentType};
use std::f64::consts::PI;

/// Represents a drill operation configuration.
#[derive(Debug, Clone)]
pub struct DrillOperation {
    pub id: String,
    pub hole_diameter: f64,
    pub drill_diameter: f64,
    pub depth: f64,
    pub feed_rate: f64,
    pub plunge_rate: f64,
    pub spindle_speed: u32,
    pub peck_depth: Option<f64>,
}

impl DrillOperation {
    /// Creates a new drill operation with default parameters.
    pub fn new(id: String, hole_diameter: f64, drill_diameter: f64, depth: f64) -> Self {
        Self {
            id,
            hole_diameter,
            drill_diameter,
            depth,
            feed_rate: 120.0,
            plunge_rate: 60.0,
            spindle_speed: 8000,
            peck_depth: None,
        }
    }

    /// Sets the cutting parameters for this drill operation.
    pub fn set_parameters(&mut self, feed_rate: f64, plunge_rate: f64, spindle_speed: u32) {
        self.feed_rate = feed_rate;
        self.plunge_rate = plunge_rate;
        self.spindle_speed = spindle_speed;
    }

    /// Enables peck drilling with the specified depth per peck.
    pub fn set_peck_drilling(&mut self, peck_depth: f64) {
        self.peck_depth = Some(peck_depth);
    }

    /// Disables peck drilling.
    pub fn disable_peck_drilling(&mut self) {
        self.peck_depth = None;
    }

    /// Calculates the number of pecks needed for the total depth.
    pub fn calculate_pecks(&self) -> u32 {
        if let Some(peck) = self.peck_depth {
            ((self.depth.abs() / peck).ceil()) as u32
        } else {
            1
        }
    }
}

/// Types of drilling patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    Linear,
    Circular,
    Grid,
    Custom,
}

impl PatternType {
    /// Returns the name of the pattern type.
    pub fn name(&self) -> &'static str {
        match self {
            PatternType::Linear => "Linear",
            PatternType::Circular => "Circular",
            PatternType::Grid => "Grid",
            PatternType::Custom => "Custom",
        }
    }
}

/// Represents a drilling pattern.
#[derive(Debug, Clone)]
pub struct DrillingPattern {
    pub pattern_type: PatternType,
    pub holes: Vec<Point>,
}

impl DrillingPattern {
    /// Creates a new drilling pattern.
    pub fn new(pattern_type: PatternType) -> Self {
        Self {
            pattern_type,
            holes: Vec::new(),
        }
    }

    /// Adds a hole to the pattern.
    pub fn add_hole(&mut self, point: Point) {
        self.holes.push(point);
    }

    /// Adds multiple holes to the pattern.
    pub fn add_holes(&mut self, points: Vec<Point>) {
        self.holes.extend(points);
    }

    /// Clears all holes from the pattern.
    pub fn clear_holes(&mut self) {
        self.holes.clear();
    }

    /// Gets the total number of holes.
    pub fn hole_count(&self) -> usize {
        self.holes.len()
    }

    /// Creates a linear hole pattern.
    pub fn linear(start: Point, end: Point, count: u32) -> Self {
        let mut pattern = Self::new(PatternType::Linear);

        for i in 0..count {
            let t = i as f64 / (count - 1).max(1) as f64;
            let x = start.x + t * (end.x - start.x);
            let y = start.y + t * (end.y - start.y);
            pattern.add_hole(Point::new(x, y));
        }

        pattern
    }

    /// Creates a circular hole pattern.
    pub fn circular(center: Point, radius: f64, count: u32) -> Self {
        let mut pattern = Self::new(PatternType::Circular);

        for i in 0..count {
            let angle = (i as f64 / count as f64) * 2.0 * PI;
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            pattern.add_hole(Point::new(x, y));
        }

        pattern
    }

    /// Creates a grid hole pattern.
    pub fn grid(start: Point, spacing_x: f64, spacing_y: f64, count_x: u32, count_y: u32) -> Self {
        let mut pattern = Self::new(PatternType::Grid);

        for row in 0..count_y {
            for col in 0..count_x {
                let x = start.x + col as f64 * spacing_x;
                let y = start.y + row as f64 * spacing_y;
                pattern.add_hole(Point::new(x, y));
            }
        }

        pattern
    }

    /// Creates a custom pattern from the given points.
    pub fn custom(points: Vec<Point>) -> Self {
        let mut pattern = Self::new(PatternType::Custom);
        pattern.add_holes(points);
        pattern
    }
}

/// Generates drilling toolpaths from patterns.
pub struct DrillingPatternGenerator {
    operation: DrillOperation,
}

impl DrillingPatternGenerator {
    /// Creates a new drilling pattern generator.
    pub fn new(operation: DrillOperation) -> Self {
        Self { operation }
    }

    /// Generates a toolpath for the given drilling pattern.
    pub fn generate_toolpath(&self, pattern: &DrillingPattern) -> Toolpath {
        let mut toolpath = Toolpath::new(self.operation.drill_diameter, self.operation.depth);

        for hole_point in &pattern.holes {
            if let Some(peck) = self.operation.peck_depth {
                let pecks = self.generate_peck_drilling(*hole_point, peck);
                for segment in pecks {
                    toolpath.add_segment(segment);
                }
            } else {
                let segment = ToolpathSegment::new(
                    ToolpathSegmentType::LinearMove,
                    *hole_point,
                    Point::new(hole_point.x, hole_point.y + self.operation.depth),
                    self.operation.plunge_rate,
                    self.operation.spindle_speed,
                );
                toolpath.add_segment(segment);
            }
        }

        toolpath
    }

    /// Generates peck drilling segments for a single hole.
    fn generate_peck_drilling(&self, hole_point: Point, peck_depth: f64) -> Vec<ToolpathSegment> {
        let mut segments = Vec::new();
        let pecks = self.operation.calculate_pecks();
        let mut current_depth = 0.0;

        for peck_num in 1..=pecks {
            let peck_amount = (peck_depth * peck_num as f64).min(self.operation.depth.abs());
            let end_point = Point::new(hole_point.x, hole_point.y - peck_amount);

            let segment = ToolpathSegment::new(
                ToolpathSegmentType::LinearMove,
                Point::new(hole_point.x, hole_point.y - current_depth),
                end_point,
                self.operation.plunge_rate,
                self.operation.spindle_speed,
            );
            segments.push(segment);

            current_depth = peck_amount;
        }

        segments
    }

    /// Generates toolpaths for a linear pattern.
    pub fn generate_linear_pattern(&self, start: Point, end: Point, count: u32) -> Toolpath {
        let pattern = DrillingPattern::linear(start, end, count);
        self.generate_toolpath(&pattern)
    }

    /// Generates toolpaths for a circular pattern.
    pub fn generate_circular_pattern(&self, center: Point, radius: f64, count: u32) -> Toolpath {
        let pattern = DrillingPattern::circular(center, radius, count);
        self.generate_toolpath(&pattern)
    }

    /// Generates toolpaths for a grid pattern.
    pub fn generate_grid_pattern(
        &self,
        start: Point,
        spacing_x: f64,
        spacing_y: f64,
        count_x: u32,
        count_y: u32,
    ) -> Toolpath {
        let pattern = DrillingPattern::grid(start, spacing_x, spacing_y, count_x, count_y);
        self.generate_toolpath(&pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drill_operation_creation() {
        let op = DrillOperation::new("drill1".to_string(), 6.35, 6.35, -15.0);
        assert_eq!(op.hole_diameter, 6.35);
        assert_eq!(op.depth, -15.0);
    }

    #[test]
    fn test_drill_operation_peck_drilling() {
        let mut op = DrillOperation::new("drill1".to_string(), 6.35, 6.35, -15.0);
        op.set_peck_drilling(5.0);

        let pecks = op.calculate_pecks();
        assert_eq!(pecks, 3);
    }

    #[test]
    fn test_drilling_pattern_linear() {
        let pattern = DrillingPattern::linear(Point::new(0.0, 0.0), Point::new(100.0, 0.0), 5);
        assert_eq!(pattern.hole_count(), 5);
        assert_eq!(pattern.holes[0], Point::new(0.0, 0.0));
        assert_eq!(pattern.holes[4], Point::new(100.0, 0.0));
    }

    #[test]
    fn test_drilling_pattern_circular() {
        let pattern = DrillingPattern::circular(Point::new(50.0, 50.0), 25.0, 8);
        assert_eq!(pattern.hole_count(), 8);
    }

    #[test]
    fn test_drilling_pattern_grid() {
        let pattern = DrillingPattern::grid(Point::new(0.0, 0.0), 10.0, 10.0, 5, 3);
        assert_eq!(pattern.hole_count(), 15);
    }

    #[test]
    fn test_drilling_pattern_custom() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(20.0, 0.0),
        ];
        let pattern = DrillingPattern::custom(points);
        assert_eq!(pattern.hole_count(), 3);
    }

    #[test]
    fn test_drilling_generator_linear() {
        let op = DrillOperation::new("drill1".to_string(), 6.35, 6.35, -15.0);
        let gen = DrillingPatternGenerator::new(op);

        let toolpath = gen.generate_linear_pattern(Point::new(0.0, 0.0), Point::new(100.0, 0.0), 5);
        assert_eq!(toolpath.segments.len(), 5);
    }

    #[test]
    fn test_drilling_generator_circular() {
        let op = DrillOperation::new("drill1".to_string(), 6.35, 6.35, -15.0);
        let gen = DrillingPatternGenerator::new(op);

        let toolpath = gen.generate_circular_pattern(Point::new(50.0, 50.0), 25.0, 8);
        assert_eq!(toolpath.segments.len(), 8);
    }

    #[test]
    fn test_drilling_generator_grid() {
        let op = DrillOperation::new("drill1".to_string(), 6.35, 6.35, -15.0);
        let gen = DrillingPatternGenerator::new(op);

        let toolpath = gen.generate_grid_pattern(Point::new(0.0, 0.0), 10.0, 10.0, 5, 3);
        assert_eq!(toolpath.segments.len(), 15);
    }

    #[test]
    fn test_drilling_pattern_type_names() {
        assert_eq!(PatternType::Linear.name(), "Linear");
        assert_eq!(PatternType::Circular.name(), "Circular");
        assert_eq!(PatternType::Grid.name(), "Grid");
    }
}
