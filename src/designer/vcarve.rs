//! # V-Carving Operations Module
//!
//! Provides V-carving toolpath generation for creating angled cuts using V-bit tools.
//!
//! V-carving is a technique that uses a V-shaped cutting tool to create variable-depth cuts
//! that follow the outline of shapes. The depth is calculated based on the tool angle and
//! the cutting width, producing professional-looking engraved or carved effects.
//!
//! Supports:
//! - Multiple V-bit angles (60°, 90°, 120°, custom)
//! - Depth calculation from cutting width
//! - Path offset for tool diameter compensation
//! - Multi-pass cutting for deeper designs
//! - Toolpath generation and optimization

use crate::designer::shapes::Point;
use anyhow::Result;

/// Represents a V-bit tool with specific geometry
#[derive(Debug, Clone, Copy)]
pub struct VBitTool {
    /// Tip angle in degrees (e.g., 60, 90, 120)
    pub tip_angle: f64,
    /// Diameter at the widest point (mm)
    pub diameter: f64,
    /// Cutting edge length (mm)
    pub cutting_length: f64,
}

impl VBitTool {
    /// Create a new V-bit tool
    pub fn new(tip_angle: f64, diameter: f64, cutting_length: f64) -> Self {
        Self {
            tip_angle,
            diameter,
            cutting_length,
        }
    }

    /// Create a 60-degree V-bit (common for fine detail)
    pub fn v60(diameter: f64) -> Self {
        Self::new(60.0, diameter, diameter * 0.86)
    }

    /// Create a 90-degree V-bit (most common)
    pub fn v90(diameter: f64) -> Self {
        Self::new(90.0, diameter, diameter * 0.707)
    }

    /// Create a 120-degree V-bit (shallow cuts)
    pub fn v120(diameter: f64) -> Self {
        Self::new(120.0, diameter, diameter * 0.577)
    }

    /// Validate V-bit parameters
    pub fn is_valid(&self) -> bool {
        self.tip_angle > 0.0
            && self.tip_angle < 180.0
            && self.diameter > 0.0
            && self.cutting_length > 0.0
    }

    /// Calculate cutting depth from cutting width
    ///
    /// For a V-bit, the depth is related to the cutting width by:
    /// depth = width / (2 * tan(tip_angle / 2))
    pub fn calculate_depth(&self, cutting_width: f64) -> f64 {
        let half_angle_rad = (self.tip_angle / 2.0).to_radians();
        cutting_width / (2.0 * half_angle_rad.tan())
    }

    /// Calculate maximum cutting width at the tool's cutting length
    pub fn max_cutting_width(&self) -> f64 {
        let half_angle_rad = (self.tip_angle / 2.0).to_radians();
        2.0 * self.cutting_length * half_angle_rad.tan()
    }

    /// Calculate the radius at a given depth
    pub fn radius_at_depth(&self, depth: f64) -> f64 {
        let half_angle_rad = (self.tip_angle / 2.0).to_radians();
        depth * half_angle_rad.tan()
    }
}

/// Parameters for V-carving operations
#[derive(Debug, Clone)]
pub struct VCarveParams {
    /// V-bit tool to use
    pub tool: VBitTool,
    /// Target cutting width (mm)
    pub cutting_width: f64,
    /// Maximum depth per pass (mm), for multi-pass operations
    pub max_depth_per_pass: f64,
    /// Spindle speed (RPM)
    pub spindle_speed: u32,
    /// Feed rate (mm/min)
    pub feed_rate: f64,
}

impl VCarveParams {
    /// Create new V-carving parameters
    pub fn new(
        tool: VBitTool,
        cutting_width: f64,
        max_depth_per_pass: f64,
        spindle_speed: u32,
        feed_rate: f64,
    ) -> Self {
        Self {
            tool,
            cutting_width,
            max_depth_per_pass,
            spindle_speed,
            feed_rate,
        }
    }

    /// Validate parameters
    pub fn is_valid(&self) -> bool {
        self.tool.is_valid()
            && self.cutting_width > 0.0
            && self.cutting_width <= self.tool.max_cutting_width()
            && self.max_depth_per_pass > 0.0
            && self.spindle_speed > 0
            && self.feed_rate > 0.0
    }

    /// Calculate total cutting depth for this operation
    pub fn total_depth(&self) -> f64 {
        self.tool.calculate_depth(self.cutting_width)
    }

    /// Calculate number of passes needed
    pub fn passes_needed(&self) -> u32 {
        let total_depth = self.total_depth();
        let raw_passes = total_depth / self.max_depth_per_pass;
        
        // Handle floating point precision - if very close to integer, round down
        let passes = if (raw_passes.fract()).abs() < 1e-9 {
            raw_passes as u32
        } else {
            raw_passes.ceil() as u32
        };
        
        passes.max(1)  // Ensure at least 1 pass
    }

    /// Calculate actual depth per pass (may be less than max for final pass)
    pub fn depth_per_pass(&self) -> f64 {
        let total_depth = self.total_depth();
        let passes = self.passes_needed();
        total_depth / passes as f64
    }
}

/// Represents a V-carving toolpath segment
#[derive(Debug, Clone)]
pub struct VCarveSegment {
    /// Start point of the segment
    pub start: Point,
    /// End point of the segment
    pub end: Point,
    /// Cutting depth at this segment (mm)
    pub depth: f64,
    /// Radius at this depth (for width calculation)
    pub radius: f64,
}

impl VCarveSegment {
    /// Create a new V-carving segment
    pub fn new(start: Point, end: Point, depth: f64, radius: f64) -> Self {
        Self {
            start,
            end,
            depth,
            radius,
        }
    }

    /// Calculate segment length
    pub fn length(&self) -> f64 {
        self.start.distance_to(&self.end)
    }
}

/// V-carving toolpath generator
pub struct VCarveGenerator;

impl VCarveGenerator {
    /// Generate V-carving depth for a given cutting width
    pub fn calculate_depth(tool: &VBitTool, cutting_width: f64) -> Result<f64> {
        if !tool.is_valid() {
            return Err(anyhow::anyhow!("Invalid V-bit tool parameters"));
        }

        if cutting_width <= 0.0 {
            return Err(anyhow::anyhow!("Cutting width must be positive"));
        }

        let max_width = tool.max_cutting_width();
        if cutting_width > max_width {
            return Err(anyhow::anyhow!(
                "Cutting width {} exceeds maximum {} for this tool",
                cutting_width,
                max_width
            ));
        }

        Ok(tool.calculate_depth(cutting_width))
    }

    /// Generate V-carving segments from a path with multiple passes
    pub fn generate_passes(
        params: &VCarveParams,
        path_points: &[Point],
    ) -> Result<Vec<Vec<VCarveSegment>>> {
        if !params.is_valid() {
            return Err(anyhow::anyhow!("Invalid V-carving parameters"));
        }

        if path_points.len() < 2 {
            return Err(anyhow::anyhow!("Path must have at least 2 points"));
        }

        let mut all_passes = Vec::new();
        let passes = params.passes_needed();
        let depth_per_pass = params.depth_per_pass();
        let tool = &params.tool;

        for pass in 0..passes {
            let mut pass_segments = Vec::new();
            let current_depth = (pass as f64 + 1.0) * depth_per_pass;

            // Clamp to maximum depth
            let actual_depth = current_depth.min(params.total_depth());
            let radius = tool.radius_at_depth(actual_depth);

            // Generate segments for this pass
            for i in 0..(path_points.len() - 1) {
                let segment = VCarveSegment::new(
                    path_points[i],
                    path_points[i + 1],
                    actual_depth,
                    radius,
                );
                pass_segments.push(segment);
            }

            all_passes.push(pass_segments);
        }

        Ok(all_passes)
    }

    /// Calculate time estimate for V-carving operation (in minutes)
    pub fn estimate_time(
        params: &VCarveParams,
        path_points: &[Point],
    ) -> Result<f64> {
        if path_points.len() < 2 {
            return Err(anyhow::anyhow!("Path must have at least 2 points"));
        }

        // Calculate total path length
        let mut total_length = 0.0;
        for i in 0..(path_points.len() - 1) {
            total_length += path_points[i].distance_to(&path_points[i + 1]);
        }

        // Account for multiple passes
        let passes = params.passes_needed() as f64;
        total_length *= passes;

        // Time = distance / feed_rate (in minutes)
        Ok(total_length / params.feed_rate)
    }

    /// Validate V-carving parameters
    pub fn validate_params(params: &VCarveParams) -> Result<()> {
        if !params.is_valid() {
            return Err(anyhow::anyhow!("Invalid V-carving parameters"));
        }

        if params.cutting_width > params.tool.max_cutting_width() {
            return Err(anyhow::anyhow!(
                "Cutting width {} exceeds tool maximum {}",
                params.cutting_width,
                params.tool.max_cutting_width()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vbit_creation() {
        let tool = VBitTool::new(90.0, 3.175, 2.25);
        assert_eq!(tool.tip_angle, 90.0);
        assert_eq!(tool.diameter, 3.175);
        assert_eq!(tool.cutting_length, 2.25);
    }

    #[test]
    fn test_vbit_presets() {
        let v60 = VBitTool::v60(3.175);
        assert_eq!(v60.tip_angle, 60.0);

        let v90 = VBitTool::v90(3.175);
        assert_eq!(v90.tip_angle, 90.0);

        let v120 = VBitTool::v120(3.175);
        assert_eq!(v120.tip_angle, 120.0);
    }

    #[test]
    fn test_vbit_validation() {
        let valid = VBitTool::new(90.0, 3.175, 2.25);
        assert!(valid.is_valid());

        let invalid_angle = VBitTool::new(0.0, 3.175, 2.25);
        assert!(!invalid_angle.is_valid());

        let invalid_diameter = VBitTool::new(90.0, 0.0, 2.25);
        assert!(!invalid_diameter.is_valid());
    }

    #[test]
    fn test_depth_calculation_90_degree() {
        let tool = VBitTool::v90(3.175);
        let depth = tool.calculate_depth(1.0);

        // For 90° V-bit: depth = width / (2 * tan(45°)) = width / 2
        assert!((depth - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_depth_calculation_60_degree() {
        let tool = VBitTool::v60(3.175);
        let depth = tool.calculate_depth(1.0);

        // For 60° V-bit: depth = width / (2 * tan(30°)) ≈ width / 1.1547
        assert!((depth - 0.866).abs() < 0.01);
    }

    #[test]
    fn test_max_cutting_width() {
        let tool = VBitTool::v90(3.175);
        let max_width = tool.max_cutting_width();

        // For 90° V-bit with cutting_length = 2.25: max_width ≈ 4.5
        assert!((max_width - 4.5).abs() < 0.1);
    }

    #[test]
    fn test_radius_at_depth() {
        let tool = VBitTool::v90(3.175);
        let radius = tool.radius_at_depth(1.0);

        // For 90° V-bit at 1mm depth: radius = 1mm
        assert!((radius - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_vcarve_params_creation() {
        let tool = VBitTool::v90(3.175);
        let params = VCarveParams::new(tool, 1.0, 0.5, 10000, 100.0);

        assert_eq!(params.cutting_width, 1.0);
        assert_eq!(params.max_depth_per_pass, 0.5);
    }

    #[test]
    fn test_vcarve_params_validation() {
        let tool = VBitTool::v90(3.175);
        let valid_params = VCarveParams::new(tool, 1.0, 0.5, 10000, 100.0);
        assert!(valid_params.is_valid());

        let invalid_width = VCarveParams::new(tool, 0.0, 0.5, 10000, 100.0);
        assert!(!invalid_width.is_valid());
    }

    #[test]
    fn test_total_depth() {
        let tool = VBitTool::v90(3.175);
        let params = VCarveParams::new(tool, 1.0, 0.5, 10000, 100.0);

        let depth = params.total_depth();
        assert!((depth - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_passes_needed_single_pass() {
        let tool = VBitTool::v90(3.175);
        let params = VCarveParams::new(tool, 1.0, 0.5, 10000, 100.0);

        let passes = params.passes_needed();
        assert_eq!(passes, 1);
    }

    #[test]
    fn test_passes_needed_multiple_passes() {
        let tool = VBitTool::v90(3.175);
        let params = VCarveParams::new(tool, 1.0, 0.2, 10000, 100.0);

        let passes = params.passes_needed();
        assert_eq!(passes, 3);
    }

    #[test]
    fn test_depth_per_pass() {
        let tool = VBitTool::v90(3.175);
        let params = VCarveParams::new(tool, 1.0, 0.2, 10000, 100.0);

        let depth_per_pass = params.depth_per_pass();
        assert!((depth_per_pass - 0.166667).abs() < 0.01);
    }

    #[test]
    fn test_vcarve_segment_creation() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(10.0, 0.0);
        let segment = VCarveSegment::new(p1, p2, 1.0, 0.5);

        assert_eq!(segment.start, p1);
        assert_eq!(segment.end, p2);
        assert_eq!(segment.depth, 1.0);
        assert_eq!(segment.radius, 0.5);
    }

    #[test]
    fn test_vcarve_segment_length() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        let segment = VCarveSegment::new(p1, p2, 1.0, 0.5);

        assert_eq!(segment.length(), 5.0);
    }

    #[test]
    fn test_calculate_depth() {
        let tool = VBitTool::v90(3.175);
        let result = VCarveGenerator::calculate_depth(&tool, 1.0);

        assert!(result.is_ok());
        let depth = result.unwrap();
        assert!((depth - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_calculate_depth_invalid() {
        let tool = VBitTool::v90(3.175);
        let result = VCarveGenerator::calculate_depth(&tool, 0.0);

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_passes_single() {
        let tool = VBitTool::v90(3.175);
        let params = VCarveParams::new(tool, 1.0, 0.5, 10000, 100.0);
        let path = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
        ];

        let result = VCarveGenerator::generate_passes(&params, &path);
        assert!(result.is_ok());

        let passes = result.unwrap();
        assert_eq!(passes.len(), 1);
        assert_eq!(passes[0].len(), 2);
    }

    #[test]
    fn test_generate_passes_multiple() {
        let tool = VBitTool::v90(3.175);
        let params = VCarveParams::new(tool, 1.0, 0.2, 10000, 100.0);
        let path = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
        ];

        let result = VCarveGenerator::generate_passes(&params, &path);
        assert!(result.is_ok());

        let passes = result.unwrap();
        assert_eq!(passes.len(), 3);
    }

    #[test]
    fn test_estimate_time() {
        let tool = VBitTool::v90(3.175);
        let params = VCarveParams::new(tool, 1.0, 0.5, 10000, 100.0);
        let path = vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)];

        let result = VCarveGenerator::estimate_time(&params, &path);
        assert!(result.is_ok());

        let time = result.unwrap();
        // 100mm path / 100 mm/min = 1 minute
        assert!((time - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_validate_params_valid() {
        let tool = VBitTool::v90(3.175);
        let params = VCarveParams::new(tool, 1.0, 0.5, 10000, 100.0);

        let result = VCarveGenerator::validate_params(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_params_invalid_width() {
        let tool = VBitTool::v90(3.175);
        let params = VCarveParams::new(tool, 10.0, 0.5, 10000, 100.0);

        let result = VCarveGenerator::validate_params(&params);
        assert!(result.is_err());
    }
}
