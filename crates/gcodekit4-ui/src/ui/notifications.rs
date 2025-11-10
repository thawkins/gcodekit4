//! Notification system for user feedback
//!
//! Provides success, warning, and error notifications with auto-dismiss.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Notification severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    /// Success notification (green)
    Success,
    /// Information notification (blue)
    Info,
    /// Warning notification (orange)
    Warning,
    /// Error notification (red)
    Error,
}

/// A single notification message
#[derive(Debug, Clone)]
pub struct Notification {
    /// Notification ID (UUID)
    pub id: String,
    /// Message content
    pub message: String,
    /// Severity level
    pub level: NotificationLevel,
    /// Creation time
    created_at: Instant,
    /// Auto-dismiss duration (None = manual only)
    auto_dismiss_after: Option<Duration>,
}

impl Notification {
    /// Create a new notification
    pub fn new(message: impl Into<String>, level: NotificationLevel) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message: message.into(),
            level,
            created_at: Instant::now(),
            auto_dismiss_after: Some(Duration::from_secs(5)),
        }
    }

    /// Create with custom auto-dismiss duration
    pub fn with_duration(
        message: impl Into<String>,
        level: NotificationLevel,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message: message.into(),
            level,
            created_at: Instant::now(),
            auto_dismiss_after: duration,
        }
    }

    /// Check if should be auto-dismissed
    pub fn should_dismiss(&self) -> bool {
        if let Some(duration) = self.auto_dismiss_after {
            self.created_at.elapsed() > duration
        } else {
            false
        }
    }

    /// Get age of notification
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Notification manager
pub struct NotificationManager {
    notifications: Arc<Mutex<Vec<Notification>>>,
}

impl NotificationManager {
    /// Create new notification manager
    pub fn new() -> Self {
        Self {
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add success notification
    pub fn success(&self, message: impl Into<String>) {
        self.add(Notification::new(message, NotificationLevel::Success));
    }

    /// Add info notification
    pub fn info(&self, message: impl Into<String>) {
        self.add(Notification::new(message, NotificationLevel::Info));
    }

    /// Add warning notification
    pub fn warning(&self, message: impl Into<String>) {
        self.add(Notification::new(message, NotificationLevel::Warning));
    }

    /// Add error notification
    pub fn error(&self, message: impl Into<String>) {
        self.add(Notification::with_duration(
            message,
            NotificationLevel::Error,
            Some(Duration::from_secs(10)),
        ));
    }

    /// Add notification manually
    pub fn add(&self, notification: Notification) {
        if let Ok(mut notifs) = self.notifications.lock() {
            notifs.push(notification);
            // Keep only last 10 notifications
            if notifs.len() > 10 {
                notifs.remove(0);
            }
        }
    }

    /// Get all active notifications
    pub fn get_all(&self) -> Vec<Notification> {
        if let Ok(notifs) = self.notifications.lock() {
            notifs.clone()
        } else {
            Vec::new()
        }
    }

    /// Remove notification by ID
    pub fn dismiss(&self, id: &str) {
        if let Ok(mut notifs) = self.notifications.lock() {
            notifs.retain(|n| n.id != id);
        }
    }

    /// Clear all notifications
    pub fn clear_all(&self) {
        if let Ok(mut notifs) = self.notifications.lock() {
            notifs.clear();
        }
    }

    /// Remove expired auto-dismiss notifications
    pub fn cleanup_expired(&self) {
        if let Ok(mut notifs) = self.notifications.lock() {
            notifs.retain(|n| !n.should_dismiss());
        }
    }

    /// Get count of active notifications
    pub fn count(&self) -> usize {
        if let Ok(notifs) = self.notifications.lock() {
            notifs.len()
        } else {
            0
        }
    }

    /// Get notifications by level
    pub fn by_level(&self, level: NotificationLevel) -> Vec<Notification> {
        if let Ok(notifs) = self.notifications.lock() {
            notifs
                .iter()
                .filter(|n| n.level == level)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for NotificationManager {
    fn clone(&self) -> Self {
        Self {
            notifications: Arc::clone(&self.notifications),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notif = Notification::new("Test message", NotificationLevel::Success);
        assert_eq!(notif.message, "Test message");
        assert_eq!(notif.level, NotificationLevel::Success);
    }

    #[test]
    fn test_notification_auto_dismiss() {
        let notif = Notification::new("Test", NotificationLevel::Info);
        assert!(!notif.should_dismiss());

        let notif_no_dismiss = Notification::with_duration("Test", NotificationLevel::Info, None);
        assert!(!notif_no_dismiss.should_dismiss());
    }

    #[test]
    fn test_manager_add_success() {
        let mgr = NotificationManager::new();
        mgr.success("Success message");
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_manager_add_error() {
        let mgr = NotificationManager::new();
        mgr.error("Error message");
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn test_manager_dismiss() {
        let mgr = NotificationManager::new();
        mgr.success("Test");
        assert_eq!(mgr.count(), 1);

        let notifs = mgr.get_all();
        if let Some(notif) = notifs.first() {
            mgr.dismiss(&notif.id);
        }
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_manager_clear_all() {
        let mgr = NotificationManager::new();
        mgr.success("Test 1");
        mgr.success("Test 2");
        mgr.success("Test 3");
        assert_eq!(mgr.count(), 3);

        mgr.clear_all();
        assert_eq!(mgr.count(), 0);
    }

    #[test]
    fn test_manager_by_level() {
        let mgr = NotificationManager::new();
        mgr.success("Success");
        mgr.error("Error");
        mgr.success("Success 2");

        let successes = mgr.by_level(NotificationLevel::Success);
        assert_eq!(successes.len(), 2);

        let errors = mgr.by_level(NotificationLevel::Error);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_manager_clone() {
        let mgr1 = NotificationManager::new();
        mgr1.success("Test");

        let mgr2 = mgr1.clone();
        assert_eq!(mgr2.count(), 1);
    }
}
