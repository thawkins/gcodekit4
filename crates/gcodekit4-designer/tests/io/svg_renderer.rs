use gcodekit4_designer::canvas::Canvas;
use gcodekit4_designer::shapes::{Circle, Point, Rectangle, Shape};
use gcodekit4_designer::svg_renderer::{render_crosshair, render_shapes};

#[test]
fn test_render_empty_canvas() {
    let canvas = Canvas::new();
    let path = render_shapes(&canvas, 800, 600);
    assert_eq!(path, "");
}

#[test]
fn test_render_crosshair() {
    let canvas = Canvas::new();
    let path = render_crosshair(&canvas, 800, 600);
    assert!(!path.is_empty());
    assert!(path.contains("M"));
    assert!(path.contains("L"));
}

#[test]
fn test_render_rectangle() {
    let mut canvas = Canvas::new();
    let rect = Rectangle::new(10.0, 10.0, 50.0, 50.0);
    canvas.add_shape(Shape::Rectangle(rect));

    let path = render_shapes(&canvas, 800, 600);
    assert!(!path.is_empty());
    assert!(path.contains("M"));
    assert!(path.contains("L"));
    assert!(path.contains("Z"));
}

#[test]
fn test_render_circle() {
    let mut canvas = Canvas::new();
    let circle = Circle::new(Point::new(30.0, 30.0), 20.0);
    canvas.add_shape(Shape::Circle(circle));

    let path = render_shapes(&canvas, 800, 600);
    assert!(!path.is_empty());
    assert!(path.contains("M"));
    assert!(path.contains("A")); // Arc commands for circle
}
