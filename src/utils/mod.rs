//! Utility functions and helpers

/// Format a float to a reasonable number of decimal places
pub fn format_float(value: f64, precision: usize) -> String {
    format!("{:.prec$}", value, prec = precision)
}

/// Convert degrees to radians
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

/// Convert radians to degrees
pub fn radians_to_degrees(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
