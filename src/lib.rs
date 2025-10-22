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
pub use core::{ControllerTrait, ControllerListener, ControllerListenerHandle, SimpleController, OverrideState,
              event::{ControllerEvent, EventDispatcher},
              message::{Message, MessageLevel, MessageDispatcher}};
pub use data::{
    CNCPoint, CommunicatorState, ControllerState, ControllerStatus, MachineStatus,
    MachineStatusSnapshot, PartialPosition, Position, Units,
};
pub use error::{ConnectionError, ControllerError, Error, FirmwareError, GcodeError, Result};
pub use firmware::ControllerType;
pub use gcode::{
    CommandId, CommandListener, CommandListenerHandle, CommandNumberGenerator, CommandProcessor,
    CommandResponse, CommandState, CommentProcessor, CommandLengthProcessor, DecimalProcessor,
    EmptyLineRemoverProcessor, GcodeCommand, GcodeParser, GcodeState, ModalState, ProcessorConfig,
    ProcessorHandle, ProcessorPipeline, ProcessorRegistry, WhitespaceProcessor,
    stream::{FileStreamReader, GcodeStreamReader, PausableStream, StringStreamReader},
};
pub use ui::{SettingsDialog, Setting, SettingValue, SettingsCategory, KeyboardShortcut, SettingsPersistence, FirmwareSettingsIntegration, DeviceConsoleManager, DeviceMessageType, ConsoleListener, GcodeEditor, GcodeLine, Token, TokenType};

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

    tracing::info!("GCodeKit4 v{} logging initialized", VERSION);
    Ok(())
}
