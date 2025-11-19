## Slint UI Research and Insights

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
- `crates/gcodekit4-gcodeeditor/ui/gcode_visualizer.slint`
- `crates/gcodekit4-ui/src/ui_panels/gcode_visualizer.slint`
- `crates/gcodekit4-ui/ui_panels/gcode_visualizer.slint`
