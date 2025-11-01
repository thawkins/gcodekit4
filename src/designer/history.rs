//! # Designer History and Undo/Redo Module
//!
//! Provides undo/redo functionality for all Designer operations.
//!
//! Tracks design state changes and allows reverting to previous states.
//! Supports multiple levels of undo/redo with configurable depth limit.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Action type for undo/redo tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    /// Shape creation
    ShapeCreated,
    /// Shape deletion
    ShapeDeleted,
    /// Shape moved
    ShapeMoved,
    /// Shape resized
    ShapeResized,
    /// Shape rotated
    ShapeRotated,
    /// Shape properties modified
    ShapeModified,
    /// Multiple shapes operation
    MultipleShapes,
    /// Tool selection changed
    ToolChanged,
    /// Material changed
    MaterialChanged,
    /// Operation changed
    OperationChanged,
    /// Batch operation
    BatchOperation,
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionType::ShapeCreated => write!(f, "Create Shape"),
            ActionType::ShapeDeleted => write!(f, "Delete Shape"),
            ActionType::ShapeMoved => write!(f, "Move Shape"),
            ActionType::ShapeResized => write!(f, "Resize Shape"),
            ActionType::ShapeRotated => write!(f, "Rotate Shape"),
            ActionType::ShapeModified => write!(f, "Modify Shape"),
            ActionType::MultipleShapes => write!(f, "Multiple Shapes"),
            ActionType::ToolChanged => write!(f, "Change Tool"),
            ActionType::MaterialChanged => write!(f, "Change Material"),
            ActionType::OperationChanged => write!(f, "Change Operation"),
            ActionType::BatchOperation => write!(f, "Batch Operation"),
        }
    }
}

/// Historical action with state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryAction {
    /// Type of action
    pub action_type: ActionType,
    /// Description of action
    pub description: String,
    /// Serialized before-state
    pub before_state: String,
    /// Serialized after-state
    pub after_state: String,
    /// Timestamp of action
    pub timestamp: String,
}

impl HistoryAction {
    /// Create new history action
    pub fn new(
        action_type: ActionType,
        description: String,
        before_state: String,
        after_state: String,
    ) -> Self {
        let timestamp = chrono::Utc::now().to_rfc3339();
        Self {
            action_type,
            description,
            before_state,
            after_state,
            timestamp,
        }
    }

    /// Create simple action without state tracking (for testing)
    pub fn simple(action_type: ActionType, description: String) -> Self {
        Self::new(action_type, description, "{}".to_string(), "{}".to_string())
    }
}

/// Undo/Redo manager for design history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoRedoManager {
    /// Stack of undo actions
    undo_stack: Vec<HistoryAction>,
    /// Stack of redo actions
    redo_stack: Vec<HistoryAction>,
    /// Maximum history depth
    max_depth: usize,
    /// Current state index
    current_index: usize,
    /// Whether history is enabled
    enabled: bool,
}

impl UndoRedoManager {
    /// Create new undo/redo manager
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
            current_index: 0,
            enabled: true,
        }
    }

    /// Create with default depth (50)
    pub fn default_depth() -> Self {
        Self::new(50)
    }

    /// Record an action
    pub fn record(&mut self, action: HistoryAction) {
        if !self.enabled {
            return;
        }

        self.undo_stack.push(action);
        self.redo_stack.clear();

        // Enforce depth limit
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }

        self.current_index = self.undo_stack.len();
    }

    /// Undo last action
    pub fn undo(&mut self) -> Option<&HistoryAction> {
        if self.undo_stack.is_empty() {
            return None;
        }

        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action);
            self.current_index = self.current_index.saturating_sub(1);
            self.redo_stack.last()
        } else {
            None
        }
    }

    /// Redo last undone action
    pub fn redo(&mut self) -> Option<&HistoryAction> {
        if self.redo_stack.is_empty() {
            return None;
        }

        if let Some(action) = self.redo_stack.pop() {
            self.undo_stack.push(action);
            self.current_index += 1;
            self.undo_stack.last()
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get last undo action description
    pub fn undo_description(&self) -> Option<String> {
        self.undo_stack.last().map(|a| a.description.clone())
    }

    /// Get last redo action description
    pub fn redo_description(&self) -> Option<String> {
        self.redo_stack.last().map(|a| a.description.clone())
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_index = 0;
    }

    /// Get undo stack depth
    pub fn undo_depth(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get redo stack depth
    pub fn redo_depth(&self) -> usize {
        self.redo_stack.len()
    }

    /// Get maximum depth
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Set maximum depth
    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = max_depth;
    }

    /// Enable history recording
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable history recording
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if history is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get all undo actions
    pub fn undo_history(&self) -> &[HistoryAction] {
        &self.undo_stack
    }

    /// Get all redo actions
    pub fn redo_history(&self) -> &[HistoryAction] {
        &self.redo_stack
    }

    /// Get full history (combined undo + redo)
    pub fn full_history(&self) -> Vec<&HistoryAction> {
        let mut history: Vec<&HistoryAction> = self.undo_stack.iter().collect();
        history.extend(self.redo_stack.iter().rev());
        history
    }

    /// Get current index in history
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Limit history to specified depth, removing oldest entries
    pub fn trim_to_depth(&mut self, depth: usize) {
        if self.undo_stack.len() > depth {
            self.undo_stack.drain(0..self.undo_stack.len() - depth);
        }
    }
}

impl Default for UndoRedoManager {
    fn default() -> Self {
        Self::default_depth()
    }
}

/// Transaction for grouping multiple related actions
#[derive(Debug)]
pub struct HistoryTransaction {
    actions: Vec<HistoryAction>,
    description: String,
}

impl HistoryTransaction {
    /// Create new transaction
    pub fn new(description: String) -> Self {
        Self {
            actions: Vec::new(),
            description,
        }
    }

    /// Add action to transaction
    pub fn add_action(&mut self, action: HistoryAction) {
        self.actions.push(action);
    }

    /// Commit transaction and return as single batch action
    pub fn commit(self) -> HistoryAction {
        let mut combined_before = String::from("{");
        let mut combined_after = String::from("{");

        for (i, action) in self.actions.iter().enumerate() {
            if i > 0 {
                combined_before.push(',');
                combined_after.push(',');
            }
            combined_before.push_str(&format!("\"{}\":{}", i, action.before_state));
            combined_after.push_str(&format!("\"{}\":{}", i, action.after_state));
        }

        combined_before.push('}');
        combined_after.push('}');

        HistoryAction::new(
            ActionType::BatchOperation,
            self.description,
            combined_before,
            combined_after,
        )
    }

    /// Get action count in transaction
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    /// Check if transaction is empty
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_history_action() {
        let action =
            HistoryAction::simple(ActionType::ShapeCreated, "Created rectangle".to_string());

        assert_eq!(action.action_type, ActionType::ShapeCreated);
        assert_eq!(action.description, "Created rectangle");
    }

    #[test]
    fn test_undo_redo_manager_creation() {
        let manager = UndoRedoManager::new(50);
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
        assert_eq!(manager.undo_depth(), 0);
        assert_eq!(manager.redo_depth(), 0);
    }

    #[test]
    fn test_record_single_action() {
        let mut manager = UndoRedoManager::new(50);
        let action = HistoryAction::simple(ActionType::ShapeCreated, "Create shape".to_string());

        manager.record(action);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
        assert_eq!(manager.undo_depth(), 1);
    }

    #[test]
    fn test_undo_single_action() {
        let mut manager = UndoRedoManager::new(50);
        let action = HistoryAction::simple(ActionType::ShapeCreated, "Create shape".to_string());

        manager.record(action);
        assert!(manager.can_undo());

        let undone = manager.undo();
        assert!(undone.is_some());
        assert!(!manager.can_undo());
        assert!(manager.can_redo());
    }

    #[test]
    fn test_redo_after_undo() {
        let mut manager = UndoRedoManager::new(50);
        let action = HistoryAction::simple(ActionType::ShapeCreated, "Create shape".to_string());

        manager.record(action);
        manager.undo();
        assert!(manager.can_redo());

        let redone = manager.redo();
        assert!(redone.is_some());
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_multiple_undo_redo() {
        let mut manager = UndoRedoManager::new(50);

        for i in 0..5 {
            let action =
                HistoryAction::simple(ActionType::ShapeCreated, format!("Create shape {}", i));
            manager.record(action);
        }

        assert_eq!(manager.undo_depth(), 5);
        assert_eq!(manager.redo_depth(), 0);

        // Undo all
        for _ in 0..5 {
            manager.undo();
        }

        assert_eq!(manager.undo_depth(), 0);
        assert_eq!(manager.redo_depth(), 5);

        // Redo all
        for _ in 0..5 {
            manager.redo();
        }

        assert_eq!(manager.undo_depth(), 5);
        assert_eq!(manager.redo_depth(), 0);
    }

    #[test]
    fn test_redo_clears_on_new_action() {
        let mut manager = UndoRedoManager::new(50);

        let a1 = HistoryAction::simple(ActionType::ShapeCreated, "A".to_string());
        let a2 = HistoryAction::simple(ActionType::ShapeCreated, "B".to_string());
        let a3 = HistoryAction::simple(ActionType::ShapeCreated, "C".to_string());

        manager.record(a1);
        manager.record(a2);
        manager.undo();

        assert_eq!(manager.redo_depth(), 1);

        manager.record(a3);
        assert_eq!(manager.redo_depth(), 0);
    }

    #[test]
    fn test_max_depth_limit() {
        let mut manager = UndoRedoManager::new(3);

        for i in 0..5 {
            let action = HistoryAction::simple(ActionType::ShapeCreated, format!("Action {}", i));
            manager.record(action);
        }

        assert_eq!(manager.undo_depth(), 3);
    }

    #[test]
    fn test_clear_history() {
        let mut manager = UndoRedoManager::new(50);

        let action = HistoryAction::simple(ActionType::ShapeCreated, "Create".to_string());
        manager.record(action);

        assert!(manager.can_undo());

        manager.undo();

        assert!(!manager.can_undo());
        assert!(manager.can_redo());

        manager.clear();

        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
        assert_eq!(manager.undo_depth(), 0);
        assert_eq!(manager.redo_depth(), 0);
    }

    #[test]
    fn test_enable_disable_history() {
        let mut manager = UndoRedoManager::new(50);
        assert!(manager.is_enabled());

        let action = HistoryAction::simple(ActionType::ShapeCreated, "Create".to_string());

        manager.record(action);
        assert_eq!(manager.undo_depth(), 1);

        manager.disable();
        let action = HistoryAction::simple(ActionType::ShapeCreated, "Create2".to_string());
        manager.record(action);

        // Should still be 1 since disabled
        assert_eq!(manager.undo_depth(), 1);

        manager.enable();
        manager.record(HistoryAction::simple(
            ActionType::ShapeCreated,
            "Create3".to_string(),
        ));
        assert_eq!(manager.undo_depth(), 2);
    }

    #[test]
    fn test_descriptions() {
        let mut manager = UndoRedoManager::new(50);

        let action = HistoryAction::simple(
            ActionType::ShapeMoved,
            "Move rectangle to (10, 20)".to_string(),
        );

        manager.record(action);
        assert_eq!(
            manager.undo_description(),
            Some("Move rectangle to (10, 20)".to_string())
        );

        manager.undo();
        assert_eq!(
            manager.redo_description(),
            Some("Move rectangle to (10, 20)".to_string())
        );
    }

    #[test]
    fn test_transaction() {
        let mut txn = HistoryTransaction::new("Multi-shape create".to_string());

        let a1 = HistoryAction::simple(ActionType::ShapeCreated, "Shape 1".to_string());
        let a2 = HistoryAction::simple(ActionType::ShapeCreated, "Shape 2".to_string());

        txn.add_action(a1);
        txn.add_action(a2);

        assert_eq!(txn.action_count(), 2);

        let batch = txn.commit();
        assert_eq!(batch.action_type, ActionType::BatchOperation);
        assert_eq!(batch.description, "Multi-shape create");
    }

    #[test]
    fn test_action_type_display() {
        assert_eq!(ActionType::ShapeCreated.to_string(), "Create Shape");
        assert_eq!(ActionType::ShapeMoved.to_string(), "Move Shape");
        assert_eq!(ActionType::ToolChanged.to_string(), "Change Tool");
    }

    #[test]
    fn test_trim_to_depth() {
        let mut manager = UndoRedoManager::new(100);

        for i in 0..10 {
            let action = HistoryAction::simple(ActionType::ShapeCreated, format!("Action {}", i));
            manager.record(action);
        }

        assert_eq!(manager.undo_depth(), 10);

        manager.trim_to_depth(5);
        assert_eq!(manager.undo_depth(), 5);
    }

    #[test]
    fn test_full_history() {
        let mut manager = UndoRedoManager::new(50);

        let a1 = HistoryAction::simple(ActionType::ShapeCreated, "A".to_string());
        let a2 = HistoryAction::simple(ActionType::ShapeMoved, "B".to_string());

        manager.record(a1);
        manager.record(a2);
        manager.undo();

        let history = manager.full_history();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_serialization() {
        let action =
            HistoryAction::simple(ActionType::ShapeCreated, "Create rectangle".to_string());

        let json = serde_json::to_string(&action).expect("Failed to serialize");
        let deserialized: HistoryAction =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(action.description, deserialized.description);
        assert_eq!(action.action_type, deserialized.action_type);
    }
}
