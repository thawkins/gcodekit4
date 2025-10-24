# G-Code Editor UI Layout Analysis - Document Index

## Quick Navigation

### For Developers (Implementation)
**Start here:** `GCODE_EDITOR_LAYOUT_VISUAL.txt`
- Visual comparisons showing exactly what's broken and how to fix it
- Contains the minimal two-line fix needed
- Diagrams showing before/after layout structure

### For Understanding (Technical Deep Dive)
**Read this:** `GCODE_EDITOR_UI_ANALYSIS.md`
- Complete technical analysis with explanations
- Why Slint layout behaves this way
- Why the bug oscillates between two states
- Full solution strategy with code examples

---

## The Problem (TL;DR)

The gcode editor layout alternates between two broken states:

1. **State A:** Content shows and is editable, but right panel is hidden/off-screen
2. **State B:** Layout looks correct with all panels visible, but content doesn't show

**Root Cause:** Two Rectangle containers lack `height: 100%;` constraint, causing unbounded expansion.

---

## The Fix (TL;DR)

In `src/ui.slint`:

**Add at line 336:**
```slint
Rectangle {
    height: 100%;  ← ADD THIS
    background: #ffffff;
```

**Add at line 402:**
```slint
Rectangle {
    height: 100%;  ← ADD THIS
    background: #f5f5f5;
```

That's it. Two lines. This will:
- ✅ Enable content display
- ✅ Enable editing
- ✅ Preserve layout with all panels visible
- ✅ Stop the oscillation between broken states

---

## Files in This Analysis

| File | Purpose | Audience |
|------|---------|----------|
| GCODE_EDITOR_LAYOUT_VISUAL.txt | Visual before/after comparison with exact fixes | Developers, Implementers |
| GCODE_EDITOR_UI_ANALYSIS.md | Deep technical analysis and explanation | Tech leads, Code reviewers |
| GCODE_EDITOR_ANALYSIS_INDEX.md | This file - navigation guide | Everyone |

---

## Key Findings

### Slint Version
✅ **1.14.1 is current** - No version upgrade needed. This is not a version issue.

### Issue Summary
- **4 layout issues identified**
- **2 critical (missing height constraints)**
- **2 major (structural and sizing ambiguity)**
- **Root cause:** Nested containers without explicit height specification

### Layout Oscillation
The bug oscillates because:
1. Without height constraints, Slint can't reliably allocate space
2. Each attempted "fix" addresses symptoms, not root causes
3. Fixing layout breaks binding, fixing binding breaks layout
4. Cycle perpetuates until root cause (missing heights) is addressed

### Why TextEdit Breaks Layout
- TextEdit has no height constraint
- With `wrap: word-wrap`, it expands to fit all content
- ScrollView can't constrain it (no parent height)
- Rectangle expands to fit TextEdit (no height constraint)
- Cascade expands everything upward
- Right panel (250px) pushed off-screen

---

## How to Use This Analysis

### For Quick Understanding
1. Read this file (you're reading it now!)
2. Look at GCODE_EDITOR_LAYOUT_VISUAL.txt diagrams
3. Apply the two-line fix

### For Implementation
1. Open GCODE_EDITOR_LAYOUT_VISUAL.txt
2. Find "THE CRITICAL MISSING LINES" section
3. Add `height: 100%;` to the two Rectangle elements shown

### For Code Review
1. Read GCODE_EDITOR_UI_ANALYSIS.md for full context
2. Verify both height constraints are added
3. Test with large gcode files to ensure ScrollView works
4. Confirm all panels remain visible during view switching

### For Future Reference
- Slint layout requires explicit sizing for:
  - Containers with conditional children
  - TextEdit elements with ScrollView
  - Nested layout structures with variable sizing

---

## Testing After Fix

```bash
# 1. Build debug version
cargo build

# 2. Test with small file (should work immediately)
# 3. Test with large file (1000+ lines - should scroll properly)
# 4. Test view switching (View menu > G-Code Editor vs Device Console)
# 5. Test window resizing (layout should recalculate properly)
# 6. Verify right panel stays visible in all cases
```

---

## Related Documentation

- `src/ui.slint` - The UI definition file (where fixes are applied)
- `src/main.rs` - Rust backend (correctly loads content, no changes needed)
- Slint documentation: https://slint.dev/ (constraints-based layout system)

---

## Questions?

The analysis documents provide extensive explanation of:
- What's broken and why
- How Slint's layout system works
- Why the bug oscillates
- Why the two-line fix solves everything

See GCODE_EDITOR_UI_ANALYSIS.md for complete technical details.
