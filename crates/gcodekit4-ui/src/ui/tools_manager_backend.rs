//! CNC Tools Manager Backend
//!
//! This module provides backend logic for the CNC Tools Manager UI,
//! including persistence for custom tools.

use gcodekit4_core::data::gtc_import::{GtcImporter, GtcImportResult};
use gcodekit4_core::data::tools::{Tool, ToolId, ToolLibrary, ToolType, ToolMaterial};
use std::path::{Path, PathBuf};

pub struct ToolsManagerBackend {
    library: ToolLibrary,
    storage_path: PathBuf,
}

impl ToolsManagerBackend {
    pub fn new() -> Self {
        let storage_path = Self::get_storage_path();
        let mut library = gcodekit4_core::data::tools::init_standard_library();
        
        // Load custom tools from disk if they exist
        if let Ok(custom_tools) = Self::load_from_file(&storage_path) {
            for tool in custom_tools {
                library.add_tool(tool);
            }
        }
        
        Self { library, storage_path }
    }
    
    fn get_storage_path() -> PathBuf {
        let mut path = dirs::config_dir()
            .or_else(|| dirs::home_dir())
            .unwrap_or_else(|| PathBuf::from("."));
        path.push("gcodekit4");
        std::fs::create_dir_all(&path).ok();
        path.push("custom_tools.json");
        path
    }
    
    fn load_from_file(path: &PathBuf) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let tools: Vec<Tool> = serde_json::from_str(&contents)?;
        Ok(tools)
    }
    
    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Only save custom tools
        let custom_tools: Vec<&Tool> = self.library
            .get_all_tools()
            .into_iter()
            .filter(|t| t.custom)
            .collect();
        
        let json = serde_json::to_string_pretty(&custom_tools)?;
        std::fs::write(&self.storage_path, json)?;
        Ok(())
    }

    pub fn get_library(&self) -> &ToolLibrary {
        &self.library
    }

    pub fn get_library_mut(&mut self) -> &mut ToolLibrary {
        &mut self.library
    }

    pub fn add_tool(&mut self, tool: Tool) {
        self.library.add_tool(tool);
        // Save custom tools to disk
        if let Err(e) = self.save_to_file() {
            tracing::warn!("Failed to save tools: {}", e);
        }
    }

    pub fn remove_tool(&mut self, id: &ToolId) -> Option<Tool> {
        let result = self.library.remove_tool(id);
        // Save custom tools to disk
        if let Err(e) = self.save_to_file() {
            tracing::warn!("Failed to save tools: {}", e);
        }
        result
    }

    pub fn get_tool(&self, id: &ToolId) -> Option<&Tool> {
        self.library.get_tool(id)
    }

    pub fn search_tools(&self, query: &str) -> Vec<&Tool> {
        if query.is_empty() {
            self.library.get_all_tools()
        } else {
            self.library.search_by_name(query)
        }
    }

    pub fn filter_by_type(&self, tool_type: ToolType) -> Vec<&Tool> {
        self.library.get_tools_by_type(tool_type)
    }

    pub fn filter_by_diameter(&self, diameter: f32, tolerance: f32) -> Vec<&Tool> {
        self.library.search_by_diameter(diameter - tolerance, diameter + tolerance)
    }

    pub fn get_all_tools(&self) -> Vec<&Tool> {
        self.library.get_all_tools()
    }
    
    /// Import tools from a GTC package (.zip file)
    pub fn import_gtc_package<P: AsRef<Path>>(
        &mut self,
        zip_path: P,
    ) -> Result<GtcImportResult, Box<dyn std::error::Error>> {
        // Determine next tool number
        let next_number = self.library.get_all_tools()
            .iter()
            .map(|t| t.number)
            .max()
            .unwrap_or(0) + 1;
        
        let mut importer = GtcImporter::new(next_number);
        let result = importer.import_from_zip(zip_path)?;
        
        // Add imported tools to library
        for tool in &result.imported_tools {
            self.library.add_tool(tool.clone());
        }
        
        // Save to disk
        if let Err(e) = self.save_to_file() {
            tracing::warn!("Failed to save tools after GTC import: {}", e);
        }
        
        Ok(result)
    }
    
    /// Import tools from a GTC JSON file
    pub fn import_gtc_json<P: AsRef<Path>>(
        &mut self,
        json_path: P,
    ) -> Result<GtcImportResult, Box<dyn std::error::Error>> {
        // Determine next tool number
        let next_number = self.library.get_all_tools()
            .iter()
            .map(|t| t.number)
            .max()
            .unwrap_or(0) + 1;
        
        let mut importer = GtcImporter::new(next_number);
        let result = importer.import_from_json(json_path)?;
        
        // Add imported tools to library
        for tool in &result.imported_tools {
            self.library.add_tool(tool.clone());
        }
        
        // Save to disk
        if let Err(e) = self.save_to_file() {
            tracing::warn!("Failed to save tools after GTC import: {}", e);
        }
        
        Ok(result)
    }
}

impl Default for ToolsManagerBackend {
    fn default() -> Self {
        Self::new()
    }
}

// Helper conversion functions for UI
pub fn string_to_tool_type(s: &str) -> Option<ToolType> {
    match s {
        "Flat End Mill" => Some(ToolType::EndMillFlat),
        "Ball End Mill" => Some(ToolType::EndMillBall),
        "Corner Radius End Mill" => Some(ToolType::EndMillCornerRadius),
        "V-Bit" => Some(ToolType::VBit),
        "Drill Bit" => Some(ToolType::DrillBit),
        "Spot Drill" => Some(ToolType::SpotDrill),
        "Engraving Bit" => Some(ToolType::EngravingBit),
        "Chamfer Tool" => Some(ToolType::ChamferTool),
        "Specialty" => Some(ToolType::Specialty),
        _ => None,
    }
}

pub fn string_to_tool_material(s: &str) -> Option<ToolMaterial> {
    match s {
        "HSS" => Some(ToolMaterial::HSS),
        "Carbide" => Some(ToolMaterial::Carbide),
        "Coated Carbide" => Some(ToolMaterial::CoatedCarbide),
        "Diamond" => Some(ToolMaterial::Diamond),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let backend = ToolsManagerBackend::new();
        assert!(!backend.get_all_tools().is_empty());
    }

    #[test]
    fn test_tool_type_conversion() {
        assert_eq!(
            string_to_tool_type("Flat End Mill"),
            Some(ToolType::EndMillFlat)
        );
        assert_eq!(
            string_to_tool_type("Drill Bit"),
            Some(ToolType::DrillBit)
        );
    }

    #[test]
    fn test_search_tools() {
        let backend = ToolsManagerBackend::new();
        let results = backend.search_tools("end");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_persistence() {
        // Create a test tool
        let test_tool = Tool::new(
            ToolId("test_persist_tool".to_string()),
            999, // tool number
            "Test Persist Tool".to_string(),
            ToolType::EndMillFlat,
            6.35, // diameter
            38.0, // length
        );
        
        // Add and save
        {
            let mut backend = ToolsManagerBackend::new();
            let mut tool = test_tool.clone();
            tool.custom = true;
            backend.add_tool(tool);
        }
        
        // Create new backend and verify tool was loaded
        {
            let backend = ToolsManagerBackend::new();
            let loaded = backend.get_tool(&ToolId("test_persist_tool".to_string()));
            assert!(loaded.is_some());
            assert_eq!(loaded.unwrap().name, "Test Persist Tool");
        }
        
        // Cleanup
        {
            let mut backend = ToolsManagerBackend::new();
            backend.remove_tool(&ToolId("test_persist_tool".to_string()));
        }
    }
}
