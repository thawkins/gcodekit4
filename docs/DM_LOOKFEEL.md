# Device Manager Aesthetic Analysis & Recommendations

## Current State Analysis
The current Device Manager implementation uses a mixed-theme approach that conflicts with the application's overall "Designer" dark aesthetic.

- **Inconsistent Theming**: The left pane uses a dark blue/grey (`#2c3e50`), while the right pane uses a light grey (`#ecf0f1`). This creates a jarring contrast and breaks the immersive dark mode experience.
- **Standard Widgets**: The view relies heavily on standard `std-widgets.slint` components (`Button`, `LineEdit`, `ComboBox`, `TabWidget`) without custom styling, resulting in a "default OS" look rather than a cohesive application UI.
- **Typography**: Text colors on the right pane are dark (`#2c3e50`, `#34495e`), which will become invisible when the background is switched to dark.
- **Layout**: The split-pane layout is functional but lacks visual hierarchy and separation.

## Recommendations

### 1. Global Dark Theme Adoption
Align the color palette with the Designer and Visualizer views:
- **Backgrounds**: 
  - Main Background: `#1e1e1e` or `#2d2d2d`
  - Panel/List Background: `#2d2d2d`
  - Input/Item Background: `#3e3e3e`
- **Text**:
  - Primary: `#e0e0e0`
  - Secondary/Labels: `#aaaaaa`
  - Active/Success: `#2ecc71` (Keep, but ensure contrast)
- **Borders**: `#404040`

### 2. Component Styling
Replace or style standard widgets to match the custom UI components:
- **Buttons**: Use the `MCToolButton` style or similar custom button implementation.
  - Primary Actions (Add/Save): Blue accent (`#4a90e2`).
  - Destructive Actions (Delete): Red accent (`#e74c3c`).
  - Secondary Actions: Dark grey (`#404040`).
- **Inputs (LineEdit/ComboBox)**:
  - Background: `#3e3e3e`
  - Text: `#ffffff`
  - Border: `#555555` (1px)
  - Rounded corners (4px)
- **List Items**:
  - Default: Transparent
  - Selected: `#4a90e2` (or a darker variant `#357abd`)
  - Hover: `#3e3e3e`

### 3. Layout Refinements
- **Header**: Unify the header style. The "Devices" title should align with the right pane's header.
- **Tabs**: If `TabWidget` cannot be easily styled, consider a custom tab bar using `HorizontalBox` and `Rectangle` indicators, or ensure the content within standard tabs follows the dark theme.
- **Spacing**: Increase padding in the right pane to let the content breathe (20px is good, but ensure internal spacing is consistent).

### 4. Specific UI Changes
- **Left Pane**: Change background to `#2d2d2d`. Update list item text to `#e0e0e0`.
- **Right Pane**: Change background to `#1e1e1e` (or `#252525` to distinguish from left pane).
- **Form Labels**: Change all `#34495e` text to `#aaaaaa`.
- **Headers**: Change `#2c3e50` headers to `#ffffff`.

## Implementation Plan
1.  Update `DeviceManagerPanel` background properties.
2.  Replace standard `Button` with custom styled buttons (or apply styles).
3.  Update all text colors to variables or hardcoded light values.
4.  Style `LineEdit` and `ComboBox` (may need to wrap them or use custom properties if `std-widgets` are limited, or simply change their colors if supported).
