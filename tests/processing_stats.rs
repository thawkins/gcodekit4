//! Tests for processing::stats

use gcodekit4_parser::processing::stats::*;

#[test]
fn test_stats_calculation() {
    let lines = vec!["G0 X10 Y20".to_string(), "G1 X30 Y40 F100".to_string()];
    let stats = StatsCalculator::calculate(&lines);
    assert_eq!(stats.total_commands, 2);
    assert_eq!(stats.rapid_count, 1);
    assert_eq!(stats.linear_count, 1);
}

#[test]
fn test_bounding_box() {
    let mut stats = Stats::default();
    stats.max_x = 100.0;
    stats.max_y = 50.0;
    let (w, h, _) = stats.bounding_box();
    assert_eq!(w, 100.0);
    assert_eq!(h, 50.0);
}
