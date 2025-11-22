//! Generic Tool Catalog (GTC) Import
//!
//! This module provides functionality to import tool catalogs from
//! GTC packages (.zip files) provided by tool suppliers.
//!
//! GTC is an industry standard format for exchanging tool catalog data
//! between CAM systems and tool suppliers.

use crate::data::tools::{Tool, ToolCoating, ToolId, ToolMaterial, ToolType};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

/// GTC Tool definition from catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GtcTool {
    #[serde(rename = "ID")]
    pub id: String,

    #[serde(rename = "Description")]
    pub description: String,

    #[serde(rename = "Type")]
    pub tool_type: String,

    #[serde(rename = "Diameter")]
    pub diameter: f32,

    #[serde(rename = "Length")]
    pub length: f32,

    #[serde(rename = "FluteLength")]
    pub flute_length: Option<f32>,

    #[serde(rename = "ShankDiameter")]
    pub shank_diameter: Option<f32>,

    #[serde(rename = "NumberOfFlutes")]
    pub number_of_flutes: Option<u32>,

    #[serde(rename = "Material")]
    pub material: Option<String>,

    #[serde(rename = "Coating")]
    pub coating: Option<String>,

    #[serde(rename = "Manufacturer")]
    pub manufacturer: Option<String>,

    #[serde(rename = "PartNumber")]
    pub part_number: Option<String>,

    #[serde(rename = "CuttingParameters")]
    pub cutting_parameters: Option<GtcCuttingParams>,
}

/// Cutting parameters from GTC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GtcCuttingParams {
    #[serde(rename = "RPM")]
    pub rpm: Option<u32>,

    #[serde(rename = "FeedRate")]
    pub feed_rate: Option<f32>,

    #[serde(rename = "PlungeRate")]
    pub plunge_rate: Option<f32>,

    #[serde(rename = "Material")]
    pub material: Option<String>,
}

/// GTC Catalog structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GtcCatalog {
    #[serde(rename = "Version")]
    pub version: String,

    #[serde(rename = "Manufacturer")]
    pub manufacturer: String,

    #[serde(rename = "Tools")]
    pub tools: Vec<GtcTool>,
}

/// GTC Import result
#[derive(Debug)]
pub struct GtcImportResult {
    pub total_tools: usize,
    pub imported_tools: Vec<Tool>,
    pub skipped_tools: usize,
    pub errors: Vec<String>,
}

/// GTC Importer
pub struct GtcImporter {
    next_tool_number: u32,
}

impl GtcImporter {
    pub fn new(starting_tool_number: u32) -> Self {
        Self {
            next_tool_number: starting_tool_number,
        }
    }

    /// Import tools from a GTC package (.zip file)
    pub fn import_from_zip<P: AsRef<Path>>(
        &mut self,
        zip_path: P,
    ) -> Result<GtcImportResult, Box<dyn std::error::Error>> {
        let file = File::open(zip_path.as_ref())?;
        let mut archive = ZipArchive::new(file)?;

        // Look for catalog.json or tools.json in the archive
        let catalog = self.find_and_parse_catalog(&mut archive)?;

        let mut result = GtcImportResult {
            total_tools: catalog.tools.len(),
            imported_tools: Vec::new(),
            skipped_tools: 0,
            errors: Vec::new(),
        };

        // Convert GTC tools to our Tool format
        for gtc_tool in catalog.tools {
            match self.convert_gtc_tool(gtc_tool) {
                Ok(tool) => {
                    result.imported_tools.push(tool);
                }
                Err(e) => {
                    result.skipped_tools += 1;
                    result.errors.push(format!("Failed to import tool: {}", e));
                }
            }
        }

        Ok(result)
    }

    /// Import tools from a JSON file directly
    pub fn import_from_json<P: AsRef<Path>>(
        &mut self,
        json_path: P,
    ) -> Result<GtcImportResult, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(json_path)?;
        let catalog: GtcCatalog = serde_json::from_str(&contents)?;

        let mut result = GtcImportResult {
            total_tools: catalog.tools.len(),
            imported_tools: Vec::new(),
            skipped_tools: 0,
            errors: Vec::new(),
        };

        for gtc_tool in catalog.tools {
            match self.convert_gtc_tool(gtc_tool) {
                Ok(tool) => {
                    result.imported_tools.push(tool);
                }
                Err(e) => {
                    result.skipped_tools += 1;
                    result.errors.push(format!("Failed to import tool: {}", e));
                }
            }
        }

        Ok(result)
    }

    fn find_and_parse_catalog(
        &self,
        archive: &mut ZipArchive<File>,
    ) -> Result<GtcCatalog, Box<dyn std::error::Error>> {
        // Try common catalog file names
        let catalog_names = vec![
            "catalog.json",
            "tools.json",
            "gtc.json",
            "tool_catalog.json",
        ];

        for name in catalog_names {
            if let Ok(mut file) = archive.by_name(name) {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                return Ok(serde_json::from_str(&contents)?);
            }
        }

        Err("No catalog file found in GTC package".into())
    }

    pub fn convert_gtc_tool(&mut self, gtc_tool: GtcTool) -> Result<Tool, Box<dyn std::error::Error>> {
        // Map GTC tool type to our ToolType
        let tool_type = self.map_tool_type(&gtc_tool.tool_type)?;

        // Map material
        let material = if let Some(mat) = &gtc_tool.material {
            self.map_tool_material(mat)
        } else {
            ToolMaterial::Carbide // Default
        };

        // Create tool ID from GTC ID or generate one
        let tool_id = ToolId(format!(
            "gtc_{}",
            gtc_tool.id.replace(" ", "_").to_lowercase()
        ));

        let tool_number = self.next_tool_number;
        self.next_tool_number += 1;

        let mut tool = Tool::new(
            tool_id,
            tool_number,
            gtc_tool.description.clone(),
            tool_type,
            gtc_tool.diameter,
            gtc_tool.length,
        );

        // Set additional properties
        if let Some(flute_length) = gtc_tool.flute_length {
            tool.flute_length = flute_length;
        }

        if let Some(shank_dia) = gtc_tool.shank_diameter {
            tool.shaft_diameter = Some(shank_dia);
        }

        if let Some(flutes) = gtc_tool.number_of_flutes {
            tool.flutes = flutes;
        }

        tool.material = material;

        if let Some(coating) = gtc_tool.coating {
            tool.coating = Some(self.map_coating(&coating));
        }

        if let Some(mfg) = gtc_tool.manufacturer {
            tool.manufacturer = Some(mfg);
        }

        if let Some(pn) = gtc_tool.part_number {
            tool.part_number = Some(pn);
        }

        // Mark as custom tool (imported)
        tool.custom = true;

        Ok(tool)
    }

    pub fn map_tool_type(&self, gtc_type: &str) -> Result<ToolType, Box<dyn std::error::Error>> {
        let gtc_type_lower = gtc_type.to_lowercase();

        if gtc_type_lower.contains("end mill") || gtc_type_lower.contains("endmill") {
            if gtc_type_lower.contains("ball") {
                Ok(ToolType::EndMillBall)
            } else if gtc_type_lower.contains("corner") || gtc_type_lower.contains("radius") {
                Ok(ToolType::EndMillCornerRadius)
            } else {
                Ok(ToolType::EndMillFlat)
            }
        } else if gtc_type_lower.contains("drill") {
            if gtc_type_lower.contains("center") {
                Ok(ToolType::SpotDrill)
            } else if gtc_type_lower.contains("spot") {
                Ok(ToolType::SpotDrill)
            } else {
                Ok(ToolType::DrillBit)
            }
        } else if gtc_type_lower.contains("v-bit") || gtc_type_lower.contains("v bit") {
            Ok(ToolType::VBit)
        } else if gtc_type_lower.contains("engrav") {
            Ok(ToolType::EngravingBit)
        } else if gtc_type_lower.contains("chamfer") {
            Ok(ToolType::ChamferTool)
        } else {
            Ok(ToolType::Specialty)
        }
    }

    pub fn map_tool_material(&self, material: &str) -> ToolMaterial {
        let material_lower = material.to_lowercase();

        if material_lower.contains("carbide") {
            if material_lower.contains("coat") {
                ToolMaterial::CoatedCarbide
            } else {
                ToolMaterial::Carbide
            }
        } else if material_lower.contains("hss") || material_lower.contains("high speed") {
            ToolMaterial::HSS
        } else if material_lower.contains("diamond") {
            ToolMaterial::Diamond
        } else {
            ToolMaterial::Carbide // Default
        }
    }

    pub fn map_coating(&self, coating: &str) -> ToolCoating {
        let coating_lower = coating.to_lowercase();

        if coating_lower.contains("tialn") {
            ToolCoating::TiAlN
        } else if coating_lower.contains("tin") {
            ToolCoating::TiN
        } else if coating_lower.contains("dlc") || coating_lower.contains("diamond") {
            ToolCoating::DLC
        } else if coating_lower.contains("alox") || coating_lower.contains("aluminum oxide") {
            ToolCoating::AlOx
        } else {
            // Default to TiN if unrecognized
            ToolCoating::TiN
        }
    }
}


