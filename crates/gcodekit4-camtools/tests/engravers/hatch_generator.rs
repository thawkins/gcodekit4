use gcodekit4_camtools::hatch_generator::generate_hatch;
use lyon::path::Path;
use lyon::math::point;

#[test]
fn test_hatch_square() {
    let mut builder = Path::builder();
    builder.begin(point(0.0, 0.0));
    builder.line_to(point(10.0, 0.0));
    builder.line_to(point(10.0, 10.0));
    builder.line_to(point(0.0, 10.0));
    builder.close();
    let path = builder.build();

    // Hatch with 1.0mm spacing at 0 degrees
    let hatches = generate_hatch(&path, 0.0, 1.0, 0.1);
    
    // Should have roughly 10 lines (allow some margin for boundary conditions)
    assert!(hatches.len() >= 9 && hatches.len() <= 13, "Expected around 10 lines, got {}", hatches.len());
}

#[test]
fn test_hatch_rotated() {
    let mut builder = Path::builder();
    builder.begin(point(0.0, 0.0));
    builder.line_to(point(10.0, 0.0));
    builder.line_to(point(10.0, 10.0));
    builder.line_to(point(0.0, 10.0));
    builder.close();
    let path = builder.build();

    // Hatch with 1.0mm spacing at 45 degrees
    let hatches = generate_hatch(&path, 45.0, 1.0, 0.1);
    
    assert!(!hatches.is_empty());
}
