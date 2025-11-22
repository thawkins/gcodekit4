//! Tool library management for CAM operations.
//!
//! Provides definitions and management for cutting tools with their geometry,
//! cutting parameters, and material-specific settings.

use std::collections::HashMap;

/// Tool types for different operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolType {
    EndMill,
    BallNose,
    VBit,
    Drill,
    Slot,
}

impl ToolType {
    /// Returns the name of the tool type.
    pub fn name(&self) -> &'static str {
        match self {
            ToolType::EndMill => "End Mill",
            ToolType::BallNose => "Ball Nose",
            ToolType::VBit => "V-Bit",
            ToolType::Drill => "Drill",
            ToolType::Slot => "Slot Cutter",
        }
    }
}

/// Represents a cutting tool with its geometry and parameters.
#[derive(Debug, Clone)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub tool_type: ToolType,
    pub diameter: f64,
    pub flutes: u32,
    pub material: String,
    pub feed_rate: f64,
    pub plunge_rate: f64,
    pub spindle_speed: u32,
    pub max_depth_per_pass: f64,
    pub stepover: f64,
    pub coolant: CoolantType,
}

/// Types of coolant used during cutting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoolantType {
    None,
    Flood,
    Mist,
    ThroughSpindle,
}

impl CoolantType {
    /// Returns the name of the coolant type.
    pub fn name(&self) -> &'static str {
        match self {
            CoolantType::None => "None",
            CoolantType::Flood => "Flood",
            CoolantType::Mist => "Mist",
            CoolantType::ThroughSpindle => "Through Spindle",
        }
    }
}

impl Tool {
    /// Creates a new tool with the given parameters.
    pub fn new(
        id: String,
        name: String,
        tool_type: ToolType,
        diameter: f64,
        flutes: u32,
        material: String,
    ) -> Self {
        Self {
            id,
            name,
            tool_type,
            diameter,
            flutes,
            material,
            feed_rate: 100.0,
            plunge_rate: 50.0,
            spindle_speed: 10000,
            max_depth_per_pass: 5.0,
            stepover: 2.0,
            coolant: CoolantType::None,
        }
    }

    /// Sets the cutting parameters for this tool.
    pub fn set_cutting_parameters(&mut self, feed_rate: f64, plunge_rate: f64, spindle_speed: u32) {
        self.feed_rate = feed_rate;
        self.plunge_rate = plunge_rate;
        self.spindle_speed = spindle_speed;
    }

    /// Sets the depth of cut parameters.
    pub fn set_depth_parameters(&mut self, max_depth: f64, stepover: f64) {
        self.max_depth_per_pass = max_depth;
        self.stepover = stepover;
    }

    /// Sets the coolant type for this tool.
    pub fn set_coolant(&mut self, coolant: CoolantType) {
        self.coolant = coolant;
    }

    /// Calculates the number of passes needed for a given total depth.
    pub fn calculate_passes(&self, total_depth: f64) -> u32 {
        ((total_depth.abs() / self.max_depth_per_pass).ceil()) as u32
    }

    /// Estimates the machining time for a given toolpath length in mm.
    pub fn estimate_machining_time(&self, toolpath_length: f64) -> f64 {
        toolpath_length / self.feed_rate * 60.0
    }
}

/// Material definitions with recommended tool parameters.
#[derive(Debug, Clone)]
pub struct MaterialProfile {
    pub name: String,
    pub density: f64,
    pub hardness: String,
    pub recommended_tools: Vec<String>,
    pub cutting_speed: HashMap<String, f64>,
}

impl MaterialProfile {
    /// Creates a new material profile.
    pub fn new(name: String, density: f64, hardness: String) -> Self {
        Self {
            name,
            density,
            hardness,
            recommended_tools: Vec::new(),
            cutting_speed: HashMap::new(),
        }
    }

    /// Adds a recommended tool for this material.
    pub fn add_recommended_tool(&mut self, tool_id: String) {
        if !self.recommended_tools.contains(&tool_id) {
            self.recommended_tools.push(tool_id);
        }
    }

    /// Sets the cutting speed for a specific tool type.
    pub fn set_cutting_speed(&mut self, tool_name: String, speed: f64) {
        self.cutting_speed.insert(tool_name, speed);
    }
}

/// Manages a library of tools and materials.
#[derive(Debug, Clone)]
pub struct ToolLibrary {
    tools: HashMap<String, Tool>,
    materials: HashMap<String, MaterialProfile>,
    default_tool: Option<String>,
}

impl ToolLibrary {
    /// Creates a new empty tool library.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            materials: HashMap::new(),
            default_tool: None,
        }
    }

    /// Adds a tool to the library.
    pub fn add_tool(&mut self, tool: Tool) {
        let tool_id = tool.id.clone();
        self.tools.insert(tool_id.clone(), tool);
        if self.default_tool.is_none() {
            self.default_tool = Some(tool_id);
        }
    }

    /// Removes a tool from the library.
    pub fn remove_tool(&mut self, tool_id: &str) -> Option<Tool> {
        self.tools.remove(tool_id)
    }

    /// Gets a tool by ID.
    pub fn get_tool(&self, tool_id: &str) -> Option<&Tool> {
        self.tools.get(tool_id)
    }

    /// Gets a mutable reference to a tool.
    pub fn get_tool_mut(&mut self, tool_id: &str) -> Option<&mut Tool> {
        self.tools.get_mut(tool_id)
    }

    /// Gets the default tool.
    pub fn get_default_tool(&self) -> Option<&Tool> {
        self.default_tool.as_ref().and_then(|id| self.tools.get(id))
    }

    /// Sets the default tool.
    pub fn set_default_tool(&mut self, tool_id: String) -> bool {
        if self.tools.contains_key(&tool_id) {
            self.default_tool = Some(tool_id);
            true
        } else {
            false
        }
    }

    /// Lists all tools in the library.
    pub fn list_tools(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }

    /// Lists tools by type.
    pub fn list_tools_by_type(&self, tool_type: ToolType) -> Vec<&Tool> {
        self.tools
            .values()
            .filter(|t| t.tool_type == tool_type)
            .collect()
    }

    /// Adds a material profile to the library.
    pub fn add_material(&mut self, material: MaterialProfile) {
        self.materials.insert(material.name.clone(), material);
    }

    /// Gets a material profile.
    pub fn get_material(&self, name: &str) -> Option<&MaterialProfile> {
        self.materials.get(name)
    }

    /// Lists all materials in the library.
    pub fn list_materials(&self) -> Vec<&MaterialProfile> {
        self.materials.values().collect()
    }

    /// Creates a default tool library with common tools.
    pub fn with_defaults() -> Self {
        let mut library = Self::new();

        let mut end_mill = Tool::new(
            "em_125".to_string(),
            "1/8\" End Mill".to_string(),
            ToolType::EndMill,
            3.175,
            2,
            "HSS".to_string(),
        );
        end_mill.set_cutting_parameters(100.0, 50.0, 12000);
        end_mill.set_depth_parameters(5.0, 2.0);
        library.add_tool(end_mill);

        let mut ball_nose = Tool::new(
            "bn_125".to_string(),
            "1/8\" Ball Nose".to_string(),
            ToolType::BallNose,
            3.175,
            2,
            "HSS".to_string(),
        );
        ball_nose.set_cutting_parameters(80.0, 40.0, 10000);
        ball_nose.set_depth_parameters(3.0, 1.5);
        library.add_tool(ball_nose);

        let mut vbit = Tool::new(
            "vbit_90".to_string(),
            "90° V-Bit".to_string(),
            ToolType::VBit,
            6.35,
            1,
            "Carbide".to_string(),
        );
        vbit.set_cutting_parameters(150.0, 75.0, 18000);
        vbit.set_depth_parameters(2.0, 1.0);
        library.add_tool(vbit);

        let mut drill = Tool::new(
            "drill_32".to_string(),
            "1/8\" Drill".to_string(),
            ToolType::Drill,
            3.175,
            2,
            "HSS".to_string(),
        );
        drill.set_cutting_parameters(120.0, 60.0, 8000);
        drill.set_depth_parameters(10.0, 0.0);
        library.add_tool(drill);

        library
    }
}

impl Default for ToolLibrary {
    fn default() -> Self {
        Self::new()
    }
}

