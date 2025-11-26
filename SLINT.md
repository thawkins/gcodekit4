## Slint UI Research and Insights

### Visualizer Analytic Bounds (2025-11-23)
- Files: `crates/gcodekit4-visualizer/src/visualizer/toolpath_rendering.rs`
- Added analytic bounding boxes for lines + arcs (cardinal checks) so zoom/fit never traverse discretized segments.
- `Toolpath::get_bounding_box` now aggregates per-segment bounds, drastically reducing Slint layout work for big files.

### Visualizer Arc Geometry Cache (2025-11-23)
- Files: `crates/gcodekit4-visualizer/src/visualizer/toolpath_rendering.rs`
- Added `ArcAngles` cache so Slint doesn't recompute atan2/TAU math during every paint.
- Arc iterators, interpolation, and length queries now reuse the cached span for consistent performance.

### Visualizer ArcLineIterator (2025-11-23)
- Files: `crates/gcodekit4-visualizer/src/visualizer/toolpath_rendering.rs`
- Added `ArcLineIterator` to lazily emit arc polylines so Slint avoids per-frame Vec allocations.
- `visit_line_segments` and `PathSegment::as_line_segments` now reuse the iterator, keeping render + metrics streaming-only.

### Visualizer Segment Visitor (2025-11-23)
- Files: `crates/gcodekit4-visualizer/src/visualizer/toolpath_rendering.rs`
- Added `visit_line_segments` so Slint canvases/metrics can stream discretized moves without allocating huge vectors.
- Bounding box + statistics now use the visitor, eliminating multi-MB clones when large G-code files load.

### Visualizer MovementMeta (2025-11-23)
- Files: `crates/gcodekit4-visualizer/src/visualizer/toolpath_rendering.rs`
- Added `MovementMeta` so line and arc segments share feed-rate + movement type data instead of duplicating fields.
- Arc discretization now simply clones metadata when producing `LineSegment`s, keeping direction + feed propagation consistent.

### Visualizer PathSegment Enum (2025-11-23)
- Files: `crates/gcodekit4-visualizer/src/visualizer/toolpath_rendering.rs`, `mod.rs`
- Consolidated the separate line/arc vectors into a single `Vec<PathSegment>` so UI layers traverse one sequence when generating stats or bounding boxes.
- Added helper methods on `PathSegment` to expose start/end, length, movement type, and discretized line approximations for arc rendering.

### Visualizer Toolpath Cache (2025-11-23)
- Files: `crates/gcodekit4-visualizer/src/visualizer/toolpath_cache.rs`, `visualizer_2d.rs`, and `canvas_renderer.rs`
- Added `ToolpathCache` to own the G-code hash, parsed command list, and SVG regeneration so we only rebuild paths when content changes.
- `render_toolpath_to_path`/`render_rapid_moves_to_path` now pull from the cache helpers instead of cloning struct fields.
- `Visualizer2D` exposes `toolpath_svg()`, `rapid_svg()`, and `commands()` to keep renderers from poking at internal buffers.

### Visualizer Viewport Transform (2025-11-23)
- Files: `crates/gcodekit4-visualizer/src/visualizer/viewport.rs`, `visualizer_2d.rs`, and `canvas_renderer.rs`
- Introduced a shared `ViewportTransform` struct so viewbox math (zoom, pan, padding) is consistent for toolpaths, grids, and origin markers.
- The parser's `Bounds` accumulator now lives alongside the viewport helper, ensuring we only pad/clip in one place before generating SVG output.
- `Visualizer2D::set_default_view` and `get_viewbox` delegate to the helper, reducing duplicated algebra and making future scaling tweaks safer.

### Designer Bulk Properties (2025-11-23)
- Files: `crates/gcodekit4-designer/ui/designer.slint`, `src/main.rs`, `crates/gcodekit4-designer/src/designer_state.rs`
- The Properties dialog now tracks whether multiple shapes are selected, swaps the header text to "Multiple Shapes", hides X/Y controls, and only emits update callbacks for fields the user actually changed.
- Rust side buffers pending values plus dirty flags so `designer_save_shape_properties` can selectively update each selected object without touching untouched attributes.
- `DesignerState::set_selected_position_and_size_with_flags` (new helper) lets us skip per-shape translations for multi-update scenarios while still resizing when requested.
- Step/pocket/text setters now iterate over every selected `DrawingObject`, marking the design dirty only when something actually changes.

### Designer Alignment Cascade (2025-11-23)
- Files: `crates/gcodekit4-ui/ui.slint`, `crates/gcodekit4-designer/ui/designer.slint`
- The context menu now exposes nested `Align` entries that fan out to Horizontal (Left/Center/Right) and Vertical (Top/Center/Bottom) submenus.
- Each submenu TouchArea calls dedicated callbacks (`align_horizontal_left`, `align_vertical_top`, etc.) that bubble up through `DesignerPanel` into `MainWindow`.
- Backend alignment helpers in `Canvas` translate each selected shape so Left/Right pins use bounding-box min/max X, while Top/Bottom snap to max/min Y in screen coordinates.
- `selected_count` drives the `align-enabled` property, so the menus only activate when at least two shapes are selected, preventing no-op commands.

### CustomTextEdit Component
- Located in: `crates/gcodekit4-ui/ui/ui_components/custom_text_edit.slint`
- Provides advanced text editing with:
  - Blinking cursor that is always visible
  - Virtual scrolling for large G-code files
  - Undo/redo support
  - Syntax highlighting
  - Line number support
  - Cursor positioned at (1,1) by default

### Empty Editor Cursor Rendering (✅ FIXED)

**Issue**: Cursor was not visible when editor had no content

**Root Causes**:
1. Backend cursor position initialized to (0, 0) in main.rs instead of (1, 1)
2. No visible content line when editor empty - Slint needs at least one line to render cursor
3. Cursor line/column defaults were 0-indexed but Slint UI expects 1-indexed

**Final Working Solution**:

**Backend Changes**:
1. `src/main.rs` line 613-614: Changed `set_cursor_line(0)` and `set_cursor_column(0)` to `set_cursor_line(1)` and `set_cursor_column(1)`
2. `crates/gcodekit4-ui/src/editor/mod.rs` in `get_visible_lines()`: When buffer is empty, push one line with space: `lines.push(" ".to_string())`

**Why This Works**:
- Cursor is now initialized to 1-indexed position (1,1) matching Slint UI expectations
- Backend always provides at least one line (with space) for Slint to render
- Cursor renders normally on this line at position (1,1)
- Space character is invisible but provides structure - when user types, space is replaced with content

**Result**: ✅ Cursor now displays at (1,1) on empty editor and blinks normally when content is added

### Focus Management Limitation

**Issue**: Focus is set correctly when first switching to gcode-editor view, but doesn't re-focus when switching back from another view.

**Root Cause**: Slint's `if current-view == "gcode-editor"` conditional shows/hides the Rectangle rather than recreating it. The `init` callback only runs once at creation.

**Current Implementation**:
- GcodeEditorPanel uses `forward-focus: editor-focus` to forward focus to the FocusScope
- The timer in the Rectangle's `init` callback ensures focus is set on first view
- Works correctly on initial app startup and first view switch

**Limitation**: When switching back to gcode-editor from another view, the `init` callback doesn't re-run because the Rectangle already exists (just hidden).

**Workaround for users**: Click once in the editor to focus it, then keyboard works normally.

## SVG Path Rendering Insights

### Multi-part SVG Path Handling (✅ FIXED 2025-11-18)

**Issue**: Long straight line segments (18mm+) appeared in gcode where SVG has curves

**Root Cause**: SVG paths can contain multiple disconnected sub-paths separated by `z` (close path) and `m` (move) commands. Example from tigershead.svg path 8:
```
m 7365,14183 c ... l ... c ... z m 670,-1853 c ... z
```

The parser treated all sub-path points as ONE continuous path, so the gap between the end of first sub-path and start of second sub-path was rendered as a single cutting line (G1 command), appearing as an 18mm straight line artifact.

**Solution**: Added discontinuity detection in gcode generation:
- When a point is >5mm from the previous point, it's treated as a path break
- Sequence becomes: M5 (laser off) → G0 (rapid move) → M3 (laser re-engage)
- This properly separates disconnected sub-paths without creating unwanted cutting lines

**Result**: ✅ Longest cutting segment reduced from 18mm to 2.5mm (normal curve approximation)

### SVG Command Implicit Repetition (✅ FIXED 2025-11-18)

**Issue**: SVG line commands with multiple coordinate pairs were partially parsed

**Details**: SVG spec allows implicit repetition in path commands:
```
l -50,50 -100,20 c 5,5 10,10 15,15
```
Should be interpreted as:
- `l -50,50` (relative line to)
- `l -100,20` (relative line to)
- `c 5,5 10,10 15,15` (cubic curve)

**Root Cause**: The L/l command handler only processed the first coordinate pair and skipped remaining ones

**Solution**: Modified parse_path_data() to loop through consecutive coordinate pairs, similar to how the C/c handler was already implemented:
```rust
let mut j = i + 1;
while j + 1 < commands.len() {
    // Process (x, y) pair
    j += 2;
    // Check if next token is another command or more line data
    if is_command(commands[j]) { break; }
}
i = j;
```

**Result**: ✅ All line segments now properly parsed according to SVG specification

**Future Fix**: Would require either:
1. Using a custom component that wraps and recreates the view
2. Upgrading Slint to support element recreation in conditionals
3. Implementing from-Rust focus callbacks

### Tooltip Implementation

**Issue**: Slint does not have a native Tooltip component yet.

**Solution**: Implemented custom tooltips using `TouchArea` and conditional rendering.

**Implementation Details**:
- Wrap the target element in a `TouchArea` (or use an existing one).
- Add a `Rectangle` that is conditionally rendered: `if (touch-area.has-hover) : Rectangle { ... }`.
- Position the tooltip rectangle relative to the parent (e.g., `y: parent.height + 2px`).
- Set `z: 100` to ensure it renders on top of other elements.
- Style with a background color, border, and text.

**Example**:
```slint
Rectangle {
    // Button content...
    touch-area := TouchArea { ... }
    
    if (touch-area.has-hover) : Rectangle {
        y: parent.height + 2px;
        z: 100;
        // Tooltip styling...
        Text { text: "Tooltip Text"; }
    }
}
```

**Usage**: Used for the VCR-style "Send", "Pause", and "Stop" buttons in the G-code editor panel to show text labels on hover.

### Toolpath Rendering Stroke Width (✅ FIXED 2025-11-20)

**Issue**: Toolpaths were rendered with a thick line (5px) that looked too heavy, especially when zoomed out or on high-DPI displays.

**Requirement**: Render toolpaths with a single-pixel wide line regardless of the scale factor.

**Solution**:
- Updated `stroke-width` property in Slint `Path` elements for the toolpath layer.
- Changed from `5px` to `1px`.
- Since the backend generates path coordinates in screen pixels (pre-scaled) and the Slint `Path` viewbox is set to 1:1 with the canvas pixels, `stroke-width: 1px` results in a crisp 1-pixel wide line on screen, independent of the zoom level applied in the backend.

**Files Updated**:
- `crates/gcodekit4-visualizer/ui/gcode_visualizer.slint`
- (Duplicate files removed in cleanup)

### Speeds and Feeds Calculator (Added 2025-11-22)
- **Backend**: `crates/gcodekit4-camtools/src/speeds_feeds.rs`
- **UI Component**: `SpeedsFeedsDialog` in `ui.slint`
- **Functionality**: Calculates RPM and Feed Rate based on Material, Tool, and Device.
- **Integration**:
  - Integrated as a card in the "CAM Tools" tab.
  - Uses `ComboBox` for selection of Material, Tool, and Device.
  - Displays calculated values dynamically.
  - **Clamping Display**: If calculated values exceed device limits, the clamped value is shown, followed by the original calculated value in red brackets (e.g., `10000 (12500)`).
  - **Layout**: Centered dialog within the CAM Tools panel using `Rectangle` wrapper for styling.

### Designer Shape Properties (November 2025)
- **Dynamic Dialog Layout**:
  - Shape Properties Dialog height adjusts dynamically based on enabled features (e.g., Pocketing).
  - Prevents empty space and ensures controls are accessible without excessive scrolling.
- **Pocketing Controls**:
  - **Strategy Selection**: `ComboBox` for selecting Raster, Contour, or Adaptive strategies.
  - **Conditional Visibility**: Raster Angle and Bidirectional controls only appear when Pocketing is enabled.
  - **Data Binding**: Controls bound to `DesignerShape` properties (`pocket_strategy`, `raster_angle`, `bidirectional`).
- **Integration**:
  - Updates are sent to backend via `update_shape_property` callbacks.
  - Backend state (`DesignerState`) is updated to reflect UI changes.

## UI Patterns (November 2025)

### Layout Constraints (2025-11-24)
- **Issue**: `VerticalBox` and `HorizontalBox` (from `std-widgets`) can cause unexpected layout expansion because they have internal padding and layout logic that may override parent constraints.
- **Solution**: Use `VerticalLayout` and `HorizontalLayout` for strict layout control when precise sizing is needed.
- **Sizing**: To prevent a sidebar from expanding when the window is resized, use `min-width` and `max-width` set to the same value (e.g., `min-width: 210px; max-width: 210px;`). Avoid using `width` alone if it conflicts with layout stretch properties.
- **Borders**: `VerticalBox` and `HorizontalBox` do not support `border-width` or `border-color`. Wrap them in a `Rectangle` if borders are needed.

### Scrollable Tabs
- Use `Flickable` to wrap content within a `TabWidget` tab to ensure content is accessible on smaller screens.
- Example: `Tab { title: "General"; Flickable { ... } }`

### Data Models
- Use `struct` definitions in Slint to map complex Rust data structures (e.g., `DeviceProfileUiModel`).
- Pass these structures via callbacks or properties to keep the UI declarative.

### Dynamic Layouts
- Use `if` conditions to show/hide UI elements based on selected options (e.g., showing "Dimple Diameter" only when "Dimple" is enabled).
- Use `ComboBox` for enumerated types (e.g., `KeyDividerType`, `BoxType`).

### Designer Grid and View Controls (Added 2025-11-21)

**Requirement**: Render a dynamic grid and origin indicator in the Designer view, similar to the Visualizer, and provide view controls (Zoom/Pan/Fit).

**Implementation**:
- **Grid Rendering**: Implemented in Rust backend (`svg_renderer.rs`) generating SVG path data strings.
  - Grid lines are generated based on viewport bounds and zoom level.
  - Path data is passed to Slint `Path` element via `canvas_grid_data` property.
  - **Fix**: Ensure canvas dimensions are synced from UI to backend to cover full width/height.
- **Origin Indicator**: Rendered as a crosshair at (0,0) using SVG paths.
- **View Controls**:
  - **Zoom/Pan**: Managed by `Viewport` struct in Rust.
  - **Fit**: Calculates bounding box of all shapes and adjusts zoom/pan to fit with padding.
  - **Reset**: Resets zoom to 1.0 and pan to default origin.
- **Default View**: Origin positioned at bottom-left with 5px margin (inset) to ensure visibility.
- **UI Controls**: Added buttons (+, -, Fit, Rst) and "Show Grid" checkbox to the right sidebar for better accessibility.

### Device Manager UI Improvements (Added 2025-11-22)
- **Explicit Labels**: Added "Min:" and "Max:" labels to axis limit fields in `DeviceManagerPanel`.
  - Previously used placeholder text which disappeared when values were present, leading to confusion.
  - Explicit labels ensure users know which field is which at all times.
- **Auto-Correction**: Implemented logic in `DeviceUiController` to automatically swap Min/Max values if entered inversely (Min > Max).
  - Prevents invalid device profiles that could cause negative dimensions in CAM tools.

### Visualizer Performance Optimization (Added 2025-11-22)
- **Issue**: Visualizer performance was poor during zoom/pan operations due to redundant G-code parsing and inefficient string generation.
- **Optimization**:
  - **Content Hashing**: Implemented content hashing in `Visualizer2D` to skip re-parsing when G-code content hasn't changed.
  - **Shared State**: Used `Arc<Mutex<Visualizer2D>>` to share the visualizer instance across UI callbacks, persisting parsed state.
  - **String Optimization**: Optimized `render_grid_to_path` and `render_origin_to_path` to use `String::with_capacity` and `std::fmt::Write` for efficient string building.
  - **Reduced Precision**: Reduced SVG path coordinate precision from 3 to 2 decimal places to reduce data size and formatting overhead.
- **Result**: Significantly smoother zoom and pan operations.

### Confirmation Dialog Pattern (Added 2025-11-22)
- **Requirement**: Prompt user for confirmation before destructive actions (e.g., deleting multiple shapes).
- **Implementation**:
  - **Dialog Component**: Created `DeleteConfirmationDialog` inheriting from `Dialog`.
  - **State Management**: Added `delete_confirmation_visible` and `delete_count` properties to `DesignerPanel`.
  - **Callbacks**:
    - `show_delete_confirmation(int)`: Called from Rust to show the dialog.
    - `confirm_delete()`: Called from Slint to Rust when user confirms.
  - **Logic**:
    - Rust checks condition (e.g., `selected_count > 1`).
    - If condition met, invokes `show_delete_confirmation`.
    - User clicks "Continue" -> `confirm_delete` callback -> Rust performs action.
    - If condition not met (single selection), Rust performs action immediately.

### CustomTextEdit Simplification (2025-11-23)
- **Files**: `crates/gcodekit4-ui/ui/ui_components/custom_text_edit.slint`
- **Cleanup**: Removed unused `TextInput` overlay that was dead code.
- **Bug Fix**: Fixed hardcoded `8px` character width in mouse click calculation to use calculated `root.char-width`.
- **Duplication**: Removed unused duplicate file `crates/gcodekit4-gcodeeditor/ui/custom_text_edit.slint`.

### Designer Grouping Functionality (2025-11-23)
- **Files**: `crates/gcodekit4-designer/src/canvas.rs`, `svg_renderer.rs`, `ui/designer.slint`, `src/main.rs`
- **Data Structure**: Added `group_id: Option<u64>` to `DrawingObject` to link shapes together.
- **Selection Logic**: Updated `select_at` to automatically select all members of a group when any member is clicked.
- **Rendering**:
  - Added `canvas_grouped_shapes_data` property and layer to `DesignerPanel`.
  - Grouped shapes are rendered in **green** (`#2ecc71`) to distinguish them from normal (blue) and selected (yellow) shapes.
  - Selected grouped shapes retain their green color but show yellow selection handles.
  - Added a dotted green bounding box around selected groups to clearly indicate the group extent.
  - Selection handles are now drawn for the *union* bounding box of all selected shapes (or the group), rather than individual shapes, improving the resizing experience.
- **Resizing**:
  - Refactored `resize_selected` to calculate the bounding box of *all* selected shapes (the group).
  - Implemented `scale` method on `Shape` trait to support group resizing by scaling relative to the group center.
  - This ensures groups resize as a single unit, maintaining relative positions and proportions.
- **UI**: Added "Grp" (Group) and "UGrp" (Ungroup) buttons to the designer toolbar.

### Designer Copy/Paste (2025-11-23)
- **Files**: `crates/gcodekit4-designer/src/designer_state.rs`, `canvas.rs`, `ui/designer.slint`, `src/main.rs`
- **Clipboard**: Added `clipboard: Vec<DrawingObject>` to `DesignerState` to store copied shapes.
- **Copy Logic**: `copy_selected` clones selected shapes into the clipboard.
- **Paste Logic**: `paste_at_location` clones shapes from clipboard, translates them to the target location (centering on the click point), and assigns new IDs.
- **Context Menu**:
  - Updated `ContextMenu` in `designer.slint` to include "Copy" and "Paste" items.
  - Added `has-selection` property to conditionally show Copy/Delete/Properties items.
  - "Paste" is always shown.
  - Right-clicking on empty space shows the context menu with only "Paste" enabled.
  - Right-clicking on a shape shows the full menu.
- **Integration**: Wired up callbacks through `DesignerPanel` and `MainWindow` to the backend.

### Designer Undo/Redo (2025-11-23)
- **Files**: `crates/gcodekit4-designer/src/designer_state.rs`, `canvas.rs`, `ui/designer.slint`, `src/main.rs`
- **State Management**: Implemented `CanvasSnapshot` to clone the entire canvas state (`shapes`, `next_id`, `spatial_index`).
- **History Stack**: Added `undo_stack` and `redo_stack` to `DesignerState`.
- **Integration**:
  - `save_history()` is called before any mutating operation in `DesignerState`.
  - For continuous operations (drag/resize), `designer-interaction-start` callback is triggered on `PointerEventKind.down` to save state once at the beginning.
  - `undo()` and `redo()` restore the snapshot and update the UI.
- **UI**:
  - Added Undo/Redo buttons to the toolbar, enabled/disabled based on stack state.
  - Added keyboard shortcuts (Ctrl+Z, Ctrl+Shift+Z, Ctrl+Y) in `DesignerPanel` FocusScope.
  - `DesignerState` struct in Slint updated to include `can_undo` and `can_redo` flags.

### Designer Context Menu Grouping (2025-11-23)
- **Files**: `crates/gcodekit4-designer/ui/designer.slint`, `src/main.rs`, `crates/gcodekit4-designer/src/designer_state.rs`
- **UI**: Removed "Grp" and "UGrp" buttons from the toolbar.
- **Context Menu**: Added "Group" and "Ungroup" items to the right-click context menu.
- **Logic**:
  - `can_group`: Active if `selected_count >= 2` AND at least one selected item is not already in a group.
  - `can_ungroup`: Active if any selected item is part of a group.
  - State is calculated in `DesignerState` and passed to UI via `can_group` and `can_ungroup` flags.

### Main Menu Refactoring (2025-11-23)
- **Files**: `crates/gcodekit4-ui/ui.slint`, `crates/gcodekit4-ui/ui/ui_components/mainmenu.slint` (deleted)
- **Consolidation**: Removed the separate `MainMenu` component definition which was a duplicate.
- **Integration**: Replaced the custom `MainMenu` component usage in `ui.slint` with the standard `MenuBar` component, ensuring all menu items (Machine Control, Machine Info, CAMTools, CNCTools) were correctly migrated.
- **Context Awareness**:
  - **File Menu**: Added a dynamic label at the top ("GCode File" vs "Design File") based on the active tab.
  - **New/Open/Save**: These actions now contextually switch between G-Code Editor operations and Designer operations depending on the active view.
  - **Edit Menu**: Added "Undo" and "Redo" items that also switch context between global undo (not yet implemented) and Designer undo/redo.
- **Safety**: Added confirmation dialogs to "New" actions in both G-Code Editor and Designer to prevent accidental data loss if content exists.

### Materials Manager UI Polish (2025-11-23)
- **Files**: `crates/gcodekit4-ui/ui_panels/materials_manager.slint`
- **Standardization**: Enforced a consistent `32px` height for all input controls (LineEdit, ComboBox, Button, SpinBox) to match the rest of the application.
- **Layout**: Fixed vertical spacing issues by using `alignment: start` in `VerticalBox` containers, ensuring controls pack neatly at the top rather than spreading out.
- **Structure**: Used `vertical-stretch: 0` on search/filter panels to prevent them from expanding unnecessarily.



## Layout Insights (2025-11-24)
- **HorizontalBox vs HorizontalLayout**: `HorizontalBox` adds implicit padding and spacing which can cause unexpected layout expansion, especially when nested or when children have flexible widths. Switching to `HorizontalLayout` provides more precise control.
- **Sidebar Expansion**: To prevent a sidebar from expanding to fill the screen, use `width` or `max-width` constraints. Avoid `width: 100%` on children if the parent's width is not strictly constrained.
- **FocusScope**: Ensure `FocusScope` is properly closed and doesn't accidentally wrap unintended elements.

### Designer Rubber Band Selection (2025-11-24)
- **Files**: `crates/gcodekit4-designer/src/canvas.rs`, `crates/gcodekit4-designer/src/designer_state.rs`, `crates/gcodekit4-designer/ui/designer.slint`, `src/main.rs`
- **Requirement**: Enable selecting multiple shapes by dragging a selection rectangle on empty space.
- **Implementation**:
  - **Canvas Logic**: Added `select_in_rect` method to `Canvas` and `DesignerState` which uses the spatial index to efficiently find shapes intersecting the selection bounds.
  - **UI Logic**:
    - Modified `TouchArea` in `designer.slint` to detect drags on empty space (when no handle is active and no shape is selected).
    - Added visual feedback: A semi-transparent yellow rectangle is drawn during the drag operation.
    - On mouse release, the selection rectangle coordinates are transformed from screen pixels to world coordinates (mm) and passed to the backend.
  - **Coordinate Transformation**:
    - Screen pixels are converted to world coordinates using the formula: `world = (screen / zoom) + pan_offset`.
    - This ensures the selection works correctly regardless of zoom level or pan position.
  - **Integration**: Added `designer-select-in-rect` callback chain from `DesignerPanel` to `MainWindow` to `main.rs`.

### Rubber Band Selection Fixes (2025-11-24)
- **Issue**: Rubber band rectangle persisted after release, and selection didn't work due to incorrect coordinate transformation.
- **Fixes**:
  - **Visual Glitch**: Moved the "release" logic from the `moved` callback (which requires mouse movement) to the `pointer-event` callback with `PointerEventKind.up`. This ensures the rubber band state is reset immediately upon mouse release.
  - **Coordinate Transformation**: Changed the `select_in_rect` callback to pass raw pixel coordinates (start and end points) to the backend instead of pre-calculating world coordinates in Slint.
  - **Backend Logic**: Updated `main.rs` to use `state.canvas.pixel_to_world` to transform the pixel coordinates. This correctly handles zoom, pan, and the Y-axis flip (screen Y down vs world Y up), ensuring the selection rectangle in world space matches what the user drew on screen.

### Designer Interaction Refinement (2025-11-24)
- **Requirement**: Change drag behavior on empty space:
  - **Default (No Shift)**: Rubber band selection.
  - **Shift Held**: Pan the canvas.
- **Implementation**:
  - **Shape Detection**: Updated `on_designer_detect_handle` in `main.rs` to return handle index `5` when the cursor is over a selected shape's body (using `contains_point`). This distinguishes "clicking on shape" from "clicking on background".
  - **UI Logic**: Updated `designer.slint`:
    - **Cursor**: Shows "move" cursor when hovering over shape body (handle 5).
    - **Drag Start**:
      - If handle is 4 (center) or 5 (body) -> Move shape.
      - If handle is -1 (background) -> Check Shift key.
        - If Shift is UP -> Start rubber band.
        - If Shift is DOWN -> Pan canvas.
  - **Backend**: Updated `on_designer_handle_drag` to accept handle index 5 as a valid move operation.
- **Result**: Users can now rubber band select even when a shape is selected (by clicking background), and pan requires Shift, matching standard design tool conventions.

### Rubber Band Group Selection (2025-11-24)
- **Requirement**: When rubber band selection intersects any part of a group, the entire group should be selected.
- **Implementation**:
  - Modified `select_in_rect` in `crates/gcodekit4-designer/src/canvas.rs`.
  - It now collects `group_id`s of all shapes that intersect the selection rectangle.
  - After the initial intersection pass, it performs a second pass to select all shapes that belong to the collected `group_id`s.
  - This ensures that even if only one member of a group is touched by the rubber band, all other members are also selected.

### Group Dragging Fix (2025-11-24)
- **Issue**: Dragging a multiple selection (including groups) failed if the click was on a shape that wasn't the "primary" selected ID.
- **Fix**: Updated `on_designer_detect_handle` in `main.rs`.
  - Previously, it only checked if the click was on the body of the *primary* selected shape.
  - Now, if the click is not on a resize handle of the primary shape, it iterates through *all* selected shapes to check if the click is on any of their bodies.
  - This ensures that clicking and dragging any part of a multiple selection correctly initiates a move operation for the entire selection.

### Designer Interaction Update (2025-11-24)
- **Requirement**: Revert drag behavior on empty space to prioritize panning, making rubber band selection require Shift.
- **Implementation**:
  - **Default (No Shift)**: Pan the canvas (restored original behavior).
  - **Shift Held**: Start rubber band selection.
  - This change was made to align with user preference for panning as the primary interaction on empty space.

### Group Interaction Improvements (2025-11-25)
- **Group Selection**: Clicking anywhere inside a group's composite bounding box now selects the entire group, treating it as a solid object.
- **Group Dragging**: Dragging anywhere inside the composite bounding box of a selected group (or multiple selection) moves the entire selection.
- **Resize Handles**: Resize handles (corners and center) are now displayed around the composite bounding box of *any* selection (single shape, multiple shapes, or groups).
- **Group Resizing**: Fixed distortion issue when resizing complex groups (like polylines) by ensuring scaling is applied relative to the group's center, preserving relative positions and proportions.

### File Menu Workflow (2025-11-25)
- **Load vs Add**:
  - **Load**: Clears the canvas before importing (standard "Open" behavior).
  - **Add**: Appends imported shapes to the existing design without clearing.
- **Auto-Grouping**: "Add" operations automatically place imported shapes into a new group for easy manipulation.
- **Auto-Fit**: Both "Load" and "Add" operations automatically execute "Fit to View" to ensure the design is visible.

### Default Properties Management (2025-11-25)
- **Requirement**: Allow users to set default properties (pocket depth, step down, etc.) for new shapes.
- **Implementation**:
  - **Virtual Shape**: Added a `default_properties_shape` to `DesignerState` which is not rendered but holds the default values.
  - **UI**: Added "Set Defaults" button to the sidebar.
  - **Dialog Reuse**: Reused `ShapePropertiesDialog` with a new `is_editing_defaults` property.
  - **Conditional Visibility**: When `is_editing_defaults` is true, the dialog hides geometry controls (X, Y, W, H) and the "Use Custom Values" checkbox, showing only operation properties.
  - **Persistence**: The virtual shape is serialized with the design file, preserving defaults across sessions.
  - **Application**: `add_shape_at` applies these defaults to new shapes, but explicitly sets `use_custom_values` to `false` so they inherit future default changes unless overridden.

### View Management (2025-11-25)
- **Auto-Fit on Open**: Opening the Designer panel now automatically triggers "Fit to View".
- **Timing**: Used a `slint::Timer` with a 100ms delay to ensure the UI layout is settled and correct canvas dimensions are available before calculating the fit.

### Visualizer Auto-Fit (2025-11-25)
- **Requirement**: Automatically "Fit to View" when the Visualizer panel is displayed.
- **Challenge**: The `init` callback of a component runs before the layout is fully calculated, meaning `canvas-width` and `canvas-height` might be incorrect (e.g., 0 or default).
- **Solution**: Use a one-shot `Timer` in the `init` block.
  - `init => { fit-timer.running = true; }`
  - `fit-timer := Timer { interval: 50ms; running: false; triggered => { root.fit-to-view(...); self.running = false; } }`
- **Result**: The timer delay allows the layout engine to perform a pass, ensuring correct dimensions are available when `fit-to-view` is called.

### Visualizer UI Overhaul (2025-11-26)
- **Dark Theme Adoption**:
  - Migrated Visualizer from mixed light/dark theme to full dark theme (`#2c3e50` panels, `#34495e` canvas).
  - Ensures visual consistency with the Designer component.
- **Floating Overlays**:
  - Implemented floating "pill" overlays for status and zoom controls.
  - Uses `Rectangle` with `opacity: 0.9`, `border-radius: 15px`, and absolute positioning (`x`, `y`).
  - **Status Pill**: Bottom-left, auto-sizing based on content (`width: status-layout.preferred-width + 20px`).
  - **Zoom Pill**: Bottom-right, fixed width.
- **Custom Components**:
  - **VisualizerToolButton**: Icon-based button with hover state (`#34495e` vs `#2c3e50`) and pointer cursor.
  - **DarkCheckBox**: Custom checkbox matching Designer's style (transparent background, white checkmark).
- **Layout**:
  - Moved controls from top toolbar to a fixed-width (200px) left sidebar.
  - Used `VerticalLayout` for strict spacing control.
  - Added a 1px right border separator using a nested `Rectangle` (workaround for missing `border-right` property).

### Designer UI Polish (2025-11-26)
- **Icon Alignment**: Fixed icon alignment in `ToolButton` by removing the `VerticalBox` wrapper (which added unwanted padding) and manually centering the `Image` component using calculated coordinates.
- **Canvas Padding**: Removed padding around the canvas area by replacing `VerticalBox` containers with `VerticalLayout`. `VerticalBox` adds default padding, while `VerticalLayout` does not.
- **Panel Sizing**: Increased the Left Sidebar width from 210px to 250px to accommodate content better.
- **Control Standardization**:
  - Standardized font sizes in the Left Sidebar to match the Right Sidebar (removed explicit `11px` overrides).
  - Removed fixed height from `CompactSpinBox` in the Left Sidebar to match the Right Sidebar's natural sizing.
  - Updated `DarkCheckBox` to use default font size.
- **Zoom Limits**: Increased maximum zoom factor from 1000% (10.0) to 5000% (50.0) in `DesignerState` and `Viewport`.

### Layers Tab Improvements (2025-11-26)
- **Keyboard Navigation**:
  - Implemented Up/Down arrow key navigation for the shape list.
  - Used `FocusScope` to capture key events.
  - Added `select_next_shape` and `select_previous_shape` logic in `DesignerState` based on draw order.
  - Clicking a list item automatically grabs focus for the `FocusScope`, enabling immediate keyboard control.
- **Layout**:
  - Used `vertical-stretch: 1` on the shape list container to ensure it fills the available vertical space.
  - Added column headers for Type, Name, and Group.

### Designer Properties Panel Refactor (2025-11-25)
- **Persistent Panel**: Replaced the modal `ShapePropertiesDialog` with a persistent right-hand sidebar (270px fixed width).
- **Immediate Updates**: Removed the "Save" button and `pending_properties` buffer. Changes to properties (Position, Size, CAM settings) are now applied immediately to the selected shape(s) via `update_shape_property` callbacks.
- **Default Properties View**:
  - When no shape is selected, the panel displays "Design Properties".
  - Transform controls (X, Y, W, H) and "Use Custom Values" are hidden.
  - Users can edit default CAM settings (Pocket Depth, Step Down, etc.) which apply to new shapes.
- **Layout Fixes**:
  - Ensured correct nesting of the Right Sidebar within the main `HorizontalLayout` to prevent it from overlaying the canvas.
  - Fixed selection handle rendering symmetry by normalizing bounding box coordinates in `svg_renderer.rs`.
  - Rounded dimensional values in the UI to 2 decimal places for cleaner display.
