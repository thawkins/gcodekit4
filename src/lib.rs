//! # GCodeKit4
//!
//! A Rust-based Universal G-Code Sender for CNC machines with support for:
//! - GRBL, TinyG, g2core, Smoothieware, FluidNC controllers
//! - Serial (USB), TCP/IP, and WebSocket connectivity
//! - 14 G-Code preprocessors for advanced toolpath processing
//! - Real-time 3D visualization and interactive UI
//!
//! ## Architecture
//!
//! GCodeKit4 is organized as a workspace with multiple crates:
//!
//! 1. **gcodekit4-core** - Core types, traits, state management, events
//! 2. **gcodekit4-parser** - G-code parsing, preprocessing, utilities
//! 3. **gcodekit4-communication** - Serial, TCP, WebSocket, firmware protocols
//! 4. **gcodekit4-ui** - Slint UI, visualizer, settings, editor
//! 5. **gcodekit4** - Main binary that integrates all crates
//!
//! ## Features
//!
//! - **Multi-Controller Support**: GRBL, TinyG, g2core, Smoothieware, FluidNC
//! - **Connection Protocols**: Serial/USB, TCP/IP, WebSocket
//! - **G-Code Processing**: Full parser with 14 preprocessors (arc expansion, mesh leveling, etc.)
//! - **Real-time Control**: Jogging, homing, probing, work coordinate systems
//! - **Advanced Features**: Macros, simulation mode, performance monitoring
//! - **Professional UI**: Syntax-highlighted editor, 3D visualization, console
//! - **Cross-Platform**: Linux, Windows, macOS support

#![allow(dead_code)]

// Re-export modules for main.rs
pub use gcodekit4_communication::firmware;
pub use gcodekit4_core::data;
pub use gcodekit4_parser::designer;
pub use gcodekit4_ui::{config, ui, visualizer};

pub use gcodekit4_core::{
    CNCPoint, CommunicatorState, ConnectionError, ControllerError, ControllerEvent,
    ControllerListener, ControllerListenerHandle, ControllerState, ControllerStatus,
    ControllerTrait, Error, EventDispatcher, FirmwareError, GcodeError, MachineStatus,
    MachineStatusSnapshot, Message, MessageDispatcher, MessageLevel, OverrideState,
    PartialPosition, Position, Result, SimpleController, Units,
};

pub use gcodekit4_parser::{
    AdvancedProber, Alarm, AlarmManager, AlarmType, AutoConnectConfig, BackupEntry, BackupManager,
    BasicProber, Bookmark, BookmarkManager, BoxParameters, BoxType, Canvas, CanvasPoint, Circle,
    CommandHistory, CommandId, CommandLengthProcessor, CommandListener, CommandListenerHandle,
    CommandNumberGenerator, CommandProcessor, CommandResponse, CommandState, CommentProcessor,
    CustomAction, CustomMacro, DataLogger, DecimalProcessor, DesignerState, DrawingMode, DropEvent,
    DropFileType, DropIndicatorState, DropTarget, DropZone, EmptyLineRemoverProcessor,
    ExportOptions, FeedRateStats, FileComparison, FileEncoding, FileExporter, FileFormat,
    FileProcessingPipeline, FileReadStats, FileStatistics, FileStreamReader, FileValidation,
    FingerJointSettings, FingerStyle, GcodeCommand, GcodeFileReader, GcodeParser, GcodeState,
    GcodeStreamReader, GcodeTemplate, HeightPoint, HistoryEntry, JigsawPuzzleMaker, Line, LogEntry,
    ModalState, NetworkConfig, PausableStream, PendantButton, PendantConfig, PerformanceMetrics,
    Point, ProbeMesh, ProbePoint, ProcessedFile, ProcessorConfig, ProcessorHandle,
    ProcessorPipeline, ProcessorRegistry, ProgramState, PuzzleParameters, RecentFileEntry,
    RecentFilesManager, Rectangle, Shape, ShapeType, SimulationPosition, Simulator, SoftLimits,
    SpindleStats, Stepper, StringStreamReader, TabbedBoxMaker, TemplateLibrary, TemplateVariable,
    ToolInfo, ToolLibrary, ToolOffset, ToolOffsetManager, Toolpath, ToolpathGenerator,
    ToolpathSegment, ToolpathSegmentType, ToolpathToGcode, ValidationIssue, ValidationResult,
    ValidationSeverity, WhitespaceProcessor, WorkCoordinateSystem, WorkOffset,
};

pub use gcodekit4_communication::{
    list_ports, CapabilityManager, CapabilityState, Communicator, CommunicatorEvent,
    CommunicatorListener, CommunicatorListenerHandle, ConnectionDriver, ConnectionParams,
    ControllerType, FirmwareDetector, NoOpCommunicator, SerialCommunicator, SerialParity,
    SerialPortInfo, TcpCommunicator, TcpConnectionInfo,
};

pub use gcodekit4_ui::{
    Config, ConnectionSettings, ConnectionType, ConsoleListener, DeviceConsoleManager,
    DeviceMessageType, FileProcessingSettings, FirmwareSettings, FirmwareSettingsIntegration,
    GcodeEditor, GcodeLine, KeyboardShortcut, MachineSettings, Setting, SettingValue,
    SettingsCategory, SettingsDialog, SettingsManager, SettingsPersistence, Token, TokenType,
    UiSettings,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build date (set at compile time)
pub const BUILD_DATE: &str = env!("BUILD_DATE");

/// Initialize logging with the default configuration
///
/// Sets up structured logging with:
/// - Console output with pretty formatting
/// - RUST_LOG environment variable support
/// - UTF timestamps
pub fn init_logging() -> anyhow::Result<()> {
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    let env_filter = EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into());

    let fmt_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true)
        .pretty();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}
