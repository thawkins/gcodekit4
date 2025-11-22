use gcodekit4_communication::firmware::device_db::*;
use gcodekit4_communication::firmware::firmware_version::FirmwareType;

#[test]
fn test_device_creation() {
    let device = Device::new(
        "device_001".to_string(),
        "Test Device".to_string(),
        FirmwareType::Grbl,
    );
    assert_eq!(device.name, "Test Device");
    assert_eq!(device.firmware_type, FirmwareType::Grbl);
}

#[test]
fn test_database_add_device() {
    let mut db = DeviceDatabase::new();
    let id = db.create_device("Test Device".to_string(), FirmwareType::Grbl);
    
    assert_eq!(db.device_count(), 1);
    assert!(db.get_device(&id).is_some());
}

#[test]
fn test_find_device_by_name() {
    let mut db = DeviceDatabase::new();
    let _id1 = db.create_device("Device 1".to_string(), FirmwareType::Grbl);
    let _id2 = db.create_device("Device 2".to_string(), FirmwareType::TinyG);
    
    let found = db.find_device_by_name("Device 2");
    assert!(found.is_some());
    assert_eq!(found.unwrap().firmware_type, FirmwareType::TinyG);
}

#[test]
fn test_find_device_by_serial() {
    let mut db = DeviceDatabase::new();
    let id = db.create_device("Test Device".to_string(), FirmwareType::Grbl);
    
    if let Some(device) = db.get_device_mut(&id) {
        device.serial_number = Some("SN12345".to_string());
    }
    
    let found = db.find_device_by_serial("SN12345");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test Device");
}

#[test]
fn test_set_primary_device() {
    let mut db = DeviceDatabase::new();
    let id1 = db.create_device("Device 1".to_string(), FirmwareType::Grbl);
    let id2 = db.create_device("Device 2".to_string(), FirmwareType::Grbl);
    
    let _ = db.set_primary_device(&id2);
    
    assert!(!db.get_device(&id1).unwrap().is_primary);
    assert!(db.get_device(&id2).unwrap().is_primary);
}

#[test]
fn test_first_device_is_primary() {
    let mut db = DeviceDatabase::new();
    let id1 = db.create_device("Device 1".to_string(), FirmwareType::Grbl);
    
    assert!(db.get_device(&id1).unwrap().is_primary);
    
    let id2 = db.create_device("Device 2".to_string(), FirmwareType::Grbl);
    assert!(!db.get_device(&id2).unwrap().is_primary);
}

#[test]
fn test_delete_device() {
    let mut db = DeviceDatabase::new();
    let id = db.create_device("Test Device".to_string(), FirmwareType::Grbl);
    
    assert!(db.delete_device(&id).is_ok());
    assert_eq!(db.device_count(), 0);
}

#[test]
fn test_create_device() {
    let mut db = DeviceDatabase::new();
    let id = db.create_device("Test Device".to_string(), FirmwareType::Grbl);
    
    let device = db.get_device(&id).unwrap();
    assert_eq!(device.name, "Test Device");
    assert_eq!(device.firmware_type, FirmwareType::Grbl);
}

#[test]
fn test_device_notes() {
    let mut db = DeviceDatabase::new();
    let id = db.create_device("Test Device".to_string(), FirmwareType::Grbl);
    
    if let Some(device) = db.get_device_mut(&id) {
        device.notes = "A test CNC machine".to_string();
    }
    
    let device = db.get_device(&id).unwrap();
    assert_eq!(device.notes, "A test CNC machine");
}

#[test]
fn test_favorite_devices() {
    let mut db = DeviceDatabase::new();
    let id1 = db.create_device("Device 1".to_string(), FirmwareType::Grbl);
    let id2 = db.create_device("Device 2".to_string(), FirmwareType::Grbl);
    
    if let Some(device) = db.get_device_mut(&id1) {
        device.is_favorite = true;
    }
    
    let favorites = db.favorite_devices();
    assert_eq!(favorites.len(), 1);
    assert_eq!(favorites[0].id, id1);
    
    // Primary is not necessarily favorite unless set
    assert!(!db.get_device(&id2).unwrap().is_favorite);
}

#[test]
fn test_record_connection() {
    let mut db = DeviceDatabase::new();
    let id = db.create_device("Test Device".to_string(), FirmwareType::Grbl);
    
    if let Some(device) = db.get_device_mut(&id) {
        device.record_connection();
    }
    
    let device = db.get_device(&id).unwrap();
    assert!(device.last_connected.is_some());
    assert_eq!(device.connection_count, 1);
}
