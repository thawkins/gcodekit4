# Materials View Aesthetics Analysis & Recommendations

## Current State Analysis
The current `materials_manager.slint` implementation uses a **Light Theme** which is inconsistent with the "Designer" dark theme aesthetic established in other parts of the application (Device Manager, CNC Tools, etc.).

### Key Issues:
1.  **Color Palette**: Uses `#ecf0f1` (light grey) background and `#2c3e50` (dark) text. The application standard is a dark background (approx `#1e1e1e` or `#2b2b2b`) with light text.
2.  **Standard Widgets**: Relies heavily on `std-widgets.slint` (`Button`, `LineEdit`, `ComboBox`, `TabWidget`, `CheckBox`). These widgets do not match the custom styling of the rest of the application.
3.  **TabWidget**: The standard `TabWidget` is used, which often has visibility issues in dark modes (as noted in previous tasks) and lacks the custom styling of the `DMTabButton` pattern used in Device Manager.
4.  **List Styling**: The material list items use a white background with a light blue selection state, which clashes with the dark theme.

## Recommendations

### 1. Theme Adoption (Dark Mode)
*   **Background**: Change main background to `#1e1e1e` or `#2d2d2d`.
*   **Text Color**: Change primary text to `#e0e0e0` or `white`, and secondary text to `#aaaaaa`.
*   **Panel Backgrounds**: Use slightly lighter dark greys (e.g., `#3e3e3e`) for panels/groups to create depth.

### 2. Component Replacement
Replace standard widgets with the custom "Designer" style widgets found in `device_manager.slint` or `tools_manager.slint`:

*   **Buttons**: Replace `Button` with `MCToolButton` (or `DMButton` if available/appropriate).
    *   *Style*: Dark background, light text, hover effects, icon support.
*   **Inputs**: Replace `LineEdit` and `TextEdit` with `DMInput` (or `CustomTextEdit`).
    *   *Style*: Dark background (`#2b2b2b`), light text, subtle border.
*   **Dropdowns**: Replace `ComboBox` with `DMComboBox`.
*   **Checkboxes**: Replace `CheckBox` with `DMCheckBox` (to fix visibility issues).
*   **Tabs**: Replace `TabWidget` with the custom tab implementation using `DMTabButton` and a content area switched by a property.

### 3. Layout & Structure
*   **Left Panel (List)**:
    *   Keep the search/filter at the top.
    *   Style the list items to have a dark background, lighter on hover, and a distinct accent color (e.g., `#007acc` or `#e67e22`) for the selected state.
    *   Use `DMLabel` or styled `Text` elements for list item details.
*   **Right Panel (Details)**:
    *   Use a `Rectangle` with a border/background to frame the details area.
    *   Implement the custom tab bar at the top of the details view.
    *   Organize fields using `GridLayout` or nested `VerticalBox`/`HorizontalBox` with consistent spacing.

### 4. Specific Styling Details
*   **Headers**: Use larger, bold text for section headers (e.g., "Search & Filter", "Material Details").
*   **Borders**: Use subtle borders (`#444444`) to separate sections.
*   **Icons**: Ensure buttons use icons where appropriate (e.g., Save, Delete, New) using the `assets/icons/` paths.

## Implementation Plan
1.  Import custom widgets (`MCToolButton`, `DMInput`, `DMComboBox`, `DMCheckBox`, `DMTabButton`, etc.) from `device_manager.slint` (or define them if not globally available).
2.  Refactor the root `Rectangle` to use the dark theme background.
3.  Rewrite the Left Panel to use custom inputs and buttons.
4.  Rewrite the List View delegate to match the dark theme.
5.  Rewrite the Right Panel to use the custom Tab system and custom form inputs.
