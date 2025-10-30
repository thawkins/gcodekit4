//! Pocket operations for CAM toolpath generation.
//!
//! Implements pocket milling operations with island detection and offset path generation.
//! Supports outline pocket and island preservation.

use super::shapes::{Circle, Point, Rectangle};
use super::toolpath::{Toolpath, ToolpathSegment, ToolpathSegmentType};
use std::f64::consts::PI;

/// Represents a pocket operation configuration.
#[derive(Debug, Clone)]
pub struct PocketOperation {
    pub id: String,
    pub depth: f64,
    pub tool_diameter: f64,
    pub stepover: f64,
    pub feed_rate: f64,
    pub spindle_speed: u32,
    pub climb_milling: bool,
}

impl PocketOperation {
    /// Creates a new pocket operation with default parameters.
    pub fn new(id: String, depth: f64, tool_diameter: f64) -> Self {
        Self {
            id,
            depth,
            tool_diameter,
            stepover: tool_diameter / 2.0,
            feed_rate: 100.0,
            spindle_speed: 10000,
            climb_milling: false,
        }
    }

    /// Sets the cutting parameters for this pocket operation.
    pub fn set_parameters(&mut self, stepover: f64, feed_rate: f64, spindle_speed: u32) {
        self.stepover = stepover;
        self.feed_rate = feed_rate;
        self.spindle_speed = spindle_speed;
    }

    /// Enables or disables climb milling.
    pub fn set_climb_milling(&mut self, enable: bool) {
        self.climb_milling = enable;
    }

    /// Calculates the offset distance for the given pass number.
    pub fn calculate_offset(&self, pass: u32) -> f64 {
        self.stepover * pass as f64
    }
}

/// Represents an island within a pocket.
#[derive(Debug, Clone)]
pub struct Island {
    pub center: Point,
    pub radius: f64,
}

impl Island {
    /// Creates a new island.
    pub fn new(center: Point, radius: f64) -> Self {
        Self { center, radius }
    }

    /// Checks if a point is inside the island.
    pub fn contains_point(&self, point: &Point) -> bool {
        self.center.distance_to(point) <= self.radius
    }
}

/// Generates pocket toolpaths with island detection.
pub struct PocketGenerator {
    operation: PocketOperation,
    islands: Vec<Island>,
}

impl PocketGenerator {
    /// Creates a new pocket generator.
    pub fn new(operation: PocketOperation) -> Self {
        Self {
            operation,
            islands: Vec::new(),
        }
    }

    /// Adds an island to the pocket.
    pub fn add_island(&mut self, island: Island) {
        self.islands.push(island);
    }

    /// Adds a circular island.
    pub fn add_circular_island(&mut self, center: Point, radius: f64) {
        self.add_island(Island::new(center, radius));
    }

    /// Clears all islands.
    pub fn clear_islands(&mut self) {
        self.islands.clear();
    }

    /// Checks if a point is in any island.
    fn is_in_island(&self, point: &Point) -> bool {
        self.islands.iter().any(|island| island.contains_point(point))
    }

    /// Generates a pocket toolpath for a rectangular outline.
    pub fn generate_rectangular_pocket(
        &self,
        rect: &Rectangle,
    ) -> Toolpath {
        let mut toolpath = Toolpath::new(self.operation.tool_diameter, self.operation.depth);

        let half_tool = self.operation.tool_diameter / 2.0;
        let passes = ((self.operation.depth.abs() / 2.0).ceil()) as u32;

        for pass in 1..=passes {
            let offset = self.operation.calculate_offset(pass);
            if offset > (rect.width.min(rect.height) / 2.0 - half_tool) {
                break;
            }

            let inset_x = rect.x + offset;
            let inset_y = rect.y + offset;
            let inset_width = (rect.width - 2.0 * offset).max(0.0);
            let inset_height = (rect.height - 2.0 * offset).max(0.0);

            if inset_width <= 0.0 || inset_height <= 0.0 {
                break;
            }

            let _depth = -(self.operation.depth * pass as f64 / passes as f64);

            let points = vec![
                Point::new(inset_x, inset_y),
                Point::new(inset_x + inset_width, inset_y),
                Point::new(inset_x + inset_width, inset_y + inset_height),
                Point::new(inset_x, inset_y + inset_height),
                Point::new(inset_x, inset_y),
            ];

            for window in points.windows(2) {
                let start = window[0];
                let end = window[1];

                if !self.is_in_island(&start) && !self.is_in_island(&end) {
                    let segment = ToolpathSegment::new(
                        ToolpathSegmentType::LinearMove,
                        start,
                        end,
                        self.operation.feed_rate,
                        self.operation.spindle_speed,
                    );
                    toolpath.add_segment(segment);
                }
            }
        }

        toolpath
    }

    /// Generates a pocket toolpath for a circular outline.
    pub fn generate_circular_pocket(&self, circle: &Circle) -> Toolpath {
        let mut toolpath = Toolpath::new(self.operation.tool_diameter, self.operation.depth);

        let half_tool = self.operation.tool_diameter / 2.0;
        let passes = ((self.operation.depth.abs() / 2.0).ceil()) as u32;

        for pass in 1..=passes {
            let offset = self.operation.calculate_offset(pass);
            if offset > (circle.radius - half_tool) {
                break;
            }

            let inset_radius = circle.radius - offset;
            if inset_radius <= half_tool {
                break;
            }

            let _depth = -(self.operation.depth * pass as f64 / passes as f64);
            let segments = 36;

            for i in 0..segments {
                let angle1 = (i as f64 / segments as f64) * 2.0 * PI;
                let angle2 = ((i + 1) as f64 / segments as f64) * 2.0 * PI;

                let x1 = circle.center.x + inset_radius * angle1.cos();
                let y1 = circle.center.y + inset_radius * angle1.sin();
                let x2 = circle.center.x + inset_radius * angle2.cos();
                let y2 = circle.center.y + inset_radius * angle2.sin();

                let start = Point::new(x1, y1);
                let end = Point::new(x2, y2);

                if !self.is_in_island(&start) && !self.is_in_island(&end) {
                    let segment = ToolpathSegment::new(
                        ToolpathSegmentType::LinearMove,
                        start,
                        end,
                        self.operation.feed_rate,
                        self.operation.spindle_speed,
                    );
                    toolpath.add_segment(segment);
                }
            }
        }

        toolpath
    }

    /// Generates offset paths for the pocket boundary.
    pub fn generate_offset_paths(
        &self,
        rect: &Rectangle,
        offset_count: u32,
    ) -> Vec<Vec<Point>> {
        let mut paths = Vec::new();

        for offset_idx in 1..=offset_count {
            let offset = self.operation.calculate_offset(offset_idx);
            if offset > (rect.width.min(rect.height) / 2.0) {
                break;
            }

            let inset_x = rect.x + offset;
            let inset_y = rect.y + offset;
            let inset_width = (rect.width - 2.0 * offset).max(0.0);
            let inset_height = (rect.height - 2.0 * offset).max(0.0);

            if inset_width <= 0.0 || inset_height <= 0.0 {
                break;
            }

            let path = vec![
                Point::new(inset_x, inset_y),
                Point::new(inset_x + inset_width, inset_y),
                Point::new(inset_x + inset_width, inset_y + inset_height),
                Point::new(inset_x, inset_y + inset_height),
            ];

            paths.push(path);
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pocket_operation_creation() {
        let op = PocketOperation::new("pocket1".to_string(), -10.0, 3.175);
        assert_eq!(op.depth, -10.0);
        assert_eq!(op.tool_diameter, 3.175);
    }

    #[test]
    fn test_pocket_operation_calculate_offset() {
        let op = PocketOperation::new("pocket1".to_string(), -10.0, 3.175);
        let offset1 = op.calculate_offset(1);
        let offset2 = op.calculate_offset(2);
        assert!(offset2 > offset1);
    }

    #[test]
    fn test_island_contains_point() {
        let island = Island::new(Point::new(50.0, 50.0), 10.0);
        assert!(island.contains_point(&Point::new(50.0, 50.0)));
        assert!(island.contains_point(&Point::new(55.0, 50.0)));
        assert!(!island.contains_point(&Point::new(65.0, 50.0)));
    }

    #[test]
    fn test_pocket_generator_rectangular() {
        let op = PocketOperation::new("pocket1".to_string(), -10.0, 3.175);
        let gen = PocketGenerator::new(op);
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);

        let toolpath = gen.generate_rectangular_pocket(&rect);
        assert!(toolpath.segments.len() > 0);
    }

    #[test]
    fn test_pocket_generator_circular() {
        let op = PocketOperation::new("pocket1".to_string(), -10.0, 3.175);
        let gen = PocketGenerator::new(op);
        let circle = Circle {
            center: Point::new(50.0, 50.0),
            radius: 25.0,
        };

        let toolpath = gen.generate_circular_pocket(&circle);
        assert!(toolpath.segments.len() > 0);
    }

    #[test]
    fn test_pocket_generator_with_islands() {
        let op = PocketOperation::new("pocket1".to_string(), -10.0, 3.175);
        let mut gen = PocketGenerator::new(op);
        gen.add_circular_island(Point::new(50.0, 50.0), 10.0);

        assert_eq!(gen.islands.len(), 1);
        assert!(gen.is_in_island(&Point::new(50.0, 50.0)));
        assert!(!gen.is_in_island(&Point::new(100.0, 100.0)));
    }

    #[test]
    fn test_pocket_generator_offset_paths() {
        let op = PocketOperation::new("pocket1".to_string(), -10.0, 3.175);
        let gen = PocketGenerator::new(op);
        let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);

        let paths = gen.generate_offset_paths(&rect, 3);
        assert!(paths.len() > 0);
    }
}
