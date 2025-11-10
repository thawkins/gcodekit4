//! Main Window - Task 67
//!
//! Core application window with menu bar, toolbar, and status bar

/// Menu item definition
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Menu label
    pub label: String,
    /// Menu ID/identifier
    pub id: String,
    /// Keyboard shortcut
    pub shortcut: Option<String>,
    /// Sub-menu items
    pub sub_items: Vec<MenuItem>,
}

impl MenuItem {
    /// Create new menu item
    pub fn new(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            shortcut: None,
            sub_items: Vec::new(),
        }
    }

    /// Set keyboard shortcut
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Add sub-menu item
    pub fn with_submenu(mut self, item: MenuItem) -> Self {
        self.sub_items.push(item);
        self
    }
}

/// Menu bar with all application menus
#[derive(Debug, Clone)]
pub struct MenuBar {
    /// File menu
    pub file_menu: MenuItem,
    /// Edit menu
    pub edit_menu: MenuItem,
    /// View menu
    pub view_menu: MenuItem,
    /// Machine menu
    pub machine_menu: MenuItem,
    /// Help menu
    pub help_menu: MenuItem,
}

impl MenuBar {
    /// Create new menu bar
    pub fn new() -> Self {
        Self {
            file_menu: Self::create_file_menu(),
            edit_menu: Self::create_edit_menu(),
            view_menu: Self::create_view_menu(),
            machine_menu: Self::create_machine_menu(),
            help_menu: Self::create_help_menu(),
        }
    }

    /// Create File menu
    fn create_file_menu() -> MenuItem {
        let mut file = MenuItem::new("File", "file");
        file.sub_items = vec![
            MenuItem::new("Open", "file_open").with_shortcut("Ctrl+O"),
            MenuItem::new("Save", "file_save").with_shortcut("Ctrl+S"),
            MenuItem::new("Exit", "file_exit").with_shortcut("Ctrl+Q"),
        ];
        file
    }

    /// Create Edit menu
    fn create_edit_menu() -> MenuItem {
        let mut edit = MenuItem::new("Edit", "edit");
        edit.sub_items = vec![
            MenuItem::new("Undo", "edit_undo").with_shortcut("Ctrl+Z"),
            MenuItem::new("Redo", "edit_redo").with_shortcut("Ctrl+Y"),
            MenuItem::new("Cut", "edit_cut").with_shortcut("Ctrl+X"),
            MenuItem::new("Copy", "edit_copy").with_shortcut("Ctrl+C"),
            MenuItem::new("Paste", "edit_paste").with_shortcut("Ctrl+V"),
        ];
        edit
    }

    /// Create View menu
    fn create_view_menu() -> MenuItem {
        let mut view = MenuItem::new("View", "view");
        view.sub_items = vec![
            MenuItem::new("Toolbars", "view_toolbars"),
            MenuItem::new("Status Bar", "view_status_bar"),
            MenuItem::new("Console", "view_console"),
            MenuItem::new("Visualizer", "view_visualizer"),
        ];
        view
    }

    /// Create Machine menu
    fn create_machine_menu() -> MenuItem {
        let mut machine = MenuItem::new("Machine", "machine");
        machine.sub_items = vec![
            MenuItem::new("Connect", "machine_connect").with_shortcut("Alt+C"),
            MenuItem::new("Disconnect", "machine_disconnect").with_shortcut("Alt+D"),
            MenuItem::new("Home", "machine_home").with_shortcut("Ctrl+H"),
            MenuItem::new("Reset", "machine_reset").with_shortcut("F5"),
        ];
        machine
    }

    /// Create Help menu
    fn create_help_menu() -> MenuItem {
        let mut help = MenuItem::new("Help", "help");
        help.sub_items = vec![
            MenuItem::new("Documentation", "help_docs"),
            MenuItem::new("About", "help_about"),
        ];
        help
    }

    /// Get all menus
    pub fn menus(&self) -> Vec<&MenuItem> {
        vec![
            &self.file_menu,
            &self.edit_menu,
            &self.view_menu,
            &self.machine_menu,
            &self.help_menu,
        ]
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Toolbar item definition
#[derive(Debug, Clone)]
pub struct ToolbarItem {
    /// Item label
    pub label: String,
    /// Item ID
    pub id: String,
    /// Icon name/path
    pub icon: Option<String>,
    /// Tooltip
    pub tooltip: Option<String>,
    /// Is enabled
    pub enabled: bool,
}

impl ToolbarItem {
    /// Create new toolbar item
    pub fn new(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            icon: None,
            tooltip: None,
            enabled: true,
        }
    }

    /// Set icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set tooltip
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}

/// Application toolbar
#[derive(Debug, Clone)]
pub struct Toolbar {
    /// Toolbar items
    pub items: Vec<ToolbarItem>,
}

impl Toolbar {
    /// Create new toolbar
    pub fn new() -> Self {
        Self {
            items: Self::default_items(),
        }
    }

    /// Create default toolbar items
    fn default_items() -> Vec<ToolbarItem> {
        vec![
            ToolbarItem::new("Connect", "connect")
                .with_icon("connect")
                .with_tooltip("Connect to controller"),
            ToolbarItem::new("Disconnect", "disconnect")
                .with_icon("disconnect")
                .with_tooltip("Disconnect from controller"),
            ToolbarItem::new("Open", "open")
                .with_icon("open")
                .with_tooltip("Open file"),
            ToolbarItem::new("Home", "home")
                .with_icon("home")
                .with_tooltip("Home machine"),
            ToolbarItem::new("Run", "run")
                .with_icon("run")
                .with_tooltip("Run G-code"),
            ToolbarItem::new("Pause", "pause")
                .with_icon("pause")
                .with_tooltip("Pause execution"),
            ToolbarItem::new("Stop", "stop")
                .with_icon("stop")
                .with_tooltip("Stop execution"),
        ]
    }

    /// Enable/disable toolbar item
    pub fn set_item_enabled(&mut self, id: &str, enabled: bool) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.enabled = enabled;
        }
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

/// Status bar message
#[derive(Debug, Clone)]
pub struct StatusMessage {
    /// Message text
    pub text: String,
    /// Message level: "info", "warning", "error"
    pub level: String,
}

impl StatusMessage {
    /// Create info message
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: "info".to_string(),
        }
    }

    /// Create warning message
    pub fn warning(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: "warning".to_string(),
        }
    }

    /// Create error message
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            level: "error".to_string(),
        }
    }
}

/// Application status bar
#[derive(Debug, Clone)]
pub struct StatusBar {
    /// Current message
    pub message: StatusMessage,
    /// Connection status
    pub connection_status: String,
    /// Machine status
    pub machine_status: String,
    /// Progress percentage (0-100)
    pub progress: u8,
}

impl StatusBar {
    /// Create new status bar
    pub fn new() -> Self {
        Self {
            message: StatusMessage::info("Ready"),
            connection_status: "Disconnected".to_string(),
            machine_status: "Idle".to_string(),
            progress: 0,
        }
    }

    /// Update status message
    pub fn set_message(&mut self, message: StatusMessage) {
        self.message = message;
    }

    /// Update connection status
    pub fn set_connection_status(&mut self, status: impl Into<String>) {
        self.connection_status = status.into();
    }

    /// Update machine status
    pub fn set_machine_status(&mut self, status: impl Into<String>) {
        self.machine_status = status.into();
    }

    /// Update progress
    pub fn set_progress(&mut self, progress: u8) {
        self.progress = progress.min(100);
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Main application window
#[derive(Debug)]
pub struct MainWindow {
    /// Window title
    pub title: String,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Menu bar
    pub menu_bar: MenuBar,
    /// Toolbar
    pub toolbar: Toolbar,
    /// Status bar
    pub status_bar: StatusBar,
    /// Is fullscreen
    pub fullscreen: bool,
    /// Is maximized
    pub maximized: bool,
}

impl MainWindow {
    /// Create new main window
    pub fn new() -> Self {
        Self {
            title: "GCodeKit4".to_string(),
            width: 1280,
            height: 960,
            menu_bar: MenuBar::new(),
            toolbar: Toolbar::new(),
            status_bar: StatusBar::new(),
            fullscreen: false,
            maximized: false,
        }
    }

    /// Set window title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Set window size
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Toggle fullscreen
    pub fn toggle_fullscreen(&mut self) {
        self.fullscreen = !self.fullscreen;
    }

    /// Toggle maximize
    pub fn toggle_maximize(&mut self) {
        self.maximized = !self.maximized;
    }

    /// Get configuration string
    pub fn config_string(&self) -> String {
        format!(
            "MainWindow(title={}, size={}x{}, fullscreen={}, maximized={})",
            self.title, self.width, self.height, self.fullscreen, self.maximized
        )
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_creation() {
        let item = MenuItem::new("File", "file");
        assert_eq!(item.label, "File");
        assert_eq!(item.id, "file");
    }

    #[test]
    fn test_menu_bar_creation() {
        let menu = MenuBar::new();
        assert_eq!(menu.file_menu.label, "File");
        assert_eq!(menu.edit_menu.label, "Edit");
    }

    #[test]
    fn test_toolbar_creation() {
        let toolbar = Toolbar::new();
        assert!(!toolbar.items.is_empty());
    }

    #[test]
    fn test_status_bar_messages() {
        let info = StatusMessage::info("Test");
        assert_eq!(info.level, "info");

        let warn = StatusMessage::warning("Warn");
        assert_eq!(warn.level, "warning");

        let err = StatusMessage::error("Error");
        assert_eq!(err.level, "error");
    }

    #[test]
    fn test_main_window_creation() {
        let window = MainWindow::new();
        assert_eq!(window.title, "GCodeKit4");
        assert_eq!(window.width, 1280);
        assert_eq!(window.height, 960);
    }

    #[test]
    fn test_main_window_resize() {
        let mut window = MainWindow::new();
        window.set_size(1920, 1080);
        assert_eq!(window.width, 1920);
        assert_eq!(window.height, 1080);
    }

    #[test]
    fn test_window_fullscreen() {
        let mut window = MainWindow::new();
        assert!(!window.fullscreen);
        window.toggle_fullscreen();
        assert!(window.fullscreen);
    }

    #[test]
    fn test_toolbar_item_enable() {
        let mut toolbar = Toolbar::new();
        toolbar.set_item_enabled("connect", false);
        let item = toolbar.items.iter().find(|i| i.id == "connect").unwrap();
        assert!(!item.enabled);
    }
}
