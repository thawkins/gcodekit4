//! Laser Image Engraving Tool
//!
//! Converts bitmap images to G-code for laser engraving using raster scanning.
//! Supports halftoning via pepecore, mirroring, rotation, grayscale power modulation,
//! bidirectional scanning, and various image formats.
//! Images are rendered from bottom to top to match device coordinate space where Y increases upward.

use anyhow::{Context, Result};
use image::DynamicImage;
use pepecore::svec::SVec;
use pepecore::svec::Shape;
use pepecore::enums::{ImgData, DotType};
use std::path::Path;

/// Image rotation angles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RotationAngle {
    /// No rotation
    Degrees0,
    /// 90 degrees clockwise
    Degrees90,
    /// 180 degrees
    Degrees180,
    /// 270 degrees clockwise
    Degrees270,
}

/// Halftoning algorithm options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HalftoneMethod {
    /// No halftoning (direct intensity mapping)
    None,
    /// Circle dot halftoning (pepecore)
    Circle,
    /// Cross dot halftoning (pepecore)
    Cross,
    /// Ellipse dot halftoning (pepecore)
    Ellipse,
    /// Line dot halftoning (pepecore)
    Line,
    /// Inverted line halftoning (pepecore)
    InvertedLine,
}

/// Scan direction for laser engraving
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScanDirection {
    /// Horizontal scanning (left to right)
    Horizontal,
    /// Vertical scanning (top to bottom)
    Vertical,
}

/// Image transformation parameters
#[derive(Debug, Clone)]
pub struct ImageTransformations {
    /// Mirror image horizontally (flip around Y axis)
    pub mirror_x: bool,
    /// Mirror image vertically (flip around X axis)
    pub mirror_y: bool,
    /// Rotation angle
    pub rotation: RotationAngle,
    /// Halftoning method
    pub halftone: HalftoneMethod,
    /// Halftone dot size (cell size in pixels, typically 2-10)
    pub halftone_dot_size: usize,
    /// Halftone threshold (0-255, deprecated - kept for compatibility)
    pub halftone_threshold: u8,
    /// Invert image (dark becomes light, light becomes dark)
    pub invert: bool,
}

impl Default for ImageTransformations {
    fn default() -> Self {
        Self {
            mirror_x: false,
            mirror_y: false,
            rotation: RotationAngle::Degrees0,
            halftone: HalftoneMethod::None,
            halftone_dot_size: 4,
            halftone_threshold: 127,
            invert: false,
        }
    }
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
    /// Line spacing multiplier (1.0 = normal, >1.0 = faster with lines)
    pub line_spacing: f32,
    /// Laser power scale (0-1000 for GRBL S parameter)
    pub power_scale: f32,
    /// Image transformations (halftoning, mirroring, rotation)
    pub transformations: ImageTransformations,
    /// X offset from machine origin
    pub offset_x: f32,
    /// Y offset from machine origin
    pub offset_y: f32,
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
            pixels_per_mm: 10.0,
            scan_direction: ScanDirection::Horizontal,
            bidirectional: true,
            line_spacing: 1.0,
            power_scale: 1000.0,
            transformations: ImageTransformations::default(),
            offset_x: 10.0,
            offset_y: 10.0,
        }
    }
}

/// Laser engraving tool for bitmap images
pub struct BitmapImageEngraver {
    svec: SVec,
    params: EngravingParameters,
    output_width: u32,
    output_height: u32,
}

impl BitmapImageEngraver {
    /// Create a new laser engraver from an image file
    pub fn from_file<P: AsRef<Path>>(path: P, params: EngravingParameters) -> Result<Self> {
        let img = image::open(path.as_ref()).context("Failed to load image file")?;
        Self::from_image(img, params)
    }

    /// Create a new laser engraver from a DynamicImage
    pub fn from_image(img: DynamicImage, params: EngravingParameters) -> Result<Self> {
        let gray = img.to_luma8();
        let height = gray.height() as usize;
        let width = gray.width() as usize;

        // Create SVec in grayscale format using ImgData::U8
        // Grayscale images must specify 1 channel explicitly for pepecore compatibility
        let shape = Shape::new(height, width, Some(1));
        let mut svec = SVec::new(shape, ImgData::U8(gray.into_raw()));

        // Apply transformations: mirroring -> rotation -> inversion -> halftoning
        if params.transformations.mirror_x {
            Self::mirror_x_svec(&mut svec)?;
        }
        if params.transformations.mirror_y {
            Self::mirror_y_svec(&mut svec)?;
        }

        if params.transformations.rotation != RotationAngle::Degrees0 {
            svec = Self::apply_rotation_svec(svec, params.transformations.rotation)?;
        }

        if params.transformations.invert {
            Self::invert_svec(&mut svec)?;
        }

        if params.transformations.halftone != HalftoneMethod::None {
            Self::apply_halftoning_svec(&mut svec, params.transformations.halftone, params.transformations.halftone_dot_size)?;
        }

        let (shape_h, shape_w, _) = svec.shape();
        let output_width = (params.width_mm * params.pixels_per_mm) as u32;
        let aspect_ratio = shape_h as f32 / shape_w as f32;
        let output_height = if let Some(h) = params.height_mm {
            (h * params.pixels_per_mm) as u32
        } else {
            (output_width as f32 * aspect_ratio) as u32
        };

        Ok(Self {
            svec,
            params,
            output_width,
            output_height,
        })
    }

    /// Mirror image horizontally (flip around Y axis) using SVec
    pub fn mirror_x_svec(svec: &mut SVec) -> Result<()> {
        let (height, width, _) = svec.shape();
        
        // Get mutable access to data
        match &mut svec.data {
            ImgData::U8(data) => {
                for y in 0..height {
                    let row_start = y * width;
                    let row_end = row_start + width;
                    data[row_start..row_end].reverse();
                }
                Ok(())
            }
            ImgData::U16(data) => {
                for y in 0..height {
                    let row_start = y * width;
                    let row_end = row_start + width;
                    data[row_start..row_end].reverse();
                }
                Ok(())
            }
            ImgData::F32(data) => {
                for y in 0..height {
                    let row_start = y * width;
                    let row_end = row_start + width;
                    data[row_start..row_end].reverse();
                }
                Ok(())
            }
        }
    }

    /// Mirror image vertically (flip around X axis) using SVec
    fn mirror_y_svec(svec: &mut SVec) -> Result<()> {
        let (height, width, _) = svec.shape();
        
        match &mut svec.data {
            ImgData::U8(data) => {
                for y in 0..height / 2 {
                    let row1_start = y * width;
                    let row2_start = (height - 1 - y) * width;
                    for x in 0..width {
                        data.swap(row1_start + x, row2_start + x);
                    }
                }
                Ok(())
            }
            ImgData::U16(data) => {
                for y in 0..height / 2 {
                    let row1_start = y * width;
                    let row2_start = (height - 1 - y) * width;
                    for x in 0..width {
                        data.swap(row1_start + x, row2_start + x);
                    }
                }
                Ok(())
            }
            ImgData::F32(data) => {
                for y in 0..height / 2 {
                    let row1_start = y * width;
                    let row2_start = (height - 1 - y) * width;
                    for x in 0..width {
                        data.swap(row1_start + x, row2_start + x);
                    }
                }
                Ok(())
            }
        }
    }

    /// Invert image brightness
    fn invert_svec(svec: &mut SVec) -> Result<()> {
        match &mut svec.data {
            ImgData::U8(data) => {
                for pixel in data.iter_mut() {
                    *pixel = 255 - *pixel;
                }
                Ok(())
            }
            ImgData::U16(data) => {
                for pixel in data.iter_mut() {
                    *pixel = u16::MAX - *pixel;
                }
                Ok(())
            }
            ImgData::F32(data) => {
                for pixel in data.iter_mut() {
                    *pixel = 1.0 - *pixel;
                }
                Ok(())
            }
        }
    }

    /// Apply rotation to SVec image
    pub fn apply_rotation_svec(
        svec: SVec,
        rotation: RotationAngle,
    ) -> Result<SVec> {
        let (height, width, channels) = svec.shape();
        
        match rotation {
            RotationAngle::Degrees0 => Ok(svec),
            RotationAngle::Degrees90 => {
                let rotated_data = match &svec.data {
                    ImgData::U8(data) => {
                        let mut rotated = vec![0u8; data.len()];
                        for y in 0..height {
                            for x in 0..width {
                                let src_idx = y * width + x;
                                let dst_idx = x * height + (height - 1 - y);
                                rotated[dst_idx] = data[src_idx];
                            }
                        }
                        ImgData::U8(rotated)
                    }
                    ImgData::U16(data) => {
                        let mut rotated = vec![0u16; data.len()];
                        for y in 0..height {
                            for x in 0..width {
                                let src_idx = y * width + x;
                                let dst_idx = x * height + (height - 1 - y);
                                rotated[dst_idx] = data[src_idx];
                            }
                        }
                        ImgData::U16(rotated)
                    }
                    ImgData::F32(data) => {
                        let mut rotated = vec![0.0; data.len()];
                        for y in 0..height {
                            for x in 0..width {
                                let src_idx = y * width + x;
                                let dst_idx = x * height + (height - 1 - y);
                                rotated[dst_idx] = data[src_idx];
                            }
                        }
                        ImgData::F32(rotated)
                    }
                };
                let shape = Shape::new(width, height, channels);
                Ok(SVec::new(shape, rotated_data))
            }
            RotationAngle::Degrees180 => {
                match svec.data {
                    ImgData::U8(mut data) => {
                        data.reverse();
                        Ok(SVec::new(svec.shape, ImgData::U8(data)))
                    }
                    ImgData::U16(mut data) => {
                        data.reverse();
                        Ok(SVec::new(svec.shape, ImgData::U16(data)))
                    }
                    ImgData::F32(mut data) => {
                        data.reverse();
                        Ok(SVec::new(svec.shape, ImgData::F32(data)))
                    }
                }
            }
            RotationAngle::Degrees270 => {
                let rotated_data = match &svec.data {
                    ImgData::U8(data) => {
                        let mut rotated = vec![0u8; data.len()];
                        for y in 0..height {
                            for x in 0..width {
                                let src_idx = y * width + x;
                                let dst_idx = (width - 1 - x) * height + y;
                                rotated[dst_idx] = data[src_idx];
                            }
                        }
                        ImgData::U8(rotated)
                    }
                    ImgData::U16(data) => {
                        let mut rotated = vec![0u16; data.len()];
                        for y in 0..height {
                            for x in 0..width {
                                let src_idx = y * width + x;
                                let dst_idx = (width - 1 - x) * height + y;
                                rotated[dst_idx] = data[src_idx];
                            }
                        }
                        ImgData::U16(rotated)
                    }
                    ImgData::F32(data) => {
                        let mut rotated = vec![0.0; data.len()];
                        for y in 0..height {
                            for x in 0..width {
                                let src_idx = y * width + x;
                                let dst_idx = (width - 1 - x) * height + y;
                                rotated[dst_idx] = data[src_idx];
                            }
                        }
                        ImgData::F32(rotated)
                    }
                };
                let shape = Shape::new(width, height, channels);
                Ok(SVec::new(shape, rotated_data))
            }
        }
    }

    /// Apply halftoning using pepecore on SVec with configurable dot size
    fn apply_halftoning_svec(svec: &mut SVec, method: HalftoneMethod, dot_size: usize) -> Result<()> {
        let dot_type = match method {
            HalftoneMethod::Circle => DotType::CIRCLE,
            HalftoneMethod::Cross => DotType::CROSS,
            HalftoneMethod::Ellipse => DotType::ELLIPSE,
            HalftoneMethod::Line => DotType::LINE,
            HalftoneMethod::InvertedLine => DotType::INVLINE,
            HalftoneMethod::None => return Ok(()),
        };

        pepecore::halftone(svec, &[dot_size], &[dot_type])
            .context("Failed to apply halftone effect")
    }

    /// Get pixel at (x, y) in the processed image
    fn get_pixel(&self, x: u32, y: u32) -> u8 {
        let (height, width, _) = self.svec.shape();
        if x >= width as u32 || y >= height as u32 {
            return 255;
        }
        
        match &self.svec.data {
            ImgData::U8(data) => {
                data[y as usize * width + x as usize]
            }
            ImgData::U16(data) => {
                (data[y as usize * width + x as usize] >> 8) as u8
            }
            ImgData::F32(data) => {
                (data[y as usize * width + x as usize] * 255.0) as u8
            }
        }
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

        let engrave_time = (width_mm * num_lines as f32) / self.params.feed_rate * 60.0;
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
    pub fn generate_gcode_with_progress<F>(&self, mut progress_callback: F) -> Result<String>
    where
        F: FnMut(f32),
    {
        let mut gcode = String::new();

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

        gcode.push_str("G21 ; Set units to millimeters\n");
        gcode.push_str("G90 ; Absolute positioning\n");
        gcode.push_str("G17 ; XY plane selection\n");
        gcode.push_str("\n");

        gcode.push_str("; Home and set work coordinate system\n");
        gcode.push_str("$H ; Home all axes (bottom-left corner)\n");
        gcode.push_str("G10 L2 P1 X0 Y0 Z0 ; Clear G54 offset\n");
        gcode.push_str("G54 ; Select work coordinate system 1\n");
        gcode.push_str(&format!(
            "G0 X{:.1} Y{:.1} ; Move to work origin\n",
            self.params.offset_x, self.params.offset_y
        ));
        gcode.push_str("G10 L20 P1 X0 Y0 Z0 ; Set current position as work zero\n");
        gcode.push_str(&format!(
            "G0 Z{:.2} F{:.0} ; Move to safe height\n",
            5.0, self.params.travel_rate
        ));
        gcode.push_str("\n");

        gcode.push_str("M5 ; Laser off\n");
        gcode.push_str("\n");

        progress_callback(0.0);

        let resized = self.resize_image(self.output_width, self.output_height);
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

        gcode.push_str("\n; End of engraving\n");
        gcode.push_str("M5 ; Laser off\n");
        gcode.push_str("G0 X0 Y0 ; Return to origin\n");

        progress_callback(1.0);

        Ok(gcode)
    }

    /// Resize image to target dimensions using bilinear interpolation
    fn resize_image(&self, target_width: u32, target_height: u32) -> ResizedImage {
        let mut resized = vec![0u8; (target_width * target_height) as usize];

        let (height, width, _) = self.svec.shape();
        let src_width = width as f32;
        let src_height = height as f32;

        for y in 0..target_height {
            for x in 0..target_width {
                let src_x = (x as f32 / target_width as f32) * src_width;
                let src_y = (y as f32 / target_height as f32) * src_height;

                let x0 = src_x.floor() as u32;
                let y0 = src_y.floor() as u32;
                let x1 = (x0 + 1).min(width as u32 - 1);
                let y1 = (y0 + 1).min(height as u32 - 1);

                let fx = src_x - x0 as f32;
                let fy = src_y - y0 as f32;

                let p00 = self.get_pixel(x0, y0) as f32;
                let p10 = self.get_pixel(x1, y0) as f32;
                let p01 = self.get_pixel(x0, y1) as f32;
                let p11 = self.get_pixel(x1, y1) as f32;

                let p0 = p00 * (1.0 - fx) + p10 * fx;
                let p1 = p01 * (1.0 - fx) + p11 * fx;
                let p = p0 * (1.0 - fy) + p1 * fy;

                resized[(y * target_width + x) as usize] = p as u8;
            }
        }

        ResizedImage {
            data: resized,
            width: target_width,
            height: target_height,
        }
    }

    fn generate_horizontal_scan_with_progress<F>(
        &self,
        gcode: &mut String,
        image: &ResizedImage,
        pixel_width: f32,
        line_spacing: f32,
        progress_callback: &mut F,
    ) -> Result<()>
    where
        F: FnMut(f32),
    {
        let height = image.height;
        let width = image.width;
        let mut left_to_right = true;

        // Render from bottom to top to match device coordinate space
        for y_reversed in 0..height {
            if y_reversed % 10 == 0 || y_reversed == height - 1 {
                let progress = 0.1 + (y_reversed as f32 / height as f32) * 0.8;
                progress_callback(progress);
            }

            let y = height - 1 - y_reversed;
            let y_pos = y_reversed as f32 * line_spacing;

            if left_to_right || !self.params.bidirectional {
                gcode.push_str(&format!("G0 X0 Y{:.3}\n", y_pos));
            } else {
                gcode.push_str(&format!(
                    "G0 X{:.3} Y{:.3}\n",
                    (width - 1) as f32 * pixel_width,
                    y_pos
                ));
            }

            let mut in_burn = false;
            let mut last_power = 0;

            let x_range: Box<dyn Iterator<Item = u32>> = if left_to_right
                || !self.params.bidirectional
            {
                Box::new(0..width)
            } else {
                Box::new((0..width).rev())
            };

            for x in x_range {
                let intensity = image.get_pixel(x, y);
                let power = self.intensity_to_power(intensity);
                let power_value = (power * self.params.power_scale / 100.0) as u32;
                let x_pos = x as f32 * pixel_width;

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
                } else if in_burn {
                    gcode.push_str("M5\n");
                    in_burn = false;
                }
            }

            if in_burn {
                gcode.push_str("M5\n");
            }

            if self.params.bidirectional {
                left_to_right = !left_to_right;
            }
        }

        Ok(())
    }

    fn generate_vertical_scan_with_progress<F>(
        &self,
        gcode: &mut String,
        image: &ResizedImage,
        pixel_width: f32,
        line_spacing: f32,
        progress_callback: &mut F,
    ) -> Result<()>
    where
        F: FnMut(f32),
    {
        let height = image.height;
        let width = image.width;
        let mut top_to_bottom = true;

        for x in 0..width {
            if x % 10 == 0 || x == width - 1 {
                let progress = 0.1 + (x as f32 / width as f32) * 0.8;
                progress_callback(progress);
            }
            let x_pos = x as f32 * line_spacing;

            if top_to_bottom || !self.params.bidirectional {
                gcode.push_str(&format!("G0 X{:.3} Y0\n", x_pos));
            } else {
                gcode.push_str(&format!(
                    "G0 X{:.3} Y{:.3}\n",
                    x_pos,
                    (height - 1) as f32 * pixel_width
                ));
            }

            let mut in_burn = false;
            let mut last_power = 0;

            let y_range: Box<dyn Iterator<Item = u32>> = if top_to_bottom
                || !self.params.bidirectional
            {
                Box::new(0..height)
            } else {
                Box::new((0..height).rev())
            };

            for y_reversed in y_range {
                let y = height - 1 - y_reversed;
                let intensity = image.get_pixel(x, y);
                let power = self.intensity_to_power(intensity);
                let power_value = (power * self.params.power_scale / 100.0) as u32;
                let y_pos = y_reversed as f32 * pixel_width;

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
                } else if in_burn {
                    gcode.push_str("M5\n");
                    in_burn = false;
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

    /// Convert pixel intensity to laser power
    fn intensity_to_power(&self, intensity: u8) -> f32 {
        let normalized = intensity as f32 / 255.0;
        self.params.min_power + (normalized * (self.params.max_power - self.params.min_power))
    }
}

/// Resized image data
struct ResizedImage {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl ResizedImage {
    fn get_pixel(&self, x: u32, y: u32) -> u8 {
        if x >= self.width || y >= self.height {
            return 255;
        }
        self.data[(y * self.width + x) as usize]
    }
}


