//! Tests for processing::toolpath

use gcodekit4::processing::toolpath::*;

#[test]
fn test_segment_length() {
    let seg = Segment {
        segment_type: SegmentType::Linear,
        start_x: 0.0,
        start_y: 0.0,
        start_z: 0.0,
        end_x: 3.0,
        end_y: 4.0,
        end_z: 0.0,
        feed_rate: Some(100.0),
    };
    assert_eq!(seg.length(), 5.0);
}

#[test]
fn test_toolpath() {
    let mut path = Toolpath::new();
    let seg = Segment {
        segment_type: SegmentType::Linear,
        start_x: 0.0,
        start_y: 0.0,
        start_z: 0.0,
        end_x: 10.0,
        end_y: 0.0,
        end_z: 0.0,
        feed_rate: Some(100.0),
    };
    path.add_segment(seg);
    assert_eq!(path.total_length(), 10.0);
}
