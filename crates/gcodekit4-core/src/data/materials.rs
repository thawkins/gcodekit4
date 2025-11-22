//! Materials Database module
//!
//! This module provides:
//! - Material categories and types
//! - Material properties (physical, mechanical, machining, safety)
//! - Cutting parameter recommendations
//! - Material library management
//! - Custom material support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Material categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum MaterialCategory {
    /// Natural wood (hardwoods, softwoods)
    Wood,
    /// Engineered wood products
    EngineeredWood,
    /// Plastic and polymer materials
    Plastic,
    /// Non-ferrous metals (aluminum, brass, copper)
    NonFerrousMetal,
    /// Ferrous metals (steel, stainless)
    FerrousMetal,
    /// Composite materials (carbon fiber, fiberglass)
    Composite,
    /// Stone and ceramic materials
    StoneAndCeramic,
}

impl std::fmt::Display for MaterialCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wood => write!(f, "Wood"),
            Self::EngineeredWood => write!(f, "Engineered Wood"),
            Self::Plastic => write!(f, "Plastic"),
            Self::NonFerrousMetal => write!(f, "Non-Ferrous Metal"),
            Self::FerrousMetal => write!(f, "Ferrous Metal"),
            Self::Composite => write!(f, "Composite"),
            Self::StoneAndCeramic => write!(f, "Stone & Ceramic"),
        }
    }
}

/// Chip formation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChipType {
    /// Continuous chips (most metals, harder plastics)
    Continuous,
    /// Segmented chips (gray cast iron)
    Segmented,
    /// Granular or powdery chips (composites, ceramics)
    Granular,
    /// Very small, breakable chips (some plastics)
    Small,
}

/// Heat sensitivity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeatSensitivity {
    /// Low heat sensitivity (woods, most metals)
    Low,
    /// Moderate heat sensitivity
    Moderate,
    /// High heat sensitivity (thermoplastics, composites)
    High,
}

/// Abrasiveness level (effect on tool wear)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Abrasiveness {
    /// Low wear (aluminum, wood)
    Low,
    /// Moderate wear (mild steel)
    Moderate,
    /// High wear (stainless, composites)
    High,
}

/// Surface finish achievability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceFinishability {
    /// Excellent surface finish possible
    Excellent,
    /// Good surface finish with proper technique
    Good,
    /// Fair surface finish, may need secondary finishing
    Fair,
    /// Rough surface finish expected
    Rough,
}

/// Hazard levels for safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HazardLevel {
    /// No special hazard
    None,
    /// Minimal hazard
    Minimal,
    /// Moderate hazard, PPE recommended
    Moderate,
    /// High hazard, PPE required
    High,
}

/// Personal Protective Equipment requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PPE {
    /// Safety glasses/face shield
    EyeProtection,
    /// Dust mask or respirator
    Respiratory,
    /// Hearing protection
    HearingProtection,
    /// Gloves
    Gloves,
    /// Apron
    Apron,
}

/// Coolant/Lubrication type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoolantType {
    /// No coolant needed
    None,
    /// Air only (dust blowout)
    AirOnly,
    /// Mineral oil based coolant
    MineralOil,
    /// Water soluble coolant
    WaterSoluble,
    /// Synthetic coolant
    Synthetic,
}

/// Cutting parameters for a specific material and tool combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuttingParameters {
    /// RPM range (min, max)
    pub rpm_range: (u32, u32),
    /// Feed rate range in mm/min (min, max) for roughing
    pub feed_rate_range: (f32, f32),
    /// Plunge rate as percentage of feed rate (0-100)
    pub plunge_rate_percent: f32,
    /// Maximum depth of cut in mm
    pub max_doc: f32,
    /// Stepover range as percentage of tool diameter (min, max)
    pub stepover_percent: (f32, f32),
    /// Recommended surface speed in m/min (SFM equivalent)
    #[serde(default)]
    pub surface_speed_m_min: Option<f32>,
    /// Recommended chip load in mm/tooth
    #[serde(default)]
    pub chip_load_mm: Option<f32>,
    /// Recommended coolant type
    pub coolant_type: CoolantType,
    /// Notes about parameters
    pub notes: String,
}

impl Default for CuttingParameters {
    fn default() -> Self {
        Self {
            rpm_range: (12000, 18000),
            feed_rate_range: (1000.0, 2000.0),
            plunge_rate_percent: 50.0,
            max_doc: 3.0,
            stepover_percent: (40.0, 60.0),
            surface_speed_m_min: None,
            chip_load_mm: None,
            coolant_type: CoolantType::None,
            notes: String::new(),
        }
    }
}

/// Material identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct MaterialId(pub String);

impl std::fmt::Display for MaterialId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Complete material definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    /// Unique material identifier
    pub id: MaterialId,
    /// Display name
    pub name: String,
    /// Material category
    pub category: MaterialCategory,
    /// Subcategory (e.g., "Red Oak" for hardwood)
    pub subcategory: String,
    /// Brief description
    pub description: String,

    // Physical properties
    /// Density in kg/m³
    pub density: f32,
    /// Machinability rating (1-10, higher is easier)
    pub machinability_rating: u8,
    /// Tensile strength in MPa (optional)
    pub tensile_strength: Option<f32>,
    /// Melting point or glass transition temperature in °C (optional)
    pub melting_point: Option<f32>,

    // Machining characteristics
    /// Type of chips formed
    pub chip_type: ChipType,
    /// Heat sensitivity when cutting
    pub heat_sensitivity: HeatSensitivity,
    /// Tool wear factor (abrasiveness)
    pub abrasiveness: Abrasiveness,
    /// Surface finish achievable
    pub surface_finish: SurfaceFinishability,

    // Safety information
    /// Dust hazard level
    pub dust_hazard: HazardLevel,
    /// Fume hazard level
    pub fume_hazard: HazardLevel,
    /// Required PPE
    pub required_ppe: Vec<PPE>,
    /// Is coolant required?
    pub coolant_required: bool,

    // Cutting parameters for different tool types
    /// Cutting parameters (tool type -> parameters)
    pub cutting_params: HashMap<String, CuttingParameters>,

    // Metadata
    /// Whether this is a user-defined custom material
    pub custom: bool,
    /// Notes and tips
    pub notes: String,
}

impl Material {
    /// Create a new material with basic properties
    pub fn new(
        id: MaterialId,
        name: String,
        category: MaterialCategory,
        subcategory: String,
    ) -> Self {
        Self {
            id,
            name,
            category,
            subcategory,
            description: String::new(),
            density: 750.0,
            machinability_rating: 7,
            tensile_strength: None,
            melting_point: None,
            chip_type: ChipType::Continuous,
            heat_sensitivity: HeatSensitivity::Low,
            abrasiveness: Abrasiveness::Low,
            surface_finish: SurfaceFinishability::Good,
            dust_hazard: HazardLevel::Minimal,
            fume_hazard: HazardLevel::None,
            required_ppe: vec![PPE::EyeProtection],
            coolant_required: false,
            cutting_params: HashMap::new(),
            custom: false,
            notes: String::new(),
        }
    }

    /// Get cutting parameters for a specific tool type
    pub fn get_cutting_params(&self, tool_type: &str) -> Option<&CuttingParameters> {
        self.cutting_params.get(tool_type)
    }

    /// Set cutting parameters for a tool type
    pub fn set_cutting_params(&mut self, tool_type: String, params: CuttingParameters) {
        self.cutting_params.insert(tool_type, params);
    }

    /// Get machinability description
    pub fn machinability_desc(&self) -> &'static str {
        match self.machinability_rating {
            1..=2 => "Very Difficult",
            3..=4 => "Difficult",
            5..=6 => "Moderate",
            7..=8 => "Easy",
            9..=10 => "Very Easy",
            _ => "Unknown",
        }
    }
}

/// Materials library - manages collection of materials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialLibrary {
    /// Collection of materials by ID
    materials: HashMap<MaterialId, Material>,
}

impl MaterialLibrary {
    /// Create a new empty library
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }

    /// Add a material to the library
    pub fn add_material(&mut self, material: Material) {
        self.materials.insert(material.id.clone(), material);
    }

    /// Get a material by ID
    pub fn get_material(&self, id: &MaterialId) -> Option<&Material> {
        self.materials.get(id)
    }

    /// Get a mutable reference to a material
    pub fn get_material_mut(&mut self, id: &MaterialId) -> Option<&mut Material> {
        self.materials.get_mut(id)
    }

    /// Remove a material from the library
    pub fn remove_material(&mut self, id: &MaterialId) -> Option<Material> {
        self.materials.remove(id)
    }

    /// Get all materials
    pub fn get_all_materials(&self) -> Vec<&Material> {
        self.materials.values().collect()
    }

    /// Get all materials in a specific category
    pub fn get_materials_by_category(&self, category: MaterialCategory) -> Vec<&Material> {
        self.materials
            .values()
            .filter(|m| m.category == category)
            .collect()
    }

    /// Search materials by name (partial match, case-insensitive)
    pub fn search_by_name(&self, query: &str) -> Vec<&Material> {
        let query_lower = query.to_lowercase();
        self.materials
            .values()
            .filter(|m| {
                m.name.to_lowercase().contains(&query_lower)
                    || m.subcategory.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get the number of materials in the library
    pub fn len(&self) -> usize {
        self.materials.len()
    }

    /// Check if library is empty
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }
}

impl Default for MaterialLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the standard materials library with common materials
pub fn init_standard_library() -> MaterialLibrary {
    let mut library = MaterialLibrary::new();

    // Red Oak (hardwood)
    let mut red_oak = Material::new(
        MaterialId("wood_oak_red".to_string()),
        "Red Oak".to_string(),
        MaterialCategory::Wood,
        "Hardwood".to_string(),
    );
    red_oak.description = "Dense American hardwood, good for general CNC work".to_string();
    red_oak.density = 750.0;
    red_oak.machinability_rating = 8;
    red_oak.surface_finish = SurfaceFinishability::Good;
    red_oak.notes = "Good grain structure, moderate dulling of tools".to_string();

    let mut oak_params = CuttingParameters::default();
    oak_params.rpm_range = (16000, 20000);
    oak_params.feed_rate_range = (1200.0, 2000.0);
    oak_params.max_doc = 6.0;
    oak_params.stepover_percent = (40.0, 60.0);
    red_oak.set_cutting_params("endmill_flat".to_string(), oak_params);

    library.add_material(red_oak);

    // Aluminum 6061 (non-ferrous metal)
    let mut al6061 = Material::new(
        MaterialId("metal_al_6061".to_string()),
        "Aluminum 6061".to_string(),
        MaterialCategory::NonFerrousMetal,
        "Alloy".to_string(),
    );
    al6061.description = "Common aluminum alloy, excellent machinability".to_string();
    al6061.density = 2700.0;
    al6061.machinability_rating = 9;
    al6061.chip_type = ChipType::Continuous;
    al6061.heat_sensitivity = HeatSensitivity::Moderate;
    al6061.coolant_required = true;
    al6061.required_ppe = vec![PPE::EyeProtection, PPE::HearingProtection];

    let mut al_params = CuttingParameters::default();
    al_params.rpm_range = (3000, 5000);
    al_params.feed_rate_range = (1500.0, 3000.0);
    al_params.max_doc = 5.0;
    al_params.coolant_type = CoolantType::WaterSoluble;
    al6061.set_cutting_params("endmill_flat".to_string(), al_params);

    library.add_material(al6061);

    // Acrylic
    let mut acrylic = Material::new(
        MaterialId("plastic_acrylic".to_string()),
        "Acrylic".to_string(),
        MaterialCategory::Plastic,
        "PMMA".to_string(),
    );
    acrylic.description = "Clear plastic, good for engraving and cutting".to_string();
    acrylic.density = 1190.0;
    acrylic.machinability_rating = 9;
    acrylic.surface_finish = SurfaceFinishability::Excellent;
    acrylic.heat_sensitivity = HeatSensitivity::High;
    acrylic.notes = "Keep tool speed high and feed moderate to avoid melting".to_string();

    let mut acrylic_params = CuttingParameters::default();
    acrylic_params.rpm_range = (18000, 24000);
    acrylic_params.feed_rate_range = (1000.0, 1800.0);
    acrylic_params.max_doc = 3.0;
    acrylic_params.coolant_type = CoolantType::AirOnly;
    acrylic.set_cutting_params("endmill_flat".to_string(), acrylic_params);

    library.add_material(acrylic);

    library
}


