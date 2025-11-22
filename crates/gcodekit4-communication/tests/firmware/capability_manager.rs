use gcodekit4_communication::firmware::capability_manager::*;
use gcodekit4_communication::firmware::firmware_version::{FirmwareType, SemanticVersion};

#[test]
fn test_capability_manager_default() {
    let manager = CapabilityManager::new();
    let state = manager.get_state();
    
    assert_eq!(state.firmware_type, None);
    assert_eq!(state.version, None);
}

#[test]
fn test_update_firmware_grbl() {
    let manager = CapabilityManager::new();
    
    manager.update_firmware(FirmwareType::Grbl, SemanticVersion::new(1, 1, 0));
    
    let state = manager.get_state();
    assert_eq!(state.firmware_type, Some(FirmwareType::Grbl));
    assert_eq!(state.version.as_ref().unwrap().major, 1);
    assert_eq!(state.version.as_ref().unwrap().minor, 1);
    
    assert!(state.supports_variable_spindle);
    assert!(state.supports_status_reports);
}

#[test]
fn test_supports_capability() {
    let manager = CapabilityManager::new();
    
    // Unknown firmware supports nothing
    assert!(!manager.supports("arcs"));
    
    // Update to GRBL 1.1
    manager.update_firmware(FirmwareType::Grbl, SemanticVersion::new(1, 1, 0));
    assert!(manager.supports("arcs"));
    assert!(manager.supports("probing"));
}

#[test]
fn test_reset() {
    let manager = CapabilityManager::new();
    
    manager.update_firmware(FirmwareType::Grbl, SemanticVersion::new(1, 1, 0));
    assert_eq!(manager.get_state().firmware_type, Some(FirmwareType::Grbl));
    
    manager.reset();
    assert_eq!(manager.get_state().firmware_type, None);
    assert!(!manager.get_state().detected);
}

#[test]
fn test_get_summary() {
    let manager = CapabilityManager::new();
    
    manager.update_firmware(FirmwareType::Grbl, SemanticVersion::new(1, 1, 0));
    
    let summary = manager.get_state().get_summary();
    assert!(summary.contains("GRBL"));
    assert!(summary.contains("v1.1.0"));
    assert!(summary.contains("3 axes"));
}
