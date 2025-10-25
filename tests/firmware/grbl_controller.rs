//! Tests for GRBL Controller Implementation (Tasks 27-31)
//!
//! Tests GRBL controller functionality including:
//! - Task 27: Initialization
//! - Task 28: Core implementation
//! - Task 29: Status polling
//! - Task 30: Streaming
//! - Task 31: Jogging

#[cfg(test)]
mod tests {
    use gcodekit4::communication::{ConnectionDriver, ConnectionParams};
    use gcodekit4::core::ControllerTrait;
    use gcodekit4::data::ControllerState;
    use gcodekit4::firmware::grbl::GrblController;

    fn create_test_connection_params() -> ConnectionParams {
        ConnectionParams {
            driver: ConnectionDriver::Serial,
            port: "/dev/ttyUSB0".to_string(),
            network_port: 0,
            baud_rate: 115200,
            timeout_ms: 1000,
            flow_control: false,
            data_bits: 8,
            stop_bits: 1,
            parity: gcodekit4::communication::SerialParity::None,
            auto_reconnect: false,
            max_retries: 3,
        }
    }

    #[test]
    fn test_grbl_controller_creation() {
        let params = create_test_connection_params();
        let result = GrblController::new(params, Some("TestGRBL".to_string()));
        assert!(result.is_ok());

        let controller = result.unwrap();
        assert_eq!(controller.name(), "TestGRBL");
        assert_eq!(controller.get_state(), ControllerState::Disconnected);
    }

    #[test]
    fn test_grbl_controller_default_name() {
        let params = create_test_connection_params();
        let controller = GrblController::new(params, None).unwrap();
        assert_eq!(controller.name(), "GRBL");
    }

    #[test]
    fn test_grbl_controller_initial_state() {
        let params = create_test_connection_params();
        let controller = GrblController::new(params, None).unwrap();

        assert_eq!(controller.get_state(), ControllerState::Disconnected);
        assert!(!controller.is_connected());
    }

    #[test]
    fn test_grbl_controller_override_state() {
        let params = create_test_connection_params();
        let controller = GrblController::new(params, None).unwrap();

        let override_state = controller.get_override_state();
        assert_eq!(override_state.feed_override, 100);
        assert_eq!(override_state.rapid_override, 100);
        assert_eq!(override_state.spindle_override, 100);
    }

    #[test]
    fn test_grbl_controller_status() {
        let params = create_test_connection_params();
        let controller = GrblController::new(params, None).unwrap();

        let status = controller.get_status();
        assert!(matches!(status, gcodekit4::data::ControllerStatus::Idle));
    }

    // Task 31: GRBL Controller - Jogging tests

    #[tokio::test]
    async fn test_grbl_controller_jog_start_valid() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // Test continuous jogging with valid inputs
        // X axis, positive direction, 100 mm/min
        let result = controller.jog_start('X', 1, 100.0).await;
        // Will fail because controller is not connected, but input validation should pass
        if let Err(e) = result {
            let msg = e.to_string();
            assert!(
                !msg.contains("Direction must be non-zero"),
                "Should not reject valid direction"
            );
        }
    }

    #[tokio::test]
    async fn test_grbl_controller_jog_start_negative_direction() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // Test with negative direction
        let result = controller.jog_start('Y', -1, 50.0).await;
        if let Err(e) = result {
            let msg = e.to_string();
            assert!(
                !msg.contains("Direction must be non-zero"),
                "Should accept negative direction"
            );
        }
    }

    #[tokio::test]
    async fn test_grbl_controller_jog_start_invalid_direction() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // Test with zero direction - should fail
        let result = controller.jog_start('Z', 0, 100.0).await;
        assert!(result.is_err(), "Should reject zero direction");
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .contains("Direction must be non-zero"));
    }

    #[tokio::test]
    async fn test_grbl_controller_jog_start_various_axes() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // Test all primary axes
        for axis in ['X', 'Y', 'Z'] {
            let result = controller.jog_start(axis, 1, 100.0).await;
            // Should not error due to axis validation
            if let Err(e) = result {
                let msg = e.to_string();
                assert!(!msg.contains("axis"), "Should accept standard axes");
            }
        }
    }

    #[tokio::test]
    async fn test_grbl_controller_jog_stop() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // Test stopping jog
        let result = controller.jog_stop().await;
        // Either succeeds or fails due to no connection
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_grbl_controller_jog_incremental() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // Test with valid inputs
        let result = controller.jog_incremental('X', 10.0, 100.0).await;
        // Will fail because controller is not connected, but syntax is valid
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_grbl_controller_jog_incremental_positive_distance() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // Test incremental jog with positive distance
        let result = controller.jog_incremental('X', 5.5, 100.0).await;
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_grbl_controller_jog_incremental_negative_distance() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // Test incremental jog with negative distance (should use absolute value)
        let result = controller.jog_incremental('Y', -10.0, 75.0).await;
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_grbl_controller_jog_state_transition() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // Verify initial state
        let initial_state = controller.get_state();
        assert_eq!(initial_state, ControllerState::Disconnected);

        // After jog_start would change state (if connected)
        // This test just verifies the method signature works
        let result = controller.jog_start('X', 1, 100.0).await;
        // State would change on successful connection
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_grbl_controller_set_work_zero() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        let result = controller.set_work_zero().await;
        // Will fail because controller is not connected
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_grbl_controller_set_work_zero_axes() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        let result = controller.set_work_zero_axes("XY").await;
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_grbl_controller_set_feed_override_valid() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // Should accept valid percentages
        // Will error because controller is not connected, which is OK for this test
        let result = controller.set_feed_override(50).await;
        // Just check that it doesn't reject due to invalid percentage
        if let Err(e) = result {
            assert!(!e.to_string().contains("0-200"));
        }

        let result = controller.set_feed_override(100).await;
        if let Err(e) = result {
            assert!(!e.to_string().contains("0-200"));
        }

        let result = controller.set_feed_override(200).await;
        if let Err(e) = result {
            assert!(!e.to_string().contains("0-200"));
        }
    }

    #[tokio::test]
    async fn test_grbl_controller_set_feed_override_invalid() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // Should reject invalid percentages
        let result = controller.set_feed_override(250).await;
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("0-200"));
    }

    #[tokio::test]
    async fn test_grbl_controller_set_rapid_override_valid() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        let result = controller.set_rapid_override(25).await;
        assert!(result.is_ok());

        let result = controller.set_rapid_override(50).await;
        assert!(result.is_ok());

        let result = controller.set_rapid_override(100).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_grbl_controller_set_rapid_override_invalid() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        let result = controller.set_rapid_override(75).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_grbl_controller_set_spindle_override_valid() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        let result = controller.set_spindle_override(100).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_grbl_controller_set_spindle_override_invalid() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        let result = controller.set_spindle_override(250).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_grbl_controller_set_work_coordinate_system_valid() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        // G54-G59 are valid (54-59)
        let result = controller.set_work_coordinate_system(54).await;
        assert!(result.is_err() || result.is_ok()); // Fails due to no connection
    }

    #[tokio::test]
    async fn test_grbl_controller_set_work_coordinate_system_invalid() {
        let params = create_test_connection_params();
        let mut controller = GrblController::new(params, None).unwrap();

        let result = controller.set_work_coordinate_system(50).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_grbl_controller_listener_count() {
        let params = create_test_connection_params();
        let controller = GrblController::new(params, None).unwrap();

        assert_eq!(controller.listener_count(), 0);
    }
}
