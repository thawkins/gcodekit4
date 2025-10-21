//! User interface module - Slint-based
//!
//! **Phase 5: UI Implementation (Tasks 66-90)**
//!
//! This module contains:
//! - UI Architecture setup (Task 66)
//! - Component hierarchy and communication
//! - State management
//! - Event handling
//! - 11 major UI panels (Tasks 67-76)
//! - Additional UI features (Tasks 77-90)

pub mod architecture;
pub mod components;
pub mod state;
pub mod events;
pub mod main_window;
pub mod connection_panel;
pub mod dro_panel;
pub mod jog_controller;

pub use architecture::UiArchitecture;
pub use state::UiState;
pub use events::{UiEvent, UiEventBus};
pub use main_window::MainWindow;
pub use connection_panel::ConnectionPanel;
pub use dro_panel::DROPanel;
pub use jog_controller::JogControllerPanel;
