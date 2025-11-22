use gcodekit4_gcodeeditor::Viewport;

#[test]
fn test_create_viewport() {
    let viewport = Viewport::new(400.0, 20.0);
    assert_eq!(viewport.visible_lines, 20);
    assert_eq!(viewport.start_line, 0);
    assert_eq!(viewport.end_line, 20);
}

#[test]
fn test_set_total_lines() {
    let mut viewport = Viewport::new(400.0, 20.0);
    viewport.set_total_lines(100);
    assert_eq!(viewport.total_lines, 100);
    assert_eq!(viewport.end_line, 20);
}

#[test]
fn test_scroll_by() {
    let mut viewport = Viewport::new(400.0, 20.0);
    viewport.set_total_lines(100);

    viewport.scroll_by(10);
    assert_eq!(viewport.start_line, 10);
    assert_eq!(viewport.end_line, 30);

    viewport.scroll_by(-5);
    assert_eq!(viewport.start_line, 5);
    assert_eq!(viewport.end_line, 25);
}

#[test]
fn test_scroll_to_line() {
    let mut viewport = Viewport::new(400.0, 20.0);
    viewport.set_total_lines(100);

    viewport.scroll_to_line(50);
    assert!(viewport.is_line_visible(50));

    viewport.scroll_to_line(5);
    assert!(viewport.is_line_visible(5));
}

#[test]
fn test_scroll_position() {
    let mut viewport = Viewport::new(400.0, 20.0);
    viewport.set_total_lines(100);

    viewport.set_scroll_offset(40); // Middle
    let pos = viewport.scroll_position();
    assert!((pos - 0.5).abs() < 0.01);

    viewport.set_scroll_position(0.0);
    assert_eq!(viewport.scroll_offset, 0);

    viewport.set_scroll_position(1.0);
    assert_eq!(viewport.scroll_offset, 80); // 100 - 20
}

#[test]
fn test_scrollbar_ratio() {
    let mut viewport = Viewport::new(400.0, 20.0);
    viewport.set_total_lines(100);

    let ratio = viewport.scrollbar_ratio();
    assert_eq!(ratio, 0.2); // 20 visible / 100 total
}

#[test]
fn test_overscan_range() {
    let mut viewport = Viewport::new(400.0, 20.0);
    viewport.set_total_lines(100);
    viewport.set_scroll_offset(40);

    let range = viewport.overscan_range(5);
    assert_eq!(range.start, 35);
    assert_eq!(range.end, 65);
}
