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
