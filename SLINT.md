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

**Future Fix**: Would require either:
1. Using a custom component that wraps and recreates the view
2. Upgrading Slint to support element recreation in conditionals
3. Implementing from-Rust focus callbacks
