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
//! GCodeKit4 is organized into 8 major modules:
//!
//! 1. **core** - Controller management, state machine, event system
//! 2. **communication** - Serial, TCP, WebSocket protocols
//! 3. **gcode** - Parser, state machine, preprocessors
//! 4. **firmware** - Protocol implementations for various controllers
//! 5. **data** - Data models (Position, Status, Commands, etc.)
//! 6. **ui** - Slint-based user interface (11 panels)
//! 7. **visualizer** - wgpu 3D rendering with interactive controls
//! 8. **utils** - Helper functions and common utilities

#![allow(dead_code)] // Allow dead code for infrastructure/future features
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

pub mod communication;
pub mod config;
pub mod core;
pub mod data;
pub mod designer;
pub mod designer_editor_integration;
pub mod designer_state;
pub mod designer_visualizer_integration;
pub mod error;
pub mod firmware;
pub mod gcode;
pub mod processing;
pub mod testing;
pub mod ui;
pub mod utils;
pub mod visualizer;

pub use communication::{
    serial::{list_ports, SerialPortInfo},
    tcp::TcpConnectionInfo,
    Communicator, CommunicatorEvent, CommunicatorListener, CommunicatorListenerHandle,
    ConnectionDriver, ConnectionParams, NoOpCommunicator, SerialCommunicator, SerialParity,
    TcpCommunicator,
};
pub use config::{
    Config, ConnectionSettings, ConnectionType, FileProcessingSettings, FirmwareSettings,
    MachineSettings, SettingsManager, UiSettings,
};
pub use core::{
    event::{ControllerEvent, EventDispatcher},
    message::{Message, MessageDispatcher, MessageLevel},
    ControllerListener, ControllerListenerHandle, ControllerTrait, OverrideState, SimpleController,
};
pub use data::{
    CNCPoint, CommunicatorState, ControllerState, ControllerStatus, MachineStatus,
    MachineStatusSnapshot, PartialPosition, Position, Units,
};
pub use designer::{
    Canvas, CanvasPoint, Circle, DrawingMode, Line, Point, Rectangle, Shape, ShapeType, Toolpath,
    ToolpathGenerator, ToolpathSegment, ToolpathSegmentType, ToolpathToGcode,
};
pub use designer_state::DesignerState;
pub use error::{ConnectionError, ControllerError, Error, FirmwareError, GcodeError, Result};
pub use firmware::{CapabilityManager, CapabilityState, ControllerType};
pub use gcode::{
    stream::{FileStreamReader, GcodeStreamReader, PausableStream, StringStreamReader},
    CommandId, CommandLengthProcessor, CommandListener, CommandListenerHandle,
    CommandNumberGenerator, CommandProcessor, CommandResponse, CommandState, CommentProcessor,
    DecimalProcessor, EmptyLineRemoverProcessor, GcodeCommand, GcodeParser, GcodeState, ModalState,
    ProcessorConfig, ProcessorHandle, ProcessorPipeline, ProcessorRegistry, WhitespaceProcessor,
};
pub use ui::{
    ConsoleListener, DeviceConsoleManager, DeviceMessageType, FirmwareSettingsIntegration,
    GcodeEditor, GcodeLine, KeyboardShortcut, Setting, SettingValue, SettingsCategory,
    SettingsDialog, SettingsPersistence, Token, TokenType,
};
pub use utils::{
    AdvancedProber, Alarm, AlarmManager, AlarmType, AutoConnectConfig, BackupEntry, BackupManager,
    BasicProber, Bookmark, BookmarkManager, BoundingBox, CommandHistory, CustomAction, CustomMacro,
    DataLogger, DropEvent, DropFileType, DropIndicatorState, DropTarget, DropZone, ExportOptions,
    FeedRateStats, FileComparison, FileEncoding, FileExporter, FileFormat, FileProcessingPipeline,
    FileReadStats, FileStatistics, FileValidation, GcodeFileReader, GcodeTemplate, HeightPoint,
    HistoryEntry, LogEntry, NetworkConfig, PendantButton, PendantConfig, PerformanceMetrics,
    ProbeMesh, ProbePoint, ProcessedFile, ProgramState, RecentFileEntry, RecentFilesManager,
    SimulationPosition, Simulator, SoftLimits, SpindleStats, Stepper, TemplateLibrary,
    TemplateVariable, ToolInfo, ToolLibrary, ToolOffset, ToolOffsetManager, ValidationIssue,
    ValidationResult, ValidationSeverity, WorkCoordinateSystem, WorkOffset,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build date
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
