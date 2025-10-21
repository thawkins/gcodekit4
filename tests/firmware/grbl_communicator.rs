//! Tests for GRBL Communicator (Task 26)
//!
//! Tests GRBL-specific communicator functionality including character counting
//! protocol and streaming support.

#[cfg(test)]
mod tests {
    use gcodekit4::communication::{ConnectionDriver, ConnectionParams, NoOpCommunicator};
    use gcodekit4::firmware::grbl::{GrblCommunicator, GrblCommunicatorConfig};

    #[test]
    fn test_grbl_communicator_config_default() {
        let config = GrblCommunicatorConfig::default();
        assert_eq!(config.rx_buffer_size, 128);
        assert_eq!(config.tx_buffer_size, 128);
    }

    #[test]
    fn test_grbl_communicator_creation() {
        let config = GrblCommunicatorConfig::default();
        let communicator = GrblCommunicator::new(Box::new(NoOpCommunicator::new()), config);
        assert!(!communicator.is_connected());
    }

    #[test]
    fn test_grbl_communicator_initial_state() {
        let config = GrblCommunicatorConfig::default();
        let communicator = GrblCommunicator::new(Box::new(NoOpCommunicator::new()), config);
        assert_eq!(communicator.get_pending_chars(), 0);
        assert_eq!(communicator.get_available_buffer(), 128);
    }

    #[test]
    fn test_grbl_communicator_ready_to_send() {
        let config = GrblCommunicatorConfig::default();
        let communicator = GrblCommunicator::new(Box::new(NoOpCommunicator::new()), config);
        
        // Should be ready to send a command within buffer
        assert!(communicator.is_ready_to_send(50));
        assert!(communicator.is_ready_to_send(128));
        
        // Should not be ready for oversized command
        assert!(!communicator.is_ready_to_send(129));
    }

    #[test]
    fn test_grbl_communicator_character_counting() {
        let config = GrblCommunicatorConfig::default();
        let communicator = GrblCommunicator::new(Box::new(NoOpCommunicator::new()), config);
        
        // Acknowledge some characters
        communicator.acknowledge_chars(50);
        assert_eq!(communicator.get_pending_chars(), 0);
        
        // After acknowledging, buffer should be fully available
        assert_eq!(communicator.get_available_buffer(), 128);
    }

    #[test]
    fn test_grbl_communicator_config_custom() {
        let config = GrblCommunicatorConfig {
            rx_buffer_size: 256,
            tx_buffer_size: 64,
        };
        let communicator = GrblCommunicator::new(Box::new(NoOpCommunicator::new()), config);
        assert_eq!(communicator.get_available_buffer(), 256);
    }

    #[test]
    fn test_grbl_communicator_running_state() {
        let config = GrblCommunicatorConfig::default();
        let communicator = GrblCommunicator::new(Box::new(NoOpCommunicator::new()), config);
        
        assert!(!communicator.is_running());
    }
}
