//! Connection watch timer
//!
//! Monitors the connection to the controller and detects timeouts.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tracing::warn;

/// Connection watch configuration
#[derive(Debug, Clone)]
pub struct ConnectionWatchConfig {
    /// Timeout duration in milliseconds
    pub timeout_ms: u64,
    /// Check interval in milliseconds
    pub check_interval_ms: u64,
    /// Enable heartbeat (periodic status queries)
    pub enable_heartbeat: bool,
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
}

impl Default for ConnectionWatchConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            check_interval_ms: 500,
            enable_heartbeat: true,
            heartbeat_interval_ms: 1000,
        }
    }
}

/// Connection watch state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionWatchState {
    /// Connection is healthy
    Healthy,
    /// Connection may be timing out
    Degraded,
    /// Connection is lost
    Lost,
}

/// Connection watcher that monitors communication
pub struct ConnectionWatcher {
    /// Configuration
    config: ConnectionWatchConfig,
    /// Last heartbeat timestamp (Unix timestamp in ms)
    last_heartbeat: Arc<AtomicU64>,
    /// Watcher task handle
    watch_task: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
    /// Current state
    state: Arc<tokio::sync::Mutex<ConnectionWatchState>>,
}

impl ConnectionWatcher {
    /// Create a new connection watcher
    pub fn new(config: ConnectionWatchConfig) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            config,
            last_heartbeat: Arc::new(AtomicU64::new(now)),
            watch_task: Arc::new(tokio::sync::Mutex::new(None)),
            state: Arc::new(tokio::sync::Mutex::new(ConnectionWatchState::Healthy)),
        }
    }

    /// Start watching the connection
    pub async fn start(&self) -> anyhow::Result<()> {
        let config = self.config.clone();
        let last_heartbeat = Arc::clone(&self.last_heartbeat);
        let state = Arc::clone(&self.state);

        let task = tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_millis(config.check_interval_ms));

            loop {
                check_interval.tick().await;

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                let last = last_heartbeat.load(Ordering::Relaxed);
                let time_since_heartbeat = now.saturating_sub(last);

                let mut current_state = state.lock().await;
                let new_state = if time_since_heartbeat > config.timeout_ms {
                    ConnectionWatchState::Lost
                } else if time_since_heartbeat > config.timeout_ms / 2 {
                    ConnectionWatchState::Degraded
                } else {
                    ConnectionWatchState::Healthy
                };

                if *current_state != new_state {
                    *current_state = new_state;
                } else if new_state == ConnectionWatchState::Lost {
                    warn!("Connection timeout detected");
                }
            }
        });

        let mut watch_task = self.watch_task.lock().await;
        *watch_task = Some(task);

        Ok(())
    }

    /// Stop watching the connection
    pub async fn stop(&self) {
        let mut watch_task = self.watch_task.lock().await;
        if let Some(task) = watch_task.take() {
            task.abort();
        }
    }

    /// Update the last heartbeat time
    pub fn heartbeat(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.last_heartbeat.store(now, Ordering::Relaxed);
    }

    /// Get current connection state
    pub async fn get_state(&self) -> ConnectionWatchState {
        *self.state.lock().await
    }

    /// Check if connection is healthy
    pub async fn is_healthy(&self) -> bool {
        self.get_state().await == ConnectionWatchState::Healthy
    }

    /// Check if connection is lost
    pub async fn is_lost(&self) -> bool {
        self.get_state().await == ConnectionWatchState::Lost
    }

    /// Get time since last heartbeat in milliseconds
    pub fn time_since_heartbeat(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let last = self.last_heartbeat.load(Ordering::Relaxed);
        now.saturating_sub(last)
    }
}

impl std::fmt::Debug for ConnectionWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionWatcher")
            .field("config", &self.config)
            .field("time_since_heartbeat", &self.time_since_heartbeat())
            .finish()
    }
}
