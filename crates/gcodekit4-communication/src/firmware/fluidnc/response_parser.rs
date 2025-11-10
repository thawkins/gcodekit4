//! FluidNC response parser
//!
//! Parses responses from FluidNC firmware including status reports,
//! errors, and standard responses.

/// Parsed FluidNC response
#[derive(Debug, Clone, PartialEq)]
pub enum FluidNCResponse {
    /// Command was successful (ok)
    Ok,
    /// Error response with code and message
    Error { code: u32, message: String },
    /// Position feedback
    Position { x: f64, y: f64, z: f64 },
    /// Firmware version
    Version(String),
    /// File list from SD card
    FileList(Vec<String>),
    /// WiFi status
    WiFiStatus(String),
    /// Raw line from controller
    Raw(String),
}

/// Parser for FluidNC protocol responses
#[derive(Debug, Clone)]
pub struct FluidNCResponseParser {
    /// Buffer for incomplete responses
    buffer: String,
}

impl FluidNCResponseParser {
    /// Create a new response parser
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Parse a single line from FluidNC
    pub fn parse_line(&mut self, line: &str) -> Option<FluidNCResponse> {
        let line = line.trim();

        if line.is_empty() {
            return None;
        }

        // Check for acknowledgment
        if line == "ok" || line == "OK" {
            return Some(FluidNCResponse::Ok);
        }

        // Check for error
        if line.starts_with("error:") {
            let error_part = line.strip_prefix("error:").unwrap_or(line).trim();
            let (code, message) = self.parse_error(error_part);
            return Some(FluidNCResponse::Error { code, message });
        }

        // Check for version
        if line.starts_with("FluidNC") {
            return Some(FluidNCResponse::Version(line.to_string()));
        }

        // Check for position feedback
        if line.contains("MPos:") && line.contains("WPos:") {
            if let Some(parsed) = self.parse_position(line) {
                return Some(parsed);
            }
        }

        // Check for file list
        if line.starts_with("[FILE]") {
            let filename = line.strip_prefix("[FILE]").unwrap_or("").trim().to_string();
            return Some(FluidNCResponse::Raw(format!("FILE: {}", filename)));
        }

        // Check for WiFi status
        if line.contains("WiFi") || line.contains("wifi") {
            return Some(FluidNCResponse::WiFiStatus(line.to_string()));
        }

        // Return as raw response
        Some(FluidNCResponse::Raw(line.to_string()))
    }

    /// Parse error code and message
    fn parse_error(&self, error_str: &str) -> (u32, String) {
        let parts: Vec<&str> = error_str.split(' ').collect();
        if let Some(first_part) = parts.first() {
            if let Ok(code) = first_part.parse::<u32>() {
                let message = parts[1..].join(" ");
                (code, message)
            } else {
                (0, error_str.to_string())
            }
        } else {
            (0, error_str.to_string())
        }
    }

    /// Parse position from response line
    fn parse_position(&self, line: &str) -> Option<FluidNCResponse> {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut found_all = false;

        for part in line.split(',') {
            let part = part.trim();
            if part.starts_with("MPos:") {
                let coords = part.strip_prefix("MPos:")?;
                let mut coord_parts = coords.split(':');
                if let Ok(val) = coord_parts.next()?.parse::<f64>() {
                    x = val;
                }
                if let Ok(val) = coord_parts.next()?.parse::<f64>() {
                    y = val;
                }
                if let Ok(val) = coord_parts.next()?.parse::<f64>() {
                    z = val;
                    found_all = true;
                }
            }
        }

        if found_all {
            Some(FluidNCResponse::Position { x, y, z })
        } else {
            None
        }
    }

    /// Clear the parser buffer
    pub fn reset(&mut self) {
        self.buffer.clear();
    }
}

impl Default for FluidNCResponseParser {
    fn default() -> Self {
        Self::new()
    }
}
