//! Tests for processing::optimizer

use gcodekit4::processing::optimizer::*;

#[test]
fn test_remove_redundant_m5() {
    let lines = vec!["M5".to_string(), "M5".to_string(), "G0 X10".to_string()];
    let result = GCodeOptimizer::remove_redundant_m5(&lines);
    assert_eq!(result.len(), 2);
}

#[test]
fn test_remove_redundant_tools() {
    let lines = vec!["T1".to_string(), "T1".to_string(), "G0 X10".to_string()];
    let result = GCodeOptimizer::remove_redundant_tools(&lines);
    assert_eq!(result.len(), 2);
}
