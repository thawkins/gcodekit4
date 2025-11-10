//! UI Events and Communication - Task 66
//!
//! Event system for inter-component communication

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

/// UI event types
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// Connection event
    Connect(String, u32), // port, baud_rate
    /// Disconnection event
    Disconnect,
    /// File opened event
    FileOpened(String),
    /// File closed event
    FileClosed,
    /// Position updated event
    PositionUpdated(f64, f64, f64), // x, y, z
    /// Feed rate changed event
    FeedRateChanged(f64),
    /// Spindle speed changed event
    SpindleSpeedChanged(u16),
    /// Machine powered on
    MachinePoweredOn,
    /// Machine powered off
    MachinePoweredOff,
    /// Home machine event
    HomeMachine,
    /// Start execution
    StartExecution,
    /// Pause execution
    PauseExecution,
    /// Resume execution
    ResumeExecution,
    /// Stop execution
    StopExecution,
    /// Jog command
    Jog(f64, f64, f64), // x, y, z (increments)
    /// Error event
    Error(String),
    /// Status update
    StatusUpdate(String),
}

impl std::fmt::Display for UiEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect(port, baud) => write!(f, "Connect({}, {})", port, baud),
            Self::Disconnect => write!(f, "Disconnect"),
            Self::FileOpened(path) => write!(f, "FileOpened({})", path),
            Self::FileClosed => write!(f, "FileClosed"),
            Self::PositionUpdated(x, y, z) => write!(f, "PositionUpdated({}, {}, {})", x, y, z),
            Self::FeedRateChanged(rate) => write!(f, "FeedRateChanged({})", rate),
            Self::SpindleSpeedChanged(speed) => write!(f, "SpindleSpeedChanged({})", speed),
            Self::MachinePoweredOn => write!(f, "MachinePoweredOn"),
            Self::MachinePoweredOff => write!(f, "MachinePoweredOff"),
            Self::HomeMachine => write!(f, "HomeMachine"),
            Self::StartExecution => write!(f, "StartExecution"),
            Self::PauseExecution => write!(f, "PauseExecution"),
            Self::ResumeExecution => write!(f, "ResumeExecution"),
            Self::StopExecution => write!(f, "StopExecution"),
            Self::Jog(x, y, z) => write!(f, "Jog({}, {}, {})", x, y, z),
            Self::Error(msg) => write!(f, "Error({})", msg),
            Self::StatusUpdate(msg) => write!(f, "StatusUpdate({})", msg),
        }
    }
}

/// Event subscription
#[derive(Debug, Clone)]
pub struct EventSubscription {
    /// Subscription ID
    pub id: String,
    /// Subscriber name
    pub subscriber: String,
}

/// UI Event Bus for inter-component communication
#[derive(Debug)]
pub struct UiEventBus {
    /// Event sender
    sender: Sender<UiEvent>,
    /// Event receiver
    receiver: Receiver<UiEvent>,
    /// Event subscriptions
    subscriptions: Arc<Mutex<Vec<EventSubscription>>>,
}

impl UiEventBus {
    /// Create new event bus
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver,
            subscriptions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Publish event
    pub fn publish(&self, event: UiEvent) -> anyhow::Result<()> {
        self.sender.send(event)?;
        Ok(())
    }

    /// Subscribe to events
    pub fn subscribe(&self, subscriber: impl Into<String>) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let subscription = EventSubscription {
            id: id.clone(),
            subscriber: subscriber.into(),
        };

        if let Ok(mut subs) = self.subscriptions.lock() {
            subs.push(subscription);
        }

        id
    }

    /// Unsubscribe from events
    pub fn unsubscribe(&self, subscription_id: &str) -> bool {
        if let Ok(mut subs) = self.subscriptions.lock() {
            subs.retain(|s| s.id != subscription_id);
            true
        } else {
            false
        }
    }

    /// Get next event (non-blocking)
    pub fn try_recv(&self) -> Option<UiEvent> {
        self.receiver.try_recv().ok()
    }

    /// Get subscriptions count
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.lock().map(|s| s.len()).unwrap_or(0)
    }
}

impl Default for UiEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_creation() {
        let bus = UiEventBus::new();
        assert_eq!(bus.subscription_count(), 0);
    }

    #[test]
    fn test_event_subscription() {
        let bus = UiEventBus::new();
        let sub_id = bus.subscribe("TestComponent");
        assert!(!sub_id.is_empty());
        assert_eq!(bus.subscription_count(), 1);
    }

    #[test]
    fn test_event_unsubscription() {
        let bus = UiEventBus::new();
        let sub_id = bus.subscribe("TestComponent");
        assert!(bus.unsubscribe(&sub_id));
        assert_eq!(bus.subscription_count(), 0);
    }

    #[test]
    fn test_event_publishing() {
        let bus = UiEventBus::new();
        let event = UiEvent::Connect("COM1".to_string(), 115200);
        assert!(bus.publish(event).is_ok());
    }

    #[test]
    fn test_event_display() {
        let event = UiEvent::PositionUpdated(10.0, 20.0, 5.0);
        assert_eq!(event.to_string(), "PositionUpdated(10, 20, 5)");
    }
}
