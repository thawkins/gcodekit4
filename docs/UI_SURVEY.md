# UI Consistency Survey

## Overview
This document reports on the consistency of styling, layout, and function across the 11 tabbed views in the GCodeKit4 application.

## Inspected Views
1.  **G-Code Editor** (`ui_panels/gcode_editor.slint`)
2.  **Machine Control** (`ui_panels/machine_control.slint`)
3.  **Device Console** (`ui_panels/device_console.slint`)
4.  **Device Info** (`ui_panels/device_info.slint`)
5.  **Device Manager** (`../gcodekit4-devicedb/ui/device_manager.slint`)
6.  **Device Config** (`ui_panels/config_settings.slint`)
7.  **Visualizer** (`../gcodekit4-visualizer/ui/gcode_visualizer.slint`)
8.  **Designer** (`../gcodekit4-designer/ui/designer.slint`)
9.  **CAM Tools** (`ui_panels/cam_tools.slint`)
10. **Materials** (`ui_panels/materials_manager.slint`)
11. **CNC Tools** (`ui_panels/tools_manager.slint`)

## Findings

### 1. Component Duplication (Critical)
Almost every panel redefines basic UI components (Buttons, Inputs, Checkboxes) with unique prefixes, leading to massive code duplication and potential styling drift.

*   **Buttons**:
    *   `EditorToolButton` (G-Code Editor)
    *   `MCToolButton` (Machine Control)
    *   `ConsoleToolButton` (Device Console)
    *   `InfoToolButton` (Device Info)
    *   `DMButton` (Device Manager)
    *   `DCButton` (Device Config)
    *   `VisualizerToolButton` (Visualizer)
    *   `ToolButton` (Designer)
    *   `MMButton` (Materials)
    *   `TMButton` (CNC Tools)
*   **Inputs**:
    *   `DMInput` (Device Manager)
    *   `DCInput` (Device Config)
    *   `MMInput` (Materials)
    *   `TMInput` (CNC Tools)
    *   Standard `LineEdit` used directly in others.

### 2. Layout Inconsistencies
*   **Sidebar Widths**:
    *   **200px**: G-Code Editor, Device Console, Visualizer.
    *   **250px**: Machine Control, Device Info, Device Manager, Designer (Left).
    *   **270px**: Designer (Right).
    *   **350px**: Materials, CNC Tools.
    *   **No Sidebar**: Device Config (uses top toolbar).
*   **Structure**:
    *   Most use a `HorizontalLayout` with a left sidebar.
    *   `config_settings.slint` uses a `VerticalBox` with a top toolbar, breaking the pattern.
    *   `cam_tools.slint` uses a grid layout (appropriate for its content, but distinct).

### 3. Styling Variations
*   **Colors**:
    *   Backgrounds vary slightly between `#1e1e1e`, `#2d2d2d`, and `#252526`.
    *   Sidebar backgrounds are mostly `#2c3e50` but some inner panels use `#34495e`.
    *   Highlight colors are generally consistent (Blue `#3498db`/`#2980b9`, Green `#2ecc71`), but specific hex values for hover/pressed states vary across the duplicated components.
*   **Icons**:
    *   Some panels use text emojis (e.g., "🔌", "🔄").
    *   Designer uses SVG images (`@image-url`).
    *   CAM Tools uses `GeneratedIcon` (SVG paths defined in code).
    *   Machine Control uses text characters like "▲", "▼".

### 4. Functional Inconsistencies
*   **CRUD Interfaces**:
    *   `materials_manager.slint` and `tools_manager.slint` are nearly identical in functionality but implemented as completely separate codebases with their own component definitions (`MM*` vs `TM*`).
*   **Status Bars**:
    *   Visualizer and Designer have floating status pills at the bottom.
    *   Other panels use fixed status areas or none.

## Recommendations

### 1. Create a Shared UI Library
Extract common components into a shared module (e.g., `ui/components/shared.slint`) and refactor all panels to use them.
*   `StandardButton` (replacing `*Button`)
*   `StandardInput` (replacing `*Input`)
*   `StandardCheckBox`
*   `StandardSpinBox`
*   `StandardSidebar` (layout container)

### 2. Standardize Layouts
*   Adopt a standard sidebar width (e.g., **250px**) for all panels with sidebars.
*   Refactor `config_settings.slint` to match the sidebar pattern if possible, or explicitly define it as a full-width view.
*   Standardize the "List on Left, Details on Right" pattern used in Device Manager, Materials, and CNC Tools.

### 3. Unify Styling
*   Define global color properties in a `theme.slint` file.
*   Use these properties in the shared components.
*   Decide on a single icon strategy (preferably SVG icons for a professional look, or a consistent set of font icons).

### 4. Refactor CRUD Panels
*   Create a generic `MasterDetailView` component or pattern that can be used by Device Manager, Materials Manager, and CNC Tools Manager to reduce code duplication.

### 5. Clean Up
*   Remove unused properties and callbacks.
*   Ensure consistent naming conventions for callbacks (e.g., `on-clicked` vs `clicked`).
