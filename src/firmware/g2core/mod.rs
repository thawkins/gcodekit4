//! g2core Firmware Support
//!
//! This module provides complete support for g2core (next generation of TinyG).
//! It includes protocol constants, capabilities, version detection, and advanced features.
//! g2core supports 6 axes, kinematics, and advanced motion modes.

pub mod constants;
pub mod capabilities;
pub mod response_parser;
pub mod command_creator;
pub mod controller;

pub use capabilities::{G2CoreCapabilities, G2CoreVersion, VersionComparison};
pub use response_parser::{G2CoreResponse, G2CoreResponseType, G2CoreResponseParser, G2CoreStatus};
pub use command_creator::{CommandCreator, RealTimeCommand, MotionType, KinematicMode};
pub use controller::G2CoreController;
pub use constants::*;
