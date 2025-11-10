//! CAM Tools Palette module - Tool definitions and library management
//!
//! This module provides:
//! - Tool types and categories
//! - Tool geometry and specifications
//! - Tool library management (add, remove, search, filter)
//! - Material-specific tool cutting parameters
//! - Standard tool library initialization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tool types for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ToolType {
    /// Flat end mill
    EndMillFlat,
    /// Ball end mill / ball nose
    EndMillBall,
    /// Corner radius end mill
    EndMillCornerRadius,
    /// V-bit engraving tool
    VBit,
    /// Drill bit (twist drill)
    DrillBit,
    /// Spot drill
    SpotDrill,
    /// Engraving tool
    EngravingBit,
    /// Chamfer tool
    ChamferTool,
    /// Specialty tool
    Specialty,
}

impl std::fmt::Display for ToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndMillFlat => write!(f, "Flat End Mill"),
            Self::EndMillBall => write!(f, "Ball End Mill"),
            Self::EndMillCornerRadius => write!(f, "Corner Radius End Mill"),
            Self::VBit => write!(f, "V-Bit"),
            Self::DrillBit => write!(f, "Drill Bit"),
            Self::SpotDrill => write!(f, "Spot Drill"),
            Self::EngravingBit => write!(f, "Engraving Bit"),
            Self::ChamferTool => write!(f, "Chamfer Tool"),
            Self::Specialty => write!(f, "Specialty"),
        }
    }
}

/// Tool material composition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolMaterial {
    /// High Speed Steel
    HSS,
    /// Carbide
    Carbide,
    /// Coated carbide
    CoatedCarbide,
    /// Diamond coated
    Diamond,
}

impl std::fmt::Display for ToolMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HSS => write!(f, "HSS"),
            Self::Carbide => write!(f, "Carbide"),
            Self::CoatedCarbide => write!(f, "Coated Carbide"),
            Self::Diamond => write!(f, "Diamond Coated"),
        }
    }
}

/// Tool coating type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolCoating {
    /// Titanium Nitride coating
    TiN,
    /// Titanium Aluminum Nitride coating
    TiAlN,
    /// Diamond-like carbon coating
    DLC,
    /// Aluminum Oxide coating
    AlOx,
}

impl std::fmt::Display for ToolCoating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TiN => write!(f, "TiN"),
            Self::TiAlN => write!(f, "TiAlN"),
            Self::DLC => write!(f, "DLC"),
            Self::AlOx => write!(f, "Al2O3"),
        }
    }
}

/// Shank type for tool holder compatibility
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShankType {
    /// Straight shank (various diameters)
    Straight(u32), // diameter in 1/10mm units (e.g., 60 = 6.0mm)
    /// Tapered shank
    Tapered,
    /// Collet size (e.g., ER-20, ER-25)
    Collet,
}

/// Tool identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ToolId(pub String);

impl std::fmt::Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tool default cutting parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCuttingParams {
    /// Recommended RPM
    pub rpm: u32,
    /// RPM range (min, max)
    pub rpm_range: (u32, u32),
    /// Default feed rate in mm/min
    pub feed_rate: f32,
    /// Default plunge rate in mm/min
    pub plunge_rate: f32,
    /// Default stepover as percentage of diameter
    pub stepover_percent: f32,
    /// Default depth per pass in mm
    pub depth_per_pass: f32,
}

impl Default for ToolCuttingParams {
    fn default() -> Self {
        Self {
            rpm: 12000,
            rpm_range: (8000, 18000),
            feed_rate: 1500.0,
            plunge_rate: 750.0,
            stepover_percent: 50.0,
            depth_per_pass: 3.0,
        }
    }
}

/// Complete tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Unique tool identifier
    pub id: ToolId,
    /// Tool number (for reference)
    pub number: u32,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Tool type
    pub tool_type: ToolType,

    // Geometry
    /// Cutting diameter in mm
    pub diameter: f32,
    /// Shaft diameter in mm (if different)
    pub shaft_diameter: Option<f32>,
    /// Overall length in mm
    pub length: f32,
    /// Flute length in mm
    pub flute_length: f32,
    /// Number of flutes
    pub flutes: u32,
    /// Corner radius in mm (for corner radius end mills)
    pub corner_radius: Option<f32>,
    /// Tip angle in degrees (for v-bits, drills)
    pub tip_angle: Option<f32>,

    // Material specs
    /// Tool material composition
    pub material: ToolMaterial,
    /// Optional coating
    pub coating: Option<ToolCoating>,
    /// Shank type
    pub shank: ShankType,

    // Parameters
    /// Default cutting parameters
    pub params: ToolCuttingParams,

    // Metadata
    /// Manufacturer name
    pub manufacturer: Option<String>,
    /// Manufacturer part number
    pub part_number: Option<String>,
    /// Cost per unit
    pub cost: Option<f32>,
    /// Notes and tips
    pub notes: String,
    /// Whether this is a user-defined custom tool
    pub custom: bool,
}

impl Tool {
    /// Create a new tool with basic properties
    pub fn new(
        id: ToolId,
        number: u32,
        name: String,
        tool_type: ToolType,
        diameter: f32,
        length: f32,
    ) -> Self {
        Self {
            id,
            number,
            name,
            description: String::new(),
            tool_type,
            diameter,
            shaft_diameter: None,
            length,
            flute_length: length - 10.0,
            flutes: 2,
            corner_radius: None,
            tip_angle: None,
            material: ToolMaterial::Carbide,
            coating: Some(ToolCoating::TiN),
            shank: ShankType::Collet,
            params: ToolCuttingParams::default(),
            manufacturer: None,
            part_number: None,
            cost: None,
            notes: String::new(),
            custom: false,
        }
    }

    /// Get a descriptive string for the tool
    pub fn description_short(&self) -> String {
        format!(
            "{} - {} dia x {} length, {} flutes",
            self.name, self.diameter, self.length, self.flutes
        )
    }

    /// Check if tool is suitable for a specific material
    pub fn is_suitable_for_material(&self, material_category: &str) -> bool {
        // For now, most tools work with most materials
        // This can be expanded with material compatibility rules
        !material_category.is_empty()
    }
}

/// Tool library - manages collection of tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolLibrary {
    /// Collection of tools by ID
    tools: HashMap<ToolId, Tool>,
    /// Next available tool number
    next_tool_number: u32,
}

impl ToolLibrary {
    /// Create a new empty tool library
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            next_tool_number: 1,
        }
    }

    /// Add a tool to the library
    pub fn add_tool(&mut self, tool: Tool) {
        if tool.number >= self.next_tool_number {
            self.next_tool_number = tool.number + 1;
        }
        self.tools.insert(tool.id.clone(), tool);
    }

    /// Get a tool by ID
    pub fn get_tool(&self, id: &ToolId) -> Option<&Tool> {
        self.tools.get(id)
    }

    /// Get a mutable reference to a tool
    pub fn get_tool_mut(&mut self, id: &ToolId) -> Option<&mut Tool> {
        self.tools.get_mut(id)
    }

    /// Remove a tool from the library
    pub fn remove_tool(&mut self, id: &ToolId) -> Option<Tool> {
        self.tools.remove(id)
    }

    /// Get all tools
    pub fn get_all_tools(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }

    /// Get tools by type
    pub fn get_tools_by_type(&self, tool_type: ToolType) -> Vec<&Tool> {
        self.tools
            .values()
            .filter(|t| t.tool_type == tool_type)
            .collect()
    }

    /// Search tools by name (partial match, case-insensitive)
    pub fn search_by_name(&self, query: &str) -> Vec<&Tool> {
        let query_lower = query.to_lowercase();
        self.tools
            .values()
            .filter(|t| {
                t.name.to_lowercase().contains(&query_lower)
                    || t.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Search tools by diameter range
    pub fn search_by_diameter(&self, min: f32, max: f32) -> Vec<&Tool> {
        self.tools
            .values()
            .filter(|t| t.diameter >= min && t.diameter <= max)
            .collect()
    }

    /// Get the next available tool number
    pub fn next_tool_number(&self) -> u32 {
        self.next_tool_number
    }

    /// Get the number of tools in the library
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if library is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize standard tool library with common tools
pub fn init_standard_library() -> ToolLibrary {
    let mut library = ToolLibrary::new();

    // 1/4" Flat End Mill
    let mut tool1 = Tool::new(
        ToolId("tool_1_4_flat".to_string()),
        1,
        "1/4\" Flat End Mill".to_string(),
        ToolType::EndMillFlat,
        6.35,
        50.0,
    );
    tool1.flutes = 2;
    tool1.flute_length = 40.0;
    tool1.material = ToolMaterial::Carbide;
    tool1.coating = Some(ToolCoating::TiN);
    tool1.manufacturer = Some("Generic".to_string());
    tool1.params.rpm = 18000;
    tool1.params.rpm_range = (12000, 24000);
    tool1.params.feed_rate = 1500.0;
    library.add_tool(tool1);

    // 1/8" Flat End Mill
    let mut tool2 = Tool::new(
        ToolId("tool_1_8_flat".to_string()),
        2,
        "1/8\" Flat End Mill".to_string(),
        ToolType::EndMillFlat,
        3.175,
        45.0,
    );
    tool2.flutes = 2;
    tool2.flute_length = 35.0;
    tool2.material = ToolMaterial::Carbide;
    tool2.coating = Some(ToolCoating::TiN);
    tool2.params.rpm = 24000;
    tool2.params.rpm_range = (18000, 30000);
    tool2.params.feed_rate = 1000.0;
    library.add_tool(tool2);

    // 90 degree V-Bit
    let mut tool3 = Tool::new(
        ToolId("tool_vbit_90".to_string()),
        3,
        "90° V-Bit".to_string(),
        ToolType::VBit,
        6.0,
        50.0,
    );
    tool3.flutes = 1;
    tool3.tip_angle = Some(90.0);
    tool3.material = ToolMaterial::Carbide;
    tool3.coating = Some(ToolCoating::TiN);
    tool3.params.rpm = 20000;
    tool3.params.rpm_range = (15000, 25000);
    tool3.params.feed_rate = 1200.0;
    tool3.params.depth_per_pass = 2.0;
    library.add_tool(tool3);

    // 1/4" Drill Bit
    let mut tool4 = Tool::new(
        ToolId("tool_drill_1_4".to_string()),
        4,
        "1/4\" Drill Bit".to_string(),
        ToolType::DrillBit,
        6.35,
        60.0,
    );
    tool4.flutes = 2;
    tool4.tip_angle = Some(118.0);
    tool4.material = ToolMaterial::HSS;
    tool4.params.rpm = 3000;
    tool4.params.rpm_range = (2000, 4000);
    tool4.params.feed_rate = 300.0;
    tool4.params.plunge_rate = 300.0;
    library.add_tool(tool4);

    // Ball End Mill 1/8"
    let mut tool5 = Tool::new(
        ToolId("tool_1_8_ball".to_string()),
        5,
        "1/8\" Ball End Mill".to_string(),
        ToolType::EndMillBall,
        3.175,
        45.0,
    );
    tool5.flutes = 2;
    tool5.flute_length = 35.0;
    tool5.material = ToolMaterial::Carbide;
    tool5.coating = Some(ToolCoating::TiAlN);
    tool5.params.rpm = 22000;
    tool5.params.rpm_range = (16000, 28000);
    tool5.params.feed_rate = 1200.0;
    tool5.params.stepover_percent = 20.0;
    library.add_tool(tool5);

    library
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
