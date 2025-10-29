//! Tests for processing::arc_expander

use gcodekit4::processing::arc_expander::*;

#[test]
fn test_arc_expander() {
    let expander = ArcExpander::default();
    let segments = expander.expand_arc(0.0, 1.0, 1.0, 0.0, 0.0, 0.0, false);
    assert!(!segments.is_empty());
}
