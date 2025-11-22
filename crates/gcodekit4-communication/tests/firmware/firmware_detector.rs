use gcodekit4_communication::firmware::firmware_detector::*;
use gcodekit4_communication::firmware::firmware_version::FirmwareType;

#[test]
fn test_parse_grbl_version_info() {
    let response = "[VER:1.1h.20190825:Some string]\n[OPT:V,15,128]";
    let result = FirmwareDetector::parse_grbl_version_info(response).unwrap();

    assert_eq!(result.firmware_type, FirmwareType::Grbl);
    assert_eq!(result.version.major, 1);
    assert_eq!(result.version.minor, 1);
    assert_eq!(result.version_string, "1.1h");
    assert_eq!(result.build_date, Some("20190825".to_string()));
    assert_eq!(result.build_info, Some("V,15,128".to_string()));
}

#[test]
fn test_parse_grbl_startup() {
    let message = "Grbl 1.1f ['$' for help]";
    let result = FirmwareDetector::parse_grbl_startup(message).unwrap();

    assert_eq!(result.firmware_type, FirmwareType::Grbl);
    assert_eq!(result.version.major, 1);
    assert_eq!(result.version.minor, 1);
    assert_eq!(result.version_string, "1.1f");
}

#[test]
fn test_parse_marlin_version_info() {
    let response = "FIRMWARE_NAME:Marlin 2.0.9.3\nPROTOCOL_VERSION:1.0";
    let result = FirmwareDetector::parse_marlin_version_info(response).unwrap();

    assert_eq!(result.version.major, 2);
    assert_eq!(result.version.minor, 0);
    assert_eq!(result.version.patch, 9);
}

#[test]
fn test_parse_generic_grbl_response() {
    let response = "[VER:1.1h.20190825:]\n[OPT:V,15,128]";
    let result = FirmwareDetector::parse_response(response).unwrap();

    assert_eq!(result.firmware_type, FirmwareType::Grbl);
    assert_eq!(result.version.major, 1);
    assert_eq!(result.version.minor, 1);
}

#[test]
fn test_get_query_command() {
    assert_eq!(
        FirmwareDetector::get_query_command(FirmwareType::Grbl),
        "$I"
    );
    assert_eq!(
        FirmwareDetector::get_query_command(FirmwareType::Unknown),
        "$I"
    );
}
