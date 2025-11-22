//! G-Code Validator - Task 65
//!
//! Validates G-code syntax, ranges, and consistency.

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Line number
    pub line: usize,
    /// Error message
    pub message: String,
}

/// Validator configuration
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Max X
    pub max_x: f64,
    /// Min X
    pub min_x: f64,
    /// Max Y
    pub max_y: f64,
    /// Min Y
    pub min_y: f64,
    /// Max Z
    pub max_z: f64,
    /// Min Z
    pub min_z: f64,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            max_x: 1000.0,
            min_x: -1000.0,
            max_y: 1000.0,
            min_y: -1000.0,
            max_z: 500.0,
            min_z: -500.0,
        }
    }
}

/// Validates G-code
#[derive(Debug)]
pub struct GCodeValidator {
    config: ValidatorConfig,
}

impl GCodeValidator {
    /// Create validator
    pub fn new(config: ValidatorConfig) -> Self {
        Self { config }
    }

    /// Validate program
    pub fn validate(&self, lines: &[String]) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            if let Err(e) = self.validate_line(line, line_num) {
                errors.extend(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_line(&self, line: &str, line_num: usize) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with(';') {
            return Ok(());
        }

        // Extract and validate coordinates using regex
        if let Some(x_pos) = self.extract_coord(trimmed, 'X') {
            if x_pos < self.config.min_x || x_pos > self.config.max_x {
                errors.push(ValidationError {
                    line: line_num,
                    message: format!(
                        "X {} out of range [{}, {}]",
                        x_pos, self.config.min_x, self.config.max_x
                    ),
                });
            }
        }

        if let Some(y_pos) = self.extract_coord(trimmed, 'Y') {
            if y_pos < self.config.min_y || y_pos > self.config.max_y {
                errors.push(ValidationError {
                    line: line_num,
                    message: format!(
                        "Y {} out of range [{}, {}]",
                        y_pos, self.config.min_y, self.config.max_y
                    ),
                });
            }
        }

        if let Some(z_pos) = self.extract_coord(trimmed, 'Z') {
            if z_pos < self.config.min_z || z_pos > self.config.max_z {
                errors.push(ValidationError {
                    line: line_num,
                    message: format!(
                        "Z {} out of range [{}, {}]",
                        z_pos, self.config.min_z, self.config.max_z
                    ),
                });
            }
        }

        // Validate feed rate
        if let Some(feed) = self.extract_coord(trimmed, 'F') {
            if feed <= 0.0 {
                errors.push(ValidationError {
                    line: line_num,
                    message: format!("Feed rate must be positive, got {}", feed),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn extract_coord(&self, line: &str, axis: char) -> Option<f64> {
        let pattern = format!("{}", axis);
        if let Some(pos) = line.find(pattern.as_str()) {
            let remainder = &line[pos + 1..];
            let mut num_str = String::new();
            for ch in remainder.chars() {
                if ch.is_ascii_digit() || ch == '.' || ch == '-' {
                    num_str.push(ch);
                } else {
                    break;
                }
            }
            num_str.parse().ok()
        } else {
            None
        }
    }
}

impl Default for GCodeValidator {
    fn default() -> Self {
        Self::new(ValidatorConfig::default())
    }
}


