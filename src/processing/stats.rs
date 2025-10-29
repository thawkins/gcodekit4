//! Statistics Calculator - Task 62
//!
//! Calculates G-code statistics including distance, time, and command counts.

use regex::Regex;

/// G-code statistics
#[derive(Debug, Clone, Default)]
pub struct Stats {
    /// Total commands
    pub total_commands: u32,
    /// Rapid movements (G0)
    pub rapid_count: u32,
    /// Linear moves (G1)
    pub linear_count: u32,
    /// Arc commands (G2/G3)
    pub arc_count: u32,
    /// Dwell commands (G4)
    pub dwell_count: u32,
    /// Min X coordinate
    pub min_x: f64,
    /// Max X coordinate
    pub max_x: f64,
    /// Min Y coordinate
    pub min_y: f64,
    /// Max Y coordinate
    pub max_y: f64,
    /// Min Z coordinate
    pub min_z: f64,
    /// Max Z coordinate
    pub max_z: f64,
}

impl Stats {
    /// Get bounding box dimensions
    pub fn bounding_box(&self) -> (f64, f64, f64) {
        (
            self.max_x - self.min_x,
            self.max_y - self.min_y,
            self.max_z - self.min_z,
        )
    }
}

/// Calculates G-code statistics
#[derive(Debug)]
pub struct StatsCalculator;

impl StatsCalculator {
    /// Calculate statistics for a program
    pub fn calculate(lines: &[String]) -> Stats {
        let mut stats = Stats {
            min_x: f64::MAX,
            min_y: f64::MAX,
            min_z: f64::MAX,
            ..Default::default()
        };

        let g_code_regex = Regex::new(r"[GgMm]\d+").unwrap();
        let coord_regex = Regex::new(r"([XYZFST])(-?\d+\.?\d*)").unwrap();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(';') {
                continue;
            }

            stats.total_commands += 1;

            // Count command types
            for cap in g_code_regex.find_iter(trimmed) {
                let code_str = cap.as_str();
                if code_str.len() > 1 {
                    if let Ok(num) = code_str[1..].parse::<u32>() {
                        match code_str.chars().next().unwrap() {
                            'G' | 'g' => match num {
                                0 => stats.rapid_count += 1,
                                1 => stats.linear_count += 1,
                                2 | 3 => stats.arc_count += 1,
                                4 => stats.dwell_count += 1,
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                }
            }

            // Extract coordinates
            for cap in coord_regex.captures_iter(trimmed) {
                if let (Some(axis), Some(value)) = (cap.get(1), cap.get(2)) {
                    if let Ok(val) = value.as_str().parse::<f64>() {
                        match axis.as_str() {
                            "X" => {
                                stats.min_x = stats.min_x.min(val);
                                stats.max_x = stats.max_x.max(val);
                            }
                            "Y" => {
                                stats.min_y = stats.min_y.min(val);
                                stats.max_y = stats.max_y.max(val);
                            }
                            "Z" => {
                                stats.min_z = stats.min_z.min(val);
                                stats.max_z = stats.max_z.max(val);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        stats
    }
}
