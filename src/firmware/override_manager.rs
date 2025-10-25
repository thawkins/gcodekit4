//! Override manager framework
//!
//! Provides traits and implementations for managing feed rate, rapid, and spindle overrides.

use tracing::debug;

/// Override state
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverrideState {
    /// Feed rate override percentage (0-200%)
    pub feed_rate_override: f64,
    /// Rapid override level (0=0%, 1=25%, 2=50%, 3=100%)
    pub rapid_override: RapidOverrideLevel,
    /// Spindle override percentage (0-200%)
    pub spindle_override: f64,
}

/// Rapid override levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RapidOverrideLevel {
    /// Rapid movements are disabled (0%)
    Off = 0,
    /// Slow rapid (25%)
    Slow = 1,
    /// Medium rapid (50%)
    Medium = 2,
    /// Full rapid (100%)
    Full = 3,
}

impl Default for OverrideState {
    fn default() -> Self {
        Self {
            feed_rate_override: 100.0,
            rapid_override: RapidOverrideLevel::Full,
            spindle_override: 100.0,
        }
    }
}

/// Trait for override management
pub trait OverrideManagerTrait: Send + Sync {
    /// Set feed rate override
    fn set_feed_rate_override(&mut self, percentage: f64) -> anyhow::Result<()>;

    /// Get current feed rate override
    fn get_feed_rate_override(&self) -> f64;

    /// Set rapid override level
    fn set_rapid_override(&mut self, level: RapidOverrideLevel) -> anyhow::Result<()>;

    /// Get current rapid override level
    fn get_rapid_override(&self) -> RapidOverrideLevel;

    /// Set spindle override
    fn set_spindle_override(&mut self, percentage: f64) -> anyhow::Result<()>;

    /// Get current spindle override
    fn get_spindle_override(&self) -> f64;

    /// Get complete override state
    fn get_state(&self) -> OverrideState;

    /// Increase feed rate override by increment
    fn increase_feed_rate(&mut self, increment: f64) -> anyhow::Result<()> {
        let new_value = (self.get_feed_rate_override() + increment)
            .min(200.0)
            .max(0.0);
        self.set_feed_rate_override(new_value)
    }

    /// Decrease feed rate override by decrement
    fn decrease_feed_rate(&mut self, decrement: f64) -> anyhow::Result<()> {
        let new_value = (self.get_feed_rate_override() - decrement)
            .min(200.0)
            .max(0.0);
        self.set_feed_rate_override(new_value)
    }

    /// Increase spindle override by increment
    fn increase_spindle(&mut self, increment: f64) -> anyhow::Result<()> {
        let new_value = (self.get_spindle_override() + increment)
            .min(200.0)
            .max(0.0);
        self.set_spindle_override(new_value)
    }

    /// Decrease spindle override by decrement
    fn decrease_spindle(&mut self, decrement: f64) -> anyhow::Result<()> {
        let new_value = (self.get_spindle_override() - decrement)
            .min(200.0)
            .max(0.0);
        self.set_spindle_override(new_value)
    }
}

/// Default implementation of override manager
#[derive(Debug, Clone)]
pub struct DefaultOverrideManager {
    state: OverrideState,
}

impl DefaultOverrideManager {
    /// Create a new override manager
    pub fn new() -> Self {
        Self {
            state: OverrideState::default(),
        }
    }
}

impl Default for DefaultOverrideManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OverrideManagerTrait for DefaultOverrideManager {
    fn set_feed_rate_override(&mut self, percentage: f64) -> anyhow::Result<()> {
        if percentage < 0.0 || percentage > 200.0 {
            return Err(anyhow::anyhow!(
                "Feed rate override must be between 0 and 200%, got {}",
                percentage
            ));
        }
        debug!("Setting feed rate override to {}%", percentage);
        self.state.feed_rate_override = percentage;
        Ok(())
    }

    fn get_feed_rate_override(&self) -> f64 {
        self.state.feed_rate_override
    }

    fn set_rapid_override(&mut self, level: RapidOverrideLevel) -> anyhow::Result<()> {
        debug!("Setting rapid override to {:?}", level);
        self.state.rapid_override = level;
        Ok(())
    }

    fn get_rapid_override(&self) -> RapidOverrideLevel {
        self.state.rapid_override
    }

    fn set_spindle_override(&mut self, percentage: f64) -> anyhow::Result<()> {
        if percentage < 0.0 || percentage > 200.0 {
            return Err(anyhow::anyhow!(
                "Spindle override must be between 0 and 200%, got {}",
                percentage
            ));
        }
        debug!("Setting spindle override to {}%", percentage);
        self.state.spindle_override = percentage;
        Ok(())
    }

    fn get_spindle_override(&self) -> f64 {
        self.state.spindle_override
    }

    fn get_state(&self) -> OverrideState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_rate_override() {
        let mut manager = DefaultOverrideManager::new();
        assert_eq!(manager.get_feed_rate_override(), 100.0);

        manager.set_feed_rate_override(150.0).unwrap();
        assert_eq!(manager.get_feed_rate_override(), 150.0);

        assert!(manager.set_feed_rate_override(300.0).is_err());
    }

    #[test]
    fn test_rapid_override() {
        let mut manager = DefaultOverrideManager::new();
        assert_eq!(manager.get_rapid_override(), RapidOverrideLevel::Full);

        manager
            .set_rapid_override(RapidOverrideLevel::Slow)
            .unwrap();
        assert_eq!(manager.get_rapid_override(), RapidOverrideLevel::Slow);
    }

    #[test]
    fn test_spindle_override() {
        let mut manager = DefaultOverrideManager::new();
        assert_eq!(manager.get_spindle_override(), 100.0);

        manager.set_spindle_override(75.0).unwrap();
        assert_eq!(manager.get_spindle_override(), 75.0);
    }

    #[test]
    fn test_increase_feed_rate() {
        let mut manager = DefaultOverrideManager::new();
        manager.increase_feed_rate(10.0).unwrap();
        assert_eq!(manager.get_feed_rate_override(), 110.0);
    }
}
