//! utils module integration tests

use gcodekit4::utils::*;

#[test]
fn test_format_float() {
    assert_eq!(format_float(std::f64::consts::PI, 2), "3.14");
    assert_eq!(format_float(10.0, 1), "10.0");
}

#[test]
fn test_degrees_to_radians() {
    let radians = degrees_to_radians(180.0);
    assert!((radians - std::f64::consts::PI).abs() < 0.0001);
}

#[test]
fn test_radians_to_degrees() {
    let degrees = radians_to_degrees(std::f64::consts::PI);
    assert!((degrees - 180.0).abs() < 0.0001);
}
