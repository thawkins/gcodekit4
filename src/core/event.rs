//! Event system for controller communication
//!
//! Provides:
//! - Event types for controller and machine state changes
//! - Event dispatcher for publishing events to subscribers
//! - Listener registration and management

use std::sync::Arc;
use tokio::sync::broadcast;
use crate::data::{ControllerStatus, ControllerState};

/// Controller event types
#[derive(Debug, Clone)]
pub enum ControllerEvent {
    /// Connection state changed
    Connected(String),
    /// Disconnection occurred
    Disconnected,
    /// Controller state changed
    StateChanged(ControllerState),
    /// Controller status changed
    StatusChanged(ControllerStatus),
    /// Alarm occurred
    Alarm(u32, String),
    /// Error occurred
    Error(String),
    /// Command completed
    CommandComplete(String),
    /// Position changed
    PositionChanged {
        machine_pos: (f64, f64, f64),
        work_pos: (f64, f64, f64),
    },
    /// Spindle speed changed
    SpindleSpeedChanged(f64),
    /// Feed rate changed
    FeedRateChanged(f64),
}

impl std::fmt::Display for ControllerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControllerEvent::Connected(name) => write!(f, "Connected to {}", name),
            ControllerEvent::Disconnected => write!(f, "Disconnected"),
            ControllerEvent::StateChanged(state) => write!(f, "State: {}", state),
            ControllerEvent::StatusChanged(status) => write!(f, "Status: {}", status),
            ControllerEvent::Alarm(code, desc) => write!(f, "Alarm {} ({})", code, desc),
            ControllerEvent::Error(msg) => write!(f, "Error: {}", msg),
            ControllerEvent::CommandComplete(cmd) => write!(f, "Command complete: {}", cmd),
            ControllerEvent::PositionChanged { machine_pos, work_pos } => {
                write!(f, "Position - Machine: {:?}, Work: {:?}", machine_pos, work_pos)
            }
            ControllerEvent::SpindleSpeedChanged(speed) => write!(f, "Spindle: {} RPM", speed),
            ControllerEvent::FeedRateChanged(rate) => write!(f, "Feed rate: {} mm/min", rate),
        }
    }
}

/// Event dispatcher for publishing events to subscribers
pub struct EventDispatcher {
    tx: broadcast::Sender<ControllerEvent>,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    ///
    /// # Arguments
    /// * `buffer_size` - Size of the broadcast buffer (default 100)
    pub fn new(buffer_size: usize) -> Self {
        let (tx, _) = broadcast::channel(buffer_size);
        Self { tx }
    }

    /// Create a new event dispatcher with default buffer size
    pub fn default_with_buffer() -> Self {
        Self::new(100)
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<ControllerEvent> {
        self.tx.subscribe()
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: ControllerEvent) -> Result<usize, broadcast::error::SendError<ControllerEvent>> {
        tracing::trace!("Publishing event: {}", event);
        self.tx.send(event)
    }

    /// Get number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Clone for EventDispatcher {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::default_with_buffer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_display() {
        let connected_event = ControllerEvent::Connected("GRBL".to_string());
        assert!(connected_event.to_string().contains("Connected"));

        let disconnected_event = ControllerEvent::Disconnected;
        assert!(disconnected_event.to_string().contains("Disconnected"));

        let error_event = ControllerEvent::Error("Test error".to_string());
        assert!(error_event.to_string().contains("Test error"));
    }

    #[tokio::test]
    async fn test_event_dispatcher_creation() {
        let dispatcher = EventDispatcher::new(50);
        assert_eq!(dispatcher.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_event_dispatcher_subscribe() {
        let dispatcher = EventDispatcher::default_with_buffer();
        let _rx = dispatcher.subscribe();
        assert_eq!(dispatcher.subscriber_count(), 1);
    }

    #[tokio::test]
    async fn test_event_dispatcher_publish() {
        let dispatcher = EventDispatcher::default_with_buffer();
        let _rx = dispatcher.subscribe();

        let event = ControllerEvent::Disconnected;
        let result = dispatcher.publish(event);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_dispatcher_broadcast() {
        let dispatcher = EventDispatcher::default_with_buffer();
        let mut rx1 = dispatcher.subscribe();
        let mut rx2 = dispatcher.subscribe();

        let event = ControllerEvent::Error("Test".to_string());
        dispatcher.publish(event).unwrap();

        // Both subscribers should receive the event
        let event1 = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            rx1.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        let event2 = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            rx2.recv(),
        )
        .await
        .unwrap()
        .unwrap();

        match (event1, event2) {
            (ControllerEvent::Error(msg1), ControllerEvent::Error(msg2)) => {
                assert_eq!(msg1, "Test");
                assert_eq!(msg2, "Test");
            }
            _ => panic!("Expected error events"),
        }
    }
}
