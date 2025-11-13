//! Bridge between Slint UI and EditorState backend

use super::{EditorState, TextLine};
use slint::{Model, ModelRc, VecModel};
use std::rc::Rc;
use std::cell::RefCell;

/// Slint-compatible text line structure
#[derive(Clone, Debug)]
pub struct SlintTextLine {
    pub line_number: i32,
    pub content: String,
    pub is_dirty: bool,
}

impl SlintTextLine {
    pub fn new(line_number: usize, content: String, is_dirty: bool) -> Self {
        Self {
            line_number: line_number as i32,
            content,
            is_dirty,
        }
    }
}

/// Bridge between EditorState and Slint UI
pub struct EditorBridge {
    editor: Rc<RefCell<EditorState>>,
    visible_lines: Rc<VecModel<SlintTextLine>>,
}

impl EditorBridge {
    /// Create a new editor bridge
    pub fn new(viewport_height: f32, line_height: f32) -> Self {
        let editor = Rc::new(RefCell::new(EditorState::new(viewport_height, line_height)));
        let visible_lines = Rc::new(VecModel::default());
        
        Self {
            editor,
            visible_lines,
        }
    }

    /// Load text into editor
    pub fn load_text(&self, text: &str) {
        let mut editor = self.editor.borrow_mut();
        editor.load_text(text);
        let line_count = editor.line_count();
        tracing::debug!("Loaded text into editor: {} lines", line_count);
        drop(editor);
        self.update_visible_lines();
    }

    /// Get all text from editor
    pub fn get_text(&self) -> String {
        self.editor.borrow().get_text()
    }

    /// Insert text at cursor
    pub fn insert_text(&self, text: &str) {
        let mut editor = self.editor.borrow_mut();
        editor.insert_text(text);
        drop(editor);
        self.update_visible_lines();
    }

    /// Delete text forward (delete key)
    pub fn delete_forward(&self, count: usize) {
        let mut editor = self.editor.borrow_mut();
        editor.delete_forward(count);
        drop(editor);
        self.update_visible_lines();
    }

    /// Delete text backward (backspace key)
    pub fn delete_backward(&self, count: usize) {
        let mut editor = self.editor.borrow_mut();
        editor.delete_backward(count);
        drop(editor);
        self.update_visible_lines();
    }

    /// Undo last change
    pub fn undo(&self) -> bool {
        let mut editor = self.editor.borrow_mut();
        let result = editor.undo();
        drop(editor);
        if result {
            self.update_visible_lines();
        }
        result
    }

    /// Redo last undone change
    pub fn redo(&self) -> bool {
        let mut editor = self.editor.borrow_mut();
        let result = editor.redo();
        drop(editor);
        if result {
            self.update_visible_lines();
        }
        result
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.editor.borrow().can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.editor.borrow().can_redo()
    }

    /// Scroll viewport to specific line
    pub fn scroll_to_line(&self, line: usize) {
        tracing::debug!("scroll_to_line: requesting line {}", line);
        let mut editor = self.editor.borrow_mut();
        let total_lines = editor.line_count();
        // Use scroll_to_line for absolute positioning (not scroll_by which is relative)
        editor.scroll_to_line(line);
        let viewport = editor.viewport();
        tracing::debug!(
            "scroll_to_line: set to line {} (total {} lines), viewport now {}-{}",
            line,
            total_lines,
            viewport.start_line,
            viewport.end_line
        );
        drop(editor);
        self.update_visible_lines();
    }

    /// Set cursor position
    pub fn set_cursor(&self, line: usize, column: usize) {
        let mut editor = self.editor.borrow_mut();
        let char_pos = editor.line_count(); // Simplified - would need proper line/col to char conversion
        editor.set_cursor(char_pos);
    }

    /// Get cursor line and column
    pub fn cursor_position(&self) -> (usize, usize) {
        self.editor.borrow().cursor_line_col()
    }

    /// Get total line count
    pub fn line_count(&self) -> usize {
        self.editor.borrow().line_count()
    }

    /// Check if modified
    pub fn is_modified(&self) -> bool {
        self.editor.borrow().is_modified()
    }

    /// Mark as unmodified
    pub fn mark_unmodified(&self) {
        self.editor.borrow_mut().mark_unmodified();
    }

    /// Get visible lines as Slint model
    pub fn get_visible_lines_model(&self) -> ModelRc<SlintTextLine> {
        ModelRc::from(self.visible_lines.clone())
    }
    
    /// Get visible lines as raw data for constructing Slint types
    pub fn get_visible_lines_data(&self) -> Vec<(i32, String, bool)> {
        self.visible_lines
            .iter()
            .map(|line| (line.line_number, line.content.clone(), line.is_dirty))
            .collect()
    }

    /// Update visible lines from editor state
    fn update_visible_lines(&self) {
        let editor = self.editor.borrow();
        let lines = editor.get_visible_lines();
        let viewport = editor.viewport();
        let start_line = viewport.start_line;
        let end_line = viewport.end_line;
        let total_lines = editor.line_count();
        
        tracing::debug!(
            "update_visible_lines: showing lines {}-{} of {} (fetched {} lines)",
            start_line,
            end_line,
            total_lines,
            lines.len()
        );
        
        // Clear and rebuild visible lines
        let mut new_lines = Vec::new();
        for (idx, content) in lines.iter().enumerate() {
            let line_number = start_line + idx;
            new_lines.push(SlintTextLine::new(
                line_number,
                content.clone(),
                false, // Could check dirty state here
            ));
        }
        
        // Update model
        self.visible_lines.set_vec(new_lines);
    }

    /// Get viewport info
    pub fn viewport_range(&self) -> (usize, usize) {
        let editor = self.editor.borrow();
        let viewport = editor.viewport();
        (viewport.start_line, viewport.end_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_load_text() {
        let bridge = EditorBridge::new(400.0, 20.0);
        bridge.load_text("Hello\nWorld");
        assert_eq!(bridge.get_text(), "Hello\nWorld");
    }

    #[test]
    fn test_bridge_undo_redo() {
        let bridge = EditorBridge::new(400.0, 20.0);
        bridge.load_text("");
        bridge.insert_text("Hello");
        
        assert!(bridge.can_undo());
        assert!(bridge.undo());
        assert_eq!(bridge.get_text(), "");
        
        assert!(bridge.can_redo());
        assert!(bridge.redo());
        assert_eq!(bridge.get_text(), "Hello");
    }

    #[test]
    fn test_bridge_line_count() {
        let bridge = EditorBridge::new(400.0, 20.0);
        bridge.load_text("Line 1\nLine 2\nLine 3");
        assert_eq!(bridge.line_count(), 3);
    }

    #[test]
    fn test_bridge_modified_state() {
        let bridge = EditorBridge::new(400.0, 20.0);
        bridge.load_text("Test");
        assert!(!bridge.is_modified());
        
        bridge.insert_text(" more");
        assert!(bridge.is_modified());
        
        bridge.mark_unmodified();
        assert!(!bridge.is_modified());
    }
}
