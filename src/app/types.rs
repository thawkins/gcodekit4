use std::collections::VecDeque;
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct GcodeSendState {
    pub lines: VecDeque<String>,
    pub pending_bytes: usize,
    pub line_lengths: VecDeque<usize>,
    pub total_sent: usize,
    pub total_lines: usize,
    pub start_time: Option<std::time::Instant>,
}

impl Default for GcodeSendState {
    fn default() -> Self {
        Self {
            lines: VecDeque::new(),
            pending_bytes: 0,
            line_lengths: VecDeque::new(),
            total_sent: 0,
            total_lines: 0,
            start_time: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VectorEngravingParams {
    pub feed_rate: f32,
    pub travel_rate: f32,
    pub cut_power: f32,
    pub engrave_power: f32,
    pub power_scale: f32,
    pub multi_pass: bool,
    pub num_passes: i32,
    pub z_increment: f32,
    pub invert_power: bool,
    pub desired_width: f32,
    pub offset_x: String,
    pub offset_y: String,
    pub enable_hatch: bool,
    pub hatch_angle: f32,
    pub hatch_spacing: f32,
    pub hatch_tolerance: f32,
    pub cross_hatch: bool,
    pub enable_dwell: bool,
    pub dwell_time: f32,
    pub vector_path: String,
}

#[derive(Serialize, Deserialize)]
pub struct BitmapEngravingParams {
    pub width_mm: f32,
    pub feed_rate: f32,
    pub travel_rate: f32,
    pub min_power: f32,
    pub max_power: f32,
    pub pixels_per_mm: f32,
    pub scan_direction: String,
    pub bidirectional: bool,
    pub invert: bool,
    pub line_spacing: f32,
    pub power_scale: f32,
    pub mirror_x: bool,
    pub mirror_y: bool,
    pub rotation: String,
    pub halftone: String,
    pub halftone_dot_size: i32,
    pub halftone_threshold: i32,
    pub offset_x: String,
    pub offset_y: String,
    pub image_path: String,
}

