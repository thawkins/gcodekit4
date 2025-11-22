use gcodekit4_gcodeeditor::{UndoManager, TextChange};

#[test]
fn test_record_and_undo() {
    let mut mgr = UndoManager::new();

    let change = TextChange::new(0..0, String::new(), "Hello".to_string(), 0, 5);

    mgr.record(change);
    assert!(mgr.can_undo());
    assert!(!mgr.can_redo());

    let undo_change = mgr.undo().unwrap();
    assert_eq!(undo_change.new_text, "");
    assert_eq!(undo_change.old_text, "Hello");
    assert!(!mgr.can_undo());
    assert!(mgr.can_redo());
}

#[test]
fn test_redo() {
    let mut mgr = UndoManager::new();

    let change = TextChange::new(0..0, String::new(), "Hello".to_string(), 0, 5);

    mgr.record(change);
    mgr.undo();

    let redo_change = mgr.redo().unwrap();
    assert_eq!(redo_change.old_text, "");
    assert_eq!(redo_change.new_text, "Hello");
}

#[test]
fn test_clear_redo_on_new_change() {
    let mut mgr = UndoManager::new();

    mgr.record(TextChange::new(0..0, String::new(), "A".to_string(), 0, 1));
    mgr.undo();
    assert!(mgr.can_redo());

    mgr.record(TextChange::new(0..0, String::new(), "B".to_string(), 0, 1));
    assert!(!mgr.can_redo());
}

#[test]
fn test_max_depth() {
    let mut mgr = UndoManager::with_depth(3);

    mgr.record(TextChange::new(0..0, String::new(), "1".to_string(), 0, 1));
    mgr.record(TextChange::new(1..1, String::new(), "2".to_string(), 1, 2));
    mgr.record(TextChange::new(2..2, String::new(), "3".to_string(), 2, 3));
    mgr.record(TextChange::new(3..3, String::new(), "4".to_string(), 3, 4));

    assert_eq!(mgr.undo_count(), 3); // Should have trimmed oldest
}

#[test]
fn test_batch() {
    let mut mgr = UndoManager::new();

    mgr.begin_batch();
    mgr.record(TextChange::new(0..0, String::new(), "H".to_string(), 0, 1));
    mgr.record(TextChange::new(1..1, String::new(), "i".to_string(), 1, 2));
    mgr.end_batch();

    assert_eq!(mgr.undo_count(), 2); // Two changes in batch
}
