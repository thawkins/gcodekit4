//! TinyG Firmware Support
//!
//! This module provides complete support for TinyG (CNC control for 3D printers and engravers).
//! It includes protocol constants, capabilities, version detection, and feature flags.

pub mod capabilities;
pub mod command_creator;
pub mod constants;
pub mod controller;
pub mod response_parser;
pub mod utils;

pub use capabilities::{TinyGCapabilities, TinyGVersion, VersionComparison};
pub use command_creator::{CommandCreator, MotionType, RealTimeCommand};
pub use constants::*;
pub use controller::TinyGController;
pub use response_parser::{TinyGResponse, TinyGResponseParser, TinyGResponseType, TinyGStatus};
