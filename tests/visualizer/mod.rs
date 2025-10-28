//! visualizer module integration tests

mod setup_tests;
mod controls_tests;
mod features_tests;
mod toolpath_tests;
mod visualizer_2d_tests;

use gcodekit4::visualizer::*;

#[test]
fn test_visualizer_creation() {
    let vis = Visualizer::new(800, 600);
    assert_eq!(vis.renderer.width, 800);
    assert_eq!(vis.renderer.height, 600);
}

#[test]
fn test_visualizer_default() {
    let vis = Visualizer::default();
    assert_eq!(vis.renderer.width, 800);
    assert_eq!(vis.renderer.height, 600);
}

#[test]
fn test_visualizer_resize() {
    let mut vis = Visualizer::new(800, 600);
    vis.resize(1024, 768);
    assert_eq!(vis.renderer.width, 1024);
    assert_eq!(vis.renderer.height, 768);
}

#[test]
fn test_visualizer_get_toolpath_stats() {
    let vis = Visualizer::new(800, 600);
    let stats = vis.get_toolpath_stats();
    assert_eq!(stats.total_segments, 0);
    assert_eq!(stats.total_length, 0.0);
}
