use gcodekit4_camtools::comment_processor::{CommentProcessor, CommentMode};

#[test]
fn test_extract_parentheses_comment() {
    let proc = CommentProcessor::new(CommentMode::Extract);
    let (_, comment) = proc.process_line("G0 X10 (move)");
    assert_eq!(comment, Some("move".to_string()));
}

#[test]
fn test_extract_semicolon_comment() {
    let proc = CommentProcessor::new(CommentMode::Extract);
    let (_, comment) = proc.process_line("G0 X10; move");
    assert_eq!(comment, Some("move".to_string()));
}

#[test]
fn test_remove_comments() {
    let proc = CommentProcessor::default();
    let (processed, _) = proc.process_line("G0 X10 (comment)");
    assert_eq!(processed, "G0 X10");
}
