//! Geometric shapes for the designer tool.

/// Represents a 2D point with X and Y coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Creates a new point with the given X and Y coordinates.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculates the distance to another point.
    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

/// Types of shapes that can be drawn on the canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeType {
    Rectangle,
    Circle,
    Line,
}

/// Base trait for all drawable shapes.
pub trait Shape: std::fmt::Debug {
    /// Gets the type of this shape.
    fn shape_type(&self) -> ShapeType;

    /// Gets the bounding box of the shape as (min_x, min_y, max_x, max_y).
    fn bounding_box(&self) -> (f64, f64, f64, f64);

    /// Checks if a point is inside or near the shape.
    fn contains_point(&self, point: &Point) -> bool;

    /// Returns a clone of the shape as a trait object.
    fn clone_shape(&self) -> Box<dyn Shape>;
}

/// A rectangle defined by its top-left corner and dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rectangle {
    /// Creates a new rectangle.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    /// Gets the four corners of the rectangle.
    pub fn corners(&self) -> [Point; 4] {
        [
            Point::new(self.x, self.y),
            Point::new(self.x + self.width, self.y),
            Point::new(self.x + self.width, self.y + self.height),
            Point::new(self.x, self.y + self.height),
        ]
    }
}

impl Shape for Rectangle {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Rectangle
    }

    fn bounding_box(&self) -> (f64, f64, f64, f64) {
        (self.x, self.y, self.x + self.width, self.y + self.height)
    }

    fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    fn clone_shape(&self) -> Box<dyn Shape> {
        Box::new(*self)
    }
}

/// A circle defined by its center and radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

impl Circle {
    /// Creates a new circle.
    pub fn new(center: Point, radius: f64) -> Self {
        Self { center, radius }
    }
}

impl Shape for Circle {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Circle
    }

    fn bounding_box(&self) -> (f64, f64, f64, f64) {
        (
            self.center.x - self.radius,
            self.center.y - self.radius,
            self.center.x + self.radius,
            self.center.y + self.radius,
        )
    }

    fn contains_point(&self, point: &Point) -> bool {
        self.center.distance_to(point) <= self.radius
    }

    fn clone_shape(&self) -> Box<dyn Shape> {
        Box::new(*self)
    }
}

/// A line defined by two endpoints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub start: Point,
    pub end: Point,
}

impl Line {
    /// Creates a new line.
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    /// Gets the length of the line.
    pub fn length(&self) -> f64 {
        self.start.distance_to(&self.end)
    }
}

impl Shape for Line {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Line
    }

    fn bounding_box(&self) -> (f64, f64, f64, f64) {
        (
            self.start.x.min(self.end.x),
            self.start.y.min(self.end.y),
            self.start.x.max(self.end.x),
            self.start.y.max(self.end.y),
        )
    }

    fn contains_point(&self, point: &Point) -> bool {
        let tolerance = 2.0;
        let dist_to_start = self.start.distance_to(point);
        let dist_to_end = self.end.distance_to(point);
        let line_length = self.length();

        (dist_to_start + dist_to_end - line_length).abs() < tolerance
    }

    fn clone_shape(&self) -> Box<dyn Shape> {
        Box::new(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_rectangle_contains_point() {
        let rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);
        assert!(rect.contains_point(&Point::new(5.0, 5.0)));
        assert!(!rect.contains_point(&Point::new(15.0, 5.0)));
    }

    #[test]
    fn test_circle_contains_point() {
        let circle = Circle::new(Point::new(0.0, 0.0), 5.0);
        assert!(circle.contains_point(&Point::new(3.0, 4.0)));
        assert!(!circle.contains_point(&Point::new(10.0, 0.0)));
    }

    #[test]
    fn test_line_length() {
        let line = Line::new(Point::new(0.0, 0.0), Point::new(3.0, 4.0));
        assert_eq!(line.length(), 5.0);
    }
}
