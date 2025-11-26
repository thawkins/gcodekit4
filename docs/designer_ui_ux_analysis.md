# Designer Tool UI/UX Analysis & Improvement Plan

## 1. Executive Summary

The current Designer tool in GCodeKit4 provides functional 2D CAD/CAM capabilities but deviates significantly from industry-standard UI/UX patterns found in vector editing software (e.g., Adobe Illustrator, Figma, Inkscape). While the underlying logic (state management, undo/redo, toolpath generation) is robust, the user interface relies heavily on modal dialogs and non-standard interaction models, which may increase cognitive load for users accustomed to standard design tools.

This analysis outlines the current patterns, compares them with industry norms, and proposes a roadmap to align the interface with user expectations.

## 2. Current UI/UX Patterns

### Layout
- **Left Sidebar**: Contains drawing tools (Select, Rectangle, etc.) and "Tool Setup" (CAM parameters like Feed/Speed).
- **Center Canvas**: Main drawing area with a top info bar showing coordinates and zoom level.
- **Right Side**: Currently unused or occupied by overlays.
- **Properties Editing**: Performed via a **modal dialog** ("Shape Properties") triggered by a context menu or button.
- **Layer Management**: Non-existent; no visual hierarchy of shapes.

### Interaction Model
- **Selection**: Click to select. Shift+Drag for marquee/rubber-band selection.
- **Navigation**: 
  - **Pan**: Left-drag on empty space.
  - **Zoom**: `+` / `-` buttons or keys.
- **Context Menu**: Right-click provides access to critical functions (Properties, Group, Align).

### Current Keystroke Bindings
The following keystrokes are currently implemented and functional:
- **Escape**: Deselect all objects.
- **Delete**: Delete selected objects.
- **Ctrl + Z**: Undo last action.
- **Ctrl + Shift + Z** / **Ctrl + Y**: Redo last action.
- **+** / **=**: Zoom in.
- **-**: Zoom out.
- **Shift**: Hold for multi-selection or rubber-band selection.
- **Note**: Tooltips suggest single-letter shortcuts (S, R, C, L, E, P, T) for tool switching, but these are not currently implemented in the code.

## 3. Comparison with Industry Norms

### Competitive Benchmark: Inkscape
[Inkscape](https://inkscape.org/) represents the open-source standard for vector graphics and is explicitly used for "making patterns for use with cutting machines and laser engravers" (Source: [Inkscape Manual](https://inkscape-manuals.readthedocs.io/en/latest/why-use-inkscape.html)).

**Key Inkscape Patterns vs. GCodeKit4:**
*   **Object-Oriented Workflow**: Inkscape treats every element as a distinct "object" with style and geometry properties. GCodeKit4 shares this underlying logic but hides it behind modal dialogs.
*   **Direct Manipulation**: In Inkscape, users expect to click an object and immediately see its properties (width, height, position) in a toolbar or side panel, editable in real-time.
*   **Scalability**: Inkscape emphasizes that vector images are "mathematically-defined lines and curves" that maintain sharpness. GCodeKit4's UI should reflect this precision by offering precise, non-blocking coordinate inputs at all times.

### Broader Market Analysis
A review of the [Top 10 Vector Graphic Software](https://veikk.com/route/blog/post?blog_post_id=88) highlights several universal standards that GCodeKit4 should aim to meet:

1.  **Adobe Illustrator & CorelDRAW (The Heavyweights)**
    *   **Standard**: "Pixel-perfect shapes" and "flawless alignment".
    *   **Lesson**: Users expect high-precision alignment tools (snapping, alignment guides) to be front-and-center, not buried in menus.
    *   **UI Pattern**: Comprehensive toolsets are organized in **dockable panels**, allowing users to customize their workspace without losing access to the canvas.

2.  **Affinity Designer (The Performer)**
    *   **Standard**: "Real-time transformations" and "60fps zooming".
    *   **Lesson**: Performance is a feature. The UI must feel responsive. Laggy dialogs or canvas redraws break the flow.
    *   **UI Pattern**: **Floating-point accuracy** allows infinite zooming, reinforcing the need for a robust coordinate system display.

3.  **Gravit Designer, Vectr, & SVG-Edit (The Web/Lightweight Class)**
    *   **Standard**: "Intuitive dashboards" and "Cross-platform compatibility".
    *   **Lesson**: Even simple tools avoid modal dialogs for core properties. They use **context-sensitive sidebars** that change based on the selected tool or object.
    *   **UI Pattern**: **Direct SVG editing** (SVG-Edit) is a niche but powerful feature for technical users, aligning well with GCodeKit4's target audience of makers/engineers.

**Synthesis for GCodeKit4:**
Across all 10 top tools, **none** rely on modal dialogs for basic shape manipulation. The industry consensus is clear: **Canvas + Contextual Sidebar** is the universal interaction model.

| Feature | GCodeKit4 Designer | Industry Standard (Figma/Illustrator/Inkscape) | Gap Analysis |
| :--- | :--- | :--- | :--- |
| **Properties Panel** | **Modal Dialog**: Blocks interaction with canvas while editing. | **Persistent Sidebar**: Updates in real-time as objects are selected. | **Critical**: Modals break flow. Users expect to tweak values and see results immediately without closing a window. |
| **Canvas Pan** | **Left-Drag on Empty Space**: Conflicts with standard selection behavior. | **Space + Drag** or **Middle Mouse**: Left-drag is reserved for marquee selection. | **High**: Non-standard panning confuses users trying to select multiple objects. |
| **Marquee Selection** | **Shift + Drag**: Requires modifier key. | **Left-Drag**: Default behavior on empty space. | **High**: Inverts standard muscle memory. |
| **Zoom** | **Buttons / Keys**: `+` / `-`. | **Scroll Wheel** (often with Ctrl/Alt/Cmd). | **Medium**: Scroll zoom is faster and more intuitive. |
| **Layers/Objects** | **None**: No visibility of shape order or grouping structure. | **Tree View**: Essential for managing complex designs. | **Medium**: Hard to select objects buried under others. |
| **Tool Options** | **Hidden/Mixed**: Some in sidebar, some in dialogs. | **Contextual Bar**: Top bar changes based on active tool. | **Low**: Current sidebar is functional but could be cleaner. |

## 4. Identified Pain Points

1.  **Modal Interruption**: Editing a shape's size or position requires opening a dialog, changing values, and clicking "Save". This prevents "tweaking" – making small adjustments and visually verifying them.
2.  **Navigation Confusion**: Users accustomed to other tools will likely try to drag-select and instead pan the canvas, or try to pan with Spacebar and fail.
3.  **Lack of Hierarchy**: Without a layers panel, managing groups or overlapping shapes is difficult.
4.  **Visual Feedback**: While handles exist, the lack of a persistent property inspector makes the state of the selected object less obvious.

## 5. Recommendations for Improvement

### Phase 1: Interaction & Navigation (High Impact, Low Effort)
*   **Swap Pan & Select**: 
    *   Make **Left-Drag on empty space** trigger **Marquee Selection**.
    *   Make **Middle-Mouse Drag** or **Space + Left-Drag** trigger **Pan**.
*   **Implement Scroll Zoom**: Enable zooming via mouse wheel (with Ctrl/Cmd modifier if needed).

### Phase 2: Persistent Properties Panel (High Impact, High Effort)
*   **Remove Modal Dialog**: Deprecate `ShapePropertiesDialog`.
*   **Create Right Sidebar**: Introduce a persistent panel on the right side of the screen.
    *   **Top Section**: Transform (X, Y, W, H, Rotation).
    *   **Middle Section**: Shape-specific props (Radius, Text content).
    *   **Bottom Section**: CAM/Toolpath properties (Pocket, Depth, Strategy).
*   **Real-time Updates**: Binding input fields directly to the selected shape's state (with debounce) or using "Apply" logic without closing the panel.

### Phase 3: Object Management (Medium Impact, Medium Effort)
*   **Layers/Objects Panel**: Add a tab in the Left or Right sidebar to list all shapes.
    *   Allow reordering (Z-index).
    *   Allow renaming shapes.
    *   Show hierarchy for Groups.

### Phase 4: Visual Polish
*   **Icons**: Replace text-based icons ("➜", "▭") with SVG icons for a professional look.
*   **Cursors**: Ensure cursor changes (move, resize, text) are consistent and responsive.
*   **Snapping**: Enhance snapping visual feedback (already partially implemented, but visual guides help).

## 6. Conclusion

The GCodeKit4 Designer is functionally capable but suffers from "programmer UI" decisions (modals for properties, non-standard inputs). By adopting the **Properties Panel** pattern and standardizing **Canvas Navigation**, the tool will feel significantly more professional and intuitive to designers and engineers already familiar with CAD/Vector software.
