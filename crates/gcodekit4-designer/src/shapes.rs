//! Geometric shapes for the designer tool.

use lyon::path::Path;
use lyon::math::point;
use lyon::algorithms::aabb::bounding_box;
use lyon::algorithms::hit_test::hit_test_path;
use lyon::path::FillRule;
use std::any::Any;
use crate::font_manager;
use rusttype::{Scale, point as rt_point};

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
    Polyline,
    Path,
    Text,
}

/// Base trait for all drawable shapes.
pub trait Shape: std::fmt::Debug + Send + Sync {
    /// Gets the type of this shape.
    fn shape_type(&self) -> ShapeType;

    /// Gets the bounding box of the shape as (min_x, min_y, max_x, max_y).
    fn bounding_box(&self) -> (f64, f64, f64, f64);

    /// Checks if a point is inside or near the shape.
    fn contains_point(&self, point: &Point) -> bool;

    /// Returns a clone of the shape as a trait object.
    fn clone_shape(&self) -> Box<dyn Shape>;

    /// Downcast helper
    fn as_any(&self) -> &dyn Any;
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

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A polyline defined by a list of vertices.
#[derive(Debug, Clone, PartialEq)]
pub struct Polyline {
    pub vertices: Vec<Point>,
}

impl Polyline {
    /// Creates a new polyline from a list of vertices.
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

impl Shape for Polyline {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Polyline
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A generic path shape wrapping lyon::path::Path
#[derive(Debug, Clone)]
pub struct PathShape {
    pub path: Path,
}

impl PathShape {
    pub fn new(path: Path) -> Self {
        Self { path }
    }
}

impl Shape for PathShape {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Path
    }

    fn bounding_box(&self) -> (f64, f64, f64, f64) {
        let aabb = bounding_box(self.path.iter());
        (aabb.min.x as f64, aabb.min.y as f64, aabb.max.x as f64, aabb.max.y as f64)
    }

    fn contains_point(&self, p: &Point) -> bool {
        hit_test_path(
            &point(p.x as f32, p.y as f32),
            self.path.iter(),
            FillRule::NonZero,
            0.1
        )
    }

    fn clone_shape(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}



#[derive(Debug, Clone)]
pub struct TextShape {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub font_size: f64,
}

impl TextShape {
    pub fn new(text: String, x: f64, y: f64, font_size: f64) -> Self {
        Self {
            text,
            x,
            y,
            font_size,
        }
    }
}

impl Shape for TextShape {
    fn shape_type(&self) -> ShapeType {
        ShapeType::Text
    }

    fn bounding_box(&self) -> (f64, f64, f64, f64) {
        let font = font_manager::get_font();
        let scale = Scale::uniform(self.font_size as f32);
        let v_metrics = font.v_metrics(scale);
        
        // We treat (x, y) as the top-left corner of the text box.
        // Text layout usually starts at a baseline.
        // ascent is the distance from baseline to top.
        // So baseline y = self.y + v_metrics.ascent.
        
        let start = rt_point(self.x as f32, self.y as f32 + v_metrics.ascent);
        
        let glyphs: Vec<_> = font.layout(&self.text, scale, start).collect();
        
        if glyphs.is_empty() {
             return (self.x, self.y, self.x, self.y + self.font_size);
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        
        let mut has_bounds = false;

        for glyph in &glyphs {
            if let Some(bb) = glyph.unpositioned().exact_bounding_box() {
                let pos = glyph.position();
                min_x = min_x.min(pos.x + bb.min.x);
                min_y = min_y.min(pos.y + bb.min.y);
                max_x = max_x.max(pos.x + bb.max.x);
                max_y = max_y.max(pos.y + bb.max.y);
                has_bounds = true;
            }
        }
        
        if !has_bounds {
             // Fallback for whitespace only
             let width = self.text.len() as f64 * self.font_size * 0.6;
             return (self.x, self.y, self.x + width, self.y + self.font_size);
        }
        
        (min_x as f64, min_y as f64, max_x as f64, max_y as f64)
    }

    fn contains_point(&self, point: &Point) -> bool {
        let (min_x, min_y, max_x, max_y) = self.bounding_box();
        point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
    }

    fn clone_shape(&self) -> Box<dyn Shape> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Type of CAM operation to perform on the shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Profile,
    Pocket,
}

impl Default for OperationType {
    fn default() -> Self {
        Self::Profile
    }
}
