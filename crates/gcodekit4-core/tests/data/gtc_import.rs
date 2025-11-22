use gcodekit4_core::data::gtc_import::*;
use gcodekit4_core::data::tools::*;

#[test]
fn test_tool_type_mapping() {
    let importer = GtcImporter::new(1);

    assert_eq!(
        importer.map_tool_type("End Mill").unwrap(),
        ToolType::EndMillFlat
    );
    assert_eq!(
        importer.map_tool_type("Ball End Mill").unwrap(),
        ToolType::EndMillBall
    );
    assert_eq!(
        importer.map_tool_type("Drill Bit").unwrap(),
        ToolType::DrillBit
    );
}

#[test]
fn test_material_mapping() {
    let importer = GtcImporter::new(1);

    assert_eq!(
        importer.map_tool_material("Solid Carbide"),
        ToolMaterial::Carbide
    );
    assert_eq!(importer.map_tool_material("HSS"), ToolMaterial::HSS);
}

#[test]
fn test_gtc_tool_conversion() {
    let mut importer = GtcImporter::new(100);

    let gtc_tool = GtcTool {
        id: "EM-001".to_string(),
        description: "6mm Carbide End Mill".to_string(),
        tool_type: "End Mill".to_string(),
        diameter: 6.0,
        length: 50.0,
        flute_length: Some(20.0),
        shank_diameter: Some(6.0),
        number_of_flutes: Some(2),
        material: Some("Carbide".to_string()),
        coating: Some("TiAlN".to_string()),
        manufacturer: Some("Test Mfg".to_string()),
        part_number: Some("EM-6-2F".to_string()),
        cutting_parameters: None,
    };

    let tool = importer.convert_gtc_tool(gtc_tool).unwrap();
    assert_eq!(tool.diameter, 6.0);
    assert_eq!(tool.tool_type, ToolType::EndMillFlat);
    assert_eq!(tool.number, 100);
}
