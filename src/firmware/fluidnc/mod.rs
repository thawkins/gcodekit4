//! FluidNC firmware support
//!
//! Provides protocol implementation, response parsing, and command creation
//! for FluidNC CNC control systems.

pub mod constants;
pub mod capabilities;
pub mod response_parser;
pub mod command_creator;
pub mod controller;

pub use capabilities::FluidNCCapabilities;
pub use response_parser::FluidNCResponseParser;
pub use command_creator::FluidNCCommandCreator;
pub use controller::FluidNCController;

/// FluidNC version information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FluidNCVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
}

impl std::fmt::Display for FluidNCVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for FluidNCVersion {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
        }
    }
}
