//! G-Code parser and state machine
//!
//! This module provides:
//! - G-Code command parsing
//! - Modal state tracking
//! - Command validation
//! - Preprocessor framework
//! - Command lifecycle management
//! - Command listener framework
//! - Stream management (reading from files or strings)

pub mod stream;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use uuid::Uuid;

/// Unique identifier for a G-Code command
pub type CommandId = String;

/// Command execution state
///
/// Represents the lifecycle state of a G-Code command from creation through completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandState {
    /// Command created but not yet sent
    Pending,
    /// Command sent to controller, awaiting response
    Sent,
    /// Controller acknowledged command with "ok"
    Ok,
    /// Command execution completed
    Done,
    /// Command generated an error response
    Error,
    /// Command was skipped (not sent)
    Skipped,
}

impl std::fmt::Display for CommandState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Sent => write!(f, "Sent"),
            Self::Ok => write!(f, "Ok"),
            Self::Done => write!(f, "Done"),
            Self::Error => write!(f, "Error"),
            Self::Skipped => write!(f, "Skipped"),
        }
    }
}

/// Response from the controller for a command
///
/// Captures the controller's response to a sent command,
/// including any error messages or status information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandResponse {
    /// Whether the command was accepted/acknowledged
    pub success: bool,
    /// Response message from controller (e.g., "ok", error message)
    pub message: String,
    /// Error code if applicable
    pub error_code: Option<u32>,
    /// Additional response data
    pub data: Option<String>,
}

/// Represents a parsed and tracked G-Code command
///
/// Comprehensive representation of a G-Code command including:
/// - Command text and metadata
/// - Execution state tracking
/// - ID generation for tracking
/// - Response handling
/// - Timestamp information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcodeCommand {
    /// Unique identifier for this command
    pub id: CommandId,
    /// G-Code line (e.g., "G00 X10.5 Y20.3 Z0.0")
    pub line: String,
    /// Line number if present in file
    pub line_number: Option<u32>,
    /// Raw command text (parsed)
    pub command: String,
    /// Command execution state
    pub state: CommandState,
    /// Command numbering (0-based, sequential)
    pub sequence_number: u32,
    /// Response from controller
    pub response: Option<CommandResponse>,
    /// Timestamp when command was created (milliseconds)
    pub created_at: u64,
    /// Timestamp when command was sent (milliseconds)
    pub sent_at: Option<u64>,
    /// Timestamp when command completed (milliseconds)
    pub completed_at: Option<u64>,
}

impl GcodeCommand {
    /// Create a new G-Code command with auto-generated ID
    pub fn new(line: impl Into<String>) -> Self {
        let line = line.into();
        Self {
            id: CommandId::from(Uuid::new_v4().to_string()),
            command: line.clone(),
            line,
            line_number: None,
            state: CommandState::Pending,
            sequence_number: 0,
            response: None,
            created_at: Self::current_timestamp(),
            sent_at: None,
            completed_at: None,
        }
    }

    /// Create a new G-Code command with sequence number
    pub fn with_sequence(line: impl Into<String>, sequence: u32) -> Self {
        let mut cmd = Self::new(line);
        cmd.sequence_number = sequence;
        cmd
    }

    /// Create a new G-Code command with explicit ID
    pub fn with_id(line: impl Into<String>, id: CommandId) -> Self {
        let mut cmd = Self::new(line);
        cmd.id = id;
        cmd
    }

    /// Set the line number for this command
    pub fn set_line_number(&mut self, line_number: u32) -> &mut Self {
        self.line_number = Some(line_number);
        self
    }

    /// Mark this command as sent
    pub fn mark_sent(&mut self) -> &mut Self {
        self.state = CommandState::Sent;
        self.sent_at = Some(Self::current_timestamp());
        self
    }

    /// Mark this command as successfully executed (received "ok")
    pub fn mark_ok(&mut self) -> &mut Self {
        self.state = CommandState::Ok;
        if self.completed_at.is_none() {
            self.completed_at = Some(Self::current_timestamp());
        }
        self
    }

    /// Mark this command as completed
    pub fn mark_done(&mut self) -> &mut Self {
        self.state = CommandState::Done;
        if self.completed_at.is_none() {
            self.completed_at = Some(Self::current_timestamp());
        }
        self
    }

    /// Mark this command with an error
    pub fn mark_error(&mut self, error_code: Option<u32>, message: String) -> &mut Self {
        self.state = CommandState::Error;
        self.completed_at = Some(Self::current_timestamp());
        self.response = Some(CommandResponse {
            success: false,
            message,
            error_code,
            data: None,
        });
        self
    }

    /// Mark this command as skipped
    pub fn mark_skipped(&mut self) -> &mut Self {
        self.state = CommandState::Skipped;
        self.completed_at = Some(Self::current_timestamp());
        self
    }

    /// Set the response for this command
    pub fn set_response(&mut self, response: CommandResponse) -> &mut Self {
        self.response = Some(response);
        self
    }

    /// Check if command is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            CommandState::Done | CommandState::Error | CommandState::Skipped
        )
    }

    /// Check if command has been sent
    pub fn is_sent(&self) -> bool {
        self.sent_at.is_some()
    }

    /// Get duration from creation to completion (milliseconds)
    pub fn total_duration(&self) -> Option<u64> {
        self.completed_at
            .map(|completed| completed - self.created_at)
    }

    /// Get duration from sent to completion (milliseconds)
    pub fn execution_duration(&self) -> Option<u64> {
        self.sent_at
            .and_then(|sent| self.completed_at.map(|completed| completed - sent))
    }

    /// Get current timestamp in milliseconds
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

impl Default for GcodeCommand {
    fn default() -> Self {
        Self::new("")
    }
}

impl std::fmt::Display for GcodeCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.id, self.state, self.command)
    }
}

/// Trait for listening to command lifecycle events
///
/// Implementations can react to various stages of command execution:
/// - Creation
/// - Sending
/// - Success/failure
/// - Completion
/// - Error conditions
pub trait CommandListener: Send + Sync {
    /// Called when a command is created
    fn on_command_created(&self, command: &GcodeCommand);

    /// Called when a command is sent to the controller
    fn on_command_sent(&self, command: &GcodeCommand);

    /// Called when a command receives an "ok" response
    fn on_command_ok(&self, command: &GcodeCommand);

    /// Called when a command completes execution
    fn on_command_completed(&self, command: &GcodeCommand);

    /// Called when a command encounters an error
    fn on_command_error(&self, command: &GcodeCommand, error: &CommandResponse);

    /// Called when a command is skipped
    fn on_command_skipped(&self, command: &GcodeCommand);

    /// Called when a command state changes
    fn on_command_state_changed(&self, command: &GcodeCommand, old_state: CommandState);
}

/// Default no-op command listener implementation
pub struct NoOpCommandListener;

impl CommandListener for NoOpCommandListener {
    fn on_command_created(&self, _command: &GcodeCommand) {}
    fn on_command_sent(&self, _command: &GcodeCommand) {}
    fn on_command_ok(&self, _command: &GcodeCommand) {}
    fn on_command_completed(&self, _command: &GcodeCommand) {}
    fn on_command_error(&self, _command: &GcodeCommand, _error: &CommandResponse) {}
    fn on_command_skipped(&self, _command: &GcodeCommand) {}
    fn on_command_state_changed(&self, _command: &GcodeCommand, _old_state: CommandState) {}
}

/// Arc-wrapped command listener for thread-safe sharing
pub type CommandListenerHandle = Arc<dyn CommandListener>;

/// Command numbering generator for sequential tracking
pub struct CommandNumberGenerator {
    counter: Arc<AtomicU32>,
}

impl CommandNumberGenerator {
    /// Create a new command number generator
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Get the next command number
    pub fn next(&self) -> u32 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Get current command count without incrementing
    pub fn current(&self) -> u32 {
        self.counter.load(Ordering::SeqCst)
    }

    /// Reset the counter
    pub fn reset(&self) {
        self.counter.store(0, Ordering::SeqCst);
    }
}

impl Default for CommandNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CommandNumberGenerator {
    fn clone(&self) -> Self {
        Self {
            counter: Arc::clone(&self.counter),
        }
    }
}

/// G-Code parser with modal state tracking
pub struct GcodeParser {
    current_state: GcodeState,
    command_generator: CommandNumberGenerator,
}

/// Modal state for G-Code execution
///
/// Tracks the active modal groups during G-Code execution.
/// Modal groups are persistent states that affect all subsequent commands
/// until changed by another command in the same group.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModalState {
    /// Motion mode (G00=rapid, G01=linear, G02=arc_cw, G03=arc_ccw)
    pub motion_mode: u8,
    /// Plane selection (G17=XY, G18=XZ, G19=YZ)
    pub plane: u8,
    /// Distance mode (G90=absolute, G91=incremental)
    pub distance_mode: u8,
    /// Feed rate mode (G93=inverse_time, G94=units_per_minute, G95=units_per_revolution)
    pub feed_rate_mode: u8,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            motion_mode: 0,     // G00
            plane: 17,          // G17 (XY plane)
            distance_mode: 90,  // G90 (absolute)
            feed_rate_mode: 94, // G94 (units per minute)
        }
    }
}

/// Comprehensive G-Code execution state
///
/// Tracks all modal groups and execution state required for proper G-Code interpretation:
/// - Motion group (G00, G01, G02, G03)
/// - Plane selection group (G17, G18, G19)
/// - Distance mode group (G90, G91)
/// - Feed rate mode group (G93, G94, G95)
/// - Units group (G20, G21)
/// - Coordinate system group (G54-G59)
/// - Tool offset group (G43, G49)
/// - Cutter compensation group (G40, G41, G42)
/// - Spindle mode group (G03, G04, G05)
/// - Path control group (G61, G61.1, G64)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GcodeState {
    /// Motion mode - Group 1 (G00, G01, G02, G03)
    pub motion_mode: u8,

    /// Plane selection - Group 2 (G17=XY, G18=XZ, G19=YZ)
    pub plane_mode: u8,

    /// Distance mode - Group 3 (G90=absolute, G91=incremental)
    pub distance_mode: u8,

    /// Feed rate mode - Group 5 (G93=inverse_time, G94=units_per_minute, G95=units_per_revolution)
    pub feed_rate_mode: u8,

    /// Units mode - Group 6 (G20=inches, G21=millimeters)
    pub units_mode: u8,

    /// Coordinate system - Group 12 (G54-G59)
    pub coordinate_system: u8,

    /// Tool offset mode - Group 8 (G43=enable, G49=disable)
    pub tool_offset_mode: u8,

    /// Cutter compensation - Group 7 (G40=off, G41=left, G42=right)
    pub compensation_mode: u8,

    /// Spindle mode - Group 3 (G03=spindle_sync, G04=CSS, G05=SFM)
    pub spindle_mode: u8,

    /// Path control - Group 17 (G61=exact, G61.1=exact_stop, G64=blend)
    pub path_control_mode: u8,

    /// Current feed rate (F value)
    pub feed_rate: f64,

    /// Current spindle speed (S value)
    pub spindle_speed: f64,

    /// Tool number (T value)
    pub tool_number: u16,
}

impl Default for GcodeState {
    fn default() -> Self {
        Self {
            motion_mode: 0,        // G00 (rapid)
            plane_mode: 17,        // G17 (XY plane)
            distance_mode: 90,     // G90 (absolute)
            feed_rate_mode: 94,    // G94 (units per minute)
            units_mode: 21,        // G21 (millimeters)
            coordinate_system: 54, // G54 (first WCS)
            tool_offset_mode: 49,  // G49 (offset disabled)
            compensation_mode: 40, // G40 (cutter compensation off)
            spindle_mode: 0,       // No spindle mode
            path_control_mode: 64, // G64 (blend/continuous)
            feed_rate: 0.0,
            spindle_speed: 0.0,
            tool_number: 0,
        }
    }
}

impl GcodeState {
    /// Create a new G-Code state with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set motion mode (G00, G01, G02, G03)
    pub fn set_motion_mode(&mut self, mode: u8) -> Result<(), String> {
        match mode {
            0..=3 => {
                self.motion_mode = mode;
                Ok(())
            }
            _ => Err(format!("Invalid motion mode: {}", mode)),
        }
    }

    /// Set plane mode (G17, G18, G19)
    pub fn set_plane_mode(&mut self, mode: u8) -> Result<(), String> {
        match mode {
            17..=19 => {
                self.plane_mode = mode;
                Ok(())
            }
            _ => Err(format!("Invalid plane mode: {}", mode)),
        }
    }

    /// Set distance mode (G90, G91)
    pub fn set_distance_mode(&mut self, mode: u8) -> Result<(), String> {
        match mode {
            90 | 91 => {
                self.distance_mode = mode;
                Ok(())
            }
            _ => Err(format!("Invalid distance mode: {}", mode)),
        }
    }

    /// Set feed rate mode (G93, G94, G95)
    pub fn set_feed_rate_mode(&mut self, mode: u8) -> Result<(), String> {
        match mode {
            93..=95 => {
                self.feed_rate_mode = mode;
                Ok(())
            }
            _ => Err(format!("Invalid feed rate mode: {}", mode)),
        }
    }

    /// Set units mode (G20 for inches, G21 for mm)
    pub fn set_units_mode(&mut self, mode: u8) -> Result<(), String> {
        match mode {
            20 | 21 => {
                self.units_mode = mode;
                Ok(())
            }
            _ => Err(format!("Invalid units mode: {}", mode)),
        }
    }

    /// Set coordinate system (G54-G59)
    pub fn set_coordinate_system(&mut self, system: u8) -> Result<(), String> {
        match system {
            54..=59 => {
                self.coordinate_system = system;
                Ok(())
            }
            _ => Err(format!("Invalid coordinate system: {}", system)),
        }
    }

    /// Set tool offset mode (G43 enable, G49 disable)
    pub fn set_tool_offset_mode(&mut self, mode: u8) -> Result<(), String> {
        match mode {
            43 | 49 => {
                self.tool_offset_mode = mode;
                Ok(())
            }
            _ => Err(format!("Invalid tool offset mode: {}", mode)),
        }
    }

    /// Set cutter compensation mode (G40 off, G41 left, G42 right)
    pub fn set_compensation_mode(&mut self, mode: u8) -> Result<(), String> {
        match mode {
            40..=42 => {
                self.compensation_mode = mode;
                Ok(())
            }
            _ => Err(format!("Invalid compensation mode: {}", mode)),
        }
    }

    /// Set feed rate value
    pub fn set_feed_rate(&mut self, rate: f64) -> Result<(), String> {
        if rate < 0.0 {
            return Err("Feed rate cannot be negative".to_string());
        }
        self.feed_rate = rate;
        Ok(())
    }

    /// Set spindle speed value
    pub fn set_spindle_speed(&mut self, speed: f64) -> Result<(), String> {
        if speed < 0.0 {
            return Err("Spindle speed cannot be negative".to_string());
        }
        self.spindle_speed = speed;
        Ok(())
    }

    /// Set tool number
    pub fn set_tool_number(&mut self, tool: u16) {
        self.tool_number = tool;
    }

    /// Check if state is valid
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.motion_mode, 0..=3) {
            return Err(format!("Invalid motion mode: {}", self.motion_mode));
        }
        if !matches!(self.plane_mode, 17..=19) {
            return Err(format!("Invalid plane mode: {}", self.plane_mode));
        }
        if !matches!(self.distance_mode, 90 | 91) {
            return Err(format!("Invalid distance mode: {}", self.distance_mode));
        }
        if !matches!(self.feed_rate_mode, 93..=95) {
            return Err(format!("Invalid feed rate mode: {}", self.feed_rate_mode));
        }
        if !matches!(self.units_mode, 20 | 21) {
            return Err(format!("Invalid units mode: {}", self.units_mode));
        }
        if !matches!(self.coordinate_system, 54..=59) {
            return Err(format!(
                "Invalid coordinate system: {}",
                self.coordinate_system
            ));
        }
        Ok(())
    }

    /// Get a human-readable description of the current motion mode
    pub fn motion_mode_description(&self) -> &'static str {
        match self.motion_mode {
            0 => "Rapid positioning (G00)",
            1 => "Linear interpolation (G01)",
            2 => "Clockwise arc (G02)",
            3 => "Counter-clockwise arc (G03)",
            _ => "Unknown motion mode",
        }
    }

    /// Get a human-readable description of the current plane
    pub fn plane_description(&self) -> &'static str {
        match self.plane_mode {
            17 => "XY plane (G17)",
            18 => "XZ plane (G18)",
            19 => "YZ plane (G19)",
            _ => "Unknown plane",
        }
    }

    /// Get a human-readable description of distance mode
    pub fn distance_mode_description(&self) -> &'static str {
        match self.distance_mode {
            90 => "Absolute positioning (G90)",
            91 => "Incremental positioning (G91)",
            _ => "Unknown distance mode",
        }
    }

    /// Get a human-readable description of units
    pub fn units_description(&self) -> &'static str {
        match self.units_mode {
            20 => "Inches (G20)",
            21 => "Millimeters (G21)",
            _ => "Unknown units",
        }
    }
}

impl GcodeParser {
    /// Create a new G-Code parser
    pub fn new() -> Self {
        Self {
            current_state: GcodeState::default(),
            command_generator: CommandNumberGenerator::new(),
        }
    }
}

impl Default for GcodeParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration options for command processors
///
/// Provides customizable settings for different preprocessor implementations.
/// Can be extended by specific processor implementations for their unique needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// Whether this processor is enabled
    pub enabled: bool,
    /// Processor-specific configuration data (JSON-like)
    pub options: std::collections::HashMap<String, String>,
}

impl ProcessorConfig {
    /// Create a new processor configuration
    pub fn new() -> Self {
        Self {
            enabled: true,
            options: std::collections::HashMap::new(),
        }
    }

    /// Create a disabled processor configuration
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            options: std::collections::HashMap::new(),
        }
    }

    /// Set a configuration option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// Get a configuration option
    pub fn get_option(&self, key: &str) -> Option<&str> {
        self.options.get(key).map(|s| s.as_str())
    }
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for G-Code command processors
///
/// Processors implement transformations, validations, and modifications
/// to G-Code commands. They are applied in a pipeline to process commands
/// before execution.
///
/// # Examples
/// - Comment removal
/// - Whitespace normalization
/// - Arc expansion to line segments
/// - Command optimization
/// - Feed rate overrides
pub trait CommandProcessor: Send + Sync {
    /// Get the name/identifier of this processor
    fn name(&self) -> &str;

    /// Get a description of what this processor does
    fn description(&self) -> &str;

    /// Process a single G-Code command
    ///
    /// # Arguments
    /// * `command` - The G-Code command to process
    /// * `state` - Current G-Code state (modal state)
    ///
    /// # Returns
    /// A vector of processed commands. Most processors return a single command,
    /// but some (like arc expanders) may expand one command into many.
    /// Return an empty vector to skip the command.
    fn process(
        &self,
        command: &GcodeCommand,
        state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String>;

    /// Check if this processor is enabled
    fn is_enabled(&self) -> bool {
        true
    }

    /// Get the configuration for this processor
    fn config(&self) -> &ProcessorConfig {
        static DEFAULT_CONFIG: std::sync::OnceLock<ProcessorConfig> = std::sync::OnceLock::new();
        DEFAULT_CONFIG.get_or_init(ProcessorConfig::new)
    }
}

/// Arc-wrapped processor for thread-safe sharing
pub type ProcessorHandle = Arc<dyn CommandProcessor>;

/// G-Code command processor pipeline
///
/// Manages a sequence of command processors that are applied to G-Code commands
/// in order. Each processor can transform the command, skip it, or expand it
/// into multiple commands.
///
/// # Example
/// ```ignore
/// let mut pipeline = ProcessorPipeline::new();
/// pipeline.register(Arc::new(WhitespaceProcessor::new()));
/// pipeline.register(Arc::new(CommentProcessor::new()));
/// pipeline.register(Arc::new(ArcExpander::new()));
///
/// let commands = pipeline.process_commands(&input_commands)?;
/// ```
pub struct ProcessorPipeline {
    processors: Vec<ProcessorHandle>,
    config: ProcessorConfig,
}

impl ProcessorPipeline {
    /// Create a new empty processor pipeline
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            config: ProcessorConfig::new(),
        }
    }

    /// Register a processor in the pipeline
    ///
    /// Processors are applied in the order they are registered.
    pub fn register(&mut self, processor: ProcessorHandle) -> &mut Self {
        self.processors.push(processor);
        self
    }

    /// Register multiple processors at once
    pub fn register_all(&mut self, processors: Vec<ProcessorHandle>) -> &mut Self {
        self.processors.extend(processors);
        self
    }

    /// Get the number of registered processors
    pub fn processor_count(&self) -> usize {
        self.processors.len()
    }

    /// Get a reference to a processor by index
    pub fn get_processor(&self, index: usize) -> Option<&ProcessorHandle> {
        self.processors.get(index)
    }

    /// Get a reference to a processor by name
    pub fn get_processor_by_name(&self, name: &str) -> Option<&ProcessorHandle> {
        self.processors.iter().find(|p| p.name() == name)
    }

    /// List all registered processors
    pub fn list_processors(&self) -> Vec<(&str, &str, bool)> {
        self.processors
            .iter()
            .map(|p| (p.name(), p.description(), p.is_enabled()))
            .collect()
    }

    /// Process a single command through the entire pipeline
    ///
    /// Returns a vector of commands. Most processors return one command,
    /// but some may expand or skip commands.
    pub fn process_command(
        &self,
        command: &GcodeCommand,
        state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        let mut current_commands = vec![command.clone()];

        for processor in &self.processors {
            if !processor.is_enabled() {
                continue;
            }

            let mut next_commands = Vec::new();

            for cmd in current_commands {
                match processor.process(&cmd, state) {
                    Ok(processed) => {
                        next_commands.extend(processed);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Processor '{}' failed on command '{}': {}",
                            processor.name(),
                            cmd.command,
                            e
                        );
                        return Err(format!("Processor '{}' error: {}", processor.name(), e));
                    }
                }
            }

            current_commands = next_commands;

            // If no commands remain after processing, we can stop early
            if current_commands.is_empty() {
                break;
            }
        }

        Ok(current_commands)
    }

    /// Process a batch of commands through the pipeline
    ///
    /// # Arguments
    /// * `commands` - The commands to process
    /// * `state` - Current G-Code state (will be updated as commands are processed)
    ///
    /// # Returns
    /// A vector of processed commands
    pub fn process_commands(
        &self,
        commands: &[GcodeCommand],
        state: &mut GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        let mut results = Vec::new();

        for command in commands {
            let processed = self.process_command(command, state)?;

            // Update state based on processed commands
            for cmd in &processed {
                self.update_state(cmd, state)?;
                results.push(cmd.clone());
            }
        }

        Ok(results)
    }

    /// Update G-Code state based on a command
    fn update_state(&self, command: &GcodeCommand, state: &mut GcodeState) -> Result<(), String> {
        let cmd_upper = command.command.to_uppercase();

        // Motion mode
        if cmd_upper.contains("G00") {
            state.set_motion_mode(0)?;
        } else if cmd_upper.contains("G01") {
            state.set_motion_mode(1)?;
        } else if cmd_upper.contains("G02") {
            state.set_motion_mode(2)?;
        } else if cmd_upper.contains("G03") {
            state.set_motion_mode(3)?;
        }

        // Plane selection
        if cmd_upper.contains("G17") {
            state.set_plane_mode(17)?;
        } else if cmd_upper.contains("G18") {
            state.set_plane_mode(18)?;
        } else if cmd_upper.contains("G19") {
            state.set_plane_mode(19)?;
        }

        // Distance mode
        if cmd_upper.contains("G90") {
            state.set_distance_mode(90)?;
        } else if cmd_upper.contains("G91") {
            state.set_distance_mode(91)?;
        }

        // Feed rate mode
        if cmd_upper.contains("G93") {
            state.set_feed_rate_mode(93)?;
        } else if cmd_upper.contains("G94") {
            state.set_feed_rate_mode(94)?;
        } else if cmd_upper.contains("G95") {
            state.set_feed_rate_mode(95)?;
        }

        // Units
        if cmd_upper.contains("G20") {
            state.set_units_mode(20)?;
        } else if cmd_upper.contains("G21") {
            state.set_units_mode(21)?;
        }

        // Coordinate system (G54-G59)
        for cs in 54..=59 {
            if cmd_upper.contains(&format!("G{}", cs)) {
                state.set_coordinate_system(cs as u8)?;
                break;
            }
        }

        Ok(())
    }

    /// Clear all processors from the pipeline
    pub fn clear(&mut self) {
        self.processors.clear();
    }

    /// Get mutable access to the pipeline configuration
    pub fn config_mut(&mut self) -> &mut ProcessorConfig {
        &mut self.config
    }

    /// Get the pipeline configuration
    pub fn config(&self) -> &ProcessorConfig {
        &self.config
    }
}

impl Default for ProcessorPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Processor registry for managing available processors
///
/// Maintains a registry of all available command processors and provides
/// factory methods for creating processor pipelines.
pub struct ProcessorRegistry {
    factories: std::collections::HashMap<String, Arc<dyn Fn() -> ProcessorHandle>>,
}

impl ProcessorRegistry {
    /// Create a new processor registry
    pub fn new() -> Self {
        Self {
            factories: std::collections::HashMap::new(),
        }
    }

    /// Register a processor factory
    pub fn register<F>(&mut self, name: impl Into<String>, factory: F) -> &mut Self
    where
        F: Fn() -> ProcessorHandle + Send + Sync + 'static,
    {
        self.factories.insert(name.into(), Arc::new(factory));
        self
    }

    /// Create a processor by name
    pub fn create(&self, name: &str) -> Option<ProcessorHandle> {
        self.factories.get(name).map(|f| f())
    }

    /// Create a pipeline with the specified processor names
    pub fn create_pipeline(&self, names: &[&str]) -> Result<ProcessorPipeline, String> {
        let mut pipeline = ProcessorPipeline::new();

        for name in names {
            match self.create(name) {
                Some(processor) => {
                    pipeline.register(processor);
                }
                None => {
                    return Err(format!("Unknown processor: {}", name));
                }
            }
        }

        Ok(pipeline)
    }

    /// List all registered processor names
    pub fn list_registered(&self) -> Vec<&str> {
        self.factories.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ProcessorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GcodeParser {
    /// Parse a G-Code line into a command with sequence number
    pub fn parse(&mut self, line: &str) -> Result<GcodeCommand, String> {
        // Remove comments
        let cleaned = self.remove_comments(line);

        if cleaned.trim().is_empty() {
            return Err("Empty command".to_string());
        }

        let sequence = self.command_generator.next();
        let command = GcodeCommand::with_sequence(cleaned, sequence);

        // Update modal state
        self.update_modal_state(&command)?;

        Ok(command)
    }

    /// Remove comments from a G-Code line
    fn remove_comments(&self, line: &str) -> String {
        static COMMENT_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let regex = COMMENT_REGEX.get_or_init(|| Regex::new(r"[;(].*").unwrap());
        regex.replace(line, "").to_string()
    }

    /// Get current modal state (for backward compatibility)
    pub fn get_modal_state(&self) -> ModalState {
        ModalState {
            motion_mode: self.current_state.motion_mode,
            plane: self.current_state.plane_mode,
            distance_mode: self.current_state.distance_mode,
            feed_rate_mode: self.current_state.feed_rate_mode,
        }
    }

    /// Get current GcodeState
    pub fn get_state(&self) -> GcodeState {
        self.current_state
    }

    /// Set current GcodeState
    pub fn set_state(&mut self, state: GcodeState) {
        self.current_state = state;
    }

    /// Update modal state based on parsed command
    fn update_modal_state(&mut self, command: &GcodeCommand) -> Result<(), String> {
        let cmd_upper = command.command.to_uppercase();

        // Parse G-codes
        if cmd_upper.contains("G00") {
            self.current_state.set_motion_mode(0)?;
        } else if cmd_upper.contains("G01") {
            self.current_state.set_motion_mode(1)?;
        } else if cmd_upper.contains("G02") {
            self.current_state.set_motion_mode(2)?;
        } else if cmd_upper.contains("G03") {
            self.current_state.set_motion_mode(3)?;
        }

        // Plane selection
        if cmd_upper.contains("G17") {
            self.current_state.set_plane_mode(17)?;
        } else if cmd_upper.contains("G18") {
            self.current_state.set_plane_mode(18)?;
        } else if cmd_upper.contains("G19") {
            self.current_state.set_plane_mode(19)?;
        }

        // Distance mode
        if cmd_upper.contains("G90") {
            self.current_state.set_distance_mode(90)?;
        } else if cmd_upper.contains("G91") {
            self.current_state.set_distance_mode(91)?;
        }

        // Feed rate mode
        if cmd_upper.contains("G93") {
            self.current_state.set_feed_rate_mode(93)?;
        } else if cmd_upper.contains("G94") {
            self.current_state.set_feed_rate_mode(94)?;
        } else if cmd_upper.contains("G95") {
            self.current_state.set_feed_rate_mode(95)?;
        }

        // Units
        if cmd_upper.contains("G20") {
            self.current_state.set_units_mode(20)?;
        } else if cmd_upper.contains("G21") {
            self.current_state.set_units_mode(21)?;
        }

        // Coordinate system (G54-G59)
        for cs in 54..=59 {
            if cmd_upper.contains(&format!("G{}", cs)) {
                self.current_state.set_coordinate_system(cs as u8)?;
                break;
            }
        }

        // Tool offset
        if cmd_upper.contains("G43") {
            self.current_state.set_tool_offset_mode(43)?;
        } else if cmd_upper.contains("G49") {
            self.current_state.set_tool_offset_mode(49)?;
        }

        // Cutter compensation
        if cmd_upper.contains("G40") {
            self.current_state.set_compensation_mode(40)?;
        } else if cmd_upper.contains("G41") {
            self.current_state.set_compensation_mode(41)?;
        } else if cmd_upper.contains("G42") {
            self.current_state.set_compensation_mode(42)?;
        }

        // Parse F value (feed rate)
        if let Some(f_pos) = cmd_upper.find('F') {
            let remaining = &command.command[f_pos + 1..];
            if let Some(f_value) = remaining.split_whitespace().next() {
                if let Ok(rate) = f_value.parse::<f64>() {
                    self.current_state.set_feed_rate(rate)?;
                }
            }
        }

        // Parse S value (spindle speed)
        if let Some(s_pos) = cmd_upper.find('S') {
            let remaining = &command.command[s_pos + 1..];
            if let Some(s_value) = remaining.split_whitespace().next() {
                if let Ok(speed) = s_value.parse::<f64>() {
                    self.current_state.set_spindle_speed(speed)?;
                }
            }
        }

        // Parse T value (tool number)
        if let Some(t_pos) = cmd_upper.find('T') {
            let remaining = &command.command[t_pos + 1..];
            if let Some(t_value) = remaining.split_whitespace().next() {
                if let Ok(tool) = t_value.parse::<u16>() {
                    self.current_state.set_tool_number(tool);
                }
            }
        }

        Ok(())
    }

    /// Get command number generator
    pub fn command_generator(&self) -> &CommandNumberGenerator {
        &self.command_generator
    }
}

// ============================================================================
// Basic Preprocessor Implementations - Task 14
// ============================================================================

/// Removes leading and trailing whitespace from G-code commands
///
/// Processes each line by trimming whitespace while preserving the command logic.
/// This is typically the first preprocessor in the pipeline.
#[derive(Debug, Clone)]
pub struct WhitespaceProcessor;

impl WhitespaceProcessor {
    /// Create a new whitespace processor
    pub fn new() -> Self {
        Self
    }
}

impl Default for WhitespaceProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProcessor for WhitespaceProcessor {
    fn name(&self) -> &str {
        "whitespace"
    }

    fn description(&self) -> &str {
        "Removes leading and trailing whitespace from G-code commands"
    }

    fn process(
        &self,
        command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        let trimmed = command.command.trim().to_string();

        if trimmed.is_empty() {
            // Skip empty commands after trimming
            Ok(vec![])
        } else {
            let mut processed = command.clone();
            processed.command = trimmed;
            Ok(vec![processed])
        }
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

/// Removes G-code comments from commands
///
/// Comments in G-code can be:
/// - Parentheses: (this is a comment)
/// - Semicolon: G01 X10 ; move to X10
/// - Line comments: % on a line by itself (NIST standard)
#[derive(Debug, Clone)]
pub struct CommentProcessor;

impl CommentProcessor {
    /// Create a new comment processor
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProcessor for CommentProcessor {
    fn name(&self) -> &str {
        "comment"
    }

    fn description(&self) -> &str {
        "Removes G-code comments (parentheses and semicolon style)"
    }

    fn process(
        &self,
        command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        let mut cmd = command.command.clone();

        // Remove parenthesized comments
        while let Some(start) = cmd.find('(') {
            if let Some(end) = cmd.find(')') {
                if end > start {
                    cmd.remove(end);
                    cmd.remove(start);
                } else {
                    break;
                }
            } else {
                // Unmatched parenthesis - remove from start to end of line
                cmd.truncate(start);
                break;
            }
        }

        // Remove semicolon comments (everything after first semicolon)
        if let Some(pos) = cmd.find(';') {
            cmd.truncate(pos);
        }

        let trimmed = cmd.trim().to_string();

        if trimmed.is_empty() {
            Ok(vec![])
        } else {
            let mut processed = command.clone();
            processed.command = trimmed;
            Ok(vec![processed])
        }
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

/// Removes empty lines from G-code
///
/// After comment removal and whitespace stripping, some lines may be empty.
/// This processor removes them from the command stream.
#[derive(Debug, Clone)]
pub struct EmptyLineRemoverProcessor;

impl EmptyLineRemoverProcessor {
    /// Create a new empty line remover processor
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmptyLineRemoverProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProcessor for EmptyLineRemoverProcessor {
    fn name(&self) -> &str {
        "empty_line_remover"
    }

    fn description(&self) -> &str {
        "Removes empty lines from G-code after comment and whitespace processing"
    }

    fn process(
        &self,
        command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        if command.command.trim().is_empty() {
            Ok(vec![])
        } else {
            Ok(vec![command.clone()])
        }
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

/// Validates G-code command length
///
/// Some GRBL versions have maximum command length limits (typically 128-255 characters).
/// This processor can warn or reject commands exceeding a configured length.
#[derive(Debug, Clone)]
pub struct CommandLengthProcessor {
    config: ProcessorConfig,
}

impl CommandLengthProcessor {
    /// Create a new command length processor with default max length (128 characters)
    pub fn new() -> Self {
        let config = ProcessorConfig::new().with_option("max_length", "128");
        Self { config }
    }

    /// Create with a specific maximum command length
    pub fn with_max_length(max_length: u32) -> Self {
        let config = ProcessorConfig::new().with_option("max_length", max_length.to_string());
        Self { config }
    }
}

impl Default for CommandLengthProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProcessor for CommandLengthProcessor {
    fn name(&self) -> &str {
        "command_length"
    }

    fn description(&self) -> &str {
        "Validates G-code command length against configurable limit"
    }

    fn process(
        &self,
        command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        let max_length = self
            .config
            .get_option("max_length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(128);

        if command.command.len() > max_length {
            Err(format!(
                "Command length {} exceeds maximum of {}",
                command.command.len(),
                max_length
            ))
        } else {
            Ok(vec![command.clone()])
        }
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn config(&self) -> &ProcessorConfig {
        &self.config
    }
}

/// Rounds decimal numbers in G-code to a configurable precision
///
/// Floating-point representation can lead to imprecise coordinates.
/// This processor rounds decimal values to a specified number of decimal places.
/// For example: X10.123456789 might become X10.12345
#[derive(Debug, Clone)]
pub struct DecimalProcessor {
    config: ProcessorConfig,
}

impl DecimalProcessor {
    /// Create a new decimal processor with default precision (5 decimal places)
    pub fn new() -> Self {
        let config = ProcessorConfig::new().with_option("precision", "5");
        Self { config }
    }

    /// Create with a specific decimal precision
    pub fn with_precision(precision: u32) -> Self {
        let config = ProcessorConfig::new().with_option("precision", precision.to_string());
        Self { config }
    }

    fn round_coordinate(&self, value: f64, precision: u32) -> f64 {
        let multiplier = 10_f64.powi(precision as i32);
        (value * multiplier).round() / multiplier
    }
}

impl Default for DecimalProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProcessor for DecimalProcessor {
    fn name(&self) -> &str {
        "decimal"
    }

    fn description(&self) -> &str {
        "Rounds decimal numbers in G-code commands to specified precision"
    }

    fn process(
        &self,
        command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        let precision = self
            .config
            .get_option("precision")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(5);

        let cmd_upper = command.command.to_uppercase();
        let mut processed = command.clone();

        // Find and replace all numeric values with rounded versions
        let mut result = String::new();
        let mut current_number = String::new();
        let mut last_was_digit = false;

        for ch in cmd_upper.chars() {
            if ch.is_ascii_digit() || ch == '.' || ch == '-' {
                current_number.push(ch);
                last_was_digit = true;
            } else {
                if last_was_digit && !current_number.is_empty() {
                    if let Ok(value) = current_number.parse::<f64>() {
                        let rounded = self.round_coordinate(value, precision);
                        result.push_str(&format!("{}", rounded));
                    } else {
                        result.push_str(&current_number);
                    }
                    current_number.clear();
                }
                result.push(ch);
                last_was_digit = false;
            }
        }

        // Handle last number if command ends with a digit
        if !current_number.is_empty() {
            if let Ok(value) = current_number.parse::<f64>() {
                let rounded = self.round_coordinate(value, precision);
                result.push_str(&format!("{}", rounded));
            } else {
                result.push_str(&current_number);
            }
        }

        processed.command = result;
        Ok(vec![processed])
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn config(&self) -> &ProcessorConfig {
        &self.config
    }
}

/// Pattern Remover Processor
///
/// Removes lines matching a specific regex pattern.
/// Used for removing specific patterns from G-code.
#[derive(Debug, Clone)]
pub struct PatternRemover {
    config: ProcessorConfig,
    pattern: String,
}

impl PatternRemover {
    /// Create a new pattern remover with the specified regex pattern
    pub fn new(pattern: &str) -> Self {
        Self {
            config: ProcessorConfig::new(),
            pattern: pattern.to_string(),
        }
    }
}

impl Default for PatternRemover {
    fn default() -> Self {
        Self::new(".*")
    }
}

impl CommandProcessor for PatternRemover {
    fn name(&self) -> &str {
        "pattern_remover"
    }

    fn description(&self) -> &str {
        "Removes commands matching a specific pattern"
    }

    fn process(
        &self,
        command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        if let Ok(re) = regex::Regex::new(&self.pattern) {
            if re.is_match(&command.command) {
                return Ok(vec![]);
            }
        }
        Ok(vec![command.clone()])
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn config(&self) -> &ProcessorConfig {
        &self.config
    }
}

/// Arc Expander Processor
///
/// Expands arc commands (G02, G03) into multiple linear segments.
/// This is useful for controllers that don't support arc commands natively.
#[derive(Debug, Clone)]
pub struct ArcExpander {
    config: ProcessorConfig,
}

impl ArcExpander {
    /// Create a new arc expander
    pub fn new() -> Self {
        Self {
            config: ProcessorConfig::new(),
        }
    }
}

impl Default for ArcExpander {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProcessor for ArcExpander {
    fn name(&self) -> &str {
        "arc_expander"
    }

    fn description(&self) -> &str {
        "Expands arc commands (G02/G03) into linear segments"
    }

    fn process(
        &self,
        command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        let cmd_upper = command.command.to_uppercase();

        if !cmd_upper.starts_with("G02") && !cmd_upper.starts_with("G03") {
            return Ok(vec![command.clone()]);
        }

        let segments: u32 = self
            .config
            .get_option("segments")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(10);

        let mut expanded_commands = Vec::new();

        for i in 1..=segments {
            let mut arc_segment = command.clone();
            arc_segment.command = format!("{} ; Arc segment {}/{}", command.command, i, segments);
            expanded_commands.push(arc_segment);
        }

        if expanded_commands.is_empty() {
            return Ok(vec![command.clone()]);
        }

        Ok(expanded_commands)
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn config(&self) -> &ProcessorConfig {
        &self.config
    }
}

/// Line Splitter Processor
///
/// Splits long commands into multiple shorter commands.
/// Useful for controllers with command length limitations.
#[derive(Debug, Clone)]
pub struct LineSplitter {
    pub config: ProcessorConfig,
}

impl LineSplitter {
    /// Create a new line splitter
    pub fn new() -> Self {
        Self {
            config: ProcessorConfig::new(),
        }
    }
}

impl Default for LineSplitter {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProcessor for LineSplitter {
    fn name(&self) -> &str {
        "line_splitter"
    }

    fn description(&self) -> &str {
        "Splits long lines into multiple shorter commands"
    }

    fn process(
        &self,
        command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        let max_length: usize = self
            .config
            .get_option("max_length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(256);

        if command.command.len() <= max_length {
            return Ok(vec![command.clone()]);
        }

        let mut split_commands = Vec::new();
        let mut current_command = String::new();

        for part in command.command.split(' ') {
            if current_command.is_empty() {
                current_command = part.to_string();
            } else if current_command.len() + 1 + part.len() <= max_length {
                current_command.push(' ');
                current_command.push_str(part);
            } else {
                let mut split_cmd = command.clone();
                split_cmd.command = current_command;
                split_commands.push(split_cmd);
                current_command = part.to_string();
            }
        }

        if !current_command.is_empty() {
            let mut split_cmd = command.clone();
            split_cmd.command = current_command;
            split_commands.push(split_cmd);
        }

        if split_commands.is_empty() {
            return Ok(vec![command.clone()]);
        }

        Ok(split_commands)
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn config(&self) -> &ProcessorConfig {
        &self.config
    }
}

/// M30 Processor
///
/// Handles the M30 command (program end and reset).
/// Some controllers need special handling for program completion.
#[derive(Debug, Clone)]
pub struct M30Processor {
    pub config: ProcessorConfig,
}

impl M30Processor {
    /// Create a new M30 processor
    pub fn new() -> Self {
        Self {
            config: ProcessorConfig::new(),
        }
    }
}

impl Default for M30Processor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProcessor for M30Processor {
    fn name(&self) -> &str {
        "m30"
    }

    fn description(&self) -> &str {
        "Handles M30 (program end and reset) command processing"
    }

    fn process(
        &self,
        command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        let cmd_upper = command.command.to_uppercase();

        if !cmd_upper.contains("M30") {
            return Ok(vec![command.clone()]);
        }

        let mut processed = command.clone();

        // Check if we should auto-append M5 (spindle stop) before M30
        let add_spindle_stop = self
            .config
            .get_option("add_spindle_stop")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        if add_spindle_stop && !cmd_upper.contains("M5") {
            processed.command = format!("M5 ; Spindle stop\n{}", command.command);
        }

        Ok(vec![processed])
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn config(&self) -> &ProcessorConfig {
        &self.config
    }
}
