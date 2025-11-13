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
pub use gcodekit4_parser::designer;
pub use gcodekit4_communication::firmware;
pub use gcodekit4_ui::{visualizer, ui, config};
pub use gcodekit4_core::data;

pub use gcodekit4_core::{
    ControllerEvent, ControllerListener, ControllerListenerHandle, ControllerTrait,
    EventDispatcher, Message, MessageDispatcher, MessageLevel, OverrideState, SimpleController,
    CNCPoint, CommunicatorState, ControllerState, ControllerStatus, MachineStatus,
    MachineStatusSnapshot, PartialPosition, Position, Units,
    ConnectionError, ControllerError, Error, FirmwareError, GcodeError, Result,
};

pub use gcodekit4_parser::{
    FileStreamReader, GcodeStreamReader, PausableStream, StringStreamReader,
    CommandId, CommandLengthProcessor, CommandListener, CommandListenerHandle,
    CommandNumberGenerator, CommandProcessor, CommandResponse, CommandState, CommentProcessor,
    DecimalProcessor, EmptyLineRemoverProcessor, GcodeCommand, GcodeParser, GcodeState, ModalState,
    ProcessorConfig, ProcessorHandle, ProcessorPipeline, ProcessorRegistry, WhitespaceProcessor,
    BoxParameters, BoxType, FingerJointSettings, FingerStyle, TabbedBoxMaker,
    JigsawPuzzleMaker, PuzzleParameters,
    Canvas, CanvasPoint, Circle, DrawingMode, Line, Point, Rectangle, Shape, ShapeType, Toolpath,
    ToolpathGenerator, ToolpathSegment, ToolpathSegmentType, ToolpathToGcode,
    DesignerState,
    AdvancedProber, Alarm, AlarmManager, AlarmType, AutoConnectConfig, BackupEntry, BackupManager,
    BasicProber, Bookmark, BookmarkManager, CommandHistory, CustomAction, CustomMacro,
    DataLogger, DropEvent, DropFileType, DropIndicatorState, DropTarget, DropZone,
    ExportOptions, FeedRateStats, FileComparison, FileEncoding, FileExporter, FileFormat,
    FileProcessingPipeline, FileReadStats, FileStatistics, FileValidation, GcodeFileReader,
    GcodeTemplate, HeightPoint, HistoryEntry, LogEntry, NetworkConfig, PendantButton,
    PendantConfig, PerformanceMetrics, ProbeMesh, ProbePoint, ProcessedFile, ProgramState,
    RecentFileEntry, RecentFilesManager, SimulationPosition, Simulator, SoftLimits, SpindleStats,
    Stepper, TemplateLibrary, TemplateVariable, ToolInfo, ToolLibrary, ToolOffset,
    ToolOffsetManager, ValidationIssue, ValidationResult, ValidationSeverity,
    WorkCoordinateSystem, WorkOffset,
};

pub use gcodekit4_communication::{
    list_ports, SerialPortInfo, TcpConnectionInfo,
    Communicator, CommunicatorEvent, CommunicatorListener, CommunicatorListenerHandle,
    ConnectionDriver, ConnectionParams, NoOpCommunicator, SerialCommunicator, SerialParity,
    TcpCommunicator,
    CapabilityManager, CapabilityState, ControllerType, FirmwareDetector,
};

pub use gcodekit4_ui::{
    ConsoleListener, DeviceConsoleManager, DeviceMessageType, FirmwareSettingsIntegration,
    GcodeEditor, GcodeLine, KeyboardShortcut, Setting, SettingValue, SettingsCategory,
    SettingsDialog, SettingsPersistence, Token, TokenType,
    Config, ConnectionSettings, ConnectionType, FileProcessingSettings, FirmwareSettings,
    MachineSettings, SettingsManager, UiSettings,
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
