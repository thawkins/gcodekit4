//! g2core Firmware Support
//!
//! This module provides complete support for g2core (next generation of TinyG).
//! It includes protocol constants, capabilities, version detection, and advanced features.
//! g2core supports 6 axes, kinematics, and advanced motion modes.

pub mod capabilities;
pub mod command_creator;
pub mod constants;
pub mod controller;
pub mod response_parser;

pub use capabilities::{G2CoreCapabilities, G2CoreVersion, VersionComparison};
pub use command_creator::{CommandCreator, KinematicMode, MotionType, RealTimeCommand};
pub use constants::*;
pub use controller::G2CoreController;
pub use response_parser::{G2CoreResponse, G2CoreResponseParser, G2CoreResponseType, G2CoreStatus};
