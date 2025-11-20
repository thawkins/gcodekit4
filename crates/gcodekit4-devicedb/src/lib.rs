pub mod model;
pub mod manager;
pub mod ui_integration;
pub mod traits;

pub use model::{DeviceProfile, DeviceType, ControllerType, AxisLimits};
pub use manager::DeviceManager;
pub use ui_integration::{DeviceUiController, DeviceProfileUiModel};
pub use traits::DeviceProfileProvider;
