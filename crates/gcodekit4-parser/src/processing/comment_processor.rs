//! Comment Processor - Task 54
//!
//! Extracts and processes G-code comments (both parentheses and semicolon styles).

/// Comment processing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentMode {
    /// Remove all comments
    Remove,
    /// Keep comments inline
    Keep,
    /// Extract comments separately
    Extract,
}

/// Processes comments in G-code
#[derive(Debug)]
pub struct CommentProcessor {
    mode: CommentMode,
}

impl CommentProcessor {
    /// Create a new comment processor
    pub fn new(mode: CommentMode) -> Self {
        Self { mode }
    }

    /// Process a line and extract comment if present
    pub fn process_line(&self, line: &str) -> (String, Option<String>) {
        let comment = Self::extract_comment(line);
        let processed = match self.mode {
            CommentMode::Remove => Self::remove_comments(line),
            CommentMode::Keep => line.to_string(),
            CommentMode::Extract => Self::remove_comments(line),
        };
        (processed, comment)
    }

    /// Extract comment from line
    fn extract_comment(line: &str) -> Option<String> {
        // Check for parentheses comment
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                if end > start {
                    return Some(line[start + 1..end].trim().to_string());
                }
            }
        }

        // Check for semicolon comment
        if let Some(start) = line.find(';') {
            return Some(line[start + 1..].trim().to_string());
        }

        None
    }

    /// Remove comments from line
    fn remove_comments(line: &str) -> String {
        let mut result = line.to_string();

        // Remove parentheses comment
        if let Some(start) = result.find('(') {
            if let Some(end) = result.find(')') {
                result = format!("{}{}", &result[..start], &result[end + 1..]);
            }
        }

        // Remove semicolon comment
        if let Some(pos) = result.find(';') {
            result.truncate(pos);
        }

        result.trim().to_string()
    }
}

impl Default for CommentProcessor {
    fn default() -> Self {
        Self::new(CommentMode::Remove)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_parentheses_comment() {
        let comment = CommentProcessor::extract_comment("G0 X10 (move)");
        assert_eq!(comment, Some("move".to_string()));
    }

    #[test]
    fn test_extract_semicolon_comment() {
        let comment = CommentProcessor::extract_comment("G0 X10; move");
        assert_eq!(comment, Some("move".to_string()));
    }

    #[test]
    fn test_remove_comments() {
        let proc = CommentProcessor::default();
        let (processed, _) = proc.process_line("G0 X10 (comment)");
        assert_eq!(processed, "G0 X10");
    }
}
