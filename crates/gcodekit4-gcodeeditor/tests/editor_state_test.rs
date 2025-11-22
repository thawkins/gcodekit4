use gcodekit4_gcodeeditor::EditorState;

#[test]
fn test_editor_insert() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.insert_text("Hello");
    assert_eq!(editor.get_text(), "Hello");
    assert_eq!(editor.cursor_pos(), 5);
    assert!(editor.is_modified());
}

#[test]
fn test_editor_undo_redo() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.insert_text("Hello");

    assert!(editor.can_undo());
    editor.undo();
    assert_eq!(editor.get_text(), "");

    assert!(editor.can_redo());
    editor.redo();
    assert_eq!(editor.get_text(), "Hello");
}

#[test]
fn test_editor_delete() {
    let mut editor = EditorState::new(400.0, 20.0);
    editor.insert_text("Hello");
    editor.delete_backward(2);
    assert_eq!(editor.get_text(), "Hel");
}
