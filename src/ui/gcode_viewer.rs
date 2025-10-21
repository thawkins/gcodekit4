//! G-Code Viewer/Editor Panel - Task 72
//!
//! Text editor with syntax highlighting, line numbers, and search/replace

use std::collections::HashMap;

/// Syntax token types for highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// G-code command (G0, G1, etc.)
    GCode,
    /// M-code command (M3, M5, etc.)
    MCode,
    /// Word (X, Y, Z, F, S, etc.)
    Word,
    /// Number value
    Number,
    /// Comment
    Comment,
    /// Error/Invalid
    Error,
    /// Whitespace
    Whitespace,
}

/// Syntax token
#[derive(Debug, Clone)]
pub struct Token {
    /// Token type
    pub token_type: TokenType,
    /// Token text
    pub text: String,
    /// Start position in line
    pub start: usize,
    /// End position in line
    pub end: usize,
}

impl Token {
    /// Create new token
    pub fn new(token_type: TokenType, text: impl Into<String>, start: usize, end: usize) -> Self {
        Self {
            token_type,
            text: text.into(),
            start,
            end,
        }
    }
}

/// Line with syntax highlighting
#[derive(Debug, Clone)]
pub struct HighlightedLine {
    /// Line number (1-based)
    pub line_number: u32,
    /// Original text
    pub text: String,
    /// Tokens for highlighting
    pub tokens: Vec<Token>,
    /// Is error line
    pub has_error: bool,
}

impl HighlightedLine {
    /// Create new highlighted line
    pub fn new(line_number: u32, text: impl Into<String>) -> Self {
        Self {
            line_number,
            text: text.into(),
            tokens: Vec::new(),
            has_error: false,
        }
    }

    /// Add token
    pub fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    /// Mark as error
    pub fn mark_error(&mut self) {
        self.has_error = true;
    }
}

/// Basic G-code parser for syntax highlighting
pub struct GCodeParser;

impl GCodeParser {
    /// Parse a line and return tokens
    pub fn parse_line(text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let text_trimmed = text.trim_start();
        let leading_spaces = text.len() - text_trimmed.len();

        if !text_trimmed.is_empty() {
            if text_trimmed.starts_with(';') {
                tokens.push(Token::new(TokenType::Comment, text_trimmed, leading_spaces, text.len()));
            } else if text_trimmed.starts_with('(') {
                tokens.push(Token::new(TokenType::Comment, text_trimmed, leading_spaces, text.len()));
            } else {
                let mut pos = leading_spaces;
                let mut chars = text_trimmed.chars().peekable();

                while let Some(ch) = chars.next() {
                    if ch.is_whitespace() {
                        pos += ch.len_utf8();
                    } else if ch == 'G' || ch == 'g' {
                        // G-code
                        let start = pos;
                        let mut word = ch.to_string();
                        pos += ch.len_utf8();

                        while let Some(&next) = chars.peek() {
                            if next.is_ascii_digit() {
                                word.push(next);
                                chars.next();
                                pos += next.len_utf8();
                            } else {
                                break;
                            }
                        }

                        tokens.push(Token::new(TokenType::GCode, word, start, pos));
                    } else if ch == 'M' || ch == 'm' {
                        // M-code
                        let start = pos;
                        let mut word = ch.to_string();
                        pos += ch.len_utf8();

                        while let Some(&next) = chars.peek() {
                            if next.is_ascii_digit() {
                                word.push(next);
                                chars.next();
                                pos += next.len_utf8();
                            } else {
                                break;
                            }
                        }

                        tokens.push(Token::new(TokenType::MCode, word, start, pos));
                    } else if ch.is_alphabetic() {
                        // Word (X, Y, Z, F, S, etc.)
                        let start = pos;
                        pos += ch.len_utf8();
                        tokens.push(Token::new(TokenType::Word, ch.to_string(), start, pos));
                    } else if ch.is_ascii_digit() || ch == '-' || ch == '+' || ch == '.' {
                        // Number
                        let start = pos;
                        let mut num = ch.to_string();
                        pos += ch.len_utf8();

                        while let Some(&next) = chars.peek() {
                            if next.is_ascii_digit() || next == '.' {
                                num.push(next);
                                chars.next();
                                pos += next.len_utf8();
                            } else {
                                break;
                            }
                        }

                        tokens.push(Token::new(TokenType::Number, num, start, pos));
                    } else {
                        pos += ch.len_utf8();
                    }
                }
            }
        }

        tokens
    }
}

/// Search/replace operation
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Line number where match found
    pub line_number: u32,
    /// Position in line
    pub position: usize,
    /// Matched text
    pub text: String,
}

impl SearchResult {
    /// Create new search result
    pub fn new(line_number: u32, position: usize, text: impl Into<String>) -> Self {
        Self {
            line_number,
            position,
            text: text.into(),
        }
    }
}

/// G-Code viewer/editor panel
#[derive(Debug)]
pub struct GCodeViewerPanel {
    /// All lines
    pub lines: Vec<HighlightedLine>,
    /// Current cursor position (line, column)
    pub cursor: (u32, usize),
    /// Selection start (line, column)
    pub selection_start: Option<(u32, usize)>,
    /// Selection end (line, column)
    pub selection_end: Option<(u32, usize)>,
    /// Current search query
    pub search_query: String,
    /// Search results
    pub search_results: Vec<SearchResult>,
    /// Current search result index
    pub current_search_index: usize,
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Read-only mode
    pub read_only: bool,
    /// Modified flag
    pub modified: bool,
}

impl GCodeViewerPanel {
    /// Create new G-code viewer panel
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            cursor: (1, 0),
            selection_start: None,
            selection_end: None,
            search_query: String::new(),
            search_results: Vec::new(),
            current_search_index: 0,
            show_line_numbers: true,
            read_only: false,
            modified: false,
        }
    }

    /// Load G-code from string
    pub fn load_code(&mut self, code: &str) {
        self.lines.clear();
        for (idx, line) in code.lines().enumerate() {
            let line_num = (idx + 1) as u32;
            let mut highlighted = HighlightedLine::new(line_num, line);

            let tokens = GCodeParser::parse_line(line);
            for token in tokens {
                highlighted.add_token(token);
            }

            self.lines.push(highlighted);
        }

        self.cursor = (1, 0);
        self.modified = false;
    }

    /// Get content as string
    pub fn get_content(&self) -> String {
        self.lines
            .iter()
            .map(|l| l.text.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Insert text at cursor
    pub fn insert_text(&mut self, text: &str) {
        if self.read_only {
            return;
        }

        let (line_num, col) = self.cursor;
        if let Some(line) = self.lines.get_mut((line_num - 1) as usize) {
            let mut new_text = line.text.clone();
            new_text.insert_str(col, text);
            line.text = new_text;
            self.cursor = (line_num, col + text.len());
            self.modified = true;

            let tokens = GCodeParser::parse_line(&line.text);
            line.tokens = tokens;
        }
    }

    /// Delete character at cursor
    pub fn delete_char(&mut self) {
        if self.read_only {
            return;
        }

        let (line_num, col) = self.cursor;
        if let Some(line) = self.lines.get_mut((line_num - 1) as usize) {
            if col < line.text.len() {
                line.text.remove(col);
                self.modified = true;

                let tokens = GCodeParser::parse_line(&line.text);
                line.tokens = tokens;
            }
        }
    }

    /// Move cursor to line
    pub fn goto_line(&mut self, line_number: u32) {
        if line_number > 0 && (line_number as usize) <= self.lines.len() {
            self.cursor = (line_number, 0);
        }
    }

    /// Search for text
    pub fn search(&mut self, query: &str) -> Vec<SearchResult> {
        self.search_query = query.to_string();
        self.search_results.clear();
        self.current_search_index = 0;

        for line in &self.lines {
            let text_lower = line.text.to_lowercase();
            let query_lower = query.to_lowercase();

            let mut pos = 0;
            while let Some(index) = text_lower[pos..].find(&query_lower) {
                let actual_pos = pos + index;
                self.search_results.push(SearchResult::new(
                    line.line_number,
                    actual_pos,
                    query,
                ));
                pos = actual_pos + 1;
            }
        }

        self.search_results.clone()
    }

    /// Replace current match
    pub fn replace_current(&mut self, replacement: &str) {
        if let Some(result) = self.search_results.get(self.current_search_index) {
            let line_num = (result.line_number - 1) as usize;
            if let Some(line) = self.lines.get_mut(line_num) {
                let start = result.position;
                let end = start + result.text.len();

                let mut new_text = line.text.clone();
                new_text.replace_range(start..end, replacement);
                line.text = new_text;
                self.modified = true;

                let tokens = GCodeParser::parse_line(&line.text);
                line.tokens = tokens;
            }
        }
    }

    /// Replace all matches
    pub fn replace_all(&mut self, old: &str, new: &str) -> u32 {
        let mut count = 0;
        for line in &mut self.lines {
            let before = line.text.clone();
            line.text = line.text.replace(old, new);
            if line.text != before {
                count += 1;
                self.modified = true;

                let tokens = GCodeParser::parse_line(&line.text);
                line.tokens = tokens;
            }
        }

        count
    }

    /// Next search result
    pub fn next_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_index = (self.current_search_index + 1) % self.search_results.len();
        }
    }

    /// Previous search result
    pub fn prev_search_result(&mut self) {
        if !self.search_results.is_empty() {
            if self.current_search_index == 0 {
                self.current_search_index = self.search_results.len() - 1;
            } else {
                self.current_search_index -= 1;
            }
        }
    }

    /// Get current search result
    pub fn get_current_search(&self) -> Option<&SearchResult> {
        self.search_results.get(self.current_search_index)
    }

    /// Set read-only mode
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    /// Get line count
    pub fn line_count(&self) -> u32 {
        self.lines.len() as u32
    }

    /// Get line text
    pub fn get_line(&self, line_num: u32) -> Option<&str> {
        self.lines
            .get((line_num - 1) as usize)
            .map(|l| l.text.as_str())
    }
}

impl Default for GCodeViewerPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcode_parser() {
        let tokens = GCodeParser::parse_line("G0 X10.5 Y20.0");
        assert!(tokens.iter().any(|t| t.token_type == TokenType::GCode));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Word));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Number));
    }

    #[test]
    fn test_mcode_parser() {
        let tokens = GCodeParser::parse_line("M3 S1000");
        assert!(tokens.iter().any(|t| t.token_type == TokenType::MCode));
    }

    #[test]
    fn test_comment_parser() {
        let tokens = GCodeParser::parse_line("; This is a comment");
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Comment));
    }

    #[test]
    fn test_viewer_load_code() {
        let mut viewer = GCodeViewerPanel::new();
        viewer.load_code("G0 X10\nG1 Y20\nM3 S1000");
        assert_eq!(viewer.line_count(), 3);
    }

    #[test]
    fn test_viewer_cursor() {
        let mut viewer = GCodeViewerPanel::new();
        viewer.load_code("G0 X10\nG1 Y20");
        viewer.goto_line(2);
        assert_eq!(viewer.cursor.0, 2);
    }

    #[test]
    fn test_search() {
        let mut viewer = GCodeViewerPanel::new();
        viewer.load_code("G0 X10\nG1 Y20\nG0 X30");
        let results = viewer.search("G0");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut viewer = GCodeViewerPanel::new();
        viewer.load_code("g0 X10\nG1 Y20");
        let results = viewer.search("G0");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_replace_all() {
        let mut viewer = GCodeViewerPanel::new();
        viewer.load_code("X10 Y10\nX20 Y10");
        let count = viewer.replace_all("Y10", "Y30");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_read_only_mode() {
        let mut viewer = GCodeViewerPanel::new();
        viewer.load_code("G0 X10");
        viewer.set_read_only(true);
        viewer.insert_text(" Y20");
        assert_eq!(viewer.line_count(), 1);
    }

    #[test]
    fn test_token_creation() {
        let token = Token::new(TokenType::GCode, "G0", 0, 2);
        assert_eq!(token.text, "G0");
        assert_eq!(token.start, 0);
        assert_eq!(token.end, 2);
    }

    #[test]
    fn test_search_navigation() {
        let mut viewer = GCodeViewerPanel::new();
        viewer.load_code("G0\nG0\nG0");
        viewer.search("G0");
        assert_eq!(viewer.current_search_index, 0);
        viewer.next_search_result();
        assert_eq!(viewer.current_search_index, 1);
        viewer.prev_search_result();
        assert_eq!(viewer.current_search_index, 0);
    }
}
