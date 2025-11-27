# G-Code Editor Aesthetics Analysis & Recommendations

## Current State Analysis

The current G-Code Editor (`GcodeEditorPanel`) uses a functional but distinct aesthetic compared to the newly updated Designer and Visualizer components.

*   **Theme**: Uses a dark grey theme (`#2d2d30` toolbar, `#1e1e1e` editor background) which differs from the Designer's blue-grey theme (`#2c3e50` panels, `#34495e` canvas).
*   **Layout**: Uses a **Top Toolbar** layout for file and transmission controls. The Designer and Visualizer now use a **Left Sidebar** layout.
*   **Controls**: Uses custom `Rectangle`-based buttons with text/icons. The Designer uses standardized `ToolButton` components.
*   **Status Information**: Displays file name, cursor position, and line counts inline within the top toolbar. The Visualizer uses **Floating Overlays** for this type of information.

## Recommendations

To unify the application's look and feel, the G-Code Editor should adopt the "Designer Standard" aesthetics.

### 1. Layout Restructuring (Left Sidebar)

Move the controls from the top toolbar to a fixed-width **Left Sidebar** (200px).

*   **Container**: `Rectangle { width: 200px; background: #2c3e50; ... }`
*   **Sections**:
    *   **Transmission**: Send, Pause, Resume, Stop buttons.
    *   **File Operations**: New, Open, Save (if not handled by main menu).
    *   **Edit Controls**: Undo, Redo.

### 2. Color Palette Harmonization

Adopt the Designer's color palette while maintaining code readability.

*   **Sidebar Background**: `#2c3e50` (Matches Designer/Visualizer sidebars).
*   **Editor Background**: `#1e1e1e` (Retain for standard code editor contrast) OR `#34495e` (Matches Designer canvas).
    *   *Recommendation*: Keep `#1e1e1e` for the text area for better syntax highlighting contrast, but ensure the surrounding frame/gutter matches `#2c3e50`.
*   **Line Number Gutter**: `#2c3e50` (Matches sidebar).
*   **Text Color**: `#ecf0f1` (White/Light Grey) instead of the current matrix green `#00ff00`, to match the professional look of the Designer.
*   **Current Line Highlight**: `#34495e` (Matches Designer canvas).

### 3. Component Modernization

Replace ad-hoc UI elements with standardized components.

*   **Buttons**: Use `VisualizerToolButton` (or rename to `ToolButton`) for all actions.
    *   Icons: ▶ (Send), ⏸ (Pause), ⏹ (Stop), ⎌ (Undo/Redo).
*   **Separators**: Use the 1px right-border separator pattern in the sidebar.

### 4. Floating Status Overlays

Move status information out of the toolbar/sidebar to maximize vertical space.

*   **Status Pill (Bottom-Right)**:
    *   Content: `Line 10:5` | `Total: 1000` | `filename.gcode`
    *   Style: Floating semi-transparent pill (`opacity: 0.9`, `background: #2c3e50`, `border-radius: 15px`).

## Implementation Plan

1.  **Refactor `GcodeEditorPanel`**:
    *   Remove the top `Rectangle` toolbar.
    *   Introduce a `HorizontalLayout` containing:
        *   **Left Sidebar**: `Rectangle { width: 200px; ... }` with `VerticalLayout` for controls.
        *   **Editor Area**: `CustomTextEdit` taking remaining space.
2.  **Update `CustomTextEdit` Styling**:
    *   Expose properties for `gutter-background` and `editor-background` if not already present.
    *   Update default colors to match the new palette.
3.  **Add Floating Overlay**:
    *   Add a `Rectangle` overlay in the editor area (z-index > editor) for status text.
