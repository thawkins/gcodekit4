//! G-Code Optimizer - Task 63
//!
//! Removes redundant commands and optimizes G-code for efficiency.

/// G-code optimization strategies
#[derive(Debug)]
pub struct GCodeOptimizer;

impl GCodeOptimizer {
    /// Remove consecutive duplicate M5 commands
    pub fn remove_redundant_m5(lines: &[String]) -> Vec<String> {
        let mut result = Vec::new();
        let mut last_was_m5 = false;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with("M5") {
                if !last_was_m5 {
                    result.push(line.clone());
                    last_was_m5 = true;
                }
            } else {
                result.push(line.clone());
                last_was_m5 = false;
            }
        }

        result
    }

    /// Remove consecutive duplicate tool selections
    pub fn remove_redundant_tools(lines: &[String]) -> Vec<String> {
        let mut result = Vec::new();
        let mut last_tool: Option<u32> = None;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with('T') {
                if let Ok(tool) = trimmed[1..].parse::<u32>() {
                    if last_tool != Some(tool) {
                        result.push(line.clone());
                        last_tool = Some(tool);
                    }
                    continue;
                }
            }
            result.push(line.clone());
        }

        result
    }

    /// Optimize G-code
    pub fn optimize(lines: &[String]) -> Vec<String> {
        let mut optimized = lines.to_vec();
        optimized = Self::remove_redundant_m5(&optimized);
        optimized = Self::remove_redundant_tools(&optimized);
        optimized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_redundant_m5() {
        let lines = vec![
            "M5".to_string(),
            "M5".to_string(),
            "G0 X10".to_string(),
        ];
        let result = GCodeOptimizer::remove_redundant_m5(&lines);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_remove_redundant_tools() {
        let lines = vec![
            "T1".to_string(),
            "T1".to_string(),
            "G0 X10".to_string(),
        ];
        let result = GCodeOptimizer::remove_redundant_tools(&lines);
        assert_eq!(result.len(), 2);
    }
}
