//! # GCodeKit4 Parser
//!
//! G-code parsing, processing, and generation for GCodeKit4.
//! Includes parser, preprocessors, stream readers, and designer tools.

pub mod gcode;
pub mod processing;
pub mod designer;
pub mod designer_state;
pub mod designer_editor_integration;
pub mod designer_visualizer_integration;
pub mod utils;

pub use gcode::{
    stream::{FileStreamReader, GcodeStreamReader, PausableStream, StringStreamReader},
    CommandId, CommandLengthProcessor, CommandListener, CommandListenerHandle,
    CommandNumberGenerator, CommandProcessor, CommandResponse, CommandState, CommentProcessor,
    DecimalProcessor, EmptyLineRemoverProcessor, GcodeCommand, GcodeParser, GcodeState, ModalState,
    ProcessorConfig, ProcessorHandle, ProcessorPipeline, ProcessorRegistry, WhitespaceProcessor,
};

pub use processing::{
    BoxParameters, BoxType, LayoutStyle, TabType, TabbedBoxMaker,
    JigsawPuzzleMaker, PuzzleParameters,
};

pub use designer::{
    Canvas, CanvasPoint, Circle, DrawingMode, Line, Point, Rectangle, Shape, ShapeType, Toolpath,
    ToolpathGenerator, ToolpathSegment, ToolpathSegmentType, ToolpathToGcode,
};

pub use designer_state::DesignerState;

pub use utils::{
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
