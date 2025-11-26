# Designer Look & Feel Analysis

## Current State Analysis

The current Designer component (`crates/gcodekit4-designer/ui/designer.slint`) implements a functional 2D CAD interface. The aesthetic is utilitarian, relying on a mix of dark and light themes.

### Color Palette
- **Dark Backgrounds**: `#2c3e50` (Midnight Blue), `#34495e` (Wet Asphalt) - Used for the left sidebar and canvas background.
- **Light Backgrounds**: `#ecf0f1` (Clouds), `#bdc3c7` (Silver), `white` - Used for the right properties panel and dialogs.
- **Accents**: 
  - Primary: `#3498db` (Peter River - Blue) - Used for selection, active tools.
  - Success: `#2ecc71` (Emerald - Green) - Used for groups.
  - Warning/Highlight: `#ffeb3b` (Yellow) - Used for selection handles and rubber band.
- **Borders**: `#ccc`, `#bdc3c7`.

### Layout Structure
- **Left Sidebar (Tools)**: Fixed width (`210px`). Contains tool icons in a grid and tool setup controls.
- **Center (Canvas)**: Flexible width. Dark background. Contains the interactive drawing area.
- **Right Sidebar (Properties/Layers)**: Fixed width (`270px`). Light background. Tabbed interface for properties and layers.

### UI Components
- **ToolButton**: 48x48px square buttons. Uses Unicode characters for icons.
- **CompactSpinBox**: Custom input control with small up/down buttons.
- **Tabs**: Simple rectangular tabs for switching between Properties and Layers.
- **Canvas Overlays**: Text indicators for scale, coordinates, and grid size.

## Critique

1.  **Inconsistent Theme**: The application uses a "split personality" theme. The left side and center are dark mode, while the right side is light mode. This creates high contrast transitions that can be visually fatiguing.
2.  **Iconography**: The use of Unicode characters (`➜`, `▭`, `●`, etc.) for tool icons is a good MVP strategy but lacks professional polish. They can render differently across systems and fonts.
3.  **Visual Density**: The "Tool Setup" section in the left sidebar and the "Properties" panel are quite dense. The `CompactSpinBox` controls are functional but visually heavy with their borders and small touch targets.
4.  **Tab Design**: The tabs in the right sidebar are boxy and lack a modern feel. The active state is indicated by color but could be more distinct.
5.  **Canvas Feedback**: The canvas info strip (Scale, X, Y) is functional but looks like a debug bar.

## Suggestions for Improvement

### 1. Unify the Color Theme
Adopt a consistent theme across all panels. A full dark theme is often preferred for CAD/CAM software as it reduces eye strain during long sessions.

*   **Recommendation**: Switch the Right Sidebar (Properties) to use the dark theme colors (`#2c3e50` / `#34495e`) with light text (`#ecf0f1`).
*   **Input Fields**: Update `CompactSpinBox` and `TextInput` to have dark backgrounds (`#2c3e50`) with light borders and text.

### 2. Modernize Iconography
Replace Unicode text icons with SVG icons. Slint supports `Image` widgets which can render SVGs.

*   **Action**: Create or source a set of simple, monoline SVG icons for: Select, Rectangle, Circle, Line, Ellipse, Polyline, Text.
*   **Benefit**: Consistent look, scalable, and professional appearance.

### 3. Refine the Properties Panel
The properties panel is a long list of controls.

*   **Grouping**: Use visual separators or "Card" style backgrounds for distinct groups (e.g., "Transform", "Geometry", "CAM Operations").
*   **Accordions**: Implement collapsible sections to hide less frequently used properties (like advanced CAM settings).
*   **Labels**: Align labels consistently. The current mix of side-by-side and top-bottom labels can be tidied up.

### 4. Enhance Canvas Controls
*   **Floating Controls**: Instead of the top bar for info, consider floating pills or a status bar at the bottom for coordinates and zoom level.
*   **On-Canvas Tools**: Add floating buttons for Zoom In/Out/Fit directly on the canvas corner for easier access.

### 5. Polish UI Components
*   **Tabs**: Change tabs to a more modern style (e.g., text with a bottom border indicator for the active tab, transparent background).
*   **SpinBoxes**: Simplify the `CompactSpinBox`. Maybe remove the visible up/down buttons in favor of a cleaner look, or make them appear only on hover.
*   **Borders**: Reduce border contrast. Use slightly lighter/darker shades of the background color for borders instead of hard `#ccc`.

### 6. Visual Hierarchy & Typography
*   **Headings**: Make section headings (like "Tool Setup", "Transform") more distinct (uppercase, smaller font, or different color).
*   **Fonts**: Ensure `Fira Code` (or the system UI font) is used consistently.

## Implementation Plan (Phased)

1.  **Phase 1 (Quick Wins)**: 
    *   Apply dark theme to the Right Sidebar.
    *   Standardize padding and spacing in the Properties panel.
2.  **Phase 2 (Assets)**:
    *   Replace Unicode icons with SVGs.
3.  **Phase 3 (Components)**:
    *   Refactor `CompactSpinBox` for better aesthetics.
    *   Implement collapsible sections in the Properties panel.
