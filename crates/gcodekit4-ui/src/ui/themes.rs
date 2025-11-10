//! Theme and styling system
//!
//! Manages application appearance, colors, fonts, and DPI scaling.

use serde::{Deserialize, Serialize};

/// Theme identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeId {
    /// Light theme (light background, dark text)
    Light,
    /// Dark theme (dark background, light text)
    Dark,
    /// High contrast theme (maximum contrast)
    HighContrast,
}

impl std::fmt::Display for ThemeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeId::Light => write!(f, "Light"),
            ThemeId::Dark => write!(f, "Dark"),
            ThemeId::HighContrast => write!(f, "High Contrast"),
        }
    }
}

/// Color definition
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create new color
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Fully opaque color
    pub fn opaque(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() < 7 {
            return None;
        }
        let hex = hex.trim_start_matches('#');
        u32::from_str_radix(hex, 16).ok().map(|n| {
            let r = ((n >> 24) & 0xFF) as u8;
            let g = ((n >> 16) & 0xFF) as u8;
            let b = ((n >> 8) & 0xFF) as u8;
            let a = (n & 0xFF) as u8;
            Self { r, g, b, a }
        })
    }
}

/// Theme colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // Background colors
    pub background_primary: Color,
    pub background_secondary: Color,
    pub background_tertiary: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,

    // UI element colors
    pub button_background: Color,
    pub button_hover: Color,
    pub button_active: Color,
    pub button_text: Color,

    // State colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // Borders
    pub border_color: Color,
    pub border_focus: Color,

    // Accents
    pub accent_primary: Color,
    pub accent_secondary: Color,
}

impl ThemeColors {
    /// Create light theme colors
    pub fn light() -> Self {
        Self {
            background_primary: Color::opaque(255, 255, 255),
            background_secondary: Color::opaque(245, 245, 245),
            background_tertiary: Color::opaque(240, 240, 240),
            text_primary: Color::opaque(0, 0, 0),
            text_secondary: Color::opaque(100, 100, 100),
            text_disabled: Color::opaque(180, 180, 180),
            button_background: Color::opaque(230, 230, 230),
            button_hover: Color::opaque(220, 220, 220),
            button_active: Color::opaque(200, 200, 200),
            button_text: Color::opaque(0, 0, 0),
            success: Color::opaque(76, 175, 80),
            warning: Color::opaque(255, 193, 7),
            error: Color::opaque(244, 67, 54),
            info: Color::opaque(33, 150, 243),
            border_color: Color::opaque(200, 200, 200),
            border_focus: Color::opaque(66, 133, 244),
            accent_primary: Color::opaque(66, 133, 244),
            accent_secondary: Color::opaque(156, 39, 176),
        }
    }

    /// Create dark theme colors
    pub fn dark() -> Self {
        Self {
            background_primary: Color::opaque(30, 30, 30),
            background_secondary: Color::opaque(45, 45, 45),
            background_tertiary: Color::opaque(60, 60, 60),
            text_primary: Color::opaque(240, 240, 240),
            text_secondary: Color::opaque(180, 180, 180),
            text_disabled: Color::opaque(100, 100, 100),
            button_background: Color::opaque(60, 60, 60),
            button_hover: Color::opaque(80, 80, 80),
            button_active: Color::opaque(100, 100, 100),
            button_text: Color::opaque(240, 240, 240),
            success: Color::opaque(129, 199, 132),
            warning: Color::opaque(255, 213, 79),
            error: Color::opaque(229, 57, 53),
            info: Color::opaque(100, 181, 246),
            border_color: Color::opaque(80, 80, 80),
            border_focus: Color::opaque(100, 181, 246),
            accent_primary: Color::opaque(100, 181, 246),
            accent_secondary: Color::opaque(186, 104, 200),
        }
    }

    /// Create high contrast theme
    pub fn high_contrast() -> Self {
        Self {
            background_primary: Color::opaque(0, 0, 0),
            background_secondary: Color::opaque(30, 30, 30),
            background_tertiary: Color::opaque(60, 60, 60),
            text_primary: Color::opaque(255, 255, 255),
            text_secondary: Color::opaque(200, 200, 200),
            text_disabled: Color::opaque(100, 100, 100),
            button_background: Color::opaque(50, 50, 50),
            button_hover: Color::opaque(100, 100, 100),
            button_active: Color::opaque(150, 150, 150),
            button_text: Color::opaque(255, 255, 255),
            success: Color::opaque(0, 255, 0),
            warning: Color::opaque(255, 255, 0),
            error: Color::opaque(255, 0, 0),
            info: Color::opaque(0, 255, 255),
            border_color: Color::opaque(200, 200, 200),
            border_focus: Color::opaque(255, 255, 255),
            accent_primary: Color::opaque(0, 255, 255),
            accent_secondary: Color::opaque(255, 0, 255),
        }
    }
}

/// Font configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Font family name
    pub family: String,
    /// Base font size in pixels
    pub base_size: f32,
    /// Line height multiplier
    pub line_height: f32,
}

impl FontConfig {
    /// Create default font config
    pub fn default_sans() -> Self {
        Self {
            family: "sans-serif".to_string(),
            base_size: 12.0,
            line_height: 1.5,
        }
    }

    /// Create monospace font config
    pub fn monospace() -> Self {
        Self {
            family: "monospace".to_string(),
            base_size: 11.0,
            line_height: 1.6,
        }
    }

    /// Get font size for heading level (1-6)
    pub fn heading_size(&self, level: u8) -> f32 {
        match level {
            1 => self.base_size * 2.0,
            2 => self.base_size * 1.7,
            3 => self.base_size * 1.4,
            4 => self.base_size * 1.2,
            _ => self.base_size * 1.1,
        }
    }
}

/// Complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme ID
    pub id: ThemeId,
    /// Color palette
    pub colors: ThemeColors,
    /// Font settings
    pub font: FontConfig,
    /// Spacing multiplier
    pub spacing: f32,
    /// Border radius
    pub border_radius: f32,
    /// Shadow blur radius
    pub shadow_blur: f32,
}

impl Theme {
    /// Create light theme
    pub fn light() -> Self {
        Self {
            id: ThemeId::Light,
            colors: ThemeColors::light(),
            font: FontConfig::default_sans(),
            spacing: 1.0,
            border_radius: 4.0,
            shadow_blur: 2.0,
        }
    }

    /// Create dark theme
    pub fn dark() -> Self {
        Self {
            id: ThemeId::Dark,
            colors: ThemeColors::dark(),
            font: FontConfig::default_sans(),
            spacing: 1.0,
            border_radius: 4.0,
            shadow_blur: 4.0,
        }
    }

    /// Create high contrast theme
    pub fn high_contrast() -> Self {
        Self {
            id: ThemeId::HighContrast,
            colors: ThemeColors::high_contrast(),
            font: FontConfig::default_sans(),
            spacing: 1.2,
            border_radius: 2.0,
            shadow_blur: 1.0,
        }
    }

    /// Get theme by ID
    pub fn by_id(id: ThemeId) -> Self {
        match id {
            ThemeId::Light => Self::light(),
            ThemeId::Dark => Self::dark(),
            ThemeId::HighContrast => Self::high_contrast(),
        }
    }
}

/// Theme manager
pub struct ThemeManager {
    current_theme: Theme,
    available_themes: Vec<ThemeId>,
}

impl ThemeManager {
    /// Create new theme manager
    pub fn new(initial: ThemeId) -> Self {
        Self {
            current_theme: Theme::by_id(initial),
            available_themes: vec![ThemeId::Light, ThemeId::Dark, ThemeId::HighContrast],
        }
    }

    /// Get current theme
    pub fn current(&self) -> &Theme {
        &self.current_theme
    }

    /// Get current theme (mutable)
    pub fn current_mut(&mut self) -> &mut Theme {
        &mut self.current_theme
    }

    /// Set theme
    pub fn set_theme(&mut self, id: ThemeId) {
        self.current_theme = Theme::by_id(id);
    }

    /// Get all available themes
    pub fn available(&self) -> &[ThemeId] {
        &self.available_themes
    }

    /// Get current theme ID
    pub fn current_id(&self) -> ThemeId {
        self.current_theme.id
    }

    /// Apply font size multiplier
    pub fn set_font_size_scale(&mut self, scale: f32) {
        let scale = scale.max(0.8).min(2.0); // Clamp between 0.8x and 2.0x
        self.current_theme.font.base_size = 12.0 * scale;
    }

    /// Apply spacing multiplier
    pub fn set_spacing_scale(&mut self, scale: f32) {
        let scale = scale.max(0.5).min(2.0);
        self.current_theme.spacing = scale;
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new(ThemeId::Dark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::opaque(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_color_hex() {
        let color = Color::opaque(255, 128, 64);
        let hex = color.to_hex();
        assert_eq!(hex, "#FF8040FF");
    }

    #[test]
    fn test_theme_colors_light() {
        let colors = ThemeColors::light();
        assert_eq!(colors.background_primary.r, 255);
    }

    #[test]
    fn test_theme_colors_dark() {
        let colors = ThemeColors::dark();
        assert_eq!(colors.background_primary.r, 30);
    }

    #[test]
    fn test_font_config() {
        let font = FontConfig::default_sans();
        assert_eq!(font.base_size, 12.0);
    }

    #[test]
    fn test_font_heading_size() {
        let font = FontConfig::default_sans();
        assert_eq!(font.heading_size(1), 24.0);
        assert_eq!(font.heading_size(3), 16.8);
    }

    #[test]
    fn test_theme_manager() {
        let mgr = ThemeManager::new(ThemeId::Dark);
        assert_eq!(mgr.current_id(), ThemeId::Dark);
    }

    #[test]
    fn test_theme_manager_set() {
        let mut mgr = ThemeManager::new(ThemeId::Dark);
        mgr.set_theme(ThemeId::Light);
        assert_eq!(mgr.current_id(), ThemeId::Light);
    }

    #[test]
    fn test_font_size_scale() {
        let mut mgr = ThemeManager::new(ThemeId::Dark);
        mgr.set_font_size_scale(1.5);
        assert!((mgr.current().font.base_size - 18.0).abs() < 0.01);
    }
}
