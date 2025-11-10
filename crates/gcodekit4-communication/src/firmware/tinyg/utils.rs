//! TinyG Utilities - JSON Parsing and Formatting Helpers
//!
//! This module provides utility functions for parsing and formatting
//! TinyG JSON responses and commands.

use serde_json::{json, Value};

/// Parse a TinyG JSON response into a Value
pub fn parse_json_response(response: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str(response)
}

/// Extract a status report from a TinyG JSON response
pub fn extract_status_report(json: &Value) -> Option<Value> {
    json.get("sr")?.as_object()?;
    Some(json["sr"].clone())
}

/// Extract position values from a status report
pub fn extract_position(sr: &Value) -> Option<(f64, f64, f64, f64)> {
    let pos = sr.get("pos")?.as_object()?;
    let x = pos.get("x")?.as_f64()?;
    let y = pos.get("y")?.as_f64()?;
    let z = pos.get("z")?.as_f64()?;
    let a = pos.get("a")?.as_f64().unwrap_or(0.0);
    Some((x, y, z, a))
}

/// Extract machine position from a status report
pub fn extract_machine_position(sr: &Value) -> Option<(f64, f64, f64, f64)> {
    let mpos = sr.get("mpos")?.as_object()?;
    let x = mpos.get("x")?.as_f64()?;
    let y = mpos.get("y")?.as_f64()?;
    let z = mpos.get("z")?.as_f64()?;
    let a = mpos.get("a")?.as_f64().unwrap_or(0.0);
    Some((x, y, z, a))
}

/// Extract work position from a status report
pub fn extract_work_position(sr: &Value) -> Option<(f64, f64, f64, f64)> {
    let wpos = sr.get("wpos")?.as_object()?;
    let x = wpos.get("x")?.as_f64()?;
    let y = wpos.get("y")?.as_f64()?;
    let z = wpos.get("z")?.as_f64()?;
    let a = wpos.get("a")?.as_f64().unwrap_or(0.0);
    Some((x, y, z, a))
}

/// Extract state from a status report
pub fn extract_state(sr: &Value) -> Option<String> {
    sr.get("stat")?
        .as_object()?
        .get("state")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
}

/// Extract feed rate from a status report
pub fn extract_feed_rate(sr: &Value) -> Option<f64> {
    sr.get("feed")?.as_f64()
}

/// Extract spindle speed from a status report
pub fn extract_spindle_speed(sr: &Value) -> Option<f64> {
    sr.get("speed")?.as_f64()
}

/// Extract line number from a response
pub fn extract_line_number(json: &Value) -> Option<u32> {
    json.get("n")?.as_u64().map(|n| n as u32)
}

/// Extract error code from a response
pub fn extract_error_code(json: &Value) -> Option<u16> {
    json.get("er")?
        .as_object()?
        .get("code")
        .and_then(Value::as_u64)
        .map(|c| c as u16)
}

/// Extract error message from a response
pub fn extract_error_message(json: &Value) -> Option<String> {
    json.get("er")?
        .as_object()?
        .get("msg")
        .and_then(Value::as_str)
        .map(|s| s.to_string())
}

/// Create a JSON command for TinyG
pub fn create_json_command(key: &str, value: Value) -> Value {
    json!({key: value})
}

/// Create a query command for TinyG
pub fn create_query_command(key: &str) -> Value {
    json!({key: Value::Null})
}

/// Create a status report request
pub fn create_status_request() -> String {
    "{\"sr\":null}".to_string()
}

/// Create a position setting command
pub fn create_position_set(
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    a: Option<f64>,
) -> Value {
    let mut pos = serde_json::Map::new();
    if let Some(x_val) = x {
        pos.insert("x".to_string(), json!(x_val));
    }
    if let Some(y_val) = y {
        pos.insert("y".to_string(), json!(y_val));
    }
    if let Some(z_val) = z {
        pos.insert("z".to_string(), json!(z_val));
    }
    if let Some(a_val) = a {
        pos.insert("a".to_string(), json!(a_val));
    }
    json!({"xpo": pos})
}

/// Format a value for TinyG JSON
pub fn format_tinyg_value(value: f64, precision: usize) -> String {
    format!("{:.prec$}", value, prec = precision)
}

/// Parse a TinyG coordinate offset value
pub fn parse_offset_value(offset_str: &str) -> Option<f64> {
    offset_str.parse().ok()
}
