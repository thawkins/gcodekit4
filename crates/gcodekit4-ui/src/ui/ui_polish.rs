//! UI Polish and Features - Tasks 84-90
//!
//! Progress indicators, status notifications, keyboard shortcuts,
//! themes, i18n, responsive layout, help system

use std::collections::HashMap;

// ============================================================================
// Task 84: Progress Indicators
// ============================================================================

/// Progress indicator type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressType {
    /// File send progress
    FileSend,
    /// Execution progress
    Execution,
    /// Loading progress
    Loading,
}

/// Progress indicator
#[derive(Debug, Clone)]
pub struct ProgressIndicator {
    /// Progress type
    pub progress_type: ProgressType,
    /// Current value (0.0-1.0)
    pub current: f32,
    /// Maximum value
    pub total: f32,
    /// Time elapsed (seconds)
    pub elapsed_time: f32,
    /// Estimated time remaining (seconds)
    pub remaining_time: Option<f32>,
    /// Status message
    pub message: String,
}

impl ProgressIndicator {
    /// Create new progress indicator
    pub fn new(progress_type: ProgressType, total: f32) -> Self {
        Self {
            progress_type,
            current: 0.0,
            total,
            elapsed_time: 0.0,
            remaining_time: None,
            message: "Starting...".to_string(),
        }
    }

    /// Get progress percentage (0-100)
    pub fn percentage(&self) -> f32 {
        if self.total > 0.0 {
            (self.current / self.total * 100.0).min(100.0)
        } else {
            0.0
        }
    }

    /// Update progress
    pub fn update(&mut self, current: f32, message: impl Into<String>) {
        self.current = current.min(self.total);
        self.message = message.into();

        if self.elapsed_time > 0.0 && self.current > 0.0 {
            let rate = self.current / self.elapsed_time;
            let remaining = (self.total - self.current) / rate;
            self.remaining_time = Some(remaining.max(0.0));
        }
    }

    /// Increment elapsed time
    pub fn tick(&mut self, delta_time: f32) {
        self.elapsed_time += delta_time;
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        (self.current - self.total).abs() < 0.01
    }

    /// Reset progress
    pub fn reset(&mut self) {
        self.current = 0.0;
        self.elapsed_time = 0.0;
        self.remaining_time = None;
        self.message = "Starting...".to_string();
    }
}

// ============================================================================
// Task 85: Status Notifications
// ============================================================================

/// Notification severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    /// Information message
    Info,
    /// Success message
    Success,
    /// Warning message
    Warning,
    /// Error message
    Error,
}

impl std::fmt::Display for NotificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "Info"),
            Self::Success => write!(f, "Success"),
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Status notification
#[derive(Debug, Clone)]
pub struct Notification {
    /// Message text
    pub message: String,
    /// Severity level
    pub level: NotificationLevel,
    /// Duration to show (seconds, None = sticky)
    pub duration: Option<f32>,
    /// Time remaining to show (seconds)
    pub remaining: Option<f32>,
    /// Notification ID
    pub id: String,
}

impl Notification {
    /// Create new notification
    pub fn new(message: impl Into<String>, level: NotificationLevel) -> Self {
        Self {
            message: message.into(),
            level,
            duration: Some(5.0),
            remaining: Some(5.0),
            id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Set duration
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = Some(duration);
        self.remaining = Some(duration);
        self
    }

    /// Make sticky (never expires)
    pub fn sticky(mut self) -> Self {
        self.duration = None;
        self.remaining = None;
        self
    }

    /// Update timer
    pub fn tick(&mut self, delta_time: f32) -> bool {
        if let Some(remaining) = &mut self.remaining {
            *remaining -= delta_time;
            *remaining > 0.0
        } else {
            true
        }
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        if let Some(remaining) = self.remaining {
            remaining <= 0.0
        } else {
            false
        }
    }
}

/// Notification manager
#[derive(Debug, Clone)]
pub struct NotificationManager {
    /// Active notifications
    pub notifications: Vec<Notification>,
    /// Maximum notifications to show
    pub max_notifications: usize,
}

impl NotificationManager {
    /// Create new notification manager
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            max_notifications: 5,
        }
    }

    /// Add notification
    pub fn add(&mut self, notification: Notification) {
        self.notifications.push(notification);
        if self.notifications.len() > self.max_notifications {
            self.notifications.remove(0);
        }
    }

    /// Add info notification
    pub fn info(&mut self, message: impl Into<String>) {
        self.add(Notification::new(message, NotificationLevel::Info));
    }

    /// Add success notification
    pub fn success(&mut self, message: impl Into<String>) {
        self.add(Notification::new(message, NotificationLevel::Success));
    }

    /// Add warning notification
    pub fn warning(&mut self, message: impl Into<String>) {
        self.add(Notification::new(message, NotificationLevel::Warning));
    }

    /// Add error notification
    pub fn error(&mut self, message: impl Into<String>) {
        self.add(Notification::new(message, NotificationLevel::Error));
    }

    /// Remove notification by ID
    pub fn remove(&mut self, id: &str) {
        self.notifications.retain(|n| n.id != id);
    }

    /// Update all notifications
    pub fn update(&mut self, delta_time: f32) {
        self.notifications.retain_mut(|n| n.tick(delta_time));
    }

    /// Clear all notifications
    pub fn clear(&mut self) {
        self.notifications.clear();
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Task 86: Keyboard Shortcuts
// ============================================================================

/// Keyboard modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Modifier {
    /// Control/Command key
    Ctrl,
    /// Shift key
    Shift,
    /// Alt key
    Alt,
    /// No modifier
    None,
}

/// Keyboard key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// Character key
    Char(char),
    /// Function key (F1-F12)
    FunctionKey(u8),
    /// Arrow key
    Arrow(ArrowKey),
    /// Special key
    Special(SpecialKey),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrowKey {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialKey {
    Enter,
    Escape,
    Space,
    Tab,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
}

/// Keyboard shortcut action
#[derive(Debug, Clone)]
pub struct KeyboardAction {
    /// Action identifier
    pub action_id: String,
    /// Action description
    pub description: String,
    /// Primary key
    pub key: Key,
    /// Modifier
    pub modifier: Modifier,
}

impl KeyboardAction {
    /// Create new keyboard action
    pub fn new(action_id: impl Into<String>, description: impl Into<String>, key: Key) -> Self {
        Self {
            action_id: action_id.into(),
            description: description.into(),
            key,
            modifier: Modifier::None,
        }
    }

    /// Add modifier
    pub fn with_modifier(mut self, modifier: Modifier) -> Self {
        self.modifier = modifier;
        self
    }
}

/// Keyboard shortcut manager
#[derive(Debug, Clone)]
pub struct KeyboardShortcutManager {
    /// Shortcuts mapping
    pub shortcuts: HashMap<String, KeyboardAction>,
}

impl KeyboardShortcutManager {
    /// Create new keyboard shortcut manager
    pub fn new() -> Self {
        Self {
            shortcuts: HashMap::new(),
        }
    }

    /// Register shortcut
    pub fn register(&mut self, action: KeyboardAction) {
        self.shortcuts.insert(action.action_id.clone(), action);
    }

    /// Get shortcut by action ID
    pub fn get(&self, action_id: &str) -> Option<&KeyboardAction> {
        self.shortcuts.get(action_id)
    }

    /// Get action by key combination
    pub fn find_action(&self, key: Key, modifier: Modifier) -> Option<&KeyboardAction> {
        self.shortcuts
            .values()
            .find(|a| a.key == key && a.modifier == modifier)
    }

    /// List all shortcuts
    pub fn list_all(&self) -> Vec<&KeyboardAction> {
        self.shortcuts.values().collect()
    }

    /// Update shortcut
    pub fn update(&mut self, action_id: &str, key: Key, modifier: Modifier) -> bool {
        if let Some(action) = self.shortcuts.get_mut(action_id) {
            action.key = key;
            action.modifier = modifier;
            true
        } else {
            false
        }
    }
}

impl Default for KeyboardShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Task 87: Themes and Styling
// ============================================================================

/// Color scheme
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// Primary color
    pub primary: (u8, u8, u8),
    /// Secondary color
    pub secondary: (u8, u8, u8),
    /// Background color
    pub background: (u8, u8, u8),
    /// Text color
    pub text: (u8, u8, u8),
    /// Accent color
    pub accent: (u8, u8, u8),
}

impl ColorScheme {
    /// Create light theme
    pub fn light() -> Self {
        Self {
            primary: (41, 128, 185),
            secondary: (52, 152, 219),
            background: (236, 240, 241),
            text: (44, 62, 80),
            accent: (22, 160, 133),
        }
    }

    /// Create dark theme
    pub fn dark() -> Self {
        Self {
            primary: (52, 152, 219),
            secondary: (41, 128, 185),
            background: (44, 62, 80),
            text: (236, 240, 241),
            accent: (22, 160, 133),
        }
    }
}

/// Theme definition
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Color scheme
    pub colors: ColorScheme,
    /// Font size multiplier
    pub font_scale: f32,
    /// Enable animations
    pub animations_enabled: bool,
}

impl Theme {
    /// Create light theme
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            colors: ColorScheme::light(),
            font_scale: 1.0,
            animations_enabled: true,
        }
    }

    /// Create dark theme
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            colors: ColorScheme::dark(),
            font_scale: 1.0,
            animations_enabled: true,
        }
    }
}

// ============================================================================
// Task 88: Multi-language Support (i18n)
// ============================================================================

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Chinese,
    Japanese,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::English => write!(f, "English"),
            Self::Spanish => write!(f, "Spanish"),
            Self::French => write!(f, "French"),
            Self::German => write!(f, "German"),
            Self::Chinese => write!(f, "Chinese"),
            Self::Japanese => write!(f, "Japanese"),
        }
    }
}

/// Internationalization manager
#[derive(Debug, Clone)]
pub struct I18nManager {
    /// Current language
    pub current_language: Language,
    /// Translation strings
    pub translations: HashMap<Language, HashMap<String, String>>,
}

impl I18nManager {
    /// Create new i18n manager
    pub fn new(language: Language) -> Self {
        Self {
            current_language: language,
            translations: HashMap::new(),
        }
    }

    /// Set language
    pub fn set_language(&mut self, language: Language) {
        self.current_language = language;
    }

    /// Add translation
    pub fn add_translation(
        &mut self,
        language: Language,
        key: impl Into<String>,
        value: impl Into<String>,
    ) {
        self.translations
            .entry(language)
            .or_default()
            .insert(key.into(), value.into());
    }

    /// Get translated string
    pub fn translate(&self, key: &str) -> String {
        self.translations
            .get(&self.current_language)
            .and_then(|map| map.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}

impl Default for I18nManager {
    fn default() -> Self {
        Self::new(Language::English)
    }
}

// ============================================================================
// Task 89: Responsive Layout
// ============================================================================

/// Layout orientation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// Responsive layout manager
#[derive(Debug, Clone)]
pub struct ResponsiveLayout {
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Current orientation
    pub orientation: Orientation,
    /// Show sidebar
    pub show_sidebar: bool,
    /// Show status bar
    pub show_statusbar: bool,
}

impl ResponsiveLayout {
    /// Create new responsive layout
    pub fn new(width: u32, height: u32) -> Self {
        let orientation = if width > height {
            Orientation::Landscape
        } else {
            Orientation::Portrait
        };

        Self {
            width,
            height,
            orientation,
            show_sidebar: width > 1024,
            show_statusbar: true,
        }
    }

    /// Update dimensions
    pub fn update(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.orientation = if width > height {
            Orientation::Landscape
        } else {
            Orientation::Portrait
        };

        self.show_sidebar = width > 1024;
    }

    /// Get scale factor for DPI
    pub fn get_scale_factor(&self) -> f32 {
        if self.width > 1920 {
            1.5
        } else if self.width > 1440 {
            1.25
        } else {
            1.0
        }
    }
}

// ============================================================================
// Task 90: Help and Documentation
// ============================================================================

/// Help topic
#[derive(Debug, Clone)]
pub struct HelpTopic {
    /// Topic ID
    pub id: String,
    /// Topic title
    pub title: String,
    /// Topic content
    pub content: String,
    /// Related topics
    pub related: Vec<String>,
}

impl HelpTopic {
    /// Create new help topic
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            related: Vec::new(),
        }
    }

    /// Add related topic
    pub fn with_related(mut self, topic_id: impl Into<String>) -> Self {
        self.related.push(topic_id.into());
        self
    }
}

/// Help system
#[derive(Debug, Clone)]
pub struct HelpSystem {
    /// Help topics
    pub topics: HashMap<String, HelpTopic>,
    /// Keyboard shortcuts help
    pub shortcuts_help: Vec<String>,
}

impl HelpSystem {
    /// Create new help system
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            shortcuts_help: Vec::new(),
        }
    }

    /// Add topic
    pub fn add_topic(&mut self, topic: HelpTopic) {
        self.topics.insert(topic.id.clone(), topic);
    }

    /// Get topic
    pub fn get_topic(&self, id: &str) -> Option<&HelpTopic> {
        self.topics.get(id)
    }

    /// Add shortcut help
    pub fn add_shortcut_help(&mut self, help: impl Into<String>) {
        self.shortcuts_help.push(help.into());
    }

    /// Get related topics
    pub fn get_related(&self, id: &str) -> Vec<&HelpTopic> {
        if let Some(topic) = self.topics.get(id) {
            topic
                .related
                .iter()
                .filter_map(|rid| self.topics.get(rid))
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for HelpSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_indicator() {
        let mut progress = ProgressIndicator::new(ProgressType::FileSend, 100.0);
        progress.update(50.0, "50% complete");
        assert_eq!(progress.percentage(), 50.0);
    }

    #[test]
    fn test_notification() {
        let notif = Notification::new("Test", NotificationLevel::Info);
        assert!(!notif.is_expired());
    }

    #[test]
    fn test_notification_manager() {
        let mut manager = NotificationManager::new();
        manager.info("Test message");
        assert_eq!(manager.notifications.len(), 1);
    }

    #[test]
    fn test_color_scheme() {
        let light = ColorScheme::light();
        let dark = ColorScheme::dark();
        assert_ne!(light.background, dark.background);
    }

    #[test]
    fn test_i18n_manager() {
        let mut i18n = I18nManager::default();
        i18n.add_translation(Language::English, "hello", "Hello");
        assert_eq!(i18n.translate("hello"), "Hello");
    }

    #[test]
    fn test_responsive_layout() {
        let mut layout = ResponsiveLayout::new(1920, 1080);
        assert_eq!(layout.orientation, Orientation::Landscape);
        layout.update(768, 1024);
        assert_eq!(layout.orientation, Orientation::Portrait);
    }

    #[test]
    fn test_help_system() {
        let mut help = HelpSystem::new();
        let topic = HelpTopic::new("topic1", "Title", "Content");
        help.add_topic(topic);
        assert!(help.get_topic("topic1").is_some());
    }
}
