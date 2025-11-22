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

/// A generic path shape wrapping lyon::path::Path
#[derive(Debug, Clone)]
pub struct PathShape {
    pub path: Path,
}

impl PathShape {
    pub fn new(path: Path) -> Self {
        Self { path }
    }

    pub fn from_points(points: &[Point], closed: bool) -> Self {
        let mut builder = Path::builder();
        if let Some(first) = points.first() {
            builder.begin(point(first.x as f32, first.y as f32));
            for p in points.iter().skip(1) {
                builder.line_to(point(p.x as f32, p.y as f32));
            }
            if closed {
                builder.close();
            } else {
                builder.end(false);
            }
        }
        Self { path: builder.build() }
    }

    pub fn translate(&self, dx: f64, dy: f64) -> Self {
        let mut builder = Path::builder();
        for event in self.path.iter() {
            match event {
                lyon::path::Event::Begin { at } => {
                    builder.begin(point(at.x + dx as f32, at.y + dy as f32));
                }
                lyon::path::Event::Line { from: _, to } => {
                    builder.line_to(point(to.x + dx as f32, to.y + dy as f32));
                }
                lyon::path::Event::Quadratic { from: _, ctrl, to } => {
                    builder.quadratic_bezier_to(
                        point(ctrl.x + dx as f32, ctrl.y + dy as f32),
                        point(to.x + dx as f32, to.y + dy as f32),
                    );
                }
                lyon::path::Event::Cubic { from: _, ctrl1, ctrl2, to } => {
                    builder.cubic_bezier_to(
                        point(ctrl1.x + dx as f32, ctrl1.y + dy as f32),
                        point(ctrl2.x + dx as f32, ctrl2.y + dy as f32),
                        point(to.x + dx as f32, to.y + dy as f32),
                    );
                }
                lyon::path::Event::End { last: _, first: _, close } => {
                    if close {
                        builder.close();
                    } else {
                        builder.end(false);
                    }
                }
            }
        }
        Self { path: builder.build() }
    }

    pub fn scale(&self, sx: f64, sy: f64, center: Point) -> Self {
        let mut builder = Path::builder();
        let transform = |p: lyon::math::Point| -> lyon::math::Point {
            let x = center.x + (p.x as f64 - center.x) * sx;
            let y = center.y + (p.y as f64 - center.y) * sy;
            point(x as f32, y as f32)
        };

        for event in self.path.iter() {
            match event {
                lyon::path::Event::Begin { at } => {
                    builder.begin(transform(at));
                }
                lyon::path::Event::Line { from: _, to } => {
                    builder.line_to(transform(to));
                }
                lyon::path::Event::Quadratic { from: _, ctrl, to } => {
                    builder.quadratic_bezier_to(transform(ctrl), transform(to));
                }
                lyon::path::Event::Cubic { from: _, ctrl1, ctrl2, to } => {
                    builder.cubic_bezier_to(transform(ctrl1), transform(ctrl2), transform(to));
                }
                lyon::path::Event::End { last: _, first: _, close } => {
                    if close {
                        builder.close();
                    } else {
                        builder.end(false);
                    }
                }
            }
        }
        Self { path: builder.build() }
    }

    pub fn to_svg_path(&self) -> String {
        let mut path_str = String::new();
        for event in self.path.iter() {
            match event {
                lyon::path::Event::Begin { at } => {
                    path_str.push_str(&format!("M {} {} ", at.x, at.y));
                }
                lyon::path::Event::Line { from: _, to } => {
                    path_str.push_str(&format!("L {} {} ", to.x, to.y));
                }
                lyon::path::Event::Quadratic { from: _, ctrl, to } => {
                    path_str.push_str(&format!("Q {} {} {} {} ", ctrl.x, ctrl.y, to.x, to.y));
                }
                lyon::path::Event::Cubic { from: _, ctrl1, ctrl2, to } => {
                    path_str.push_str(&format!("C {} {} {} {} {} {} ", ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y));
                }
                lyon::path::Event::End { last: _, first: _, close } => {
                    if close {
                        path_str.push_str("Z ");
                    }
                }
            }
        }
        path_str
    }

    pub fn from_svg_path(data_str: &str) -> Option<Self> {
        let mut builder = Path::builder();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;
        let mut start_x = 0.0f32;
        let mut start_y = 0.0f32;
        let mut subpath_active = false;

        let commands = Self::tokenize_svg_path(data_str);
        let mut i = 0;

        while i < commands.len() {
            let cmd = &commands[i];

            match cmd.as_str() {
                "M" | "m" => {
                    if i + 2 < commands.len() {
                        let x: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        let y: f32 = commands[i + 2].parse().unwrap_or(0.0);

                        if cmd == "m" {
                            current_x += x;
                            current_y += y;
                        } else {
                            current_x = x;
                            current_y = y;
                        }
                        
                        if subpath_active {
                            builder.end(false);
                        }
                        
                        start_x = current_x;
                        start_y = current_y;
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                        i += 3;
                    } else {
                        i += 1;
                    }
                }
                "L" | "l" => {
                    if !subpath_active {
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                    }
                    let mut j = i + 1;
                    while j + 1 < commands.len() {
                        let x: f32 = commands[j].parse().unwrap_or(0.0);
                        let y: f32 = commands[j + 1].parse().unwrap_or(0.0);

                        if cmd == "l" {
                            current_x += x;
                            current_y += y;
                        } else {
                            current_x = x;
                            current_y = y;
                        }

                        builder.line_to(point(current_x, current_y));
                        j += 2;

                        if j < commands.len() {
                            let next = &commands[j];
                            if next.len() == 1 && next.chars().all(|c| c.is_alphabetic()) {
                                break;
                            } else if next.parse::<f32>().is_err() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    i = j;
                }
                "H" | "h" => {
                    if !subpath_active {
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                    }
                    if i + 1 < commands.len() {
                        let x: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        if cmd == "h" {
                            current_x += x;
                        } else {
                            current_x = x;
                        }
                        builder.line_to(point(current_x, current_y));
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "V" | "v" => {
                    if !subpath_active {
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                    }
                    if i + 1 < commands.len() {
                        let y: f32 = commands[i + 1].parse().unwrap_or(0.0);
                        if cmd == "v" {
                            current_y += y;
                        } else {
                            current_y = y;
                        }
                        builder.line_to(point(current_x, current_y));
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "C" | "c" => {
                    if !subpath_active {
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                    }
                    let mut j = i + 1;
                    while j + 5 < commands.len() {
                        let x1: f32 = commands[j].parse().unwrap_or(0.0);
                        let y1: f32 = commands[j + 1].parse().unwrap_or(0.0);
                        let x2: f32 = commands[j + 2].parse().unwrap_or(0.0);
                        let y2: f32 = commands[j + 3].parse().unwrap_or(0.0);
                        let x: f32 = commands[j + 4].parse().unwrap_or(0.0);
                        let y: f32 = commands[j + 5].parse().unwrap_or(0.0);

                        let mut cp1_x = x1;
                        let mut cp1_y = y1;
                        let mut cp2_x = x2;
                        let mut cp2_y = y2;
                        let mut end_x = x;
                        let mut end_y = y;

                        if cmd == "c" {
                            cp1_x += current_x;
                            cp1_y += current_y;
                            cp2_x += current_x;
                            cp2_y += current_y;
                            end_x += current_x;
                            end_y += current_y;
                        }

                        builder.cubic_bezier_to(
                            point(cp1_x, cp1_y),
                            point(cp2_x, cp2_y),
                            point(end_x, end_y)
                        );

                        current_x = end_x;
                        current_y = end_y;
                        j += 6;

                        if j < commands.len() {
                            let next = &commands[j];
                            if next.len() == 1 && next.chars().all(|c| c.is_alphabetic()) {
                                break;
                            } else if next.parse::<f32>().is_err() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    i = j;
                }
                "Q" | "q" => {
                    if !subpath_active {
                        builder.begin(point(current_x, current_y));
                        subpath_active = true;
                    }
                    let mut j = i + 1;
                    while j + 3 < commands.len() {
                        let x1: f32 = commands[j].parse().unwrap_or(0.0);
                        let y1: f32 = commands[j + 1].parse().unwrap_or(0.0);
                        let x: f32 = commands[j + 2].parse().unwrap_or(0.0);
                        let y: f32 = commands[j + 3].parse().unwrap_or(0.0);

                        let mut cp_x = x1;
                        let mut cp_y = y1;
                        let mut end_x = x;
                        let mut end_y = y;

                        if cmd == "q" {
                            cp_x += current_x;
                            cp_y += current_y;
                            end_x += current_x;
                            end_y += current_y;
                        }

                        builder.quadratic_bezier_to(
                            point(cp_x, cp_y),
                            point(end_x, end_y)
                        );

                        current_x = end_x;
                        current_y = end_y;
                        j += 4;

                        if j < commands.len() {
                            let next = &commands[j];
                            if next.len() == 1 && next.chars().all(|c| c.is_alphabetic()) {
                                break;
                            } else if next.parse::<f32>().is_err() {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    i = j;
                }
                "Z" | "z" => {
                    if subpath_active {
                        builder.close();
                        subpath_active = false;
                    }
                    current_x = start_x;
                    current_y = start_y;
                    i += 1;
                }
                _ => i += 1,
            }
        }
        
        if subpath_active {
            builder.end(false);
        }
        Some(Self { path: builder.build() })
    }

    fn tokenize_svg_path(path_data: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();

        for ch in path_data.chars() {
            match ch {
                'M' | 'm' | 'L' | 'l' | 'H' | 'h' | 'V' | 'v' | 'C' | 'c' | 'S' | 's' | 'Q'
                | 'q' | 'T' | 't' | 'A' | 'a' | 'Z' | 'z' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    tokens.push(ch.to_string());
                }
                ' ' | ',' | '\n' | '\r' | '\t' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                _ => current_token.push(ch),
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
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
