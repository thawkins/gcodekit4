//! Tests for event system

use gcodekit4::data::ControllerStatus;
use gcodekit4::{ControllerEvent, EventDispatcher};

#[test]
fn test_event_creation() {
    let _event = ControllerEvent::Disconnected;
    let _event = ControllerEvent::Connected("GRBL".to_string());
    let _event = ControllerEvent::Error("Test error".to_string());
}

#[test]
fn test_event_display() {
    let event = ControllerEvent::Connected("TestCTRL".to_string());
    assert!(event.to_string().contains("TestCTRL"));

    let event = ControllerEvent::Alarm(10, "Safety door opened".to_string());
    assert!(event.to_string().contains("Alarm 10"));
    assert!(event.to_string().contains("Safety door"));
}

#[tokio::test]
async fn test_event_dispatcher_creation() {
    let dispatcher = EventDispatcher::default_with_buffer();
    assert_eq!(dispatcher.subscriber_count(), 0);
}

#[tokio::test]
async fn test_event_dispatcher_subscription() {
    let dispatcher = EventDispatcher::new(50);
    let _rx1 = dispatcher.subscribe();
    let _rx2 = dispatcher.subscribe();
    assert_eq!(dispatcher.subscriber_count(), 2);
}

#[tokio::test]
async fn test_event_dispatcher_publish() {
    let dispatcher = EventDispatcher::default_with_buffer();
    let mut rx = dispatcher.subscribe();

    let event = ControllerEvent::Connected("TestCTRL".to_string());
    assert!(dispatcher.publish(event.clone()).is_ok());

    let received = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();

    match (event, received) {
        (ControllerEvent::Connected(expected), ControllerEvent::Connected(actual)) => {
            assert_eq!(expected, actual);
        }
        _ => panic!("Event mismatch"),
    }
}

#[tokio::test]
async fn test_event_dispatcher_broadcast_multiple_subscribers() {
    let dispatcher = EventDispatcher::default_with_buffer();
    let mut rx1 = dispatcher.subscribe();
    let mut rx2 = dispatcher.subscribe();
    let mut rx3 = dispatcher.subscribe();

    let event = ControllerEvent::Error("Broadcast test".to_string());
    dispatcher.publish(event).unwrap();

    // All subscribers should receive
    let msg1 = rx1.recv().await.unwrap();
    let msg2 = rx2.recv().await.unwrap();
    let msg3 = rx3.recv().await.unwrap();

    match (msg1, msg2, msg3) {
        (
            ControllerEvent::Error(msg1),
            ControllerEvent::Error(msg2),
            ControllerEvent::Error(msg3),
        ) => {
            assert_eq!(msg1, "Broadcast test");
            assert_eq!(msg2, "Broadcast test");
            assert_eq!(msg3, "Broadcast test");
        }
        _ => panic!("Expected error events"),
    }
}

#[tokio::test]
async fn test_event_dispatcher_clone() {
    let dispatcher1 = EventDispatcher::default_with_buffer();
    let dispatcher2 = dispatcher1.clone();

    let mut rx = dispatcher1.subscribe();

    let event = ControllerEvent::Disconnected;
    dispatcher2.publish(event).unwrap();

    let received = rx.recv().await.unwrap();
    assert!(matches!(received, ControllerEvent::Disconnected));
}

#[test]
fn test_event_position_change() {
    let event = ControllerEvent::PositionChanged {
        machine_pos: (10.0, 20.0, 5.0),
        work_pos: (0.0, 0.0, 0.0),
    };
    let display = event.to_string();
    assert!(display.contains("Position"));
    assert!(display.contains("(10"));
}

#[test]
fn test_event_spindle_speed_change() {
    let event = ControllerEvent::SpindleSpeedChanged(1000.0);
    let display = event.to_string();
    assert!(display.contains("Spindle"));
    assert!(display.contains("1000"));
}

#[test]
fn test_event_feed_rate_change() {
    let event = ControllerEvent::FeedRateChanged(500.0);
    let display = event.to_string();
    assert!(display.contains("Feed rate"));
    assert!(display.contains("500"));
}
