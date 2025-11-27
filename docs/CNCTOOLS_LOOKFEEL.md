# CNC Tools View Aesthetics Analysis & Recommendations

## Current State Analysis
The current `tools_manager.slint` implementation uses a "Light Theme" which clashes with the overall "Dark Theme" aesthetic of the Designer and other recently updated views (Visualizer, Device Console, etc.).

*   **Backgrounds:** Uses `#ecf0f1` (light grey) and `white` for panels.
*   **Text:** Uses dark colors (`#2c3e50`, `#7f8c8d`).
*   **Widgets:** Uses standard `std-widgets.slint` components (`Button`, `LineEdit`, `ComboBox`, `GroupBox`) without custom styling, resulting in a native/default look that doesn't match the custom UI of the rest of the application.
*   **Selection:** Uses standard blue `#3498db` which is fine but needs to be integrated into a dark context.
*   **Layout:** The Split-View (List on left, Details on right) is functional and good, but the visual execution needs modernization.

## Recommendations for "Designer" Look & Feel

To match the **Designer** and **Dark Theme** aesthetic, the following changes are recommended:

### 1. Color Palette (Dark Theme)
*   **Main Background:** Change from `#ecf0f1` to `#1e1e1e` (or the app's standard dark background).
*   **Panel Backgrounds:** Change from `white` to `#2d2d2d` or `#252526` (Card/Panel background).
*   **Text Colors:**
    *   Primary Text: Change from `#2c3e50` to `#e0e0e0` or `white`.
    *   Secondary Text: Change from `#7f8c8d` to `#aaaaaa` or `#888888`.
    *   Accent Text: Keep `#3498db` (Blue) or use the app's primary accent color (e.g., Orange/Yellow if that's the theme, but Blue is often used for selection).
*   **Borders:** Change from `#bdc3c7` to `#404040` or `#505050`.

### 2. Component Styling
*   **Buttons:** Replace standard `Button` with custom styled buttons (e.g., `MCToolButton` style or similar flat, dark buttons with hover states).
    *   Primary Action (Save/New): distinct color (e.g., Blue or Green).
    *   Secondary Action (Cancel): muted dark grey.
    *   Destructive Action (Delete): Red.
*   **Inputs (`LineEdit`, `TextEdit`, `SpinBox`):**
    *   Background: Darker than the panel (e.g., `#1a1a1a`).
    *   Text: White.
    *   Border: Thin, subtle grey (`#404040`).
*   **ComboBox:** Needs a dark background popup and styled arrow.
*   **GroupBox:** The standard `GroupBox` often has a white background or light border. It might need to be replaced with a custom `Rectangle` with a `Text` header to control the look fully.

### 3. List View Styling
*   **Item Background:** Transparent or very dark grey (`#2d2d2d`).
*   **Selected Item:** Dark Blue (`#0d47a1` or similar) or a lighter grey (`#3e3e42`) to indicate selection without being too jarring.
*   **Hover State:** Slight lighten effect.
*   **Icons:** Ensure any icons (if added) are light/white.

### 4. Layout Refinements
*   **Header:** Make the "CNC Tools Manager" header consistent with other panel headers (font size, padding).
*   **Spacing:** Maintain the 10px spacing but ensure it feels "grid-like" and aligned.
*   **Scrollbars:** Ensure `ScrollView` uses a dark scrollbar if possible (or custom styled).

### 5. Specific Code Changes
*   Replace `import { ... } from "std-widgets.slint"` with imports of custom widgets if they exist in the project (e.g., `DMInput`, `DMButton` from Device Manager if reusable, or create local custom definitions).
*   Refactor `GroupBox` usages to custom layouts if standard `GroupBox` doesn't support dark theme well.
*   Update `ToolData` visualization in the list to use a "Card" style if appropriate, or a clean "Row" style.

## Summary
The goal is to "invert" the colors from Light to Dark and apply the custom "flat" styling found in the Designer view to make the CNC Tools Manager feel like an integrated part of the suite.
