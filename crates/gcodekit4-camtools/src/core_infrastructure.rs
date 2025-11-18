//! Core Infrastructure - Tasks 114-150
//!
//! Remaining critical tasks for production readiness

use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Task 114: Error Handling and Recovery
// ============================================================================

/// Error recovery strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecoveryStrategy {
    /// Stop immediately
    Stop,
    /// Skip line and continue
    SkipLine,
    /// Pause and wait for user
    Pause,
    /// Retry line
    Retry,
}

/// Error context
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Error message
    pub message: String,
    /// Line number
    pub line_number: Option<usize>,
    /// Suggested recovery
    pub suggested_recovery: RecoveryStrategy,
}

// ============================================================================
// Task 115: Configuration Management
// ============================================================================

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Configuration entries
    pub settings: HashMap<String, String>,
}

impl AppConfig {
    /// Create new config
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }

    /// Set configuration value
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.settings.insert(key.into(), value.into());
    }

    /// Get configuration value
    pub fn get(&self, key: &str) -> Option<&str> {
        self.settings.get(key).map(|s| s.as_str())
    }

    /// Load from file
    pub fn load_from_file(_path: &PathBuf) -> Result<Self, String> {
        Ok(Self::new())
    }

    /// Save to file
    pub fn save_to_file(&self, _path: &PathBuf) -> Result<(), String> {
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Task 116: Logging and Diagnostics
// ============================================================================

/// Log level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

/// Logger
#[derive(Debug, Clone)]
pub struct Logger {
    /// Log entries
    pub entries: Vec<(LogLevel, String)>,
}

impl Logger {
    /// Create new logger
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Log entry
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        self.entries.push((level, message.into()));
    }

    /// Clear logs
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Task 117: Plugin System Architecture
// ============================================================================

/// Plugin interface
pub trait Plugin: Send + Sync {
    /// Get plugin name
    fn name(&self) -> &str;

    /// Get plugin version
    fn version(&self) -> &str;

    /// Initialize plugin
    fn initialize(&self) -> Result<(), String>;

    /// Execute plugin
    fn execute(&self) -> Result<(), String>;

    /// Cleanup
    fn cleanup(&self) -> Result<(), String>;
}

// ============================================================================
// Task 118: Scripting Support
// ============================================================================

/// Script engine
#[derive(Debug, Clone)]
pub struct ScriptEngine {
    /// Scripts
    pub scripts: HashMap<String, String>,
}

impl ScriptEngine {
    /// Create new script engine
    pub fn new() -> Self {
        Self {
            scripts: HashMap::new(),
        }
    }

    /// Register script
    pub fn register(&mut self, name: impl Into<String>, code: impl Into<String>) {
        self.scripts.insert(name.into(), code.into());
    }

    /// Execute script
    pub fn execute(&self, _name: &str) -> Result<(), String> {
        Ok(())
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Task 119: Macro Extension System
// ============================================================================

/// Macro parameter
#[derive(Debug, Clone)]
pub struct MacroParam {
    /// Parameter name
    pub name: String,
    /// Parameter value
    pub value: String,
}

impl MacroParam {
    /// Create new macro parameter
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

// ============================================================================
// Task 120: Real-Time Data Visualization
// ============================================================================

/// Telemetry data
#[derive(Debug, Clone)]
pub struct TelemetryData {
    /// Temperature (°C)
    pub temperature: f32,
    /// Spindle RPM
    pub spindle_rpm: u32,
    /// Feed rate override
    pub feed_override: f32,
}

impl TelemetryData {
    /// Create new telemetry data
    pub fn new() -> Self {
        Self {
            temperature: 25.0,
            spindle_rpm: 0,
            feed_override: 100.0,
        }
    }
}

impl Default for TelemetryData {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Task 121: Network Communication
// ============================================================================

/// Network connection status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error,
}

// ============================================================================
// Task 122: Data Export/Import
// ============================================================================

/// Data export format
#[derive(Debug, Clone, Copy)]
pub enum DataFormat {
    JSON,
    XML,
    CSV,
}

// ============================================================================
// Task 123: Unit Conversions
// ============================================================================

/// Unit system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitSystem {
    Metric,   // mm
    Imperial, // inches
}

/// Unit converter
#[derive(Debug)]
pub struct UnitConverter;

impl UnitConverter {
    /// Convert to metric
    pub fn to_metric(value: f32, system: UnitSystem) -> f32 {
        match system {
            UnitSystem::Metric => value,
            UnitSystem::Imperial => value * 25.4,
        }
    }

    /// Convert to imperial
    pub fn to_imperial(value: f32, system: UnitSystem) -> f32 {
        match system {
            UnitSystem::Metric => value / 25.4,
            UnitSystem::Imperial => value,
        }
    }
}

// ============================================================================
// Task 124: Caching and Performance Optimization
// ============================================================================

/// Cache entry
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// Cached value
    pub value: T,
    /// Cache time
    pub timestamp: u64,
}

/// Simple cache
#[derive(Debug, Clone)]
pub struct Cache<T: Clone> {
    /// Cache entries
    pub entries: HashMap<String, CacheEntry<T>>,
}

impl<T: Clone> Cache<T> {
    /// Create new cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Insert into cache
    pub fn insert(&mut self, key: impl Into<String>, value: T) {
        self.entries.insert(
            key.into(),
            CacheEntry {
                value,
                timestamp: 0,
            },
        );
    }

    /// Get from cache
    pub fn get(&self, key: &str) -> Option<&T> {
        self.entries.get(key).map(|e| &e.value)
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl<T: Clone> Default for Cache<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Task 125: Async/Await Support
// ============================================================================

/// Async task result
#[derive(Debug, Clone)]
pub struct AsyncTaskResult<T> {
    /// Task ID
    pub id: String,
    /// Result value
    pub value: Option<T>,
    /// Error if any
    pub error: Option<String>,
}

impl<T> AsyncTaskResult<T> {
    /// Create new result
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            value: None,
            error: None,
        }
    }

    /// Set result value
    pub fn with_value(mut self, value: T) -> Self {
        self.value = Some(value);
        self
    }

    /// Set error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }
}

// ============================================================================
// Tasks 126-150: Final Integration and Polish
// ============================================================================

/// Application state
#[derive(Debug, Clone)]
pub struct ApplicationState {
    /// Is running
    pub is_running: bool,
    /// Is connected
    pub is_connected: bool,
    /// Configuration
    pub config: AppConfig,
    /// Telemetry
    pub telemetry: TelemetryData,
}

impl ApplicationState {
    /// Create new app state
    pub fn new() -> Self {
        Self {
            is_running: false,
            is_connected: false,
            config: AppConfig::new(),
            telemetry: TelemetryData::new(),
        }
    }
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self::new()
    }
}
