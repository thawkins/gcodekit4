// Integration tests for CAM Tools Palette module
//
// Tests tool creation, library operations, and parameter management

use gcodekit4::data::tools::*;

#[test]
fn test_tools_library_creation() {
    let library = init_standard_library();
    assert!(library.len() >= 5, "Library should contain at least 5 tools");
}

#[test]
fn test_tool_add_and_remove() {
    let mut library = ToolLibrary::new();
    let tool = Tool::new(
        ToolId("test_tool".to_string()),
        1,
        "Test Tool".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );

    assert_eq!(library.len(), 0);
    library.add_tool(tool);
    assert_eq!(library.len(), 1);

    let removed = library.remove_tool(&ToolId("test_tool".to_string()));
    assert!(removed.is_some());
    assert_eq!(library.len(), 0);
}

#[test]
fn test_tool_search_by_name() {
    let library = init_standard_library();

    let flat_results = library.search_by_name("flat");
    assert!(!flat_results.is_empty(), "Should find flat end mills");

    let vbit_results = library.search_by_name("v-bit");
    assert!(!vbit_results.is_empty(), "Should find V-bits");

    let not_found = library.search_by_name("nonexistent");
    assert!(not_found.is_empty(), "Should not find non-existent tool");
}

#[test]
fn test_tool_type_filtering() {
    let library = init_standard_library();

    let flat_mills = library.get_tools_by_type(ToolType::EndMillFlat);
    assert!(!flat_mills.is_empty(), "Should have flat end mills");

    let vbits = library.get_tools_by_type(ToolType::VBit);
    assert!(!vbits.is_empty(), "Should have V-bits");

    let drills = library.get_tools_by_type(ToolType::DrillBit);
    assert!(!drills.is_empty(), "Should have drill bits");
}

#[test]
fn test_tool_diameter_search() {
    let library = init_standard_library();

    let small_tools = library.search_by_diameter(0.0, 4.0);
    assert!(!small_tools.is_empty(), "Should find small diameter tools");

    let large_tools = library.search_by_diameter(6.0, 10.0);
    assert!(!large_tools.is_empty(), "Should find large diameter tools");

    let mid_range = library.search_by_diameter(3.0, 7.0);
    assert!(!mid_range.is_empty(), "Should find mid-range tools");
}

#[test]
fn test_cutting_parameters_for_tool() {
    let mut tool = Tool::new(
        ToolId("param_test".to_string()),
        1,
        "Param Test".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );

    let mut params = ToolCuttingParams::default();
    params.rpm = 18000;
    params.feed_rate = 1500.0;

    tool.params = params;

    assert_eq!(tool.params.rpm, 18000);
    assert_eq!(tool.params.feed_rate, 1500.0);
}

#[test]
fn test_tool_properties() {
    let library = init_standard_library();
    let flat_mill = library.get_tool(&ToolId("tool_1_4_flat".to_string()));

    assert!(flat_mill.is_some(), "Should find 1/4 flat end mill");

    if let Some(t) = flat_mill {
        assert_eq!(t.name, "1/4\" Flat End Mill");
        assert_eq!(t.tool_type, ToolType::EndMillFlat);
        assert_eq!(t.diameter, 6.35);
        assert_eq!(t.flutes, 2);
    }
}

#[test]
fn test_vbit_tool_properties() {
    let library = init_standard_library();
    let vbit = library.get_tool(&ToolId("tool_vbit_90".to_string()));

    assert!(vbit.is_some(), "Should find 90 degree V-bit");

    if let Some(t) = vbit {
        assert_eq!(t.name, "90° V-Bit");
        assert_eq!(t.tool_type, ToolType::VBit);
        assert_eq!(t.tip_angle, Some(90.0));
        assert_eq!(t.flutes, 1);
    }
}

#[test]
fn test_drill_bit_properties() {
    let library = init_standard_library();
    let drill = library.get_tool(&ToolId("tool_drill_1_4".to_string()));

    assert!(drill.is_some(), "Should find 1/4 drill bit");

    if let Some(t) = drill {
        assert_eq!(t.tool_type, ToolType::DrillBit);
        assert_eq!(t.material, ToolMaterial::HSS);
        assert!(t.tip_angle.is_some());
    }
}

#[test]
fn test_custom_tool_creation() {
    let mut tool = Tool::new(
        ToolId("custom_tool".to_string()),
        99,
        "Custom Tool".to_string(),
        ToolType::Specialty,
        5.0,
        45.0,
    );
    tool.custom = true;

    assert!(tool.custom, "Tool should be marked as custom");
    assert_eq!(tool.id.0, "custom_tool");
}

#[test]
fn test_tool_library_get_all() {
    let library = init_standard_library();
    let all_tools = library.get_all_tools();

    assert!(!all_tools.is_empty(), "Library should have tools");
    assert_eq!(all_tools.len(), library.len());
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
fn test_tool_material_types() {
    let library = init_standard_library();

    // HSS drill bit
    let drill = library.get_tool(&ToolId("tool_drill_1_4".to_string()));
    if let Some(t) = drill {
        assert_eq!(t.material, ToolMaterial::HSS);
    }

    // Carbide end mill
    let endmill = library.get_tool(&ToolId("tool_1_4_flat".to_string()));
    if let Some(t) = endmill {
        assert_eq!(t.material, ToolMaterial::Carbide);
    }
}

#[test]
fn test_tool_coating_types() {
    let library = init_standard_library();

    let endmill = library.get_tool(&ToolId("tool_1_4_flat".to_string()));
    if let Some(t) = endmill {
        assert!(t.coating.is_some());
        assert_eq!(t.coating, Some(ToolCoating::TiN));
    }

    let ball_mill = library.get_tool(&ToolId("tool_1_8_ball".to_string()));
    if let Some(t) = ball_mill {
        assert!(t.coating.is_some());
        assert_eq!(t.coating, Some(ToolCoating::TiAlN));
    }
}

#[test]
fn test_tool_flute_count() {
    let library = init_standard_library();

    let vbit = library.get_tool(&ToolId("tool_vbit_90".to_string()));
    if let Some(t) = vbit {
        assert_eq!(t.flutes, 1, "V-bit should have 1 flute");
    }

    let flat_mill = library.get_tool(&ToolId("tool_1_4_flat".to_string()));
    if let Some(t) = flat_mill {
        assert_eq!(t.flutes, 2, "Flat mill should have 2 flutes");
    }
}

#[test]
fn test_tool_next_number() {
    let mut library = ToolLibrary::new();
    assert_eq!(library.next_tool_number(), 1);

    let tool = Tool::new(
        ToolId("test".to_string()),
        10,
        "Test".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );
    library.add_tool(tool);
    assert_eq!(library.next_tool_number(), 11);
}

#[test]
fn test_tool_case_insensitive_search() {
    let library = init_standard_library();

    let results_lower = library.search_by_name("drill");
    let results_upper = library.search_by_name("DRILL");
    let results_mixed = library.search_by_name("DrILl");

    assert_eq!(results_lower.len(), results_upper.len());
    assert_eq!(results_lower.len(), results_mixed.len());
}

#[test]
fn test_tool_library_empty() {
    let library = ToolLibrary::new();
    assert!(library.is_empty());
    assert_eq!(library.len(), 0);
    assert!(library.get_all_tools().is_empty());
}

#[test]
fn test_standard_tools_have_valid_params() {
    let library = init_standard_library();

    for tool in library.get_all_tools() {
        assert!(tool.params.rpm > 0, "Tool {} should have non-zero RPM", tool.name);
        assert!(
            tool.params.feed_rate > 0.0,
            "Tool {} should have non-zero feed rate",
            tool.name
        );
        assert!(
            tool.params.plunge_rate > 0.0,
            "Tool {} should have non-zero plunge rate",
            tool.name
        );
    }
}

#[test]
fn test_tool_diameter_conversion() {
    // 1/4 inch = 6.35 mm
    let tool = Tool::new(
        ToolId("quarter_inch".to_string()),
        1,
        "1/4 inch tool".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );

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
    assert!(desc.contains("2 flutes"));
}

#[test]
fn test_tool_cost_tracking() {
    let mut tool = Tool::new(
        ToolId("cost_test".to_string()),
        1,
        "Cost Test".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );

    tool.cost = Some(15.50);
    assert!(tool.cost.is_some());
    assert_eq!(tool.cost, Some(15.50));
}

#[test]
fn test_tool_manufacturer_info() {
    let mut tool = Tool::new(
        ToolId("mfg_test".to_string()),
        1,
        "MFG Test".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );

    tool.manufacturer = Some("Amana".to_string());
    tool.part_number = Some("45100-Z".to_string());

    assert_eq!(tool.manufacturer, Some("Amana".to_string()));
    assert_eq!(tool.part_number, Some("45100-Z".to_string()));
}

#[test]
fn test_all_standard_tools_accessible() {
    let library = init_standard_library();

    // Check all known tools are present
    assert!(library.get_tool(&ToolId("tool_1_4_flat".to_string())).is_some());
    assert!(library.get_tool(&ToolId("tool_1_8_flat".to_string())).is_some());
    assert!(library.get_tool(&ToolId("tool_vbit_90".to_string())).is_some());
    assert!(library.get_tool(&ToolId("tool_drill_1_4".to_string())).is_some());
    assert!(library.get_tool(&ToolId("tool_1_8_ball".to_string())).is_some());
}

#[test]
fn test_tool_type_coverage() {
    let library = init_standard_library();

    // Check that we have multiple tool types represented
    let types: Vec<ToolType> = library
        .get_all_tools()
        .iter()
        .map(|t| t.tool_type)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    assert!(
        types.len() > 2,
        "Should have at least 3 different tool types in standard library"
    );
}

#[test]
fn test_tool_rpm_ranges_reasonable() {
    let library = init_standard_library();

    for tool in library.get_all_tools() {
        assert!(
            tool.params.rpm_range.0 > 0,
            "Tool {} should have positive min RPM",
            tool.name
        );
        assert!(
            tool.params.rpm_range.1 > tool.params.rpm_range.0,
            "Tool {} max RPM should exceed min RPM",
            tool.name
        );
        assert!(
            tool.params.rpm > 0,
            "Tool {} should have positive recommended RPM",
            tool.name
        );
    }
}

#[test]
fn test_tool_feed_rate_relationships() {
    let library = init_standard_library();

    for tool in library.get_all_tools() {
        assert!(
            tool.params.plunge_rate <= tool.params.feed_rate,
            "Tool {} plunge rate should not exceed feed rate",
            tool.name
        );
    }
}
