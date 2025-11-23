//! 3D Visualizer - Toolpath Rendering - Task 81
//!
//! Render G-code toolpath with color-coding by movement type,
//! current position indicator, and arc rendering support

use crate::visualizer::setup::{Color, Vector3};

/// Movement type for toolpath segments
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementType {
    /// Rapid movement (no cutting)
    Rapid,
    /// Feed movement (cutting)
    Feed,
    /// Arc movement clockwise
    ArcClockwise,
    /// Arc movement counter-clockwise
    ArcCounterClockwise,
}

/// Shared metadata for any movement segment
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementMeta {
    /// Segment classification
    pub movement_type: MovementType,
    /// Optional feed rate
    pub feed_rate: Option<f32>,
}

impl MovementMeta {
    pub fn new(movement_type: MovementType) -> Self {
        Self {
            movement_type,
            feed_rate: None,
        }
    }

    pub fn with_feed_rate(mut self, feed_rate: f32) -> Self {
        self.feed_rate = Some(feed_rate);
        self
    }
}

/// Line segment in toolpath
#[derive(Debug, Clone)]
pub struct LineSegment {
    /// Start point
    pub start: Vector3,
    /// End point
    pub end: Vector3,
    /// Movement meta
    pub meta: MovementMeta,
    /// Segment color (override default)
    pub color: Option<Color>,
}

impl LineSegment {
    /// Create new line segment
    pub fn new(start: Vector3, end: Vector3, movement_type: MovementType) -> Self {
        Self {
            start,
            end,
            meta: MovementMeta::new(movement_type),
            color: None,
        }
    }

    /// Set feed rate
    pub fn with_feed_rate(mut self, feed_rate: f32) -> Self {
        self.meta.feed_rate = Some(feed_rate);
        self
    }

    /// Set custom color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Get segment length
    pub fn length(&self) -> f32 {
        self.start.subtract(self.end).magnitude()
    }

    /// Get segment color (using movement type if not customized)
    pub fn get_color(&self) -> Color {
        if let Some(color) = self.color {
            color
        } else {
            match self.meta.movement_type {
                MovementType::Rapid => Color::orange(),
                MovementType::Feed => Color::new(0.0, 1.0, 0.0),
                MovementType::ArcClockwise => Color::new(1.0, 0.0, 0.0),
                MovementType::ArcCounterClockwise => Color::new(1.0, 0.0, 0.0),
            }
        }
    }
}

/// Arc segment in toolpath
#[derive(Debug, Clone)]
pub struct ArcSegment {
    /// Start point
    pub start: Vector3,
    /// End point
    pub end: Vector3,
    /// Arc center
    pub center: Vector3,
    /// Arc radius
    pub radius: f32,
    /// Movement metadata (direction + feed rate)
    pub meta: MovementMeta,
    /// Number of line segments to approximate arc
    pub segments: u32,
}

impl ArcSegment {
    /// Create new arc segment
    pub fn new(start: Vector3, end: Vector3, center: Vector3, clockwise: bool) -> Self {
        let radius = start.subtract(center).magnitude();
        let movement_type = if clockwise {
            MovementType::ArcClockwise
        } else {
            MovementType::ArcCounterClockwise
        };
        Self {
            start,
            end,
            center,
            radius,
            meta: MovementMeta::new(movement_type),
            segments: 20,
        }
    }

    /// Set feed rate
    pub fn with_feed_rate(mut self, feed_rate: f32) -> Self {
        self.meta.feed_rate = Some(feed_rate);
        self
    }

    /// Set number of segments for arc approximation
    pub fn with_segments(mut self, segments: u32) -> Self {
        self.segments = segments.max(2);
        self
    }

    fn is_clockwise(&self) -> bool {
        self.meta.movement_type == MovementType::ArcClockwise
    }

    /// Calculate arc length
    pub fn length(&self) -> f32 {
        let start_dir = (self.start - self.center).normalize();
        let end_dir = (self.end - self.center).normalize();

        let angle_start = start_dir.y.atan2(start_dir.x);
        let angle_end = end_dir.y.atan2(end_dir.x);

        let mut angle_diff = angle_end - angle_start;
        if self.is_clockwise() && angle_diff > 0.0 {
            angle_diff -= std::f32::consts::TAU;
        } else if !self.is_clockwise() && angle_diff < 0.0 {
            angle_diff += std::f32::consts::TAU;
        }

        // Arc length = radius * angle (in radians)
        // Also account for helical movement (Z change)
        let arc_len = self.radius * angle_diff.abs();
        let z_diff = (self.end.z - self.start.z).abs();

        (arc_len.powi(2) + z_diff.powi(2)).sqrt()
    }

    /// Convert arc to line segments
    pub fn to_line_segments(&self) -> Vec<LineSegment> {
        let mut segments = Vec::new();
        let movement_type = self.meta.movement_type;

        let mut current = self.start;
        let step = 1.0 / (self.segments as f32);

        for i in 1..=self.segments {
            let t = step * i as f32;
            let next = self.interpolate_arc(t);

            let mut segment = LineSegment::new(current, next, movement_type);
            if let Some(feed_rate) = self.meta.feed_rate {
                segment = segment.with_feed_rate(feed_rate);
            }
            segments.push(segment);
            current = next;
        }

        segments
    }

    /// Interpolate point on arc at parameter t (0.0 to 1.0)
    pub fn interpolate_arc(&self, t: f32) -> Vector3 {
        let angle = self.calculate_arc_angle(t);
        Vector3::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
            self.start.z + (self.end.z - self.start.z) * t,
        )
    }

    fn calculate_arc_angle(&self, t: f32) -> f32 {
        let start_dir = (self.start - self.center).normalize();
        let end_dir = (self.end - self.center).normalize();

        let angle_start = start_dir.y.atan2(start_dir.x);
        let angle_end = end_dir.y.atan2(end_dir.x);

        let mut angle_diff = angle_end - angle_start;
        if self.is_clockwise() && angle_diff > 0.0 {
            angle_diff -= std::f32::consts::TAU;
        } else if !self.is_clockwise() && angle_diff < 0.0 {
            angle_diff += std::f32::consts::TAU;
        }

        angle_start + angle_diff * t
    }
}

/// Unified path segment representation
#[derive(Debug, Clone)]
pub enum PathSegment {
    /// Linear movement segment
    Line(LineSegment),
    /// Arc movement segment
    Arc(ArcSegment),
}

impl PathSegment {
    pub fn start(&self) -> Vector3 {
        match self {
            PathSegment::Line(line) => line.start,
            PathSegment::Arc(arc) => arc.start,
        }
    }

    pub fn end(&self) -> Vector3 {
        match self {
            PathSegment::Line(line) => line.end,
            PathSegment::Arc(arc) => arc.end,
        }
    }

    pub fn length(&self) -> f32 {
        match self {
            PathSegment::Line(line) => line.length(),
            PathSegment::Arc(arc) => arc.length(),
        }
    }

    pub fn movement_type(&self) -> MovementType {
        match self {
            PathSegment::Line(line) => line.meta.movement_type,
            PathSegment::Arc(arc) => arc.meta.movement_type,
        }
    }

    pub fn as_line_segments(&self) -> Vec<LineSegment> {
        match self {
            PathSegment::Line(line) => vec![line.clone()],
            PathSegment::Arc(arc) => arc.to_line_segments(),
        }
    }
}

/// Toolpath visualization data
#[derive(Debug, Clone)]
pub struct Toolpath {
    /// Ordered path segments (line or arc)
    pub segments: Vec<PathSegment>,
    /// Current tool position
    pub current_position: Vector3,
    /// Start position
    pub start_position: Vector3,
    /// Total path length
    pub total_length: f32,
    /// Estimated execution time (seconds)
    pub estimated_time: Option<f32>,
}

impl Toolpath {
    /// Create new toolpath
    pub fn new(start_position: Vector3) -> Self {
        Self {
            segments: Vec::new(),
            current_position: start_position,
            start_position,
            total_length: 0.0,
            estimated_time: None,
        }
    }

    /// Add line segment
    pub fn add_line_segment(&mut self, segment: LineSegment) {
        self.total_length += segment.length();
        self.current_position = segment.end;
        self.segments.push(PathSegment::Line(segment));
    }

    /// Add arc segment
    pub fn add_arc_segment(&mut self, segment: ArcSegment) {
        self.total_length += segment.length();
        self.current_position = segment.end;
        self.segments.push(PathSegment::Arc(segment));
    }

    /// Update current position
    pub fn update_current_position(&mut self, position: Vector3) {
        self.current_position = position;
    }

    /// Get all segments as line segments (arcs converted)
    pub fn get_all_line_segments(&self) -> Vec<LineSegment> {
        let mut all_segments = Vec::new();
        self.visit_line_segments(|segment| all_segments.push(segment));
        all_segments
    }

    /// Visit every line segment (arcs discretized on the fly)
    pub fn visit_line_segments<F>(&self, mut visitor: F)
    where
        F: FnMut(LineSegment),
    {
        for segment in &self.segments {
            match segment {
                PathSegment::Line(line) => visitor(line.clone()),
                PathSegment::Arc(arc) => {
                    for line in arc.to_line_segments() {
                        visitor(line);
                    }
                }
            }
        }
    }

    /// Calculate bounding box
    pub fn get_bounding_box(&self) -> Option<(Vector3, Vector3)> {
        let mut min: Option<Vector3> = None;
        let mut max: Option<Vector3> = None;

        self.visit_line_segments(|segment| {
            for point in [segment.start, segment.end] {
                min = Some(match min {
                    Some(current) => Vector3::new(
                        current.x.min(point.x),
                        current.y.min(point.y),
                        current.z.min(point.z),
                    ),
                    None => point,
                });

                max = Some(match max {
                    Some(current) => Vector3::new(
                        current.x.max(point.x),
                        current.y.max(point.y),
                        current.z.max(point.z),
                    ),
                    None => point,
                });
            }
        });

        match (min, max) {
            (Some(min), Some(max)) => Some((min, max)),
            _ => None,
        }
    }

    /// Get center of bounding box
    pub fn get_center(&self) -> Option<Vector3> {
        self.get_bounding_box().map(|(min, max)| {
            Vector3::new(
                (min.x + max.x) / 2.0,
                (min.y + max.y) / 2.0,
                (min.z + max.z) / 2.0,
            )
        })
    }

    /// Set estimated execution time
    pub fn set_estimated_time(&mut self, time_seconds: f32) {
        self.estimated_time = Some(time_seconds.max(0.0));
    }

    /// Clear all segments
    pub fn clear(&mut self) {
        self.segments.clear();
        self.current_position = self.start_position;
        self.total_length = 0.0;
        self.estimated_time = None;
    }

    /// Get statistics
    pub fn get_statistics(&self) -> ToolpathStats {
        let mut totals = SegmentTotals::default();

        self.visit_line_segments(|segment| {
            totals.total += 1;
            match segment.meta.movement_type {
                MovementType::Rapid => totals.rapid += 1,
                MovementType::Feed => totals.feed += 1,
                _ => {}
            }
        });

        ToolpathStats {
            total_segments: totals.total,
            rapid_segments: totals.rapid,
            feed_segments: totals.feed,
            total_length: self.total_length,
            estimated_time: self.estimated_time,
        }
    }
}

#[derive(Default)]
struct SegmentTotals {
    total: usize,
    rapid: usize,
    feed: usize,
}

impl Default for Toolpath {
    fn default() -> Self {
        Self::new(Vector3::zero())
    }
}

/// Toolpath statistics
#[derive(Debug, Clone)]
pub struct ToolpathStats {
    /// Total number of segments
    pub total_segments: usize,
    /// Number of rapid segments
    pub rapid_segments: usize,
    /// Number of feed segments
    pub feed_segments: usize,
    /// Total path length
    pub total_length: f32,
    /// Estimated execution time
    pub estimated_time: Option<f32>,
}
