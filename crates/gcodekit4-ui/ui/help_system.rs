//! Help system and documentation integration
//!
//! Provides help menu, keyboard shortcuts reference, tooltips, and about dialog.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Help topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpTopic {
    /// Topic ID
    pub id: String,
    /// Display title
    pub title: String,
    /// Help content (markdown)
    pub content: String,
    /// Related topics (IDs)
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
    pub fn with_related(mut self, related_id: impl Into<String>) -> Self {
        self.related.push(related_id.into());
        self
    }
}

/// Keyboard shortcut reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutReference {
    /// Shortcut description
    pub description: String,
    /// Key combination
    pub keys: String,
    /// Category (File, Edit, Machine, etc.)
    pub category: String,
}

impl ShortcutReference {
    /// Create new shortcut reference
    pub fn new(
        description: impl Into<String>,
        keys: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            description: description.into(),
            keys: keys.into(),
            category: category.into(),
        }
    }
}

/// Help system
pub struct HelpSystem {
    topics: HashMap<String, HelpTopic>,
    shortcuts: Vec<ShortcutReference>,
}

impl HelpSystem {
    /// Create new help system
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            shortcuts: Vec::new(),
        }
    }

    /// Add help topic
    pub fn add_topic(&mut self, topic: HelpTopic) {
        self.topics.insert(topic.id.clone(), topic);
    }

    /// Get help topic by ID
    pub fn get_topic(&self, id: &str) -> Option<&HelpTopic> {
        self.topics.get(id)
    }

    /// Add keyboard shortcut
    pub fn add_shortcut(&mut self, shortcut: ShortcutReference) {
        self.shortcuts.push(shortcut);
    }

    /// Get shortcuts by category
    pub fn shortcuts_by_category(&self, category: &str) -> Vec<&ShortcutReference> {
        self.shortcuts
            .iter()
            .filter(|s| s.category == category)
            .collect()
    }

    /// Get all shortcut categories
    pub fn shortcut_categories(&self) -> Vec<String> {
        let mut categories: Vec<_> = self.shortcuts.iter().map(|s| s.category.clone()).collect();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Get all shortcuts
    pub fn all_shortcuts(&self) -> &[ShortcutReference] {
        &self.shortcuts
    }

    /// Initialize default help content
    pub fn init_defaults(&mut self) {
        // Connection Help
        self.add_topic(HelpTopic::new(
            "connection",
            "Connecting to Your Machine",
            r#"# Connecting to Your Machine

## Serial Connection
1. Select your serial port from the dropdown
2. Choose the appropriate baud rate (typically 115200 for GRBL)
3. Click Connect

## TCP Connection
1. Enter the hostname or IP address
2. Enter the port number
3. Click Connect

## WebSocket Connection
1. Enter the WebSocket URL
2. Click Connect

## Troubleshooting
- Check that the cable is properly connected
- Verify the correct port is selected
- Ensure baud rate matches your controller
- Check device permissions on Linux/Mac"#,
        ));

        // Jogging Help
        self.add_topic(HelpTopic::new(
            "jogging",
            "Jogging the Machine",
            r#"# Jogging Controls

## Keyboard Jogging
- W/Up Arrow: +Y
- S/Down Arrow: -Y
- A/Left Arrow: -X
- D/Right Arrow: +X
- Q: +Z
- Z: -Z

## Incremental Jogging
1. Select jog increment (0.1mm, 1mm, 10mm, 100mm)
2. Set jog feed rate
3. Click direction buttons to move

## Continuous Jogging
1. Select feed rate
2. Hold direction button
3. Machine moves continuously
4. Release button to stop"#,
        ));

        // File Operations Help
        self.add_topic(HelpTopic::new(
            "file_operations",
            "File Operations",
            r#"# Working with G-Code Files

## Opening Files
- Click File → Open
- Select your .gcode, .ngc, or .tap file
- File is parsed and displayed

## File Validation
- GCodeKit automatically validates files
- Errors are highlighted
- Warnings show potential issues

## File Processing
- Comments are removed
- Commands are normalized
- File is ready to stream

## Streaming
- Click Start to begin streaming
- Click Pause to hold machine
- Click Stop to cancel
- Progress shows in status bar"#,
        ));

        // Add default shortcuts
        self.add_shortcut(ShortcutReference::new("Open File", "Ctrl+O", "File"));
        self.add_shortcut(ShortcutReference::new("Save File", "Ctrl+S", "File"));
        self.add_shortcut(ShortcutReference::new("Exit Application", "Ctrl+Q", "File"));

        self.add_shortcut(ShortcutReference::new("Home All Axes", "Ctrl+H", "Machine"));
        self.add_shortcut(ShortcutReference::new("Soft Reset", "Ctrl+R", "Machine"));
        self.add_shortcut(ShortcutReference::new(
            "Kill Alarm Lock",
            "Ctrl+L",
            "Machine",
        ));

        self.add_shortcut(ShortcutReference::new(
            "Pause/Resume Stream",
            "Space",
            "Streaming",
        ));
        self.add_shortcut(ShortcutReference::new("Stop Stream", "Escape", "Streaming"));

        self.add_shortcut(ShortcutReference::new("Jog +X", "D", "Jog"));
        self.add_shortcut(ShortcutReference::new("Jog -X", "A", "Jog"));
        self.add_shortcut(ShortcutReference::new("Jog +Y", "W", "Jog"));
        self.add_shortcut(ShortcutReference::new("Jog -Y", "S", "Jog"));
        self.add_shortcut(ShortcutReference::new("Jog +Z", "Q", "Jog"));
        self.add_shortcut(ShortcutReference::new("Jog -Z", "Z", "Jog"));
    }

    /// Search help topics
    pub fn search_topics(&self, query: &str) -> Vec<&HelpTopic> {
        let query_lower = query.to_lowercase();
        self.topics
            .values()
            .filter(|topic| {
                topic.title.to_lowercase().contains(&query_lower)
                    || topic.content.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get topic count
    pub fn topic_count(&self) -> usize {
        self.topics.len()
    }

    /// Get shortcut count
    pub fn shortcut_count(&self) -> usize {
        self.shortcuts.len()
    }
}

impl Default for HelpSystem {
    fn default() -> Self {
        let mut system = Self::new();
        system.init_defaults();
        system
    }
}

/// Application information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// Application name
    pub name: String,
    /// Version
    pub version: String,
    /// Build date
    pub build_date: String,
    /// Git commit hash
    pub git_commit: String,
    /// License
    pub license: String,
    /// Homepage URL
    pub homepage: String,
    /// Documentation URL
    pub documentation: String,
    /// Repository URL
    pub repository: String,
    /// Bug report URL
    pub bug_report: String,
    /// Authors
    pub authors: Vec<String>,
}

impl AppInfo {
    /// Create application info
    pub fn new() -> Self {
        Self {
            name: "GCodeKit4".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_date: "2025-10-25".to_string(),
            git_commit: "development".to_string(),
            license: "GPL-3.0".to_string(),
            homepage: "https://github.com/thawkins/gcodekit4".to_string(),
            documentation: "https://github.com/thawkins/gcodekit4/wiki".to_string(),
            repository: "https://github.com/thawkins/gcodekit4".to_string(),
            bug_report: "https://github.com/thawkins/gcodekit4/issues".to_string(),
            authors: vec!["GCodeKit Contributors".to_string()],
        }
    }

    /// Get full about text
    pub fn about_text(&self) -> String {
        format!(
            "{} v{}\n\n\
             Build Date: {}\n\
             Commit: {}\n\
             License: {}\n\n\
             A modern G-Code sender for CNC machines.\n\n\
             Supported: GRBL, TinyG, g2core, FluidNC, Smoothieware",
            self.name, self.version, self.build_date, self.git_commit, self.license
        )
    }
}

impl Default for AppInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Tooltip system
pub struct TooltipProvider {
    tooltips: HashMap<String, String>,
}

impl TooltipProvider {
    /// Create new tooltip provider
    pub fn new() -> Self {
        Self {
            tooltips: HashMap::new(),
        }
    }

    /// Add tooltip
    pub fn add(&mut self, target: impl Into<String>, tooltip: impl Into<String>) {
        self.tooltips.insert(target.into(), tooltip.into());
    }

    /// Get tooltip
    pub fn get(&self, target: &str) -> Option<&str> {
        self.tooltips.get(target).map(|s| s.as_str())
    }

    /// Initialize default tooltips
    pub fn init_defaults(&mut self) {
        self.add("connect_btn", "Connect to CNC machine");
        self.add("disconnect_btn", "Disconnect from machine");
        self.add("home_btn", "Home all axes");
        self.add("reset_btn", "Soft reset controller");
        self.add("start_btn", "Start streaming G-code");
        self.add("pause_btn", "Pause/Resume streaming");
        self.add("stop_btn", "Stop streaming and clear queue");
        self.add("feed_rate_slider", "Adjust feed rate override (0-200%)");
        self.add("spindle_slider", "Adjust spindle speed override (0-200%)");
        self.add("rapid_100", "Set rapid traverse to 100%");
    }
}

impl Default for TooltipProvider {
    fn default() -> Self {
        let mut provider = Self::new();
        provider.init_defaults();
        provider
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_topic_creation() {
        let topic = HelpTopic::new("test", "Test Topic", "Content");
        assert_eq!(topic.id, "test");
        assert_eq!(topic.title, "Test Topic");
    }

    #[test]
    fn test_help_system_add_topic() {
        let mut system = HelpSystem::new();
        system.add_topic(HelpTopic::new("test", "Test", "Content"));
        assert_eq!(system.topic_count(), 1);
    }

    #[test]
    fn test_help_system_get_topic() {
        let mut system = HelpSystem::new();
        system.add_topic(HelpTopic::new("test", "Test", "Content"));
        assert!(system.get_topic("test").is_some());
    }

    #[test]
    fn test_help_system_defaults() {
        let system = HelpSystem::default();
        assert!(system.topic_count() > 0);
        assert!(system.shortcut_count() > 0);
    }

    #[test]
    fn test_shortcut_categories() {
        let system = HelpSystem::default();
        let categories = system.shortcut_categories();
        assert!(!categories.is_empty());
    }

    #[test]
    fn test_app_info() {
        let info = AppInfo::new();
        assert_eq!(info.name, "GCodeKit4");
        let about = info.about_text();
        assert!(about.contains("GCodeKit4"));
    }

    #[test]
    fn test_tooltip_provider() {
        let mut provider = TooltipProvider::default();
        assert!(provider.get("connect_btn").is_some());
    }

    #[test]
    fn test_help_search() {
        let system = HelpSystem::default();
        let results = system.search_topics("connection");
        assert!(!results.is_empty());
    }
}
