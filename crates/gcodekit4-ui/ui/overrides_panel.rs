//! Overrides Panel - Task 75
//!
//! Feed rate and spindle speed override controls

/// Override type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverrideType {
    /// Feed rate override (0-200%)
    FeedRate,
    /// Spindle speed override (0-200%)
    SpindleSpeed,
    /// Rapid feed override (fixed percentages)
    RapidFeed,
}

impl std::fmt::Display for OverrideType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FeedRate => write!(f, "Feed Rate"),
            Self::SpindleSpeed => write!(f, "Spindle Speed"),
            Self::RapidFeed => write!(f, "Rapid Feed"),
        }
    }
}

/// Rapid override preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RapidOverride {
    /// 25% rapid
    Percent25,
    /// 50% rapid
    Percent50,
    /// 100% rapid (full speed)
    Percent100,
}

impl RapidOverride {
    /// Get override value as percentage
    pub fn value(&self) -> u8 {
        match self {
            Self::Percent25 => 25,
            Self::Percent50 => 50,
            Self::Percent100 => 100,
        }
    }

    /// Get label
    pub fn label(&self) -> &str {
        match self {
            Self::Percent25 => "25%",
            Self::Percent50 => "50%",
            Self::Percent100 => "100%",
        }
    }

    /// Get all rapid overrides
    pub fn all() -> Vec<Self> {
        vec![Self::Percent25, Self::Percent50, Self::Percent100]
    }
}

impl std::fmt::Display for RapidOverride {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Feed rate override state
#[derive(Debug, Clone)]
pub struct FeedRateOverride {
    /// Current percentage (0-200)
    pub percentage: u8,
    /// Minimum percentage
    pub min: u8,
    /// Maximum percentage
    pub max: u8,
    /// Default percentage
    pub default: u8,
}

impl FeedRateOverride {
    /// Create new feed rate override
    pub fn new() -> Self {
        Self {
            percentage: 100,
            min: 0,
            max: 200,
            default: 100,
        }
    }

    /// Set percentage
    pub fn set_percentage(&mut self, value: u8) {
        self.percentage = value.clamp(self.min, self.max);
    }

    /// Increase percentage by delta
    pub fn increase(&mut self, delta: u8) {
        self.set_percentage(self.percentage.saturating_add(delta));
    }

    /// Decrease percentage by delta
    pub fn decrease(&mut self, delta: u8) {
        self.set_percentage(self.percentage.saturating_sub(delta));
    }

    /// Reset to default
    pub fn reset(&mut self) {
        self.percentage = self.default;
    }

    /// Get percentage as factor (100 = 1.0)
    pub fn as_factor(&self) -> f64 {
        self.percentage as f64 / 100.0
    }
}

impl Default for FeedRateOverride {
    fn default() -> Self {
        Self::new()
    }
}

/// Spindle speed override state
#[derive(Debug, Clone)]
pub struct SpindleSpeedOverride {
    /// Current percentage (0-200)
    pub percentage: u8,
    /// Minimum percentage
    pub min: u8,
    /// Maximum percentage
    pub max: u8,
    /// Default percentage
    pub default: u8,
}

impl SpindleSpeedOverride {
    /// Create new spindle speed override
    pub fn new() -> Self {
        Self {
            percentage: 100,
            min: 0,
            max: 200,
            default: 100,
        }
    }

    /// Set percentage
    pub fn set_percentage(&mut self, value: u8) {
        self.percentage = value.clamp(self.min, self.max);
    }

    /// Increase percentage by delta
    pub fn increase(&mut self, delta: u8) {
        self.set_percentage(self.percentage.saturating_add(delta));
    }

    /// Decrease percentage by delta
    pub fn decrease(&mut self, delta: u8) {
        self.set_percentage(self.percentage.saturating_sub(delta));
    }

    /// Reset to default
    pub fn reset(&mut self) {
        self.percentage = self.default;
    }

    /// Get percentage as factor (100 = 1.0)
    pub fn as_factor(&self) -> f64 {
        self.percentage as f64 / 100.0
    }
}

impl Default for SpindleSpeedOverride {
    fn default() -> Self {
        Self::new()
    }
}

/// Rapid feed override state
#[derive(Debug, Clone)]
pub struct RapidFeedOverride {
    /// Current override
    pub current: RapidOverride,
}

impl RapidFeedOverride {
    /// Create new rapid feed override
    pub fn new() -> Self {
        Self {
            current: RapidOverride::Percent100,
        }
    }

    /// Set rapid override
    pub fn set(&mut self, override_val: RapidOverride) {
        self.current = override_val;
    }

    /// Get current percentage
    pub fn percentage(&self) -> u8 {
        self.current.value()
    }

    /// Get percentage as factor (100 = 1.0)
    pub fn as_factor(&self) -> f64 {
        self.percentage() as f64 / 100.0
    }
}

impl Default for RapidFeedOverride {
    fn default() -> Self {
        Self::new()
    }
}

/// Overrides panel
#[derive(Debug)]
pub struct OverridesPanel {
    /// Feed rate override
    pub feed_rate: FeedRateOverride,
    /// Spindle speed override
    pub spindle_speed: SpindleSpeedOverride,
    /// Rapid feed override
    pub rapid_feed: RapidFeedOverride,
    /// Enable/disable overrides
    pub enabled: bool,
}

impl OverridesPanel {
    /// Create new overrides panel
    pub fn new() -> Self {
        Self {
            feed_rate: FeedRateOverride::new(),
            spindle_speed: SpindleSpeedOverride::new(),
            rapid_feed: RapidFeedOverride::new(),
            enabled: true,
        }
    }

    /// Enable overrides
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable overrides
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Toggle overrides
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Set feed rate percentage
    pub fn set_feed_rate(&mut self, percentage: u8) {
        if self.enabled {
            self.feed_rate.set_percentage(percentage);
        }
    }

    /// Increase feed rate
    pub fn increase_feed_rate(&mut self, delta: u8) {
        if self.enabled {
            self.feed_rate.increase(delta);
        }
    }

    /// Decrease feed rate
    pub fn decrease_feed_rate(&mut self, delta: u8) {
        if self.enabled {
            self.feed_rate.decrease(delta);
        }
    }

    /// Reset feed rate to default
    pub fn reset_feed_rate(&mut self) {
        self.feed_rate.reset();
    }

    /// Set spindle speed percentage
    pub fn set_spindle_speed(&mut self, percentage: u8) {
        if self.enabled {
            self.spindle_speed.set_percentage(percentage);
        }
    }

    /// Increase spindle speed
    pub fn increase_spindle_speed(&mut self, delta: u8) {
        if self.enabled {
            self.spindle_speed.increase(delta);
        }
    }

    /// Decrease spindle speed
    pub fn decrease_spindle_speed(&mut self, delta: u8) {
        if self.enabled {
            self.spindle_speed.decrease(delta);
        }
    }

    /// Reset spindle speed to default
    pub fn reset_spindle_speed(&mut self) {
        self.spindle_speed.reset();
    }

    /// Set rapid override
    pub fn set_rapid_override(&mut self, override_val: RapidOverride) {
        if self.enabled {
            self.rapid_feed.set(override_val);
        }
    }

    /// Reset all overrides to default
    pub fn reset_all(&mut self) {
        self.feed_rate.reset();
        self.spindle_speed.reset();
        self.rapid_feed = RapidFeedOverride::new();
    }

    /// Get current override values
    pub fn get_current_values(&self) -> (u8, u8, u8) {
        (
            self.feed_rate.percentage,
            self.spindle_speed.percentage,
            self.rapid_feed.percentage(),
        )
    }

    /// Get override factors
    pub fn get_factors(&self) -> (f64, f64, f64) {
        (
            self.feed_rate.as_factor(),
            self.spindle_speed.as_factor(),
            self.rapid_feed.as_factor(),
        )
    }

    /// Get override status string
    pub fn status_string(&self) -> String {
        format!(
            "Feed: {}% | Spindle: {}% | Rapid: {}%",
            self.feed_rate.percentage,
            self.spindle_speed.percentage,
            self.rapid_feed.percentage()
        )
    }
}

impl Default for OverridesPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rapid_override_value() {
        assert_eq!(RapidOverride::Percent25.value(), 25);
        assert_eq!(RapidOverride::Percent50.value(), 50);
        assert_eq!(RapidOverride::Percent100.value(), 100);
    }

    #[test]
    fn test_rapid_override_all() {
        let all = RapidOverride::all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_feed_rate_override() {
        let mut override_val = FeedRateOverride::new();
        assert_eq!(override_val.percentage, 100);
        override_val.set_percentage(150);
        assert_eq!(override_val.percentage, 150);
    }

    #[test]
    fn test_feed_rate_clamp() {
        let mut override_val = FeedRateOverride::new();
        override_val.set_percentage(200);
        assert_eq!(override_val.percentage, 200);
        override_val.set_percentage(0);
        assert_eq!(override_val.percentage, 0);
    }

    #[test]
    fn test_feed_rate_increase() {
        let mut override_val = FeedRateOverride::new();
        override_val.increase(25);
        assert_eq!(override_val.percentage, 125);
    }

    #[test]
    fn test_feed_rate_decrease() {
        let mut override_val = FeedRateOverride::new();
        override_val.decrease(25);
        assert_eq!(override_val.percentage, 75);
    }

    #[test]
    fn test_feed_rate_reset() {
        let mut override_val = FeedRateOverride::new();
        override_val.set_percentage(150);
        override_val.reset();
        assert_eq!(override_val.percentage, 100);
    }

    #[test]
    fn test_feed_rate_factor() {
        let override_val = FeedRateOverride::new();
        assert_eq!(override_val.as_factor(), 1.0);
        let mut override_val = FeedRateOverride::new();
        override_val.set_percentage(50);
        assert_eq!(override_val.as_factor(), 0.5);
    }

    #[test]
    fn test_spindle_override() {
        let mut override_val = SpindleSpeedOverride::new();
        override_val.set_percentage(200);
        assert_eq!(override_val.percentage, 200);
    }

    #[test]
    fn test_rapid_feed_override() {
        let mut override_val = RapidFeedOverride::new();
        override_val.set(RapidOverride::Percent50);
        assert_eq!(override_val.percentage(), 50);
    }

    #[test]
    fn test_overrides_panel() {
        let panel = OverridesPanel::new();
        assert!(panel.enabled);
        assert_eq!(panel.feed_rate.percentage, 100);
    }

    #[test]
    fn test_overrides_disabled() {
        let mut panel = OverridesPanel::new();
        panel.disable();
        let before = panel.feed_rate.percentage;
        panel.set_feed_rate(150);
        assert_eq!(panel.feed_rate.percentage, before);
    }

    #[test]
    fn test_overrides_current_values() {
        let panel = OverridesPanel::new();
        let (feed, spindle, rapid) = panel.get_current_values();
        assert_eq!(feed, 100);
        assert_eq!(spindle, 100);
        assert_eq!(rapid, 100);
    }

    #[test]
    fn test_overrides_reset_all() {
        let mut panel = OverridesPanel::new();
        panel.set_feed_rate(150);
        panel.set_spindle_speed(75);
        panel.reset_all();
        assert_eq!(panel.feed_rate.percentage, 100);
        assert_eq!(panel.spindle_speed.percentage, 100);
    }

    #[test]
    fn test_overrides_status_string() {
        let panel = OverridesPanel::new();
        let status = panel.status_string();
        assert!(status.contains("Feed:"));
        assert!(status.contains("Spindle:"));
        assert!(status.contains("Rapid:"));
    }
}
