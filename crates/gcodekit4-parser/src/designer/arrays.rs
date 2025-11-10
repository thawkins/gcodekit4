//! # Array Operations Module
//!
//! Provides functionality for creating multiple copies of shapes in linear, circular, and grid patterns.
//!
//! Supports:
//! - Linear arrays (X/Y direction copies with uniform spacing)
//! - Circular arrays (rotational copies around a center point)
//! - Grid arrays (2D rectangular arrays with row/column spacing)
//! - Configurable spacing, count, and orientation
//! - Integration with existing shapes and toolpath generation

use crate::designer::shapes::Point;
use anyhow::Result;

/// Represents different types of array operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayType {
    /// Linear array in X and Y directions
    Linear,
    /// Circular array around a center point
    Circular,
    /// Grid/rectangular array in 2D
    Grid,
}

/// Parameters for linear array operations
#[derive(Debug, Clone)]
pub struct LinearArrayParams {
    /// Number of copies in X direction
    pub count_x: u32,
    /// Number of copies in Y direction
    pub count_y: u32,
    /// Spacing between copies in X direction (mm)
    pub spacing_x: f64,
    /// Spacing between copies in Y direction (mm)
    pub spacing_y: f64,
}

impl LinearArrayParams {
    /// Create new linear array parameters
    pub fn new(count_x: u32, count_y: u32, spacing_x: f64, spacing_y: f64) -> Self {
        Self {
            count_x,
            count_y,
            spacing_x,
            spacing_y,
        }
    }

    /// Validate parameters
    pub fn is_valid(&self) -> bool {
        self.count_x > 0 && self.count_y > 0 && self.spacing_x >= 0.0 && self.spacing_y >= 0.0
    }

    /// Get total number of copies
    pub fn total_copies(&self) -> u32 {
        self.count_x * self.count_y
    }

    /// Calculate bounding box of the array
    pub fn calculate_bounds(&self, original_bounds: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
        let (min_x, min_y, max_x, max_y) = original_bounds;
        let width = max_x - min_x;
        let height = max_y - min_y;

        let array_width = width + (self.count_x - 1) as f64 * self.spacing_x;
        let array_height = height + (self.count_y - 1) as f64 * self.spacing_y;

        (min_x, min_y, min_x + array_width, min_y + array_height)
    }
}

/// Parameters for circular array operations
#[derive(Debug, Clone)]
pub struct CircularArrayParams {
    /// Number of copies to create
    pub count: u32,
    /// Center point of the array
    pub center: Point,
    /// Radius from center to original shape
    pub radius: f64,
    /// Starting angle in degrees (0-360)
    pub start_angle: f64,
    /// Rotation direction: true = clockwise, false = counter-clockwise
    pub clockwise: bool,
}

impl CircularArrayParams {
    /// Create new circular array parameters
    pub fn new(count: u32, center: Point, radius: f64, start_angle: f64, clockwise: bool) -> Self {
        Self {
            count,
            center,
            radius,
            start_angle,
            clockwise,
        }
    }

    /// Validate parameters
    pub fn is_valid(&self) -> bool {
        self.count > 0 && self.radius >= 0.0 && self.start_angle >= 0.0 && self.start_angle <= 360.0
    }

    /// Calculate angle step between copies
    pub fn angle_step(&self) -> f64 {
        360.0 / self.count as f64
    }

    /// Calculate the position of the Nth copy relative to original
    pub fn get_offset(&self, copy_index: u32) -> (f64, f64) {
        if copy_index == 0 {
            return (0.0, 0.0);
        }

        let angle_step = self.angle_step();
        let angle = if self.clockwise {
            self.start_angle - (copy_index as f64) * angle_step
        } else {
            self.start_angle + (copy_index as f64) * angle_step
        };

        let angle_rad = angle.to_radians();
        (self.radius * angle_rad.cos(), self.radius * angle_rad.sin())
    }
}

/// Parameters for grid array operations
#[derive(Debug, Clone)]
pub struct GridArrayParams {
    /// Number of columns
    pub columns: u32,
    /// Number of rows
    pub rows: u32,
    /// Horizontal spacing between columns (mm)
    pub column_spacing: f64,
    /// Vertical spacing between rows (mm)
    pub row_spacing: f64,
}

impl GridArrayParams {
    /// Create new grid array parameters
    pub fn new(columns: u32, rows: u32, column_spacing: f64, row_spacing: f64) -> Self {
        Self {
            columns,
            rows,
            column_spacing,
            row_spacing,
        }
    }

    /// Validate parameters
    pub fn is_valid(&self) -> bool {
        self.columns > 0 && self.rows > 0 && self.column_spacing >= 0.0 && self.row_spacing >= 0.0
    }

    /// Get total number of copies
    pub fn total_copies(&self) -> u32 {
        self.columns * self.rows
    }

    /// Calculate position offset for a specific cell in the grid
    pub fn get_offset(&self, column: u32, row: u32) -> Option<(f64, f64)> {
        if column >= self.columns || row >= self.rows {
            return None;
        }

        Some((
            column as f64 * self.column_spacing,
            row as f64 * self.row_spacing,
        ))
    }

    /// Calculate bounding box of the grid array
    pub fn calculate_bounds(&self, original_bounds: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
        let (min_x, min_y, max_x, max_y) = original_bounds;
        let width = max_x - min_x;
        let height = max_y - min_y;

        let array_width = width + (self.columns - 1) as f64 * self.column_spacing;
        let array_height = height + (self.rows - 1) as f64 * self.row_spacing;

        (min_x, min_y, min_x + array_width, min_y + array_height)
    }
}

/// Main array operation combining type and parameters
#[derive(Debug, Clone)]
pub enum ArrayOperation {
    /// Linear array with its parameters
    Linear(LinearArrayParams),
    /// Circular array with its parameters
    Circular(CircularArrayParams),
    /// Grid array with its parameters
    Grid(GridArrayParams),
}

impl ArrayOperation {
    /// Get the array type
    pub fn array_type(&self) -> ArrayType {
        match self {
            ArrayOperation::Linear(_) => ArrayType::Linear,
            ArrayOperation::Circular(_) => ArrayType::Circular,
            ArrayOperation::Grid(_) => ArrayType::Grid,
        }
    }

    /// Validate the array operation
    pub fn is_valid(&self) -> bool {
        match self {
            ArrayOperation::Linear(params) => params.is_valid(),
            ArrayOperation::Circular(params) => params.is_valid(),
            ArrayOperation::Grid(params) => params.is_valid(),
        }
    }

    /// Get total number of copies
    pub fn total_copies(&self) -> u32 {
        match self {
            ArrayOperation::Linear(params) => params.total_copies(),
            ArrayOperation::Circular(params) => params.count,
            ArrayOperation::Grid(params) => params.total_copies(),
        }
    }
}

/// Generator for array copies
pub struct ArrayGenerator;

impl ArrayGenerator {
    /// Generate copy offsets for a linear array
    pub fn generate_linear(params: &LinearArrayParams) -> Result<Vec<(f64, f64)>> {
        if !params.is_valid() {
            return Err(anyhow::anyhow!(
                "Invalid linear array parameters: count_x={}, count_y={}, spacing_x={}, spacing_y={}",
                params.count_x,
                params.count_y,
                params.spacing_x,
                params.spacing_y
            ));
        }

        let mut offsets = Vec::new();
        for y in 0..params.count_y {
            for x in 0..params.count_x {
                let offset_x = x as f64 * params.spacing_x;
                let offset_y = y as f64 * params.spacing_y;
                offsets.push((offset_x, offset_y));
            }
        }

        Ok(offsets)
    }

    /// Generate copy offsets for a circular array
    pub fn generate_circular(params: &CircularArrayParams) -> Result<Vec<(f64, f64)>> {
        if !params.is_valid() {
            return Err(anyhow::anyhow!(
                "Invalid circular array parameters: count={}, radius={}",
                params.count,
                params.radius
            ));
        }

        let mut offsets = Vec::new();
        for i in 0..params.count {
            let offset = params.get_offset(i);
            offsets.push(offset);
        }

        Ok(offsets)
    }

    /// Generate copy offsets for a grid array
    pub fn generate_grid(params: &GridArrayParams) -> Result<Vec<(f64, f64)>> {
        if !params.is_valid() {
            return Err(anyhow::anyhow!(
                "Invalid grid array parameters: columns={}, rows={}, column_spacing={}, row_spacing={}",
                params.columns,
                params.rows,
                params.column_spacing,
                params.row_spacing
            ));
        }

        let mut offsets = Vec::new();
        for row in 0..params.rows {
            for col in 0..params.columns {
                if let Some(offset) = params.get_offset(col, row) {
                    offsets.push(offset);
                }
            }
        }

        Ok(offsets)
    }

    /// Generate copy offsets for any array operation
    pub fn generate(operation: &ArrayOperation) -> Result<Vec<(f64, f64)>> {
        match operation {
            ArrayOperation::Linear(params) => Self::generate_linear(params),
            ArrayOperation::Circular(params) => Self::generate_circular(params),
            ArrayOperation::Grid(params) => Self::generate_grid(params),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_array_params_creation() {
        let params = LinearArrayParams::new(3, 2, 10.0, 20.0);
        assert_eq!(params.count_x, 3);
        assert_eq!(params.count_y, 2);
        assert_eq!(params.spacing_x, 10.0);
        assert_eq!(params.spacing_y, 20.0);
    }

    #[test]
    fn test_linear_array_validation() {
        let valid = LinearArrayParams::new(3, 2, 10.0, 20.0);
        assert!(valid.is_valid());

        let invalid = LinearArrayParams::new(0, 2, 10.0, 20.0);
        assert!(!invalid.is_valid());

        let negative_spacing = LinearArrayParams::new(3, 2, -10.0, 20.0);
        assert!(!negative_spacing.is_valid());
    }

    #[test]
    fn test_linear_array_total_copies() {
        let params = LinearArrayParams::new(3, 4, 10.0, 20.0);
        assert_eq!(params.total_copies(), 12);
    }

    #[test]
    fn test_linear_array_bounds() {
        let params = LinearArrayParams::new(3, 2, 10.0, 20.0);
        let original_bounds = (0.0, 0.0, 5.0, 5.0);
        let bounds = params.calculate_bounds(original_bounds);

        assert_eq!(bounds.0, 0.0); // min_x
        assert_eq!(bounds.1, 0.0); // min_y
        assert_eq!(bounds.2, 25.0); // max_x = 5 + 2*10
        assert_eq!(bounds.3, 25.0); // max_y = 5 + 1*20
    }

    #[test]
    fn test_circular_array_params_creation() {
        let center = Point::new(50.0, 50.0);
        let params = CircularArrayParams::new(8, center, 30.0, 0.0, false);
        assert_eq!(params.count, 8);
        assert_eq!(params.radius, 30.0);
    }

    #[test]
    fn test_circular_array_validation() {
        let center = Point::new(50.0, 50.0);
        let valid = CircularArrayParams::new(8, center, 30.0, 0.0, false);
        assert!(valid.is_valid());

        let invalid_count = CircularArrayParams::new(0, center, 30.0, 0.0, false);
        assert!(!invalid_count.is_valid());

        let invalid_angle = CircularArrayParams::new(8, center, 30.0, 400.0, false);
        assert!(!invalid_angle.is_valid());
    }

    #[test]
    fn test_circular_array_angle_step() {
        let center = Point::new(50.0, 50.0);
        let params = CircularArrayParams::new(4, center, 30.0, 0.0, false);
        assert_eq!(params.angle_step(), 90.0);

        let params8 = CircularArrayParams::new(8, center, 30.0, 0.0, false);
        assert_eq!(params8.angle_step(), 45.0);
    }

    #[test]
    fn test_circular_array_offset_zero() {
        let center = Point::new(0.0, 0.0);
        let params = CircularArrayParams::new(4, center, 10.0, 0.0, false);
        let (x, y) = params.get_offset(0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn test_grid_array_params_creation() {
        let params = GridArrayParams::new(5, 3, 15.0, 25.0);
        assert_eq!(params.columns, 5);
        assert_eq!(params.rows, 3);
        assert_eq!(params.column_spacing, 15.0);
        assert_eq!(params.row_spacing, 25.0);
    }

    #[test]
    fn test_grid_array_validation() {
        let valid = GridArrayParams::new(5, 3, 15.0, 25.0);
        assert!(valid.is_valid());

        let invalid = GridArrayParams::new(0, 3, 15.0, 25.0);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_grid_array_total_copies() {
        let params = GridArrayParams::new(5, 4, 15.0, 25.0);
        assert_eq!(params.total_copies(), 20);
    }

    #[test]
    fn test_grid_array_get_offset() {
        let params = GridArrayParams::new(3, 2, 10.0, 20.0);

        let offset00 = params.get_offset(0, 0);
        assert_eq!(offset00, Some((0.0, 0.0)));

        let offset10 = params.get_offset(1, 0);
        assert_eq!(offset10, Some((10.0, 0.0)));

        let offset01 = params.get_offset(0, 1);
        assert_eq!(offset01, Some((0.0, 20.0)));

        let offset11 = params.get_offset(1, 1);
        assert_eq!(offset11, Some((10.0, 20.0)));

        let out_of_bounds = params.get_offset(5, 5);
        assert_eq!(out_of_bounds, None);
    }

    #[test]
    fn test_grid_array_bounds() {
        let params = GridArrayParams::new(3, 2, 10.0, 20.0);
        let original_bounds = (0.0, 0.0, 5.0, 5.0);
        let bounds = params.calculate_bounds(original_bounds);

        assert_eq!(bounds.0, 0.0); // min_x
        assert_eq!(bounds.1, 0.0); // min_y
        assert_eq!(bounds.2, 25.0); // max_x = 5 + 2*10
        assert_eq!(bounds.3, 25.0); // max_y = 5 + 1*20
    }

    #[test]
    fn test_array_operation_enum() {
        let linear = ArrayOperation::Linear(LinearArrayParams::new(2, 2, 10.0, 10.0));
        assert_eq!(linear.array_type(), ArrayType::Linear);
        assert!(linear.is_valid());
        assert_eq!(linear.total_copies(), 4);

        let center = Point::new(0.0, 0.0);
        let circular =
            ArrayOperation::Circular(CircularArrayParams::new(6, center, 20.0, 0.0, false));
        assert_eq!(circular.array_type(), ArrayType::Circular);
        assert!(circular.is_valid());
        assert_eq!(circular.total_copies(), 6);

        let grid = ArrayOperation::Grid(GridArrayParams::new(3, 3, 10.0, 10.0));
        assert_eq!(grid.array_type(), ArrayType::Grid);
        assert!(grid.is_valid());
        assert_eq!(grid.total_copies(), 9);
    }

    #[test]
    fn test_array_generator_linear() {
        let params = LinearArrayParams::new(2, 2, 10.0, 20.0);
        let result = ArrayGenerator::generate_linear(&params);
        assert!(result.is_ok());

        let offsets = result.unwrap();
        assert_eq!(offsets.len(), 4);
        assert_eq!(offsets[0], (0.0, 0.0));
        assert_eq!(offsets[1], (10.0, 0.0));
        assert_eq!(offsets[2], (0.0, 20.0));
        assert_eq!(offsets[3], (10.0, 20.0));
    }

    #[test]
    fn test_array_generator_circular() {
        let center = Point::new(0.0, 0.0);
        let params = CircularArrayParams::new(4, center, 10.0, 0.0, false);
        let result = ArrayGenerator::generate_circular(&params);
        assert!(result.is_ok());

        let offsets = result.unwrap();
        assert_eq!(offsets.len(), 4);
    }

    #[test]
    fn test_array_generator_grid() {
        let params = GridArrayParams::new(2, 3, 10.0, 20.0);
        let result = ArrayGenerator::generate_grid(&params);
        assert!(result.is_ok());

        let offsets = result.unwrap();
        assert_eq!(offsets.len(), 6);
        assert_eq!(offsets[0], (0.0, 0.0));
        assert_eq!(offsets[1], (10.0, 0.0));
        assert_eq!(offsets[2], (0.0, 20.0));
        assert_eq!(offsets[3], (10.0, 20.0));
        assert_eq!(offsets[4], (0.0, 40.0));
        assert_eq!(offsets[5], (10.0, 40.0));
    }

    #[test]
    fn test_array_generator_main() {
        let linear = ArrayOperation::Linear(LinearArrayParams::new(2, 2, 10.0, 10.0));
        let result = ArrayGenerator::generate(&linear);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 4);
    }

    #[test]
    fn test_invalid_linear_array() {
        let invalid = LinearArrayParams::new(0, 0, 10.0, 10.0);
        let result = ArrayGenerator::generate_linear(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_circular_array() {
        let center = Point::new(0.0, 0.0);
        let invalid = CircularArrayParams::new(0, center, 10.0, 0.0, false);
        let result = ArrayGenerator::generate_circular(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_grid_array() {
        let invalid = GridArrayParams::new(0, 0, 10.0, 10.0);
        let result = ArrayGenerator::generate_grid(&invalid);
        assert!(result.is_err());
    }
}
