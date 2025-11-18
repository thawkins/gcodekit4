//! Message service for logging and displaying messages
//!
//! Provides:
//! - Message types with severity levels
//! - Message dispatcher for publishing messages
//! - Console output formatting
//! - Message filtering by level

use std::sync::Arc;
use tokio::sync::broadcast;

/// Message severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageLevel {
    /// Verbose/Debug level
    Verbose = 0,
    /// Information level
    Info = 1,
    /// Warning level
    Warning = 2,
    /// Error level
    Error = 3,
}

impl std::fmt::Display for MessageLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verbose => write!(f, "VERB"),
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERR!"),
        }
    }
}

/// Message from controller or system
#[derive(Debug, Clone)]
pub struct Message {
    /// Timestamp of message
    pub timestamp: String,
    /// Message level/severity
    pub level: MessageLevel,
    /// Message source (controller, system, etc.)
    pub source: String,
    /// Message text
    pub text: String,
}

impl Message {
    /// Create a new message
    pub fn new(level: MessageLevel, source: impl Into<String>, text: impl Into<String>) -> Self {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        Self {
            timestamp,
            level,
            source: source.into(),
            text: text.into(),
        }
    }

    /// Create an info message
    pub fn info(source: impl Into<String>, text: impl Into<String>) -> Self {
        Self::new(MessageLevel::Info, source, text)
    }

    /// Create a warning message
    pub fn warning(source: impl Into<String>, text: impl Into<String>) -> Self {
        Self::new(MessageLevel::Warning, source, text)
    }

    /// Create an error message
    pub fn error(source: impl Into<String>, text: impl Into<String>) -> Self {
        Self::new(MessageLevel::Error, source, text)
    }

    /// Create a verbose message
    pub fn verbose(source: impl Into<String>, text: impl Into<String>) -> Self {
        Self::new(MessageLevel::Verbose, source, text)
    }

    /// Format message for console output
    pub fn format_console(&self) -> String {
        format!(
            "[{}] {} | {}: {}",
            self.timestamp, self.level, self.source, self.text
        )
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_console())
    }
}

/// Message dispatcher for publishing messages to subscribers
pub struct MessageDispatcher {
    tx: broadcast::Sender<Message>,
    min_level: Arc<parking_lot::RwLock<MessageLevel>>,
}

impl MessageDispatcher {
    /// Create a new message dispatcher
    ///
    /// # Arguments
    /// * `buffer_size` - Size of the broadcast buffer
    /// * `min_level` - Minimum message level to dispatch
    pub fn new(buffer_size: usize, min_level: MessageLevel) -> Self {
        let (tx, _) = broadcast::channel(buffer_size);
        Self {
            tx,
            min_level: Arc::new(parking_lot::RwLock::new(min_level)),
        }
    }

    /// Create a new dispatcher with default settings (INFO level, buffer 100)
    pub fn default_with_buffer() -> Self {
        Self::new(100, MessageLevel::Info)
    }

    /// Subscribe to messages
    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.tx.subscribe()
    }

    /// Publish a message
    pub fn publish(&self, message: Message) -> Result<usize, broadcast::error::SendError<Message>> {
        let min_level = *self.min_level.read();
        if message.level >= min_level {
            match message.level {
                MessageLevel::Verbose => {
                    // Skip verbose logging
                }
                MessageLevel::Info => {
                    // Skip logging info messages to reduce noise
                }
                MessageLevel::Warning => tracing::warn!("{}", message.format_console()),
                MessageLevel::Error => tracing::error!("{}", message.format_console()),
            }
            self.tx.send(message)
        } else {
            Ok(0)
        }
    }

    /// Set minimum message level
    pub fn set_min_level(&self, level: MessageLevel) {
        *self.min_level.write() = level;
    }

    /// Get minimum message level
    pub fn get_min_level(&self) -> MessageLevel {
        *self.min_level.read()
    }

    /// Get number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// Publish info message
    pub fn info(
        &self,
        source: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<usize, broadcast::error::SendError<Message>> {
        self.publish(Message::info(source, text))
    }

    /// Publish warning message
    pub fn warning(
        &self,
        source: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<usize, broadcast::error::SendError<Message>> {
        self.publish(Message::warning(source, text))
    }

    /// Publish error message
    pub fn error(
        &self,
        source: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<usize, broadcast::error::SendError<Message>> {
        self.publish(Message::error(source, text))
    }

    /// Publish verbose message
    pub fn verbose(
        &self,
        source: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<usize, broadcast::error::SendError<Message>> {
        self.publish(Message::verbose(source, text))
    }
}

impl Clone for MessageDispatcher {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            min_level: Arc::clone(&self.min_level),
        }
    }
}

impl Default for MessageDispatcher {
    fn default() -> Self {
        Self::default_with_buffer()
    }
}
