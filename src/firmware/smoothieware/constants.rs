//! Smoothieware protocol constants and error codes

/// Smoothieware response messages
pub mod responses {
    /// Acknowledgment of command received
    pub const ACK: &str = "ok";
    /// Error in command
    pub const ERROR: &str = "Error";
    /// Smoothieware is ready
    pub const READY: &str = "Smoothieware";
}

/// Smoothieware M-codes
pub mod mcodes {
    /// Spindle on (clockwise)
    pub const M3: u32 = 3;
    /// Spindle on (counter-clockwise)
    pub const M4: u32 = 4;
    /// Spindle off
    pub const M5: u32 = 5;
    /// Tool change
    pub const M6: u32 = 6;
}

/// Smoothieware G-codes
pub mod gcodes {
    /// Rapid positioning
    pub const G0: u32 = 0;
    /// Linear interpolation
    pub const G1: u32 = 1;
    /// Clockwise arc
    pub const G2: u32 = 2;
    /// Counter-clockwise arc
    pub const G3: u32 = 3;
    /// Dwell
    pub const G4: u32 = 4;
}

/// Default Smoothieware serial communication settings
pub const DEFAULT_BAUD_RATE: u32 = 115200;
pub const DEFAULT_TIMEOUT_MS: u64 = 1000;

/// Maximum axes supported by Smoothieware
pub const MAX_AXES: u8 = 5;

/// Smoothieware buffer size
pub const BUFFER_SIZE: usize = 128;
