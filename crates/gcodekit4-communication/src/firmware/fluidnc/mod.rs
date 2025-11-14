//! FluidNC firmware support
//!
//! Provides protocol implementation, response parsing, and command creation
//! for FluidNC CNC control systems.

pub mod capabilities;
pub mod command_creator;
pub mod constants;
pub mod controller;
pub mod response_parser;

pub use capabilities::FluidNCCapabilities;
pub use command_creator::FluidNCCommandCreator;
pub use controller::FluidNCController;
pub use response_parser::FluidNCResponseParser;

/// FluidNC version information
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
