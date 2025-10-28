//! Toolpath rendering module integration tests

use gcodekit4::visualizer::{ArcSegment, LineSegment, MovementType, Toolpath, Vector3};

#[test]
fn test_line_segment_creation() {
    let segment = LineSegment::new(
        Vector3::zero(),
        Vector3::new(10.0, 0.0, 0.0),
        MovementType::Feed,
    );
    
    assert_eq!(segment.start, Vector3::zero());
    assert_eq!(segment.end, Vector3::new(10.0, 0.0, 0.0));
    assert_eq!(segment.movement_type, MovementType::Feed);
    assert_eq!(segment.length(), 10.0);
}

#[test]
fn test_line_segment_with_feed_rate() {
    let segment = LineSegment::new(
        Vector3::zero(),
        Vector3::new(10.0, 0.0, 0.0),
        MovementType::Feed,
    )
    .with_feed_rate(100.0);

    assert_eq!(segment.feed_rate, Some(100.0));
}

#[test]
fn test_line_segment_colors() {
    let rapid = LineSegment::new(Vector3::zero(), Vector3::new(10.0, 0.0, 0.0), MovementType::Rapid);
    let feed = LineSegment::new(Vector3::zero(), Vector3::new(10.0, 0.0, 0.0), MovementType::Feed);
    let arc_cw = LineSegment::new(Vector3::zero(), Vector3::new(10.0, 0.0, 0.0), MovementType::ArcClockwise);
    let arc_ccw = LineSegment::new(Vector3::zero(), Vector3::new(10.0, 0.0, 0.0), MovementType::ArcCounterClockwise);

    assert_ne!(rapid.get_color(), feed.get_color());
    assert_ne!(arc_cw.get_color(), arc_ccw.get_color());
}

#[test]
fn test_arc_segment_creation() {
    let arc = ArcSegment::new(
        Vector3::new(10.0, 0.0, 0.0),
        Vector3::new(0.0, 10.0, 0.0),
        Vector3::zero(),
        false,
    );

    assert_eq!(arc.start, Vector3::new(10.0, 0.0, 0.0));
    assert_eq!(arc.end, Vector3::new(0.0, 10.0, 0.0));
    assert_eq!(arc.center, Vector3::zero());
    assert!(!arc.clockwise);
    assert!((arc.radius - 10.0).abs() < 0.01);
}

#[test]
fn test_arc_segment_with_feed_rate() {
    let arc = ArcSegment::new(
        Vector3::new(10.0, 0.0, 0.0),
        Vector3::new(0.0, 10.0, 0.0),
        Vector3::zero(),
        true,
    )
    .with_feed_rate(150.0);

    assert_eq!(arc.feed_rate, Some(150.0));
}

#[test]
fn test_arc_segment_segments() {
    let arc = ArcSegment::new(
        Vector3::new(10.0, 0.0, 0.0),
        Vector3::new(0.0, 10.0, 0.0),
        Vector3::zero(),
        false,
    )
    .with_segments(10);

    assert_eq!(arc.segments, 10);
}

#[test]
fn test_arc_to_line_segments() {
    let arc = ArcSegment::new(
        Vector3::new(10.0, 0.0, 0.0),
        Vector3::new(0.0, 10.0, 0.0),
        Vector3::zero(),
        false,
    )
    .with_segments(4);

    let lines = arc.to_line_segments();
    assert_eq!(lines.len(), 4);

    assert_eq!(lines[0].start, arc.start);
    let end = lines[lines.len() - 1].end;
    assert!((end.x - arc.end.x).abs() < 0.001);
    assert!((end.y - arc.end.y).abs() < 0.001);
    assert!((end.z - arc.end.z).abs() < 0.001);
}

#[test]
fn test_arc_interpolation() {
    let arc = ArcSegment::new(
        Vector3::new(10.0, 0.0, 0.0),
        Vector3::new(0.0, 10.0, 0.0),
        Vector3::zero(),
        false,
    );

    let start_point = arc.interpolate_arc(0.0);
    let mid_point = arc.interpolate_arc(0.5);
    let end_point = arc.interpolate_arc(1.0);

    assert!((start_point.x - arc.start.x).abs() < 0.1);
    assert!((end_point.x - arc.end.x).abs() < 0.1);
    assert!(mid_point.magnitude() > 0.0);
}

#[test]
fn test_toolpath_creation() {
    let toolpath = Toolpath::new(Vector3::new(5.0, 5.0, 5.0));
    
    assert_eq!(toolpath.start_position, Vector3::new(5.0, 5.0, 5.0));
    assert_eq!(toolpath.current_position, Vector3::new(5.0, 5.0, 5.0));
    assert_eq!(toolpath.total_length, 0.0);
    assert!(toolpath.line_segments.is_empty());
    assert!(toolpath.arc_segments.is_empty());
}

#[test]
fn test_toolpath_default() {
    let toolpath = Toolpath::default();
    assert_eq!(toolpath.start_position, Vector3::zero());
}

#[test]
fn test_toolpath_add_line_segment() {
    let mut toolpath = Toolpath::new(Vector3::zero());
    let segment = LineSegment::new(
        Vector3::zero(),
        Vector3::new(10.0, 0.0, 0.0),
        MovementType::Feed,
    );
    
    toolpath.add_line_segment(segment);

    assert_eq!(toolpath.line_segments.len(), 1);
    assert_eq!(toolpath.total_length, 10.0);
    assert_eq!(toolpath.current_position, Vector3::new(10.0, 0.0, 0.0));
}

#[test]
fn test_toolpath_add_arc_segment() {
    let mut toolpath = Toolpath::new(Vector3::zero());
    let arc = ArcSegment::new(
        Vector3::new(10.0, 0.0, 0.0),
        Vector3::new(0.0, 10.0, 0.0),
        Vector3::zero(),
        false,
    );
    
    toolpath.add_arc_segment(arc);

    assert_eq!(toolpath.arc_segments.len(), 1);
    assert!(toolpath.total_length > 0.0);
    assert_eq!(toolpath.current_position, Vector3::new(0.0, 10.0, 0.0));
}

#[test]
fn test_toolpath_update_position() {
    let mut toolpath = Toolpath::new(Vector3::zero());
    toolpath.update_current_position(Vector3::new(5.0, 5.0, 5.0));
    
    assert_eq!(toolpath.current_position, Vector3::new(5.0, 5.0, 5.0));
}

#[test]
fn test_toolpath_get_all_line_segments() {
    let mut toolpath = Toolpath::new(Vector3::zero());
    
    toolpath.add_line_segment(LineSegment::new(
        Vector3::zero(),
        Vector3::new(10.0, 0.0, 0.0),
        MovementType::Feed,
    ));
    
    let arc = ArcSegment::new(
        Vector3::new(10.0, 0.0, 0.0),
        Vector3::new(0.0, 10.0, 0.0),
        Vector3::zero(),
        false,
    )
    .with_segments(4);
    toolpath.add_arc_segment(arc);

    let all_segments = toolpath.get_all_line_segments();
    assert_eq!(all_segments.len(), 5);
}

#[test]
fn test_toolpath_bounding_box() {
    let mut toolpath = Toolpath::new(Vector3::zero());
    
    assert!(toolpath.get_bounding_box().is_none());

    toolpath.add_line_segment(LineSegment::new(
        Vector3::zero(),
        Vector3::new(10.0, 10.0, 10.0),
        MovementType::Feed,
    ));

    let bbox = toolpath.get_bounding_box();
    assert!(bbox.is_some());
    
    let (min, max) = bbox.unwrap();
    assert_eq!(min, Vector3::zero());
    assert_eq!(max, Vector3::new(10.0, 10.0, 10.0));
}

#[test]
fn test_toolpath_center() {
    let mut toolpath = Toolpath::new(Vector3::zero());
    
    assert!(toolpath.get_center().is_none());

    toolpath.add_line_segment(LineSegment::new(
        Vector3::zero(),
        Vector3::new(20.0, 30.0, 40.0),
        MovementType::Feed,
    ));

    let center = toolpath.get_center();
    assert!(center.is_some());
    assert_eq!(center.unwrap(), Vector3::new(10.0, 15.0, 20.0));
}

#[test]
fn test_toolpath_estimated_time() {
    let mut toolpath = Toolpath::new(Vector3::zero());
    assert!(toolpath.estimated_time.is_none());

    toolpath.set_estimated_time(120.5);
    assert_eq!(toolpath.estimated_time, Some(120.5));

    toolpath.set_estimated_time(-10.0);
    assert_eq!(toolpath.estimated_time, Some(0.0));
}

#[test]
fn test_toolpath_clear() {
    let mut toolpath = Toolpath::new(Vector3::new(5.0, 5.0, 5.0));
    
    toolpath.add_line_segment(LineSegment::new(
        Vector3::new(5.0, 5.0, 5.0),
        Vector3::new(10.0, 10.0, 10.0),
        MovementType::Feed,
    ));
    toolpath.set_estimated_time(60.0);

    assert!(!toolpath.line_segments.is_empty());
    assert!(toolpath.total_length > 0.0);

    toolpath.clear();

    assert!(toolpath.line_segments.is_empty());
    assert!(toolpath.arc_segments.is_empty());
    assert_eq!(toolpath.total_length, 0.0);
    assert!(toolpath.estimated_time.is_none());
    assert_eq!(toolpath.current_position, toolpath.start_position);
}

#[test]
fn test_toolpath_statistics() {
    let mut toolpath = Toolpath::new(Vector3::zero());
    
    toolpath.add_line_segment(LineSegment::new(
        Vector3::zero(),
        Vector3::new(10.0, 0.0, 0.0),
        MovementType::Feed,
    ));
    
    toolpath.add_line_segment(LineSegment::new(
        Vector3::new(10.0, 0.0, 0.0),
        Vector3::new(20.0, 0.0, 0.0),
        MovementType::Rapid,
    ));

    toolpath.set_estimated_time(30.0);

    let stats = toolpath.get_statistics();
    assert_eq!(stats.total_segments, 2);
    assert_eq!(stats.feed_segments, 1);
    assert_eq!(stats.rapid_segments, 1);
    assert_eq!(stats.total_length, 20.0);
    assert_eq!(stats.estimated_time, Some(30.0));
}
