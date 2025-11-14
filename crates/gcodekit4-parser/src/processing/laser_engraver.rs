//! Laser Image Engraving Tool
//!
//! Converts bitmap images to G-code for laser engraving using raster scanning.
//! Supports grayscale power modulation, bidirectional scanning, and various image formats.

use anyhow::{Context, Result};
use image::{DynamicImage, GrayImage};
use std::path::Path;

/// Scan direction for laser engraving
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScanDirection {
    /// Horizontal scanning (left to right)
    Horizontal,
    /// Vertical scanning (top to bottom)
    Vertical,
}

/// Laser engraving parameters
#[derive(Debug, Clone)]
pub struct EngravingParameters {
    /// Output width in millimeters
    pub width_mm: f32,
    /// Output height in millimeters (auto-calculated if None based on aspect ratio)
    pub height_mm: Option<f32>,
    /// Feed rate for engraving moves (mm/min)
    pub feed_rate: f32,
    /// Travel feed rate for rapid moves (mm/min)
    pub travel_rate: f32,
    /// Minimum laser power (0-100%)
    pub min_power: f32,
    /// Maximum laser power (0-100%)
    pub max_power: f32,
    /// Resolution in pixels per millimeter
    pub pixels_per_mm: f32,
    /// Scan direction
    pub scan_direction: ScanDirection,
    /// Use bidirectional scanning (faster but may reduce quality)
    pub bidirectional: bool,
    /// Invert image (dark becomes light, light becomes dark)
    pub invert: bool,
    /// Line spacing multiplier (1.0 = normal, >1.0 = faster with lines)
    pub line_spacing: f32,
    /// Laser power scale (0-1000 for GRBL S parameter)
    pub power_scale: f32,
}

impl Default for EngravingParameters {
    fn default() -> Self {
        Self {
            width_mm: 100.0,
            height_mm: None,
            feed_rate: 1000.0,
            travel_rate: 3000.0,
            min_power: 0.0,
            max_power: 100.0,
            pixels_per_mm: 10.0, // 10 pixels per mm = ~254 DPI
            scan_direction: ScanDirection::Horizontal,
            bidirectional: true,
            invert: false,
            line_spacing: 1.0,
            power_scale: 1000.0, // GRBL default S0-S1000
        }
    }
}

/// Laser engraving tool for bitmap images
pub struct LaserEngraver {
    image: GrayImage,
    params: EngravingParameters,
    output_width: u32,
    output_height: u32,
}

impl LaserEngraver {
    /// Create a new laser engraver from an image file
    pub fn from_file<P: AsRef<Path>>(path: P, params: EngravingParameters) -> Result<Self> {
        let img = image::open(path.as_ref()).context("Failed to load image file")?;
        Self::from_image(img, params)
    }

    /// Create a new laser engraver from a DynamicImage
    pub fn from_image(img: DynamicImage, params: EngravingParameters) -> Result<Self> {
        // Convert to grayscale
        let mut gray = img.to_luma8();

        // Apply inversion if requested
        if params.invert {
            for pixel in gray.pixels_mut() {
                pixel.0[0] = 255 - pixel.0[0];
            }
        }

        // Calculate output dimensions
        let output_width = (params.width_mm * params.pixels_per_mm) as u32;
        let aspect_ratio = gray.height() as f32 / gray.width() as f32;
        let output_height = if let Some(h) = params.height_mm {
            (h * params.pixels_per_mm) as u32
        } else {
            (output_width as f32 * aspect_ratio) as u32
        };

        Ok(Self {
            image: gray,
            params,
            output_width,
            output_height,
        })
    }

    /// Get the output dimensions in millimeters
    pub fn output_size_mm(&self) -> (f32, f32) {
        (
            self.output_width as f32 / self.params.pixels_per_mm,
            self.output_height as f32 / self.params.pixels_per_mm,
        )
    }

    /// Estimate engraving time in seconds
    pub fn estimate_time(&self) -> f32 {
        let (width_mm, height_mm) = self.output_size_mm();
        let line_spacing = 1.0 / self.params.pixels_per_mm * self.params.line_spacing;
        let num_lines = (height_mm / line_spacing) as u32;

        // Calculate engraving time
        let engrave_time = (width_mm * num_lines as f32) / self.params.feed_rate * 60.0;

        // Calculate travel time (return to start for unidirectional)
        let travel_time = if self.params.bidirectional {
            (height_mm / self.params.travel_rate) * 60.0
        } else {
            (width_mm * num_lines as f32) / self.params.travel_rate * 60.0
        };

        engrave_time + travel_time
    }

    /// Generate G-code for laser engraving
    pub fn generate_gcode(&self) -> Result<String> {
        self.generate_gcode_with_progress(|_| {})
    }

    /// Generate G-code for laser engraving with progress callback
    /// The callback receives progress from 0.0 to 1.0
    pub fn generate_gcode_with_progress<F>(&self, mut progress_callback: F) -> Result<String>
    where
        F: FnMut(f32),
    {
        let mut gcode = String::new();

        // Header comments
        gcode.push_str("; Laser Image Engraving G-code\n");
        gcode.push_str(&format!(
            "; Generated: {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));
        let (width_mm, height_mm) = self.output_size_mm();
        gcode.push_str(&format!(
            "; Image size: {:.2}mm x {:.2}mm\n",
            width_mm, height_mm
        ));
        gcode.push_str(&format!(
            "; Resolution: {:.1} pixels/mm\n",
            self.params.pixels_per_mm
        ));
        gcode.push_str(&format!(
            "; Feed rate: {:.0} mm/min\n",
            self.params.feed_rate
        ));
        gcode.push_str(&format!(
            "; Power range: {:.0}%-{:.0}%\n",
            self.params.min_power, self.params.max_power
        ));
        gcode.push_str(&format!(
            "; Estimated time: {:.1} minutes\n",
            self.estimate_time() / 60.0
        ));
        gcode.push_str(";\n");

        // Initialization sequence
        gcode.push_str("G21 ; Set units to millimeters\n");
        gcode.push_str("G90 ; Absolute positioning\n");
        gcode.push_str("G17 ; XY plane selection\n");
        gcode.push_str("\n");

        gcode.push_str("; Home and set work coordinate system\n");
        gcode.push_str("$H ; Home all axes (bottom-left corner)\n");
        gcode.push_str("G10 L2 P1 X0 Y0 Z0 ; Clear G54 offset\n");
        gcode.push_str("G54 ; Select work coordinate system 1\n");
        gcode.push_str("G0 X10.0 Y10.0 ; Move to work origin (10mm from corner)\n");
        gcode.push_str("G10 L20 P1 X0 Y0 Z0 ; Set current position as work zero\n");
        gcode.push_str(&format!(
            "G0 Z{:.2} F{:.0} ; Move to safe height\n",
            5.0, self.params.travel_rate
        ));
        gcode.push_str("\n");

        gcode.push_str("M5 ; Laser off\n");
        gcode.push_str("\n");

        progress_callback(0.0);

        // Resize image to output dimensions
        let resized = image::imageops::resize(
            &self.image,
            self.output_width,
            self.output_height,
            image::imageops::FilterType::Lanczos3,
        );

        progress_callback(0.1);

        let line_spacing = 1.0 / self.params.pixels_per_mm * self.params.line_spacing;
        let pixel_width = 1.0 / self.params.pixels_per_mm;

        match self.params.scan_direction {
            ScanDirection::Horizontal => {
                self.generate_horizontal_scan_with_progress(
                    &mut gcode,
                    &resized,
                    pixel_width,
                    line_spacing,
                    &mut progress_callback,
                )?;
            }
            ScanDirection::Vertical => {
                self.generate_vertical_scan_with_progress(
                    &mut gcode,
                    &resized,
                    pixel_width,
                    line_spacing,
                    &mut progress_callback,
                )?;
            }
        }

        progress_callback(0.9);

        // Footer
        gcode.push_str("\n; End of engraving\n");
        gcode.push_str("M5 ; Laser off\n");
        gcode.push_str("G0 X0 Y0 ; Return to origin\n");

        progress_callback(1.0);

        Ok(gcode)
    }

    /// Generate horizontal scanning G-code with progress callback
    fn generate_horizontal_scan_with_progress<F>(
        &self,
        gcode: &mut String,
        image: &GrayImage,
        pixel_width: f32,
        line_spacing: f32,
        progress_callback: &mut F,
    ) -> Result<()>
    where
        F: FnMut(f32),
    {
        let height = image.height();
        let width = image.width();
        let mut left_to_right = true;

        for y in 0..height {
            // Report progress every 10 lines to avoid overwhelming the UI thread
            if y % 10 == 0 || y == height - 1 {
                let progress = 0.1 + (y as f32 / height as f32) * 0.8;
                progress_callback(progress);
            }
            let y_pos = y as f32 * line_spacing;

            // Skip to start of line
            if left_to_right || !self.params.bidirectional {
                gcode.push_str(&format!("G0 X0 Y{:.3}\n", y_pos));
            } else {
                gcode.push_str(&format!(
                    "G0 X{:.3} Y{:.3}\n",
                    (width - 1) as f32 * pixel_width,
                    y_pos
                ));
            }

            // Process pixels in this line
            let mut in_burn = false;
            let mut last_power = 0;

            let x_range: Box<dyn Iterator<Item = u32>> =
                if left_to_right || !self.params.bidirectional {
                    Box::new(0..width)
                } else {
                    Box::new((0..width).rev())
                };

            for x in x_range {
                let pixel = image.get_pixel(x, y);
                let intensity = pixel.0[0];

                // Convert intensity to laser power
                let power = self.intensity_to_power(intensity);
                let power_value = (power * self.params.power_scale / 100.0) as u32;

                let x_pos = x as f32 * pixel_width;

                if power_value > 0 {
                    if !in_burn || power_value != last_power {
                        // Start burning or change power
                        gcode.push_str(&format!(
                            "G1 X{:.3} Y{:.3} F{:.0} M3 S{}\n",
                            x_pos, y_pos, self.params.feed_rate, power_value
                        ));
                        in_burn = true;
                        last_power = power_value;
                    } else {
                        // Continue burning at same power
                        gcode.push_str(&format!("G1 X{:.3} Y{:.3}\n", x_pos, y_pos));
                    }
                } else {
                    if in_burn {
                        // Stop burning
                        gcode.push_str("M5\n");
                        in_burn = false;
                    }
                    // Skip over white space (no command needed)
                }
            }

            // Turn off laser at end of line
            if in_burn {
                gcode.push_str("M5\n");
            }

            // Alternate direction for bidirectional mode
            if self.params.bidirectional {
                left_to_right = !left_to_right;
            }
        }

        Ok(())
    }

    /// Generate vertical scanning G-code with progress callback
    fn generate_vertical_scan_with_progress<F>(
        &self,
        gcode: &mut String,
        image: &GrayImage,
        pixel_width: f32,
        line_spacing: f32,
        progress_callback: &mut F,
    ) -> Result<()>
    where
        F: FnMut(f32),
    {
        let height = image.height();
        let width = image.width();
        let mut top_to_bottom = true;

        for x in 0..width {
            // Report progress every 10 columns to avoid overwhelming the UI thread
            if x % 10 == 0 || x == width - 1 {
                let progress = 0.1 + (x as f32 / width as f32) * 0.8;
                progress_callback(progress);
            }
            let x_pos = x as f32 * line_spacing;

            // Skip to start of column
            if top_to_bottom || !self.params.bidirectional {
                gcode.push_str(&format!("G0 X{:.3} Y0\n", x_pos));
            } else {
                gcode.push_str(&format!(
                    "G0 X{:.3} Y{:.3}\n",
                    x_pos,
                    (height - 1) as f32 * pixel_width
                ));
            }

            // Process pixels in this column
            let mut in_burn = false;
            let mut last_power = 0;

            let y_range: Box<dyn Iterator<Item = u32>> =
                if top_to_bottom || !self.params.bidirectional {
                    Box::new(0..height)
                } else {
                    Box::new((0..height).rev())
                };

            for y in y_range {
                let pixel = image.get_pixel(x, y);
                let intensity = pixel.0[0];

                let power = self.intensity_to_power(intensity);
                let power_value = (power * self.params.power_scale / 100.0) as u32;

                let y_pos = y as f32 * pixel_width;

                if power_value > 0 {
                    if !in_burn || power_value != last_power {
                        gcode.push_str(&format!(
                            "G1 X{:.3} Y{:.3} F{:.0} M3 S{}\n",
                            x_pos, y_pos, self.params.feed_rate, power_value
                        ));
                        in_burn = true;
                        last_power = power_value;
                    } else {
                        gcode.push_str(&format!("G1 X{:.3} Y{:.3}\n", x_pos, y_pos));
                    }
                } else {
                    if in_burn {
                        gcode.push_str("M5\n");
                        in_burn = false;
                    }
                }
            }

            if in_burn {
                gcode.push_str("M5\n");
            }

            if self.params.bidirectional {
                top_to_bottom = !top_to_bottom;
            }
        }

        Ok(())
    }

    /// Convert pixel intensity (0-255) to laser power (0-100%)
    fn intensity_to_power(&self, intensity: u8) -> f32 {
        let normalized = intensity as f32 / 255.0;
        self.params.min_power + (normalized * (self.params.max_power - self.params.min_power))
    }

    /// Get the processed grayscale image
    pub fn get_image(&self) -> &GrayImage {
        &self.image
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_parameters() {
        let params = EngravingParameters::default();
        assert_eq!(params.width_mm, 100.0);
        assert_eq!(params.feed_rate, 1000.0);
        assert!(params.bidirectional);
    }

    #[test]
    fn test_intensity_to_power() {
        let params = EngravingParameters {
            min_power: 0.0,
            max_power: 100.0,
            ..Default::default()
        };

        let img = DynamicImage::new_luma8(10, 10);
        let engraver = LaserEngraver::from_image(img, params).unwrap();

        assert_eq!(engraver.intensity_to_power(0), 0.0);
        assert_eq!(engraver.intensity_to_power(255), 100.0);
        assert!((engraver.intensity_to_power(127) - 49.8).abs() < 0.5);
    }

    #[test]
    fn test_output_size_calculation() {
        let params = EngravingParameters {
            width_mm: 50.0,
            height_mm: None,
            pixels_per_mm: 10.0,
            ..Default::default()
        };

        let img = DynamicImage::new_luma8(100, 50); // 2:1 aspect ratio
        let engraver = LaserEngraver::from_image(img, params).unwrap();

        let (w, h) = engraver.output_size_mm();
        assert_eq!(w, 50.0);
        assert_eq!(h, 25.0); // Maintains 2:1 aspect ratio
    }
}
