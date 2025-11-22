use gcodekit4_designer::pocket_operations::{Island, PocketGenerator, PocketOperation};
use gcodekit4_designer::shapes::{Circle, Point, Rectangle};

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
    assert!(gen.islands[0].contains_point(&Point::new(50.0, 50.0)));
    assert!(!gen.islands[0].contains_point(&Point::new(100.0, 100.0)));
}

#[test]
fn test_pocket_generator_offset_paths() {
    let op = PocketOperation::new("pocket1".to_string(), -10.0, 3.175);
    let gen = PocketGenerator::new(op);
    let rect = Rectangle::new(0.0, 0.0, 100.0, 100.0);

    let paths = gen.generate_offset_paths(&rect, 3);
    assert!(paths.len() > 0);
}
