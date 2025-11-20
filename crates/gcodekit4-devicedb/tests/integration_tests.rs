use gcodekit4_devicedb::{DeviceManager, DeviceProfile, DeviceType, ControllerType};
use tempfile::tempdir;

#[test]
fn test_device_manager_crud() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("devices.json");
    let manager = DeviceManager::new(config_path.clone());

    // Test Load (should create default)
    manager.load().unwrap();
    let profiles = manager.get_all_profiles();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].name, "New Device");
    
    let active = manager.get_active_profile();
    assert!(active.is_some());
    assert_eq!(active.unwrap().id, profiles[0].id);

    // Test Create
    let mut new_profile = DeviceProfile::default();
    new_profile.name = "My Laser".to_string();
    new_profile.device_type = DeviceType::LaserCutter;
    manager.save_profile(new_profile.clone()).unwrap();

    let profiles = manager.get_all_profiles();
    assert_eq!(profiles.len(), 2);

    // Test Read
    let fetched = manager.get_profile(&new_profile.id).unwrap();
    assert_eq!(fetched.name, "My Laser");
    assert_eq!(fetched.device_type, DeviceType::LaserCutter);

    // Test Update
    let mut updated = fetched.clone();
    updated.controller_type = ControllerType::Smoothieware;
    manager.save_profile(updated).unwrap();
    
    let fetched_updated = manager.get_profile(&new_profile.id).unwrap();
    assert_eq!(fetched_updated.controller_type, ControllerType::Smoothieware);

    // Test Set Active
    manager.set_active_profile(&new_profile.id).unwrap();
    let active = manager.get_active_profile().unwrap();
    assert_eq!(active.id, new_profile.id);

    // Test Persistence
    let manager2 = DeviceManager::new(config_path);
    manager2.load().unwrap();
    assert_eq!(manager2.get_all_profiles().len(), 2);
    assert_eq!(manager2.get_active_profile().unwrap().id, new_profile.id);

    // Test Delete
    manager2.delete_profile(&new_profile.id).unwrap();
    assert_eq!(manager2.get_all_profiles().len(), 1);
    assert!(manager2.get_active_profile().is_none()); // Active profile was deleted
}
