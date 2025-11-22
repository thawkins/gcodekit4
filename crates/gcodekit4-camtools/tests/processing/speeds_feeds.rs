use gcodekit4_camtools::speeds_feeds::SpeedsFeedsCalculator;
use gcodekit4_core::data::materials::{Material, MaterialCategory, MaterialId};
use gcodekit4_core::data::tools::{Tool, ToolId, ToolType};
use gcodekit4_devicedb::model::DeviceProfile;

#[test]
fn test_calculation_with_defaults() {
    let material = Material::new(
        MaterialId("test".to_string()),
        "Test".to_string(),
        MaterialCategory::Wood,
        "Test".to_string(),
    );
    
    let tool = Tool::new(
        ToolId("test".to_string()),
        1,
        "Test".to_string(),
        ToolType::EndMillFlat,
        6.35, // 1/4 inch
        50.0,
    );

    let device = DeviceProfile::default();

    let result = SpeedsFeedsCalculator::calculate(&material, &tool, &device);
    
    assert!(result.rpm > 0);
    assert!(result.feed_rate > 0.0);
    assert!(result.source.contains("Tool Defaults"));
}

#[test]
fn test_calculation_with_material_properties() {
    let mut material = Material::new(
        MaterialId("test".to_string()),
        "Test".to_string(),
        MaterialCategory::Wood,
        "Test".to_string(),
    );
    
    // Set explicit surface speed and chip load via CuttingParameters
    let mut params = gcodekit4_core::data::materials::CuttingParameters::default();
    params.surface_speed_m_min = Some(300.0); // High speed for wood
    params.chip_load_mm = Some(0.1);
    
    material.set_cutting_params("endmill_flat".to_string(), params);

    let tool = Tool::new(
        ToolId("test".to_string()),
        1,
        "Test".to_string(),
        ToolType::EndMillFlat,
        6.35, // ~6mm
        50.0,
    );

    let mut device = DeviceProfile::default();
    device.max_feed_rate = 5000.0;

    let result = SpeedsFeedsCalculator::calculate(&material, &tool, &device);

    // RPM = (300 * 1000) / (pi * 6.35) ≈ 15037
    assert!(result.rpm > 15000 && result.rpm < 15100);
    
    // Feed = 15037 * 0.1 * 2 ≈ 3007
    assert!(result.feed_rate > 3000.0 && result.feed_rate < 3020.0);
    
    assert!(result.source.contains("Material Surface Speed"));
}
