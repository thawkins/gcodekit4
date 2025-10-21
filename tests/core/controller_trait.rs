//! Tests for controller interface and simple implementation

use gcodekit4::{ControllerTrait, SimpleController};

#[tokio::test]
async fn test_simple_controller_creation() {
    let controller = SimpleController::new("test-controller");
    assert_eq!(controller.name(), "test-controller");
}

#[tokio::test]
async fn test_simple_controller_default() {
    let controller = SimpleController::default();
    assert_eq!(controller.name(), "default");
}

#[tokio::test]
async fn test_simple_controller_connect_disconnect() {
    let mut controller = SimpleController::new("test");
    
    assert!(!controller.is_connected());
    
    controller.connect().await.unwrap();
    assert!(controller.is_connected());
    
    controller.disconnect().await.unwrap();
    assert!(!controller.is_connected());
}

#[tokio::test]
async fn test_simple_controller_send_command() {
    let mut controller = SimpleController::new("test");
    
    // Should work without being connected (for testing)
    controller.send_command("G28").await.unwrap();
    controller.send_command("G0 X10 Y10").await.unwrap();
}

#[tokio::test]
async fn test_simple_controller_multiple_commands() {
    let mut controller = SimpleController::new("test");
    
    let commands = vec!["G28", "G0 X0", "M30"];
    controller.send_commands(&commands).await.unwrap();
}

#[tokio::test]
async fn test_simple_controller_home() {
    let mut controller = SimpleController::new("test");
    controller.connect().await.unwrap();
    
    controller.home().await.unwrap();
}

#[tokio::test]
async fn test_simple_controller_reset() {
    let mut controller = SimpleController::new("test");
    controller.connect().await.unwrap();
    
    controller.reset().await.unwrap();
    assert!(!controller.is_connected());
}

#[tokio::test]
async fn test_simple_controller_clear_alarm() {
    let mut controller = SimpleController::new("test");
    
    controller.clear_alarm().await.unwrap();
}

#[tokio::test]
async fn test_simple_controller_unlock() {
    let mut controller = SimpleController::new("test");
    
    controller.unlock().await.unwrap();
}

#[tokio::test]
async fn test_simple_controller_jog_operations() {
    let mut controller = SimpleController::new("test");
    
    controller.jog_start('X', 1, 100.0).await.unwrap();
    controller.jog_stop().await.unwrap();
    controller.jog_incremental('Y', 5.0, 50.0).await.unwrap();
    controller.jog_incremental('Z', -2.5, 25.0).await.unwrap();
}

#[tokio::test]
async fn test_simple_controller_streaming_operations() {
    let mut controller = SimpleController::new("test");
    controller.connect().await.unwrap();
    
    controller.start_streaming().await.unwrap();
    controller.pause_streaming().await.unwrap();
    controller.resume_streaming().await.unwrap();
    controller.cancel_streaming().await.unwrap();
}

#[tokio::test]
async fn test_simple_controller_probing() {
    let mut controller = SimpleController::new("test");
    
    let z_pos = controller.probe_z(100.0).await.unwrap();
    assert_eq!(z_pos.z, None);
    
    let x_pos = controller.probe_x(50.0).await.unwrap();
    assert_eq!(x_pos.x, None);
    
    let y_pos = controller.probe_y(50.0).await.unwrap();
    assert_eq!(y_pos.y, None);
}

#[tokio::test]
async fn test_simple_controller_overrides() {
    let mut controller = SimpleController::new("test");
    
    controller.set_feed_override(150).await.unwrap();
    controller.set_rapid_override(50).await.unwrap();
    controller.set_spindle_override(200).await.unwrap();
}

#[tokio::test]
async fn test_simple_controller_work_coordinate_system() {
    let mut controller = SimpleController::new("test");
    
    controller.set_work_zero().await.unwrap();
    controller.set_work_zero_axes("XY").await.unwrap();
    controller.go_to_work_zero().await.unwrap();
    controller.set_work_coordinate_system(54).await.unwrap();
    
    let _offset = controller.get_wcs_offset(54).await.unwrap();
}

#[tokio::test]
async fn test_simple_controller_query_operations() {
    let mut controller = SimpleController::new("test");
    
    let status = controller.query_status().await.unwrap();
    // Just verify it succeeds
    assert_eq!(status.to_string(), "Idle");
    
    controller.query_settings().await.unwrap();
    controller.query_parser_state().await.unwrap();
}

#[tokio::test]
async fn test_simple_controller_listener_management() {
    let mut controller = SimpleController::new("test");
    
    assert_eq!(controller.listener_count(), 0);
    
    // Listeners are managed but simplified in SimpleController
    // Just verify the interface works
}

#[test]
fn test_override_state_default() {
    use gcodekit4::OverrideState;
    
    let override_state = OverrideState::default();
    assert_eq!(override_state.feed_override, 100);
    assert_eq!(override_state.rapid_override, 100);
    assert_eq!(override_state.spindle_override, 100);
}
