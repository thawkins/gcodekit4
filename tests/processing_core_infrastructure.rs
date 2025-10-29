//! Tests for processing::core_infrastructure

use gcodekit4::processing::core_infrastructure::*;

#[test]
fn test_app_config() {
    let mut config = AppConfig::new();
    config.set("key", "value");
    assert_eq!(config.get("key"), Some("value"));
}

#[test]
fn test_logger() {
    let mut logger = Logger::new();
    logger.log(LogLevel::Info, "Test message");
    assert_eq!(logger.entries.len(), 1);
}

#[test]
fn test_unit_converter() {
    let mm = UnitConverter::to_metric(1.0, UnitSystem::Imperial);
    assert!((mm - 25.4).abs() < 0.01);
}

#[test]
fn test_cache() {
    let mut cache = Cache::new();
    cache.insert("key", "value");
    assert_eq!(cache.get("key"), Some(&"value"));
}

#[test]
fn test_app_state() {
    let state = ApplicationState::new();
    assert!(!state.is_running);
    assert!(!state.is_connected);
}
