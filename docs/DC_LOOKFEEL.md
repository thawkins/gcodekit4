# Device Config View - Aesthetic Analysis & Recommendations

## Current State Analysis
The current `ConfigSettingsPanel` (`crates/gcodekit4-ui/ui_panels/config_settings.slint`) implements a functional but basic "Light Mode" design that is inconsistent with the recently updated "Dark Mode" aesthetic of the Device Manager and other system components.

### Key Issues
1.  **Inconsistent Theme**: The panel uses a light color palette (`#ffffff`, `#ecf0f1`) which clashes with the application's dark theme direction (`#1e1e1e`, `#2d2d2d`).
2.  **Standard Widgets**: Usage of unstyled `std-widgets` (`Button`, `LineEdit`, `ComboBox`) creates a disjointed look compared to the custom `DMButton`, `DMInput`, etc.
3.  **List Readability**:
    *   Alternating row colors (`#ecf0f1`/`#ffffff`) are too bright.
    *   Text contrast is designed for light mode (dark text on light bg), making it hard to adapt without a full overhaul.
    *   The "Edit" button inside rows adds visual clutter.
4.  **Dialog Styling**: The modal dialog uses a light background with standard widgets, breaking immersion.
5.  **Header & Status Bar**: The header color (`#34495e`) and status bar (`#ecf0f1`) are holdovers from a previous design language.

## Recommendations

### 1. Global Theme Adoption
*   **Background**: Change main background to `#1e1e1e`.
*   **Panels**: Use `#2d2d2d` for distinct sections (like the list container or sidebars if added).
*   **Text**:
    *   Primary: `#e0e0e0` (High emphasis)
    *   Secondary: `#aaaaaa` (Labels, units, descriptions)
    *   Accent: `#4a90e2` (Active states, highlights)

### 2. Component Replacement
Replace standard widgets with custom implementations matching `DeviceManager`:
*   `Button` -> `DCButton` (based on `DMButton` style: flat, colored states).
*   `LineEdit` -> `DCInput` (based on `DMInput`: dark bg `#3e3e3e`, border `#555555`).
*   `ComboBox` -> Styled `ComboBox` or custom dropdown if possible (standard `ComboBox` is hard to style in Slint without custom implementation, but we can wrap it or style the surrounding container).

### 3. List View Overhaul
*   **Header**: Dark background `#252525`, bold light text `#ffffff`.
*   **Rows**:
    *   Background: Alternating `#1e1e1e` and `#252525` (or slightly lighter `#232323`).
    *   Hover State: `#333333` or slight tint of accent color.
    *   Selection: `#4a90e2` border or background tint.
*   **Columns**:
    *   ID: Monospace font or distinct color (e.g., `#f1c40f` or `#aaaaaa`).
    *   Value: Green (`#2ecc71`) for valid, Red for invalid/error.
    *   Action: Replace "Edit" button with a row-click action or a subtle icon button.

### 4. Dialog Redesign
*   **Background**: `#2d2d2d` with a subtle border `#404040` and shadow.
*   **Overlay**: `rgba(0, 0, 0, 0.7)` for better focus.
*   **Inputs**: Use `DCInput` for value editing.
*   **Typography**: Clear hierarchy with bold titles and descriptive helper text.

### 5. Layout Refinements
*   **Toolbar**: Group related actions. Use icons if available, otherwise consistent button sizing.
*   **Filter Bar**: Integrate seamlessly into the top area.
*   **Status Bar**: Dark background `#252525`, subtle text.

## Implementation Plan
1.  Define `DCButton` and `DCInput` components (reusing `DM*` logic).
2.  Update `ConfigSettingsPanel` background and layout containers.
3.  Refactor `ListView` to use the new color scheme.
4.  Redesign the `Edit Dialog` with the dark theme.
5.  Update text colors and font weights for readability.
