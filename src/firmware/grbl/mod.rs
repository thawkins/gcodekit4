//! GRBL Firmware Support
//!
//! This module provides complete support for GRBL (open-source CNC control software).
//! It includes protocol constants, capabilities, version detection, and feature flags.

pub mod constants;
pub mod capabilities;
pub mod response_parser;
pub mod status_parser;
pub mod utils;
pub mod command_creator;

pub use capabilities::{GrblCapabilities, GrblVersion, GrblFeatureSet, VersionComparison, GrblFeature};
pub use response_parser::{GrblResponse, GrblResponseParser, StatusReport, BufferState};
pub use status_parser::{StatusParser, MachinePosition, WorkPosition, WorkCoordinateOffset, BufferRxState, FeedSpindleState, FullStatus};
pub use command_creator::{CommandCreator, RealTimeCommand, SystemCommand, JogCommand, ProbeCommand, ProbeType, JogMode};
pub use constants::*;
