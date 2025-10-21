//! Configuration and settings management for GCodeKit4
//!
//! Provides configuration file handling, settings management, and validation.
//! Supports JSON and TOML file formats stored in platform-specific directories.
//!
//! Configuration is organized into logical sections:
//! - Connection settings (ports, timeouts, protocols)
//! - UI preferences (theme, layout, fonts)
//! - File processing defaults (preprocessors, arc settings)
//! - Machine preferences (limits, jog settings)
//! - Firmware-specific settings

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Connection protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    /// Serial/USB connection
    Serial,
    /// TCP/IP connection
    Tcp,
    /// WebSocket connection
    WebSocket,
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serial => write!(f, "serial"),
            Self::Tcp => write!(f, "tcp"),
            Self::WebSocket => write!(f, "websocket"),
        }
    }
}

/// Connection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSettings {
    /// Last used connection type
    pub connection_type: ConnectionType,
    /// Last used port (serial) or hostname (TCP/WebSocket)
    pub port: String,
    /// Baud rate for serial connections
    pub baud_rate: u32,
    /// TCP port for network connections
    pub tcp_port: u16,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Auto-reconnect on connection loss
    pub auto_reconnect: bool,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            connection_type: ConnectionType::Serial,
            port: "/dev/ttyUSB0".to_string(),
            baud_rate: 115200,
            tcp_port: 8888,
            timeout_ms: 5000,
            auto_reconnect: true,
        }
    }
}

/// UI preference settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    /// Window width
    pub window_width: u32,
    /// Window height
    pub window_height: u32,
    /// Whether panels are visible (by name)
    pub panel_visibility: HashMap<String, bool>,
    /// Selected theme (light/dark)
    pub theme: String,
    /// Font size in points
    pub font_size: u8,
    /// UI language code (e.g., "en", "es", "fr")
    pub language: String,
}

impl Default for UiSettings {
    fn default() -> Self {
        let mut visibility = HashMap::new();
        visibility.insert("connection".to_string(), true);
        visibility.insert("dro".to_string(), true);
        visibility.insert("jog".to_string(), true);
        visibility.insert("console".to_string(), true);

        Self {
            window_width: 1400,
            window_height: 900,
            panel_visibility: visibility,
            theme: "dark".to_string(),
            font_size: 12,
            language: "en".to_string(),
        }
    }
}

/// File processing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileProcessingSettings {
    /// Default preprocessors to enable
    pub enabled_preprocessors: Vec<String>,
    /// Arc segment length in mm
    pub arc_segment_length: f64,
    /// Maximum line length in characters
    pub max_line_length: u32,
    /// Whether to preserve comments
    pub preserve_comments: bool,
    /// Default output directory
    pub output_directory: PathBuf,
    /// Number of recent files to track
    pub recent_files_count: usize,
}

impl Default for FileProcessingSettings {
    fn default() -> Self {
        Self {
            enabled_preprocessors: vec![
                "comment_remover".to_string(),
                "whitespace_cleaner".to_string(),
            ],
            arc_segment_length: 0.5,
            max_line_length: 256,
            preserve_comments: false,
            output_directory: PathBuf::from("."),
            recent_files_count: 10,
        }
    }
}

/// Machine preference settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineSettings {
    /// Default jog increment in mm
    pub jog_increment: f64,
    /// Default jog feed rate in units/min
    pub jog_feed_rate: f64,
    /// Machine X limit (max)
    pub x_limit: f64,
    /// Machine Y limit (max)
    pub y_limit: f64,
    /// Machine Z limit (max)
    pub z_limit: f64,
    /// Default unit (mm or in)
    pub default_unit: String,
    /// Homing direction per axis (true = negative, false = positive)
    pub homing_direction: HashMap<String, bool>,
}

impl Default for MachineSettings {
    fn default() -> Self {
        let mut homing = HashMap::new();
        homing.insert("X".to_string(), true);
        homing.insert("Y".to_string(), true);
        homing.insert("Z".to_string(), true);

        Self {
            jog_increment: 1.0,
            jog_feed_rate: 1000.0,
            x_limit: 200.0,
            y_limit: 200.0,
            z_limit: 100.0,
            default_unit: "mm".to_string(),
            homing_direction: homing,
        }
    }
}

/// Firmware-specific settings trait
///
/// Allows firmware implementations to define their own settings structure.
pub trait FirmwareSettings: Serialize {
    /// Get the firmware name
    fn firmware_name(&self) -> &str;

    /// Validate the settings
    fn validate(&self) -> Result<()>;

    /// Get setting by key
    fn get_setting(&self, key: &str) -> Option<String>;

    /// Set setting by key
    fn set_setting(&mut self, key: &str, value: String) -> Result<()>;
}

/// Complete application configuration
///
/// Aggregates all settings sections and provides file I/O operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Connection settings
    pub connection: ConnectionSettings,
    /// UI preferences
    pub ui: UiSettings,
    /// File processing settings
    pub file_processing: FileProcessingSettings,
    /// Machine preferences
    pub machine: MachineSettings,
    /// Recent files list
    pub recent_files: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            connection: ConnectionSettings::default(),
            ui: UiSettings::default(),
            file_processing: FileProcessingSettings::default(),
            machine: MachineSettings::default(),
            recent_files: Vec::new(),
        }
    }
}

impl Config {
    /// Create new config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load config from file (JSON or TOML)
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::other(format!("Failed to read config file: {}", e)))?;

        let config: Self = if path.extension().map_or(false, |ext| ext == "json") {
            serde_json::from_str(&content)
                .map_err(|e| Error::other(format!("Invalid JSON config: {}", e)))?
        } else if path.extension().map_or(false, |ext| ext == "toml") {
            toml::from_str(&content)
                .map_err(|e| Error::other(format!("Invalid TOML config: {}", e)))?
        } else {
            return Err(Error::other(
                "Config file must be .json or .toml".to_string(),
            ));
        };

        config.validate()?;
        Ok(config)
    }

    /// Save config to file (JSON or TOML)
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        self.validate()?;

        let content = if path.extension().map_or(false, |ext| ext == "json") {
            serde_json::to_string_pretty(self)
                .map_err(|e| Error::other(format!("Failed to serialize config: {}", e)))?
        } else if path.extension().map_or(false, |ext| ext == "toml") {
            toml::to_string_pretty(self)
                .map_err(|e| Error::other(format!("Failed to serialize config: {}", e)))?
        } else {
            return Err(Error::other(
                "Config file must be .json or .toml".to_string(),
            ));
        };

        std::fs::write(path, content)
            .map_err(|e| Error::other(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate connection settings
        if self.connection.timeout_ms == 0 {
            return Err(Error::other("Connection timeout must be > 0".to_string()));
        }

        if self.connection.baud_rate == 0 {
            return Err(Error::other("Baud rate must be > 0".to_string()));
        }

        // Validate UI settings
        if self.ui.window_width == 0 || self.ui.window_height == 0 {
            return Err(Error::other("Window dimensions must be > 0".to_string()));
        }

        if self.ui.font_size == 0 {
            return Err(Error::other("Font size must be > 0".to_string()));
        }

        // Validate file processing
        if self.file_processing.arc_segment_length <= 0.0 {
            return Err(Error::other("Arc segment length must be > 0".to_string()));
        }

        if self.file_processing.max_line_length == 0 {
            return Err(Error::other("Max line length must be > 0".to_string()));
        }

        // Validate machine settings
        if self.machine.jog_feed_rate <= 0.0 {
            return Err(Error::other("Jog feed rate must be > 0".to_string()));
        }

        if self.machine.x_limit <= 0.0 || self.machine.y_limit <= 0.0 || self.machine.z_limit <= 0.0
        {
            return Err(Error::other("Machine limits must be > 0".to_string()));
        }

        Ok(())
    }

    /// Add file to recent files list
    pub fn add_recent_file(&mut self, path: PathBuf) {
        // Remove if already in list
        self.recent_files.retain(|f| f != &path);

        // Add to front
        self.recent_files.insert(0, path);

        // Trim to max size
        self.recent_files
            .truncate(self.file_processing.recent_files_count);
    }

    /// Merge another config into this one (preserves existing values for missing sections)
    pub fn merge(&mut self, other: &Config) {
        // Only merge non-zero/non-default values
        if other.connection.timeout_ms > 0 {
            self.connection = other.connection.clone();
        }
        if !other.ui.theme.is_empty() {
            self.ui = other.ui.clone();
        }
        if other.file_processing.arc_segment_length > 0.0 {
            self.file_processing = other.file_processing.clone();
        }
        if other.machine.jog_feed_rate > 0.0 {
            self.machine = other.machine.clone();
        }
    }
}

/// Settings manager for different firmware types
///
/// Provides default settings for each supported firmware and manages configuration persistence.
pub struct SettingsManager {
    config: Config,
    #[allow(dead_code)]
    firmware_settings: HashMap<String, Box<dyn std::any::Any>>,
}

impl SettingsManager {
    /// Create new settings manager with default config
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            firmware_settings: HashMap::new(),
        }
    }

    /// Create settings manager with loaded config
    pub fn with_config(config: Config) -> Self {
        Self {
            config,
            firmware_settings: HashMap::new(),
        }
    }

    /// Load config from file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let config = Config::load_from_file(path)?;
        Ok(Self::with_config(config))
    }

    /// Get current config
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get mutable config
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Save config to file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        self.config.save_to_file(path)
    }

    /// Get default settings for GRBL firmware
    pub fn default_grbl_settings() -> Config {
        let mut config = Config::default();
        config.connection.baud_rate = 115200;
        config.connection.timeout_ms = 5000;
        config.machine.x_limit = 200.0;
        config.machine.y_limit = 200.0;
        config.machine.z_limit = 100.0;
        config
    }

    /// Get default settings for TinyG firmware
    pub fn default_tinyg_settings() -> Config {
        let mut config = Config::default();
        config.connection.baud_rate = 115200;
        config.connection.timeout_ms = 5000;
        config.machine.x_limit = 250.0;
        config.machine.y_limit = 250.0;
        config.machine.z_limit = 150.0;
        config
    }

    /// Get default settings for g2core firmware
    pub fn default_g2core_settings() -> Config {
        let mut config = Config::default();
        config.connection.connection_type = ConnectionType::Tcp;
        config.connection.timeout_ms = 10000;
        config.machine.x_limit = 300.0;
        config.machine.y_limit = 300.0;
        config.machine.z_limit = 200.0;
        config
    }

    /// Get default settings for Smoothieware firmware
    pub fn default_smoothieware_settings() -> Config {
        let mut config = Config::default();
        config.connection.baud_rate = 115200;
        config.connection.timeout_ms = 5000;
        config.machine.x_limit = 200.0;
        config.machine.y_limit = 200.0;
        config.machine.z_limit = 100.0;
        config
    }

    /// Get default settings for FluidNC firmware
    pub fn default_fluidnc_settings() -> Config {
        let mut config = Config::default();
        config.connection.connection_type = ConnectionType::WebSocket;
        config.connection.timeout_ms = 10000;
        config.machine.x_limit = 300.0;
        config.machine.y_limit = 300.0;
        config.machine.z_limit = 200.0;
        config
    }

    /// Get platform-specific config directory
    pub fn config_directory() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA")
                .map_err(|_| Error::other("APPDATA environment variable not set".to_string()))?;
            Ok(PathBuf::from(appdata).join("gcodekit4"))
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| Error::other("HOME environment variable not set".to_string()))?;
            Ok(PathBuf::from(home).join("Library/Application Support/gcodekit4"))
        }

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| Error::other("HOME environment variable not set".to_string()))?;
            Ok(PathBuf::from(home).join(".config/gcodekit4"))
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(Error::other("Unsupported platform".to_string()))
        }
    }

    /// Get config file path for platform
    pub fn config_file_path() -> Result<PathBuf> {
        let dir = Self::config_directory()?;
        Ok(dir.join("config.json"))
    }

    /// Ensure config directory exists
    pub fn ensure_config_dir() -> Result<PathBuf> {
        let dir = Self::config_directory()?;
        std::fs::create_dir_all(&dir)
            .map_err(|e| Error::other(format!("Failed to create config directory: {}", e)))?;
        Ok(dir)
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}
