use gcodekit4_designer::toolpath::ToolpathGenerator;
use gcodekit4_designer::shapes::Rectangle;

#[test]
fn test_toolpath_generator_rectangle() {
    let gen = ToolpathGenerator::new();
    let rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);
    let toolpath = gen.generate_rectangle_contour(&rect);

    assert!(toolpath.segments.len() > 0);
    assert_eq!(toolpath.tool_diameter, 3.175);
    assert_eq!(toolpath.depth, -5.0);
}

#[test]
fn test_toolpath_total_length() {
    let gen = ToolpathGenerator::new();
    let rect = Rectangle::new(0.0, 0.0, 10.0, 10.0);
    let toolpath = gen.generate_rectangle_contour(&rect);

    let length = toolpath.total_length();
    assert!(length > 0.0);
}
