//! Unit conversion utilities
//!
//! Handles conversion between Metric (mm) and Imperial (inch) systems.
//! Supports decimal and fractional inch parsing and formatting.

use std::fmt;
use std::str::FromStr;

/// Measurement system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementSystem {
    Metric,
    Imperial,
}

impl fmt::Display for MeasurementSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Metric => write!(f, "Metric"),
            Self::Imperial => write!(f, "Imperial"),
        }
    }
}

impl FromStr for MeasurementSystem {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "metric" | "mm" => Ok(Self::Metric),
            "imperial" | "inch" | "in" => Ok(Self::Imperial),
            _ => Err(format!("Unknown measurement system: {}", s)),
        }
    }
}

/// Convert internal value (mm) to display string based on system
pub fn to_display_string(mm: f32, system: MeasurementSystem) -> String {
    match system {
        MeasurementSystem::Metric => {
            // Format to reasonable precision, removing trailing zeros
            let s = format!("{:.3}", mm);
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        }
        MeasurementSystem::Imperial => {
            let inches = mm / 25.4;
            // Format to 3 decimal places (thou)
            let s = format!("{:.3}", inches);
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }
}

/// Get the unit label for the given system ("mm" or "in")
pub fn get_unit_label(system: MeasurementSystem) -> &'static str {
    match system {
        MeasurementSystem::Metric => "mm",
        MeasurementSystem::Imperial => "in",
    }
}

/// Parse display string to internal value (mm) based on system
/// Supports fractional inches (e.g. "5 1/8")
pub fn parse_from_string(input: &str, system: MeasurementSystem) -> Result<f32, String> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(0.0);
    }

    match system {
        MeasurementSystem::Metric => {
            input.parse::<f32>().map_err(|e| e.to_string())
        }
        MeasurementSystem::Imperial => {
            // Check for fraction
            if input.contains('/') {
                let parts: Vec<&str> = input.split_whitespace().collect();
                let mut total_inches = 0.0;

                for part in parts {
                    if part.contains('/') {
                        let frac_parts: Vec<&str> = part.split('/').collect();
                        if frac_parts.len() == 2 {
                            let num = frac_parts[0].parse::<f32>().map_err(|_| "Invalid numerator")?;
                            let den = frac_parts[1].parse::<f32>().map_err(|_| "Invalid denominator")?;
                            if den == 0.0 {
                                return Err("Division by zero".to_string());
                            }
                            total_inches += num / den;
                        } else {
                            return Err("Invalid fraction format".to_string());
                        }
                    } else {
                        total_inches += part.parse::<f32>().map_err(|_| "Invalid number part")?;
                    }
                }
                Ok(total_inches * 25.4)
            } else {
                // Decimal inches
                let inches = input.parse::<f32>().map_err(|e| e.to_string())?;
                Ok(inches * 25.4)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_conversion() {
        assert_eq!(to_display_string(10.5, MeasurementSystem::Metric), "10.5");
        assert_eq!(parse_from_string("10.5", MeasurementSystem::Metric).unwrap(), 10.5);
    }

    #[test]
    fn test_imperial_decimal() {
        // 1 inch = 25.4 mm
        assert_eq!(to_display_string(25.4, MeasurementSystem::Imperial), "1");
        assert_eq!(parse_from_string("1", MeasurementSystem::Imperial).unwrap(), 25.4);
        
        // 0.5 inch = 12.7 mm
        assert_eq!(to_display_string(12.7, MeasurementSystem::Imperial), "0.5");
        assert_eq!(parse_from_string("0.5", MeasurementSystem::Imperial).unwrap(), 12.7);
    }

    #[test]
    fn test_imperial_fraction() {
        // 1 1/2 inch = 1.5 inch = 38.1 mm
        assert_eq!(parse_from_string("1 1/2", MeasurementSystem::Imperial).unwrap(), 38.1);
        
        // 5 1/8 inch = 5.125 inch = 130.175 mm
        assert_eq!(parse_from_string("5 1/8", MeasurementSystem::Imperial).unwrap(), 130.175);
        
        // Just fraction: 1/4 inch = 0.25 inch = 6.35 mm
        assert_eq!(parse_from_string("1/4", MeasurementSystem::Imperial).unwrap(), 6.35);
    }

    #[test]
    fn test_unit_labels() {
        assert_eq!(get_unit_label(MeasurementSystem::Metric), "mm");
        assert_eq!(get_unit_label(MeasurementSystem::Imperial), "in");
    }

    #[test]
    fn test_negative_values() {
        assert_eq!(parse_from_string("-10.5", MeasurementSystem::Metric).unwrap(), -10.5);
        assert_eq!(parse_from_string("-1", MeasurementSystem::Imperial).unwrap(), -25.4);
        assert_eq!(parse_from_string("-1/2", MeasurementSystem::Imperial).unwrap(), -12.7);
    }

    #[test]
    fn test_zero_values() {
        assert_eq!(parse_from_string("0", MeasurementSystem::Metric).unwrap(), 0.0);
        assert_eq!(parse_from_string("0", MeasurementSystem::Imperial).unwrap(), 0.0);
        assert_eq!(parse_from_string("", MeasurementSystem::Metric).unwrap(), 0.0);
    }

    #[test]
    fn test_whitespace_handling() {
        assert_eq!(parse_from_string("  10.5  ", MeasurementSystem::Metric).unwrap(), 10.5);
        assert_eq!(parse_from_string("  1  1/2  ", MeasurementSystem::Imperial).unwrap(), 38.1);
    }

    #[test]
    fn test_invalid_inputs() {
        assert!(parse_from_string("abc", MeasurementSystem::Metric).is_err());
        assert!(parse_from_string("1/0", MeasurementSystem::Imperial).is_err()); // Division by zero
        assert!(parse_from_string("1/2/3", MeasurementSystem::Imperial).is_err()); // Invalid fraction
    }

    #[test]
    fn test_rounding_behavior() {
        // 1/3 inch = 8.4666... mm
        // Display should round to 3 decimal places
        let val = 1.0 / 3.0 * 25.4;
        assert_eq!(to_display_string(val, MeasurementSystem::Imperial), "0.333");
        
        // Metric rounding
        assert_eq!(to_display_string(10.12345, MeasurementSystem::Metric), "10.123");
        assert_eq!(to_display_string(10.12355, MeasurementSystem::Metric), "10.124");
    }
}
