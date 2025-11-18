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
    Ellipse,
    Polygon,
    RoundRectangle,
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
        Self {
            x,
            y,
            width,
            height,
        }
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

/// An ellipse defined by its center, horizontal radius, and vertical radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipse {
    pub center: Point,
    pub rx: f64,
    pub ry: f64,
}

impl Ellipse {
    /// Creates a new ellipse with specified center and radii.
    pub fn new(center: Point, rx: f64, ry: f64) -> Self {
        Self { center, rx, ry }
    }
}

impl Shape for Ellipse {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Ellipse
    }

    fn bounding_box(&self) -> (f64, f64, f64, f64) {
        (
            self.center.x - self.rx,
            self.center.y - self.ry,
            self.center.x + self.rx,
            self.center.y + self.ry,
        )
    }

    fn contains_point(&self, point: &Point) -> bool {
        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;
        (dx * dx) / (self.rx * self.rx) + (dy * dy) / (self.ry * self.ry) <= 1.0
    }

    fn clone_shape(&self) -> Box<dyn Shape> {
        Box::new(*self)
    }
}

/// A polygon defined by a list of vertices.
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    pub vertices: Vec<Point>,
}

impl Polygon {
    /// Creates a new polygon from a list of vertices.
    pub fn new(vertices: Vec<Point>) -> Self {
        Self { vertices }
    }

    /// Creates a regular polygon with n sides.
    pub fn regular(center: Point, radius: f64, sides: usize) -> Self {
        let mut vertices = Vec::with_capacity(sides);
        for i in 0..sides {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (sides as f64);
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            vertices.push(Point::new(x, y));
        }
        Self { vertices }
    }
}

impl Shape for Polygon {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Polygon
    }

    fn bounding_box(&self) -> (f64, f64, f64, f64) {
        if self.vertices.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let mut min_x = self.vertices[0].x;
        let mut min_y = self.vertices[0].y;
        let mut max_x = self.vertices[0].x;
        let mut max_y = self.vertices[0].y;

        for v in &self.vertices {
            if v.x < min_x {
                min_x = v.x;
            }
            if v.x > max_x {
                max_x = v.x;
            }
            if v.y < min_y {
                min_y = v.y;
            }
            if v.y > max_y {
                max_y = v.y;
            }
        }
        (min_x, min_y, max_x, max_y)
    }

    fn contains_point(&self, point: &Point) -> bool {
        if self.vertices.len() < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = self.vertices.len() - 1;

        for i in 0..self.vertices.len() {
            let xi = self.vertices[i].x;
            let yi = self.vertices[i].y;
            let xj = self.vertices[j].x;
            let yj = self.vertices[j].y;

            if (yi > point.y) != (yj > point.y)
                && (point.x < (xj - xi) * (point.y - yi) / (yj - yi) + xi)
            {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    fn clone_shape(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }
}

/// A rectangle with rounded corners defined by position, dimensions, and corner radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoundRectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub radius: f64,
}

impl RoundRectangle {
    /// Creates a new round rectangle.
    pub fn new(x: f64, y: f64, width: f64, height: f64, radius: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            radius,
        }
    }

    /// Creates a new round rectangle with default radius (5% of height).
    pub fn with_default_radius(x: f64, y: f64, width: f64, height: f64) -> Self {
        let radius = (height * 0.20).max(1.0); // Default to 20% of height, minimum 1.0
        Self {
            x,
            y,
            width,
            height,
            radius,
        }
    }
}

impl Shape for RoundRectangle {
    fn shape_type(&self) -> ShapeType {
        ShapeType::RoundRectangle
    }

    fn bounding_box(&self) -> (f64, f64, f64, f64) {
        (self.x, self.y, self.x + self.width, self.y + self.height)
    }

    fn contains_point(&self, point: &Point) -> bool {
        let px = point.x;
        let py = point.y;
        let x1 = self.x;
        let y1 = self.y;
        let x2 = self.x + self.width;
        let y2 = self.y + self.height;
        let r = self.radius;

        // Check if point is within bounding box
        if px < x1 || px > x2 || py < y1 || py > y2 {
            return false;
        }

        // Check if point is in the main rectangular area
        if (px >= x1 + r && px <= x2 - r) || (py >= y1 + r && py <= y2 - r) {
            return true;
        }

        // Check rounded corners
        let corners = [
            (x1 + r, y1 + r), // top-left
            (x2 - r, y1 + r), // top-right
            (x2 - r, y2 - r), // bottom-right
            (x1 + r, y2 - r), // bottom-left
        ];

        for (cx, cy) in corners {
            let dist = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
            if dist <= r {
                // Check if point is in the corner arc
                if px <= cx && py <= cy {
                    return true; // top-left corner
                } else if px >= cx && py <= cy {
                    return true; // top-right corner
                } else if px >= cx && py >= cy {
                    return true; // bottom-right corner
                } else if px <= cx && py >= cy {
                    return true; // bottom-left corner
                }
            }
        }

        // Check rectangular edges with rounded corners
        if px >= x1 && px <= x2 && py >= y1 + r && py <= y2 - r {
            return true; // vertical edges
        }
        if py >= y1 && py <= y2 && px >= x1 + r && px <= x2 - r {
            return true; // horizontal edges
        }

        false
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

    #[test]
    fn test_ellipse_contains_point() {
        let ellipse = Ellipse::new(Point::new(0.0, 0.0), 5.0, 3.0);
        assert!(ellipse.contains_point(&Point::new(0.0, 0.0)));
        assert!(ellipse.contains_point(&Point::new(4.0, 0.0)));
        assert!(!ellipse.contains_point(&Point::new(6.0, 0.0)));
    }

    #[test]
    fn test_ellipse_bounding_box() {
        let ellipse = Ellipse::new(Point::new(10.0, 10.0), 5.0, 3.0);
        let (min_x, min_y, max_x, max_y) = ellipse.bounding_box();
        assert_eq!(min_x, 5.0);
        assert_eq!(min_y, 7.0);
        assert_eq!(max_x, 15.0);
        assert_eq!(max_y, 13.0);
    }

    #[test]
    fn test_polygon_regular() {
        let polygon = Polygon::regular(Point::new(0.0, 0.0), 10.0, 4);
        assert_eq!(polygon.vertices.len(), 4);
    }

    #[test]
    fn test_polygon_bounding_box() {
        let polygon = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ]);
        let (min_x, min_y, max_x, max_y) = polygon.bounding_box();
        assert_eq!(min_x, 0.0);
        assert_eq!(min_y, 0.0);
        assert_eq!(max_x, 10.0);
        assert_eq!(max_y, 10.0);
    }

    #[test]
    fn test_polygon_contains_point() {
        let polygon = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ]);
        assert!(polygon.contains_point(&Point::new(5.0, 5.0)));
        assert!(!polygon.contains_point(&Point::new(15.0, 5.0)));
    }

    #[test]
    fn test_round_rectangle_with_default_radius() {
        let rrect = RoundRectangle::with_default_radius(0.0, 0.0, 100.0, 20.0);
        assert_eq!(rrect.radius, 4.0); // 20% of 20 = 4.0
    }

    #[test]
    fn test_round_rectangle_bounding_box() {
        let rrect = RoundRectangle::new(10.0, 10.0, 50.0, 30.0, 5.0);
        let (min_x, min_y, max_x, max_y) = rrect.bounding_box();
        assert_eq!(min_x, 10.0);
        assert_eq!(min_y, 10.0);
        assert_eq!(max_x, 60.0);
        assert_eq!(max_y, 40.0);
    }

    #[test]
    fn test_round_rectangle_contains_point_center() {
        let rrect = RoundRectangle::new(0.0, 0.0, 10.0, 10.0, 2.0);
        assert!(rrect.contains_point(&Point::new(5.0, 5.0)));
    }

    #[test]
    fn test_round_rectangle_contains_point_edges() {
        let rrect = RoundRectangle::new(0.0, 0.0, 10.0, 10.0, 2.0);
        assert!(rrect.contains_point(&Point::new(5.0, 0.0)));
        assert!(rrect.contains_point(&Point::new(0.0, 5.0)));
        assert!(rrect.contains_point(&Point::new(10.0, 5.0)));
        assert!(rrect.contains_point(&Point::new(5.0, 10.0)));
    }

    #[test]
    fn test_round_rectangle_excludes_outside() {
        let rrect = RoundRectangle::new(0.0, 0.0, 10.0, 10.0, 2.0);
        assert!(!rrect.contains_point(&Point::new(-1.0, 5.0)));
        assert!(!rrect.contains_point(&Point::new(11.0, 5.0)));
        assert!(!rrect.contains_point(&Point::new(5.0, -1.0)));
        assert!(!rrect.contains_point(&Point::new(5.0, 11.0)));
    }
}
