//! TinyG Firmware Support
//!
//! This module provides complete support for TinyG (CNC control for 3D printers and engravers).
//! It includes protocol constants, capabilities, version detection, and feature flags.

pub mod constants;
pub mod capabilities;
pub mod response_parser;
pub mod utils;
pub mod command_creator;

pub use capabilities::{TinyGCapabilities, TinyGVersion, VersionComparison};
pub use response_parser::{TinyGResponse, TinyGResponseType, TinyGResponseParser, TinyGStatus};
pub use command_creator::{CommandCreator, RealTimeCommand, MotionType};
pub use constants::*;
