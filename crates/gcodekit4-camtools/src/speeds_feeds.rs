//! Speeds and Feeds Calculator
//!
//! Calculates spindle speeds and feed rates based on material properties,
//! tool geometry, and machine capabilities.
//!
//! Based on standard machining formulas:
//! RPM = (Surface Speed * 1000) / (π * Diameter)
//! Feed Rate = RPM * Chip Load * Number of Flutes

use gcodekit4_core::data::materials::Material;
use gcodekit4_core::data::tools::Tool;
use gcodekit4_devicedb::model::DeviceProfile;

/// Result of a speeds and feeds calculation
#[derive(Debug, Clone)]
pub struct CalculationResult {
    /// Calculated Spindle Speed in RPM
    pub rpm: u32,
    /// Calculated Feed Rate in mm/min
    pub feed_rate: f32,
    /// Surface Speed used for calculation (m/min)
    pub surface_speed: f32,
    /// Chip Load used for calculation (mm/tooth)
    pub chip_load: f32,
    /// Source of the calculation data
    pub source: String,
    /// Warnings generated during calculation
    pub warnings: Vec<String>,
    /// Unclamped RPM if clamping occurred
    pub unclamped_rpm: Option<u32>,
    /// Unclamped Feed Rate if clamping occurred
    pub unclamped_feed_rate: Option<f32>,
}

/// Calculator for speeds and feeds
pub struct SpeedsFeedsCalculator;

impl SpeedsFeedsCalculator {
    /// Calculate speeds and feeds for a given material, tool, and device
    pub fn calculate(
        material: &Material,
        tool: &Tool,
        device: &DeviceProfile,
    ) -> CalculationResult {
        let mut warnings = Vec::new();
        let mut source = String::new();
        let mut unclamped_rpm = None;
        let mut unclamped_feed_rate = None;

        // 1. Determine Surface Speed (SFM equivalent in m/min)
        // Priority:
        // 1. Material property (surface_speed_m_min)
        // 2. Material cutting params for this tool type (implied from RPM range?)
        // 3. Tool default params
        
        let tool_type_key = match tool.tool_type {
            gcodekit4_core::data::tools::ToolType::EndMillFlat => "endmill_flat",
            gcodekit4_core::data::tools::ToolType::EndMillBall => "endmill_ball",
            gcodekit4_core::data::tools::ToolType::VBit => "vbit",
            gcodekit4_core::data::tools::ToolType::DrillBit => "drill",
            _ => "generic",
        };

        let material_params = material.get_cutting_params(tool_type_key);

        let surface_speed = if let Some(params) = material_params {
            if let Some(speed) = params.surface_speed_m_min {
                source.push_str("Material Surface Speed");
                speed
            } else {
                // Fallback: Estimate from Material RPM range if available
                let avg_rpm = (params.rpm_range.0 + params.rpm_range.1) as f32 / 2.0;
                // SFM = (RPM * pi * Dia) / 1000
                let speed = (avg_rpm * std::f32::consts::PI * tool.diameter) / 1000.0;
                source.push_str("Estimated from Material RPM");
                speed
            }
        } else {
            // Fallback: Estimate from Tool default RPM
            let rpm = tool.params.rpm as f32;
            let speed = (rpm * std::f32::consts::PI * tool.diameter) / 1000.0;
            source.push_str("Estimated from Tool Defaults");
            speed
        };

        // 2. Calculate RPM
        // RPM = (Surface Speed * 1000) / (π * Diameter)
        let mut rpm = (surface_speed * 1000.0) / (std::f32::consts::PI * tool.diameter);

        // 3. Determine Chip Load
        // Priority:
        // 1. Material property (chip_load_mm)
        // 2. Derived from Material Feed Rate range
        // 3. Derived from Tool default Feed Rate
        
        let chip_load = if let Some(params) = material_params {
            if let Some(load) = params.chip_load_mm {
                if !source.contains("Material") {
                    source.push_str(" + Material Chip Load");
                }
                load
            } else {
                let avg_feed = (params.feed_rate_range.0 + params.feed_rate_range.1) / 2.0;
                let avg_rpm = (params.rpm_range.0 + params.rpm_range.1) as f32 / 2.0;
                // Chip Load = Feed / (RPM * Flutes)
                let load = avg_feed / (avg_rpm * tool.flutes as f32);
                if !source.contains("Material") {
                    source.push_str(" + Material Feed");
                }
                load
            }
        } else {
            let load = tool.params.feed_rate / (tool.params.rpm as f32 * tool.flutes as f32);
            if !source.contains("Tool") {
                source.push_str(" + Tool Defaults");
            }
            load
        };

        // 4. Calculate Feed Rate
        // Feed = RPM * Chip Load * Flutes
        let mut feed_rate = rpm * chip_load * tool.flutes as f32;

        // 5. Apply Device Limits
        
        // Check Max RPM
        let max_rpm = 24000.0; // Common default
        if rpm > max_rpm {
            warnings.push(format!("Calculated RPM ({:.0}) exceeds standard max ({:.0}). Clamped.", rpm, max_rpm));
            unclamped_rpm = Some(rpm as u32);
            rpm = max_rpm;
            // Recalculate feed rate to maintain chip load? 
            // Usually yes, to protect tool.
            feed_rate = rpm * chip_load * tool.flutes as f32;
        }

        // Check Max Feed Rate
        if feed_rate > device.max_feed_rate as f32 {
            warnings.push(format!("Calculated Feed ({:.0}) exceeds device max ({:.0}). Clamped.", feed_rate, device.max_feed_rate));
            unclamped_feed_rate = Some(feed_rate);
            feed_rate = device.max_feed_rate as f32;
            // If we clamp feed, chip load decreases.
        }

        // Check Min RPM (Spindle usually has a min speed)
        let min_rpm = 1000.0;
        if rpm < min_rpm {
             warnings.push(format!("Calculated RPM ({:.0}) is below standard min ({:.0}). Clamped.", rpm, min_rpm));
             unclamped_rpm = Some(rpm as u32);
             rpm = min_rpm;
             feed_rate = rpm * chip_load * tool.flutes as f32;
        }

        CalculationResult {
            rpm: rpm as u32,
            feed_rate,
            surface_speed,
            chip_load,
            source,
            warnings,
            unclamped_rpm,
            unclamped_feed_rate,
        }
    }
}


