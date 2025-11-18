//! Common test utilities

use gcodekit4_communication::CommunicatorListener;
use std::sync::Mutex;

/// Test listener for testing event callbacks
pub struct TestListener {
    events: Mutex<Vec<String>>,
}

impl TestListener {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    pub fn record_event(&self, event: &str) {
        self.events.lock().unwrap().push(event.to_string());
    }

    pub fn events(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }
}

impl CommunicatorListener for TestListener {
    fn on_connected(&self) {
        self.record_event("connected");
    }

    fn on_disconnected(&self) {
        self.record_event("disconnected");
    }

    fn on_error(&self, error: &str) {
        self.record_event(&format!("error: {}", error));
    }

    fn on_data_received(&self, data: &[u8]) {
        self.record_event(&format!("data_received: {} bytes", data.len()));
    }

    fn on_data_sent(&self, data: &[u8]) {
        self.record_event(&format!("data_sent: {} bytes", data.len()));
    }

    fn on_timeout(&self) {
        self.record_event("timeout");
    }
}
