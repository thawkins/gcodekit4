use gcodekit4_core::data::tools::*;

#[test]
fn test_tool_id_display() {
    let id = ToolId("test_tool".to_string());
    assert_eq!(id.to_string(), "test_tool");
}

#[test]
fn test_tool_creation() {
    let tool = Tool::new(
        ToolId("test".to_string()),
        1,
        "Test Tool".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );

    assert_eq!(tool.id.0, "test");
    assert_eq!(tool.name, "Test Tool");
    assert_eq!(tool.tool_type, ToolType::EndMillFlat);
    assert_eq!(tool.diameter, 6.35);
}

#[test]
fn test_tool_description_short() {
    let tool = Tool::new(
        ToolId("test".to_string()),
        1,
        "Test Tool".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );

    let desc = tool.description_short();
    assert!(desc.contains("Test Tool"));
    assert!(desc.contains("6.35"));
    assert!(desc.contains("50"));
}

#[test]
fn test_tool_cutting_params_default() {
    let params = ToolCuttingParams::default();
    assert!(params.rpm > 0);
    assert!(params.feed_rate > 0.0);
    assert!(params.plunge_rate > 0.0);
}

#[test]
fn test_tool_library_add_and_get() {
    let mut library = ToolLibrary::new();
    let tool = Tool::new(
        ToolId("test".to_string()),
        1,
        "Test".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );

    library.add_tool(tool);
    assert_eq!(library.len(), 1);

    let retrieved = library.get_tool(&ToolId("test".to_string()));
    assert!(retrieved.is_some());
}

#[test]
fn test_tool_library_search() {
    let library = init_standard_library();
    let results = library.search_by_name("flat");
    assert!(!results.is_empty());
    assert!(results.iter().any(|t| t.name.contains("Flat")));
}

#[test]
fn test_tool_library_type_filter() {
    let library = init_standard_library();
    let end_mills = library.get_tools_by_type(ToolType::EndMillFlat);
    assert!(!end_mills.is_empty());

    let vbits = library.get_tools_by_type(ToolType::VBit);
    assert!(!vbits.is_empty());
}

#[test]
fn test_standard_library_initialization() {
    let library = init_standard_library();
    assert!(library.len() >= 5);

    assert!(library
        .get_tool(&ToolId("tool_1_4_flat".to_string()))
        .is_some());
    assert!(library
        .get_tool(&ToolId("tool_vbit_90".to_string()))
        .is_some());
    assert!(library
        .get_tool(&ToolId("tool_drill_1_4".to_string()))
        .is_some());
}

#[test]
fn test_tool_library_diameter_search() {
    let library = init_standard_library();
    let small_tools = library.search_by_diameter(0.0, 4.0);
    assert!(!small_tools.is_empty());

    let large_tools = library.search_by_diameter(6.0, 8.0);
    assert!(!large_tools.is_empty());
}

#[test]
fn test_tool_library_remove() {
    let mut library = init_standard_library();
    let initial_count = library.len();

    let removed = library.remove_tool(&ToolId("tool_1_4_flat".to_string()));
    assert!(removed.is_some());
    assert_eq!(library.len(), initial_count - 1);
}

#[test]
fn test_tool_library_mutable_access() {
    let mut library = ToolLibrary::new();
    let tool = Tool::new(
        ToolId("mut_test".to_string()),
        1,
        "Mutable Test".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );
    library.add_tool(tool);

    let tool_mut = library.get_tool_mut(&ToolId("mut_test".to_string()));
    assert!(tool_mut.is_some());

    if let Some(t) = tool_mut {
        t.notes = "Modified notes".to_string();
    }

    let tool_check = library.get_tool(&ToolId("mut_test".to_string()));
    if let Some(t) = tool_check {
        assert_eq!(t.notes, "Modified notes");
    }
}

#[test]
fn test_tool_material_display() {
    assert_eq!(ToolMaterial::HSS.to_string(), "HSS");
    assert_eq!(ToolMaterial::Carbide.to_string(), "Carbide");
}

#[test]
fn test_tool_type_display() {
    assert_eq!(ToolType::EndMillFlat.to_string(), "Flat End Mill");
    assert_eq!(ToolType::VBit.to_string(), "V-Bit");
}

#[test]
fn test_tool_library_get_all() {
    let library = init_standard_library();
    let all_tools = library.get_all_tools();
    assert!(!all_tools.is_empty());
    assert_eq!(all_tools.len(), library.len());
}

#[test]
fn test_tool_next_number() {
    let mut library = ToolLibrary::new();
    assert_eq!(library.next_tool_number(), 1);

    let tool = Tool::new(
        ToolId("test".to_string()),
        5,
        "Test".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );
    library.add_tool(tool);
    assert_eq!(library.next_tool_number(), 6);
}

#[test]
fn test_tool_case_insensitive_search() {
    let library = init_standard_library();

    let results_lower = library.search_by_name("flat");
    let results_upper = library.search_by_name("FLAT");
    let results_mixed = library.search_by_name("FLat");

    assert_eq!(results_lower.len(), results_upper.len());
    assert_eq!(results_lower.len(), results_mixed.len());
}
