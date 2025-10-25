//! Tests for message service

use gcodekit4::{Message, MessageDispatcher, MessageLevel};

#[test]
fn test_message_level_display() {
    assert_eq!(MessageLevel::Verbose.to_string(), "VERB");
    assert_eq!(MessageLevel::Info.to_string(), "INFO");
    assert_eq!(MessageLevel::Warning.to_string(), "WARN");
    assert_eq!(MessageLevel::Error.to_string(), "ERR!");
}

#[test]
fn test_message_level_ordering() {
    assert!(MessageLevel::Verbose < MessageLevel::Info);
    assert!(MessageLevel::Info < MessageLevel::Warning);
    assert!(MessageLevel::Warning < MessageLevel::Error);
}

#[test]
fn test_message_creation_helpers() {
    let info_msg = Message::info("system", "Info message");
    assert_eq!(info_msg.level, MessageLevel::Info);
    assert_eq!(info_msg.source, "system");
    assert_eq!(info_msg.text, "Info message");

    let warn_msg = Message::warning("controller", "Warning message");
    assert_eq!(warn_msg.level, MessageLevel::Warning);

    let error_msg = Message::error("comm", "Error message");
    assert_eq!(error_msg.level, MessageLevel::Error);

    let verbose_msg = Message::verbose("debug", "Verbose message");
    assert_eq!(verbose_msg.level, MessageLevel::Verbose);
}

#[test]
fn test_message_format_console() {
    let msg = Message::error("controller", "Connection lost");
    let formatted = msg.format_console();

    assert!(formatted.contains("ERR!"));
    assert!(formatted.contains("controller"));
    assert!(formatted.contains("Connection lost"));
    assert!(formatted.contains(":")); // HH:MM:SS format
}

#[test]
fn test_message_display() {
    let msg = Message::info("system", "Test message");
    let displayed = msg.to_string();
    assert_eq!(displayed, msg.format_console());
}

#[tokio::test]
async fn test_message_dispatcher_creation() {
    let dispatcher = MessageDispatcher::default_with_buffer();
    assert_eq!(dispatcher.get_min_level(), MessageLevel::Info);
    assert_eq!(dispatcher.subscriber_count(), 0);
}

#[tokio::test]
async fn test_message_dispatcher_level_setting() {
    let dispatcher = MessageDispatcher::default_with_buffer();

    dispatcher.set_min_level(MessageLevel::Error);
    assert_eq!(dispatcher.get_min_level(), MessageLevel::Error);

    dispatcher.set_min_level(MessageLevel::Verbose);
    assert_eq!(dispatcher.get_min_level(), MessageLevel::Verbose);
}

#[tokio::test]
async fn test_message_dispatcher_publish() {
    let dispatcher = MessageDispatcher::default_with_buffer();
    let mut rx = dispatcher.subscribe();

    let msg = Message::info("system", "Test publish");
    assert!(dispatcher.publish(msg.clone()).is_ok());

    let received = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(received.source, "system");
    assert_eq!(received.text, "Test publish");
}

#[tokio::test]
async fn test_message_dispatcher_convenience_methods() {
    let dispatcher = MessageDispatcher::default_with_buffer();
    let mut rx = dispatcher.subscribe();

    // Use convenience method
    assert!(dispatcher.info("src", "Info message").is_ok());

    let received = rx.recv().await.unwrap();
    assert_eq!(received.level, MessageLevel::Info);
    assert_eq!(received.text, "Info message");
}

#[tokio::test]
async fn test_message_dispatcher_filtering() {
    let dispatcher = MessageDispatcher::new(100, MessageLevel::Warning);
    let mut rx = dispatcher.subscribe();

    // This should not be sent (below Warning level)
    let result_verbose = dispatcher.verbose("src", "This is verbose");
    assert_eq!(result_verbose.unwrap(), 0); // 0 subscribers received it

    // This should be sent (Warning level)
    let result_warn = dispatcher.warning("src", "This is warning");
    assert!(result_warn.is_ok());

    // Receive the warning
    let received = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(received.level, MessageLevel::Warning);
}

#[tokio::test]
async fn test_message_dispatcher_broadcast() {
    let dispatcher = MessageDispatcher::default_with_buffer();
    let mut rx1 = dispatcher.subscribe();
    let mut rx2 = dispatcher.subscribe();

    let msg = Message::error("system", "Error message");
    dispatcher.publish(msg).unwrap();

    let msg1 = rx1.recv().await.unwrap();
    let msg2 = rx2.recv().await.unwrap();

    assert_eq!(msg1.text, "Error message");
    assert_eq!(msg2.text, "Error message");
}

#[tokio::test]
async fn test_message_dispatcher_clone() {
    let dispatcher1 = MessageDispatcher::default_with_buffer();
    let dispatcher2 = dispatcher1.clone();

    let mut rx = dispatcher1.subscribe();

    dispatcher2.info("src", "Message from clone").unwrap();

    let received = rx.recv().await.unwrap();
    assert_eq!(received.text, "Message from clone");
}

#[tokio::test]
async fn test_message_dispatcher_all_levels() {
    let dispatcher = MessageDispatcher::default_with_buffer();
    let mut rx = dispatcher.subscribe();

    dispatcher.verbose("src", "V").unwrap();
    dispatcher.info("src", "I").unwrap();
    dispatcher.warning("src", "W").unwrap();
    dispatcher.error("src", "E").unwrap();

    let mut messages = Vec::new();
    for _ in 0..4 {
        if let Ok(Some(msg)) = tokio::time::timeout(std::time::Duration::from_millis(100), async {
            Some(rx.recv().await.ok())
        })
        .await
        {
            if let Some(msg) = msg {
                messages.push(msg);
            }
        }
    }

    // We expect to receive all 4 messages since default min level is Info
    // Actually, Verbose might be filtered. Let me check: min_level = Info, so Verbose is filtered
    // We should get I, W, E (3 messages)
    let levels: Vec<_> = messages.iter().map(|m| m.level).collect();
    assert!(levels.contains(&MessageLevel::Info));
    assert!(levels.contains(&MessageLevel::Warning));
    assert!(levels.contains(&MessageLevel::Error));
}
