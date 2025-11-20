use crate::model::DeviceProfile;

pub trait DeviceProfileProvider: Send + Sync {
    fn get_active_profile(&self) -> Option<DeviceProfile>;
    fn get_profile(&self, id: &str) -> Option<DeviceProfile>;
}
