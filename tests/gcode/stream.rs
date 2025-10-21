//! Tests for G-Code stream management

use gcodekit4::{FileStreamReader, GcodeStreamReader, PausableStream, StringStreamReader};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_string_stream_basic_reading() {
    let content = "G28\nG0 X10 Y10\nG1 Z5 F100\n";
    let mut reader = StringStreamReader::new(content);

    assert_eq!(reader.current_line_number(), 0);
    assert_eq!(reader.total_lines(), Some(3));
    assert!(!reader.is_eof());

    let line1 = reader.read_line();
    assert_eq!(line1, Some("G28".to_string()));
    assert_eq!(reader.current_line_number(), 1);

    let line2 = reader.read_line();
    assert_eq!(line2, Some("G0 X10 Y10".to_string()));
    assert_eq!(reader.current_line_number(), 2);

    let line3 = reader.read_line();
    assert_eq!(line3, Some("G1 Z5 F100".to_string()));
    assert_eq!(reader.current_line_number(), 3);

    assert!(reader.is_eof());
    assert_eq!(reader.read_line(), None);
}

#[test]
fn test_string_stream_progress_tracking() {
    let content = "L0\nL1\nL2\nL3\nL4\n";
    let mut reader = StringStreamReader::new(content);

    assert_eq!(reader.progress_percent(), 0.0);

    reader.read_line();
    assert!(reader.progress_percent() > 0.0);
    assert!(reader.progress_percent() < 100.0);

    // Read all lines
    while reader.read_line().is_some() {}
    assert_eq!(reader.progress_percent(), 100.0);
}

#[test]
fn test_string_stream_reset() {
    let content = "G28\nG0 X10\nG1 Z5\n";
    let mut reader = StringStreamReader::new(content);

    reader.read_line();
    reader.read_line();
    assert_eq!(reader.current_line_number(), 2);

    reader.reset().unwrap();
    assert_eq!(reader.current_line_number(), 0);
    assert!(!reader.is_eof());
    assert_eq!(reader.read_line(), Some("G28".to_string()));
}

#[test]
fn test_string_stream_seek_to_line() {
    let content = "L0\nL1\nL2\nL3\nL4\n";
    let mut reader = StringStreamReader::new(content);

    reader.seek_to_line(2).unwrap();
    assert_eq!(reader.current_line_number(), 2);
    assert_eq!(reader.read_line(), Some("L2".to_string()));

    reader.seek_to_line(0).unwrap();
    assert_eq!(reader.current_line_number(), 0);
    assert_eq!(reader.read_line(), Some("L0".to_string()));
}

#[test]
fn test_string_stream_seek_beyond_end() {
    let content = "L0\nL1\nL2\n";
    let mut reader = StringStreamReader::new(content);

    let result = reader.seek_to_line(10);
    assert!(result.is_err());
}

#[test]
fn test_file_stream_basic_reading() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "G28").unwrap();
    writeln!(file, "G0 X10 Y10").unwrap();
    writeln!(file, "G1 Z5 F100").unwrap();
    file.flush().unwrap();

    let mut reader = FileStreamReader::new(file.path()).unwrap();

    assert_eq!(reader.current_line_number(), 0);
    assert_eq!(reader.total_lines(), Some(3));

    let line1 = reader.read_line();
    assert_eq!(line1, Some("G28\n".to_string()));
    assert_eq!(reader.current_line_number(), 1);
}

#[test]
fn test_file_stream_reset() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "L0").unwrap();
    writeln!(file, "L1").unwrap();
    writeln!(file, "L2").unwrap();
    file.flush().unwrap();

    let mut reader = FileStreamReader::new(file.path()).unwrap();

    reader.read_line();
    reader.read_line();
    assert_eq!(reader.current_line_number(), 2);

    reader.reset().unwrap();
    assert_eq!(reader.current_line_number(), 0);
}

#[test]
fn test_pausable_stream_pause_resume() {
    let content = "G28\nG0 X10\nG1 Z5\n";
    let reader = StringStreamReader::new(content);
    let mut pausable = PausableStream::new(Box::new(reader));

    assert!(!pausable.is_paused());

    pausable.pause();
    assert!(pausable.is_paused());

    pausable.resume();
    assert!(!pausable.is_paused());
}

#[test]
fn test_pausable_stream_blocks_reads_when_paused() {
    let content = "G28\nG0 X10\nG1 Z5\n";
    let reader = StringStreamReader::new(content);
    let mut pausable = PausableStream::new(Box::new(reader));

    // Read first line
    let line1 = pausable.read_line();
    assert_eq!(line1, Some("G28".to_string()));

    // Pause
    pausable.pause();

    // Should not read while paused
    assert_eq!(pausable.read_line(), None);
    assert_eq!(pausable.current_line(), 1); // Position unchanged
}

#[test]
fn test_pausable_stream_resume_enables_reads() {
    let content = "G28\nG0 X10\nG1 Z5\n";
    let reader = StringStreamReader::new(content);
    let mut pausable = PausableStream::new(Box::new(reader));

    pausable.read_line(); // G28
    pausable.pause();
    pausable.resume();

    let line = pausable.read_line();
    assert_eq!(line, Some("G0 X10".to_string()));
}

#[test]
fn test_pausable_stream_pause_line_tracking() {
    let content = "L0\nL1\nL2\nL3\n";
    let reader = StringStreamReader::new(content);
    let mut pausable = PausableStream::new(Box::new(reader));

    pausable.read_line(); // L0, now at 1
    pausable.read_line(); // L1, now at 2

    pausable.pause();
    assert_eq!(pausable.pause_line(), 2);
}

#[test]
fn test_pausable_stream_reset_clears_pause() {
    let content = "G28\nG0 X10\n";
    let reader = StringStreamReader::new(content);
    let mut pausable = PausableStream::new(Box::new(reader));

    pausable.pause();
    assert!(pausable.is_paused());

    pausable.reset().unwrap();
    assert!(!pausable.is_paused());
    assert_eq!(pausable.current_line(), 0);
}

#[test]
fn test_pausable_stream_total_lines() {
    let content = "L0\nL1\nL2\n";
    let reader = StringStreamReader::new(content);
    let pausable = PausableStream::new(Box::new(reader));

    assert_eq!(pausable.total_lines(), Some(3));
}

#[test]
fn test_empty_stream() {
    let content = "";
    let mut reader = StringStreamReader::new(content);

    assert_eq!(reader.total_lines(), Some(0));
    assert!(reader.is_eof());
    assert_eq!(reader.read_line(), None);
}

#[test]
fn test_single_line_stream() {
    let content = "G28";
    let mut reader = StringStreamReader::new(content);

    assert_eq!(reader.total_lines(), Some(1));
    assert_eq!(reader.read_line(), Some("G28".to_string()));
    assert!(reader.is_eof());
}
