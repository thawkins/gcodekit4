//! # Adaptive Clearing Operations Module
//!
//! Provides adaptive clearing strategies for pocket operations that optimize material removal
//! while maintaining constant cutting force and tool load.
//!
//! Adaptive clearing dynamically adjusts cutting parameters (stepover/stepdown) based on
//! real-time or estimated cutting conditions. This extends tool life, improves surface finish,
//! and reduces machining time compared to fixed-parameter strategies.
//!
//! Supports:
//! - Load-based parameter adjustment
//! - Material-specific cutting parameters
//! - Dynamic stepover/stepdown calculation
//! - Tool wear tracking
//! - Performance monitoring

use anyhow::Result;

/// Material properties for adaptive cutting calculations
#[derive(Debug, Clone, Copy)]
pub struct MaterialProperties {
    /// Material type identifier
    pub material_type: MaterialType,
    /// Maximum recommended cutting speed (mm/min)
    pub max_feed_rate: f64,
    /// Recommended chip load (mm/tooth)
    pub chip_load: f64,
    /// Material hardness factor (relative, 1.0 = aluminum)
    pub hardness_factor: f64,
    /// Machinability rating (0.0-1.0, higher = easier to cut)
    pub machinability: f64,
}

/// Supported material types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialType {
    /// Aluminum and aluminum alloys
    Aluminum,
    /// Plastic and composites
    Plastic,
    /// Wood
    Wood,
    /// Brass and copper
    Brass,
    /// Steel
    Steel,
    /// Stainless steel
    StainlessSteel,
}

impl MaterialProperties {
    /// Create profile for aluminum
    pub fn aluminum() -> Self {
        Self {
            material_type: MaterialType::Aluminum,
            max_feed_rate: 200.0,
            chip_load: 0.05,
            hardness_factor: 1.0,
            machinability: 0.9,
        }
    }

    /// Create profile for plastic
    pub fn plastic() -> Self {
        Self {
            material_type: MaterialType::Plastic,
            max_feed_rate: 150.0,
            chip_load: 0.04,
            hardness_factor: 0.8,
            machinability: 0.95,
        }
    }

    /// Create profile for wood
    pub fn wood() -> Self {
        Self {
            material_type: MaterialType::Wood,
            max_feed_rate: 300.0,
            chip_load: 0.08,
            hardness_factor: 0.6,
            machinability: 0.98,
        }
    }

    /// Create profile for brass
    pub fn brass() -> Self {
        Self {
            material_type: MaterialType::Brass,
            max_feed_rate: 120.0,
            chip_load: 0.03,
            hardness_factor: 1.2,
            machinability: 0.85,
        }
    }

    /// Create profile for steel
    pub fn steel() -> Self {
        Self {
            material_type: MaterialType::Steel,
            max_feed_rate: 80.0,
            chip_load: 0.02,
            hardness_factor: 1.8,
            machinability: 0.7,
        }
    }

    /// Create profile for stainless steel
    pub fn stainless_steel() -> Self {
        Self {
            material_type: MaterialType::StainlessSteel,
            max_feed_rate: 60.0,
            chip_load: 0.015,
            hardness_factor: 2.5,
            machinability: 0.5,
        }
    }

    /// Validate profile parameters
    pub fn is_valid(&self) -> bool {
        self.max_feed_rate > 0.0
            && self.chip_load > 0.0
            && self.hardness_factor > 0.0
            && self.machinability > 0.0
            && self.machinability <= 1.0
    }
}

/// Tool load monitor for tracking cutting conditions
#[derive(Debug, Clone)]
pub struct LoadMonitor {
    /// Current estimated cutting load (0.0-1.0, where 1.0 = maximum)
    pub current_load: f64,
    /// Target cutting load for optimal conditions
    pub target_load: f64,
    /// Average load over recent operations
    pub average_load: f64,
    /// Load samples for averaging
    load_samples: Vec<f64>,
    /// Maximum sample history to keep
    max_samples: usize,
}

impl LoadMonitor {
    /// Create a new load monitor
    pub fn new(target_load: f64) -> Self {
        Self {
            current_load: 0.0,
            target_load,
            average_load: 0.0,
            load_samples: Vec::new(),
            max_samples: 100,
        }
    }

    /// Record a load sample
    pub fn record_sample(&mut self, load: f64) {
        if !(0.0..=1.0).contains(&load) {
            return;
        }

        self.load_samples.push(load);
        self.current_load = load;

        // Keep only recent samples
        if self.load_samples.len() > self.max_samples {
            self.load_samples.remove(0);
        }

        // Update average
        if !self.load_samples.is_empty() {
            self.average_load =
                self.load_samples.iter().sum::<f64>() / self.load_samples.len() as f64;
        }
    }

    /// Get load adjustment factor (1.0 = no change, <1.0 = reduce, >1.0 = increase)
    pub fn adjustment_factor(&self) -> f64 {
        if self.average_load == 0.0 {
            return 1.0;
        }

        // Linear adjustment toward target load
        self.target_load / self.average_load
    }

    /// Check if load is within acceptable range
    pub fn is_load_healthy(&self) -> bool {
        let lower_bound = self.target_load * 0.7;
        let upper_bound = self.target_load * 1.3;
        self.average_load >= lower_bound && self.average_load <= upper_bound
    }

    /// Clear load history
    pub fn reset(&mut self) {
        self.load_samples.clear();
        self.current_load = 0.0;
        self.average_load = 0.0;
    }
}

/// Dynamic stepover/stepdown calculator
#[derive(Debug, Clone)]
pub struct DynamicStepover {
    /// Base stepover value (mm)
    pub base_stepover: f64,
    /// Base stepdown value (mm)
    pub base_stepdown: f64,
    /// Current stepover with adjustments (mm)
    pub current_stepover: f64,
    /// Current stepdown with adjustments (mm)
    pub current_stepdown: f64,
    /// Minimum stepover (mm)
    pub min_stepover: f64,
    /// Maximum stepover (mm)
    pub max_stepover: f64,
}

impl DynamicStepover {
    /// Create a new dynamic stepover calculator
    pub fn new(base_stepover: f64, base_stepdown: f64) -> Self {
        let min_stepover = base_stepover * 0.3;
        let max_stepover = base_stepover * 1.5;

        Self {
            base_stepover,
            base_stepdown,
            current_stepover: base_stepover,
            current_stepdown: base_stepdown,
            min_stepover,
            max_stepover,
        }
    }

    /// Apply load-based adjustment to parameters
    pub fn apply_adjustment(&mut self, adjustment_factor: f64) {
        self.current_stepover =
            (self.base_stepover * adjustment_factor).clamp(self.min_stepover, self.max_stepover);

        self.current_stepdown = (self.base_stepdown * adjustment_factor)
            .clamp(self.base_stepdown * 0.5, self.base_stepdown * 1.5);
    }

    /// Get efficiency ratio (how aggressive the parameters are)
    pub fn efficiency_ratio(&self) -> f64 {
        (self.current_stepover / self.base_stepover + self.current_stepdown / self.base_stepdown)
            / 2.0
    }
}

/// Adaptive clearing strategy configuration
#[derive(Debug, Clone)]
pub struct AdaptiveClearing {
    /// Material profile being used
    pub material: MaterialProperties,
    /// Load monitor for tracking conditions
    pub load_monitor: LoadMonitor,
    /// Dynamic stepover/stepdown calculator
    pub stepover: DynamicStepover,
    /// Aggressiveness level (0.0-1.0, higher = more aggressive)
    pub aggressiveness: f64,
    /// Enable tool wear compensation
    pub enable_wear_compensation: bool,
    /// Estimated tool wear factor (0.0-1.0, where 1.0 = new tool)
    pub tool_condition: f64,
}

impl AdaptiveClearing {
    /// Create new adaptive clearing strategy
    pub fn new(
        material: MaterialProperties,
        base_stepover: f64,
        base_stepdown: f64,
        aggressiveness: f64,
    ) -> Self {
        let target_load = 0.7 + (aggressiveness * 0.2); // Target 70-90% load

        Self {
            material,
            load_monitor: LoadMonitor::new(target_load),
            stepover: DynamicStepover::new(base_stepover, base_stepdown),
            aggressiveness: aggressiveness.clamp(0.0, 1.0),
            enable_wear_compensation: true,
            tool_condition: 1.0,
        }
    }

    /// Validate configuration
    pub fn is_valid(&self) -> bool {
        self.material.is_valid()
            && self.stepover.base_stepover > 0.0
            && self.stepover.base_stepdown > 0.0
            && self.tool_condition > 0.0
            && self.tool_condition <= 1.0
    }

    /// Update strategy based on current load
    pub fn update(&mut self, load_reading: f64) {
        self.load_monitor.record_sample(load_reading);
        let adjustment = self.load_monitor.adjustment_factor();
        self.stepover.apply_adjustment(adjustment);
    }

    /// Apply tool wear compensation
    pub fn apply_wear_compensation(&mut self) {
        if !self.enable_wear_compensation {
            return;
        }

        // Reduce parameters based on tool wear
        let wear_factor = self.tool_condition;
        let adjusted_stepover = self.stepover.current_stepover * wear_factor;
        let adjusted_stepdown = self.stepover.current_stepdown * wear_factor;

        self.stepover.current_stepover =
            adjusted_stepover.clamp(self.stepover.min_stepover, self.stepover.max_stepover);
        self.stepover.current_stepdown = adjusted_stepdown.clamp(
            self.stepover.base_stepdown * 0.5,
            self.stepover.base_stepdown * 1.5,
        );
    }

    /// Simulate tool wear (decrease tool condition)
    pub fn simulate_wear(&mut self, wear_amount: f64) {
        self.tool_condition = (self.tool_condition - wear_amount).max(0.0);
    }

    /// Get estimated machining time reduction percentage
    pub fn time_reduction_estimate(&self) -> f64 {
        // Estimate based on efficiency vs. stock parameters
        let efficiency = self.stepover.efficiency_ratio();
        (efficiency - 1.0) * 100.0
    }
}

/// Adaptive clearing algorithm generator
pub struct AdaptiveAlgorithm;

impl AdaptiveAlgorithm {
    /// Calculate estimated load for given parameters
    pub fn estimate_load(
        tool_diameter: f64,
        feed_rate: f64,
        stepover: f64,
        spindle_speed: u32,
        material: &MaterialProperties,
    ) -> Result<f64> {
        if tool_diameter <= 0.0 || feed_rate <= 0.0 || stepover <= 0.0 || spindle_speed == 0 {
            return Err(anyhow::anyhow!("Invalid parameters for load estimation"));
        }

        // Simplified load calculation
        let chip_area = stepover * (feed_rate / spindle_speed as f64);
        let material_factor = 1.0 / material.machinability;
        let load = (chip_area * material_factor).min(1.0);

        Ok(load)
    }

    /// Generate adaptive pass sequence for a pocket operation
    pub fn generate_passes(
        clearing: &AdaptiveClearing,
        material_volume: f64,
        passes_needed: u32,
    ) -> Result<Vec<(f64, f64)>> {
        if material_volume <= 0.0 || passes_needed == 0 {
            return Err(anyhow::anyhow!("Invalid material volume or pass count"));
        }

        if !clearing.is_valid() {
            return Err(anyhow::anyhow!("Invalid adaptive clearing configuration"));
        }

        let mut passes = Vec::new();
        let _volume_per_pass = material_volume / passes_needed as f64;

        for pass in 0..passes_needed {
            // Simulate load increase as tool wears
            let pass_wear = pass as f64 * 0.02;
            let wear_factor = (1.0 - pass_wear).max(0.5);

            let adjusted_stepover = clearing.stepover.current_stepover * wear_factor;
            let adjusted_stepdown = clearing.stepover.current_stepdown * wear_factor;

            passes.push((adjusted_stepover, adjusted_stepdown));
        }

        Ok(passes)
    }

    /// Optimize feed rate based on material and tool condition
    pub fn optimize_feed_rate(
        material: &MaterialProperties,
        tool_flutes: u32,
        spindle_speed: u32,
        tool_condition: f64,
    ) -> Result<f64> {
        if tool_flutes == 0 || spindle_speed == 0 {
            return Err(anyhow::anyhow!("Invalid tool parameters"));
        }

        let base_feed = material.chip_load * tool_flutes as f64 * spindle_speed as f64;
        let optimized_feed = base_feed * tool_condition;

        Ok(optimized_feed.min(material.max_feed_rate))
    }
}


