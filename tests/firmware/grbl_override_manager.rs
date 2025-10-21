//! Tests for GRBL Override Manager (Task 33)
//!
//! Tests GRBL override management including:
//! - Feed rate override (0-200%)
//! - Rapid override (25%, 50%, 100%)
//! - Spindle override (0-200%)
//! - Real-time override commands

#[cfg(test)]
mod tests {
    use gcodekit4::firmware::grbl::{OverrideManager, RealTimeOverrideCommand};

    #[test]
    fn test_override_manager_creation() {
        let manager = OverrideManager::new();
        assert_eq!(manager.get_feed_override(), 100);
        assert_eq!(manager.get_rapid_override(), 100);
        assert_eq!(manager.get_spindle_override(), 100);
        assert!(!manager.is_overridden());
    }

    #[test]
    fn test_feed_override_range() {
        let mut manager = OverrideManager::new();

        // Valid values
        assert!(manager.set_feed_override(0).is_ok());
        assert_eq!(manager.get_feed_override(), 0);

        assert!(manager.set_feed_override(100).is_ok());
        assert_eq!(manager.get_feed_override(), 100);

        assert!(manager.set_feed_override(200).is_ok());
        assert_eq!(manager.get_feed_override(), 200);

        // Invalid values
        assert!(manager.set_feed_override(250).is_err());
        assert_eq!(manager.get_feed_override(), 200); // unchanged
    }

    #[test]
    fn test_feed_override_increment() {
        let mut manager = OverrideManager::new();

        // Increase by 1%
        for _ in 0..5 {
            let prev = manager.get_feed_override();
            assert!(manager.increase_feed_1().is_ok());
            assert_eq!(manager.get_feed_override(), prev + 1);
        }

        // Should not exceed 200%
        manager.set_feed_override(199).ok();
        assert!(manager.increase_feed_10().is_ok());
        assert_eq!(manager.get_feed_override(), 200);
    }

    #[test]
    fn test_feed_override_decrement() {
        let mut manager = OverrideManager::new();
        manager.set_feed_override(100).ok();

        // Decrease by 1%
        for _ in 0..5 {
            let prev = manager.get_feed_override();
            assert!(manager.decrease_feed_1().is_ok());
            assert_eq!(manager.get_feed_override(), prev - 1);
        }

        // Should not go below 0%
        manager.set_feed_override(5).ok();
        assert!(manager.decrease_feed_10().is_ok());
        assert_eq!(manager.get_feed_override(), 0);
    }

    #[test]
    fn test_feed_override_10_percent_changes() {
        let mut manager = OverrideManager::new();

        // 10% increase
        assert!(manager.increase_feed_10().is_ok());
        assert_eq!(manager.get_feed_override(), 110);

        // 10% decrease
        assert!(manager.decrease_feed_10().is_ok());
        assert_eq!(manager.get_feed_override(), 100);

        // Multiple 10% increases
        for _ in 0..5 {
            assert!(manager.increase_feed_10().is_ok());
        }
        assert_eq!(manager.get_feed_override(), 150);
    }

    #[test]
    fn test_rapid_override_valid_values() {
        let mut manager = OverrideManager::new();

        // 25%
        assert!(manager.set_rapid_override(25).is_ok());
        assert_eq!(manager.get_rapid_override(), 25);

        // 50%
        assert!(manager.set_rapid_override(50).is_ok());
        assert_eq!(manager.get_rapid_override(), 50);

        // 100%
        assert!(manager.set_rapid_override(100).is_ok());
        assert_eq!(manager.get_rapid_override(), 100);
    }

    #[test]
    fn test_rapid_override_invalid_values() {
        let mut manager = OverrideManager::new();

        // Invalid percentages
        assert!(manager.set_rapid_override(10).is_err());
        assert!(manager.set_rapid_override(75).is_err());
        assert!(manager.set_rapid_override(200).is_err());

        // Should remain at default
        assert_eq!(manager.get_rapid_override(), 100);
    }

    #[test]
    fn test_rapid_override_commands() {
        let mut manager = OverrideManager::new();

        manager.set_rapid_override(25).ok();
        assert_eq!(
            manager.get_rapid_override_command(),
            RealTimeOverrideCommand::RapidOv25
        );

        manager.set_rapid_override(50).ok();
        assert_eq!(
            manager.get_rapid_override_command(),
            RealTimeOverrideCommand::RapidOv50
        );

        manager.set_rapid_override(100).ok();
        assert_eq!(
            manager.get_rapid_override_command(),
            RealTimeOverrideCommand::RapidOv100
        );
    }

    #[test]
    fn test_spindle_override_range() {
        let mut manager = OverrideManager::new();

        // Valid values
        assert!(manager.set_spindle_override(0).is_ok());
        assert_eq!(manager.get_spindle_override(), 0);

        assert!(manager.set_spindle_override(100).is_ok());
        assert_eq!(manager.get_spindle_override(), 100);

        assert!(manager.set_spindle_override(200).is_ok());
        assert_eq!(manager.get_spindle_override(), 200);

        // Invalid values
        assert!(manager.set_spindle_override(250).is_err());
        assert_eq!(manager.get_spindle_override(), 200); // unchanged
    }

    #[test]
    fn test_spindle_override_increment_decrement() {
        let mut manager = OverrideManager::new();
        manager.set_spindle_override(100).ok();

        // Increase by 1%
        assert!(manager.increase_spindle_1().is_ok());
        assert_eq!(manager.get_spindle_override(), 101);

        // Decrease by 1%
        assert!(manager.decrease_spindle_1().is_ok());
        assert_eq!(manager.get_spindle_override(), 100);

        // Increase by 10%
        assert!(manager.increase_spindle_10().is_ok());
        assert_eq!(manager.get_spindle_override(), 110);

        // Decrease by 10%
        assert!(manager.decrease_spindle_10().is_ok());
        assert_eq!(manager.get_spindle_override(), 100);
    }

    #[test]
    fn test_spindle_override_clipping() {
        let mut manager = OverrideManager::new();

        // Should not exceed 200%
        manager.set_spindle_override(199).ok();
        assert!(manager.increase_spindle_10().is_ok());
        assert_eq!(manager.get_spindle_override(), 200);

        // Should not go below 0%
        manager.set_spindle_override(5).ok();
        assert!(manager.decrease_spindle_10().is_ok());
        assert_eq!(manager.get_spindle_override(), 0);
    }

    #[test]
    fn test_stop_spindle() {
        let mut manager = OverrideManager::new();
        manager.set_spindle_override(150).ok();

        assert!(manager.stop_spindle().is_ok());
        assert_eq!(manager.get_spindle_override(), 0);
    }

    #[test]
    fn test_reset_all_overrides() {
        let mut manager = OverrideManager::new();

        // Set various overrides
        manager.set_feed_override(50).ok();
        manager.set_rapid_override(25).ok();
        manager.set_spindle_override(150).ok();

        assert!(manager.is_overridden());

        // Reset all
        manager.reset_all();

        assert_eq!(manager.get_feed_override(), 100);
        assert_eq!(manager.get_rapid_override(), 100);
        assert_eq!(manager.get_spindle_override(), 100);
        assert!(!manager.is_overridden());
    }

    #[test]
    fn test_is_overridden_detection() {
        let mut manager = OverrideManager::new();
        assert!(!manager.is_overridden());

        // Feed override
        manager.set_feed_override(50).ok();
        assert!(manager.is_overridden());

        manager.reset_all();
        assert!(!manager.is_overridden());

        // Rapid override
        manager.set_rapid_override(25).ok();
        assert!(manager.is_overridden());

        manager.reset_all();
        assert!(!manager.is_overridden());

        // Spindle override
        manager.set_spindle_override(150).ok();
        assert!(manager.is_overridden());
    }

    #[test]
    fn test_real_time_override_command_byte_values() {
        assert_eq!(RealTimeOverrideCommand::FeedHold.as_byte(), 0x21);
        assert_eq!(RealTimeOverrideCommand::CycleStart.as_byte(), 0x7E);
        assert_eq!(RealTimeOverrideCommand::Reset.as_byte(), 0x18);
        assert_eq!(RealTimeOverrideCommand::FeedDecrease10.as_byte(), 0x91);
        assert_eq!(RealTimeOverrideCommand::FeedDecrease1.as_byte(), 0x92);
        assert_eq!(RealTimeOverrideCommand::FeedIncrease1.as_byte(), 0x93);
        assert_eq!(RealTimeOverrideCommand::FeedIncrease10.as_byte(), 0x94);
        assert_eq!(RealTimeOverrideCommand::RapidOv25.as_byte(), 0x95);
        assert_eq!(RealTimeOverrideCommand::RapidOv50.as_byte(), 0x96);
        assert_eq!(RealTimeOverrideCommand::RapidOv100.as_byte(), 0x97);
        assert_eq!(RealTimeOverrideCommand::SpindleDecrease10.as_byte(), 0x99);
        assert_eq!(RealTimeOverrideCommand::SpindleDecrease1.as_byte(), 0x9A);
        assert_eq!(RealTimeOverrideCommand::SpindleIncrease1.as_byte(), 0x9B);
        assert_eq!(RealTimeOverrideCommand::SpindleIncrease10.as_byte(), 0x9C);
        assert_eq!(RealTimeOverrideCommand::SpindleStop.as_byte(), 0x9D);
    }

    #[test]
    fn test_all_overrides_combined() {
        let mut manager = OverrideManager::new();

        // Set all to different values
        manager.set_feed_override(75).ok();
        manager.set_rapid_override(50).ok();
        manager.set_spindle_override(125).ok();

        // Verify all are set
        assert_eq!(manager.get_feed_override(), 75);
        assert_eq!(manager.get_rapid_override(), 50);
        assert_eq!(manager.get_spindle_override(), 125);
        assert!(manager.is_overridden());

        // Modify one shouldn't affect others
        manager.increase_feed_10().ok();
        assert_eq!(manager.get_feed_override(), 85);
        assert_eq!(manager.get_rapid_override(), 50);
        assert_eq!(manager.get_spindle_override(), 125);
    }

    #[test]
    fn test_default_override_manager() {
        let manager = OverrideManager::default();
        assert_eq!(manager.get_feed_override(), 100);
        assert_eq!(manager.get_rapid_override(), 100);
        assert_eq!(manager.get_spindle_override(), 100);
    }

    #[test]
    fn test_feed_override_command_tracking() {
        let mut manager = OverrideManager::new();

        // No change from default
        assert!(manager.get_feed_override_command().is_none());

        // Increase by 10%
        manager.increase_feed_10().ok();
        let cmd = manager.get_feed_override_command();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap(), RealTimeOverrideCommand::FeedIncrease10);

        // Increase by 1%
        manager.reset_all();
        manager.set_feed_override(100).ok();
        manager.increase_feed_1().ok();
        let cmd = manager.get_feed_override_command();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap(), RealTimeOverrideCommand::FeedIncrease1);
    }

    #[test]
    fn test_spindle_override_command_tracking() {
        let mut manager = OverrideManager::new();

        // No change
        assert!(manager.get_spindle_override_command().is_none());

        // Increase by 10%
        manager.increase_spindle_10().ok();
        let cmd = manager.get_spindle_override_command();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap(), RealTimeOverrideCommand::SpindleIncrease10);

        // Stop spindle
        manager.reset_all();
        manager.stop_spindle().ok();
        let cmd = manager.get_spindle_override_command();
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap(), RealTimeOverrideCommand::SpindleStop);
    }
}
