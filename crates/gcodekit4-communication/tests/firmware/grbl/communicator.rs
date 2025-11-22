use gcodekit4_communication::firmware::grbl::communicator::*;

#[test]
fn test_character_counting_state_default() {
    let state = CharacterCountingState::default();
    assert_eq!(state.pending_chars, 0);
    assert_eq!(state.acked_chars, 0);
}

#[test]
fn test_grbl_config_default() {
    let config = GrblCommunicatorConfig::default();
    assert_eq!(config.rx_buffer_size, 128);
    assert_eq!(config.tx_buffer_size, 128);
}
