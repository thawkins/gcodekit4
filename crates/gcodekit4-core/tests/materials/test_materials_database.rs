// Integration tests for Materials Database module
//
// Tests material creation, library operations, and cutting parameter management

use gcodekit4_core::data::materials::*;

#[test]
fn test_materials_database_creation() {
    let library = init_standard_library();
    assert!(
        library.len() >= 3,
        "Library should contain at least 3 materials"
    );
}

#[test]
fn test_material_add_and_remove() {
    let mut library = MaterialLibrary::new();
    let material = Material::new(
        MaterialId("test_mat".to_string()),
        "Test Material".to_string(),
        MaterialCategory::Wood,
        "Test".to_string(),
    );

    assert_eq!(library.len(), 0);
    library.add_material(material);
    assert_eq!(library.len(), 1);

    let removed = library.remove_material(&MaterialId("test_mat".to_string()));
    assert!(removed.is_some());
    assert_eq!(library.len(), 0);
}

#[test]
fn test_material_search_by_name() {
    let library = init_standard_library();

    let oak_results = library.search_by_name("oak");
    assert!(!oak_results.is_empty(), "Should find oak materials");

    let aluminum_results = library.search_by_name("aluminum");
    assert!(
        !aluminum_results.is_empty(),
        "Should find aluminum materials"
    );

    let not_found = library.search_by_name("unobtainium");
    assert!(
        not_found.is_empty(),
        "Should not find non-existent material"
    );
}

#[test]
fn test_material_category_filtering() {
    let library = init_standard_library();

    let wood_mats = library.get_materials_by_category(MaterialCategory::Wood);
    assert!(!wood_mats.is_empty(), "Should have wood materials");

    let plastic_mats = library.get_materials_by_category(MaterialCategory::Plastic);
    assert!(!plastic_mats.is_empty(), "Should have plastic materials");

    let metal_mats = library.get_materials_by_category(MaterialCategory::NonFerrousMetal);
    assert!(!metal_mats.is_empty(), "Should have non-ferrous metals");
}

#[test]
fn test_cutting_parameters_for_material() {
    let mut material = Material::new(
        MaterialId("param_test".to_string()),
        "Param Test".to_string(),
        MaterialCategory::Wood,
        "Test".to_string(),
    );

    let mut params = CuttingParameters::default();
    params.rpm_range = (16000, 20000);
    params.feed_rate_range = (1200.0, 2000.0);
    params.max_doc = 6.0;

    material.set_cutting_params("endmill_flat".to_string(), params);

    let retrieved = material.get_cutting_params("endmill_flat");
    assert!(retrieved.is_some(), "Should retrieve cutting parameters");

    if let Some(p) = retrieved {
        assert_eq!(p.rpm_range, (16000, 20000));
        assert_eq!(p.max_doc, 6.0);
    }
}

#[test]
fn test_material_properties() {
    let library = init_standard_library();
    let oak = library.get_material(&MaterialId("wood_oak_red".to_string()));

    assert!(oak.is_some(), "Should find red oak material");

    if let Some(m) = oak {
        assert_eq!(m.name, "Red Oak");
        assert_eq!(m.category, MaterialCategory::Wood);
        assert!(m.machinability_rating > 0);
        assert_eq!(m.machinability_desc(), "Easy");
    }
}

#[test]
fn test_aluminum_material_properties() {
    let library = init_standard_library();
    let aluminum = library.get_material(&MaterialId("metal_al_6061".to_string()));

    assert!(aluminum.is_some(), "Should find aluminum 6061");

    if let Some(m) = aluminum {
        assert_eq!(m.name, "Aluminum 6061");
        assert_eq!(m.category, MaterialCategory::NonFerrousMetal);
        assert!(m.coolant_required, "Aluminum should require coolant");
        assert!(m.required_ppe.contains(&PPE::EyeProtection));
    }
}

#[test]
fn test_acrylic_material_properties() {
    let library = init_standard_library();
    let acrylic = library.get_material(&MaterialId("plastic_acrylic".to_string()));

    assert!(acrylic.is_some(), "Should find acrylic material");

    if let Some(m) = acrylic {
        assert_eq!(m.name, "Acrylic");
        assert_eq!(m.category, MaterialCategory::Plastic);
        assert_eq!(m.surface_finish, SurfaceFinishability::Excellent);
        assert_eq!(m.heat_sensitivity, HeatSensitivity::High);
    }
}

#[test]
fn test_custom_material_creation() {
    let mut material = Material::new(
        MaterialId("custom_material".to_string()),
        "Custom Material".to_string(),
        MaterialCategory::Plastic,
        "Custom".to_string(),
    );
    material.custom = true;

    assert!(material.custom, "Material should be marked as custom");
    assert_eq!(material.id.0, "custom_material");
}

#[test]
fn test_material_library_get_all() {
    let library = init_standard_library();
    let all_materials = library.get_all_materials();

    assert!(!all_materials.is_empty(), "Library should have materials");
    assert_eq!(all_materials.len(), library.len());
}

#[test]
fn test_material_library_mutable_access() {
    let mut library = MaterialLibrary::new();
    let material = Material::new(
        MaterialId("mut_test".to_string()),
        "Mutable Test".to_string(),
        MaterialCategory::Wood,
        "Test".to_string(),
    );
    library.add_material(material);

    let mat_mut = library.get_material_mut(&MaterialId("mut_test".to_string()));
    assert!(mat_mut.is_some());

    if let Some(m) = mat_mut {
        m.notes = "Modified notes".to_string();
    }

    let mat_check = library.get_material(&MaterialId("mut_test".to_string()));
    if let Some(m) = mat_check {
        assert_eq!(m.notes, "Modified notes");
    }
}

#[test]
fn test_hazard_levels() {
    let material = Material::new(
        MaterialId("hazard_test".to_string()),
        "Hazard Test".to_string(),
        MaterialCategory::Wood,
        "Test".to_string(),
    );

    assert_eq!(material.dust_hazard, HazardLevel::Minimal);
    assert_eq!(material.fume_hazard, HazardLevel::None);
}

#[test]
fn test_chip_type_variations() {
    let mut material = Material::new(
        MaterialId("chip_test".to_string()),
        "Chip Test".to_string(),
        MaterialCategory::NonFerrousMetal,
        "Test".to_string(),
    );

    material.chip_type = ChipType::Continuous;
    assert_eq!(material.chip_type, ChipType::Continuous);

    material.chip_type = ChipType::Segmented;
    assert_eq!(material.chip_type, ChipType::Segmented);
}

#[test]
fn test_coolant_types() {
    let mut params = CuttingParameters::default();
    params.coolant_type = CoolantType::WaterSoluble;
    assert_eq!(params.coolant_type, CoolantType::WaterSoluble);

    params.coolant_type = CoolantType::MineralOil;
    assert_eq!(params.coolant_type, CoolantType::MineralOil);
}

#[test]
fn test_material_category_display() {
    assert_eq!(MaterialCategory::Wood.to_string(), "Wood");
    assert_eq!(
        MaterialCategory::NonFerrousMetal.to_string(),
        "Non-Ferrous Metal"
    );
    assert_eq!(
        MaterialCategory::EngineeredWood.to_string(),
        "Engineered Wood"
    );
}

#[test]
fn test_multiple_cutting_parameters() {
    let mut material = Material::new(
        MaterialId("multi_param_test".to_string()),
        "Multi Param Test".to_string(),
        MaterialCategory::Wood,
        "Test".to_string(),
    );

    let mut endmill_params = CuttingParameters::default();
    endmill_params.rpm_range = (16000, 20000);
    material.set_cutting_params("endmill_flat".to_string(), endmill_params);

    let mut vbit_params = CuttingParameters::default();
    vbit_params.rpm_range = (12000, 15000);
    material.set_cutting_params("vbit".to_string(), vbit_params);

    assert!(material.get_cutting_params("endmill_flat").is_some());
    assert!(material.get_cutting_params("vbit").is_some());
    assert!(material.get_cutting_params("ballnose").is_none());
}

#[test]
fn test_machinability_ratings_and_descriptions() {
    let mut material = Material::new(
        MaterialId("machinability_test".to_string()),
        "Machinability Test".to_string(),
        MaterialCategory::Wood,
        "Test".to_string(),
    );

    // Test all machinability levels
    let test_cases = vec![
        (1, "Very Difficult"),
        (2, "Very Difficult"),
        (3, "Difficult"),
        (4, "Difficult"),
        (5, "Moderate"),
        (6, "Moderate"),
        (7, "Easy"),
        (8, "Easy"),
        (9, "Very Easy"),
        (10, "Very Easy"),
    ];

    for (rating, expected_desc) in test_cases {
        material.machinability_rating = rating;
        assert_eq!(material.machinability_desc(), expected_desc);
    }
}

#[test]
fn test_material_case_insensitive_search() {
    let library = init_standard_library();

    let results_lower = library.search_by_name("aluminum");
    let results_upper = library.search_by_name("ALUMINUM");
    let results_mixed = library.search_by_name("AlUmInUm");

    assert_eq!(results_lower.len(), results_upper.len());
    assert_eq!(results_lower.len(), results_mixed.len());
}

#[test]
fn test_material_library_empty() {
    let library = MaterialLibrary::new();
    assert!(library.is_empty());
    assert_eq!(library.len(), 0);
    assert!(library.get_all_materials().is_empty());
}

#[test]
fn test_material_density_values() {
    let library = init_standard_library();

    let oak = library.get_material(&MaterialId("wood_oak_red".to_string()));
    if let Some(m) = oak {
        // Oak should have density around 750 kg/m³
        assert!(m.density > 700.0 && m.density < 800.0);
    }

    let aluminum = library.get_material(&MaterialId("metal_al_6061".to_string()));
    if let Some(m) = aluminum {
        // Aluminum should have density around 2700 kg/m³
        assert!(m.density > 2600.0 && m.density < 2800.0);
    }

    let acrylic = library.get_material(&MaterialId("plastic_acrylic".to_string()));
    if let Some(m) = acrylic {
        // Acrylic should have density around 1190 kg/m³
        assert!(m.density > 1100.0 && m.density < 1300.0);
    }
}

#[test]
fn test_cutting_parameter_defaults() {
    let params = CuttingParameters::default();

    assert!(params.rpm_range.0 > 0);
    assert!(params.rpm_range.1 > params.rpm_range.0);
    assert!(params.feed_rate_range.0 > 0.0);
    assert!(params.feed_rate_range.1 > params.feed_rate_range.0);
    assert!(params.plunge_rate_percent > 0.0 && params.plunge_rate_percent <= 100.0);
    assert!(params.max_doc > 0.0);
    assert!(params.stepover_percent.0 > 0.0 && params.stepover_percent.1 > 0.0);
}

#[test]
fn test_pse_requirements() {
    let library = init_standard_library();

    let oak = library.get_material(&MaterialId("wood_oak_red".to_string()));
    if let Some(m) = oak {
        assert!(m.required_ppe.contains(&PPE::EyeProtection));
    }

    let aluminum = library.get_material(&MaterialId("metal_al_6061".to_string()));
    if let Some(m) = aluminum {
        assert!(m.required_ppe.contains(&PPE::EyeProtection));
        assert!(m.required_ppe.contains(&PPE::HearingProtection));
    }
}
