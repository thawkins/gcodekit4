//! # GCodeKit4 Core
//!
//! Core types, traits, and utilities for GCodeKit4.
//! Provides the fundamental abstractions for controller management,
//! state machines, events, and data models.

pub mod core;
pub mod data;
pub mod error;
pub mod units;

pub use core::{
    event::{ControllerEvent, EventDispatcher},
    message::{Message, MessageDispatcher, MessageLevel},
    ControllerListener, ControllerListenerHandle, ControllerTrait, OverrideState, SimpleController,
};

pub use data::{
    CNCPoint, CommunicatorState, ControllerState, ControllerStatus, MachineStatus,
    MachineStatusSnapshot, PartialPosition, Position, Units,
};

pub use error::{ConnectionError, ControllerError, Error, FirmwareError, GcodeError, Result};
