//! Smoothieware firmware support
//!
//! Provides protocol implementation, response parsing, and command creation
//! for Smoothieware CNC control systems.

pub mod capabilities;
pub mod command_creator;
pub mod constants;
pub mod controller;
pub mod response_parser;

pub use capabilities::SmoothiewareCapabilities;
pub use command_creator::SmoothiewareCommandCreator;
pub use controller::SmoothiewareController;
pub use response_parser::SmoothiewareResponseParser;

/// Smoothieware version information
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SmoothiewareVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
}

impl std::fmt::Display for SmoothiewareVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
