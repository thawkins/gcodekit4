# G-Code Editor UI Layout Issues - Technical Analysis

## Executive Summary

The g-code editor exhibits a layout problem that oscillates between two broken states:
- **State A:** Gcode content displays and can be edited, but layout is broken (central panel too large, right panel hidden/pushed off)
- **State B:** Layout appears correct with all panels visible, but gcode content doesn't show and editing is disabled

The root cause is **poor layout container sizing** in the Slint UI file combined with **conditional rendering** of the editor view. This analysis identifies the exact issues and their causes.

## Slint Version Status

✅ **Current Version: 1.14.1** - Using recent stable version, no upgrade needed.

The project is using the latest 1.14.x branch of Slint, so this is not a version-related issue.

---

## Critical Issues Found

### Issue 1: View Container Rectangle Missing Height Constraint (PRIMARY ISSUE)

**Location:** `src/ui.slint`, lines 336-453

**Code:**
```slint
// View Container - maintains consistent layout
Rectangle {
    background: #ffffff;
    
    if current-view == "gcode-editor" : VerticalBox {
        spacing: 0px;
        // ...TextEdit and toolbar...
    }
    
    if current-view == "device-console" : VerticalBox {
        // ...console content...
    }
}
```

**Problem:**
The Rectangle wrapper has **no explicit `height` property**. In Slint's layout engine:
- Rectangle without height defaults to sizing based on content
- When conditional `if` blocks change, the Rectangle's height calculation becomes unreliable
- TextEdit can expand to fit all content, causing the Rectangle to grow beyond viewport
- This pushes the right panel off-screen or hides it entirely

**Impact:**
- When gcode-editor view loads large files, TextEdit expands
- Rectangle expands with it
- Right panel (250px width) gets pushed outside the visible window
- Layout becomes broken despite all panels technically existing

---

### Issue 2: TextEdit Has No Height Constraint Inside ScrollView

**Location:** `src/ui.slint`, lines 407-415

**Code:**
```slint
Rectangle {
    background: #f5f5f5;
    border-width: 2px;
    border-color: #000000;
    
    ScrollView {
        TextEdit {
            enabled: true;
            text <=> root.gcode-content;
            read-only: false;
            wrap: word-wrap;
        }
    }
}
```

**Problem:**
The TextEdit element inside ScrollView has **no height constraint**. This means:
- TextEdit will expand to fit all its content (all lines of gcode)
- ScrollView becomes ineffective - there's nothing to scroll within
- The containing Rectangle (which has no height) expands to fit TextEdit
- This cascades up causing the entire center panel to expand

**Impact:**
- Large gcode files cause TextEdit to be extremely tall
- ScrollView doesn't function as intended
- Surrounding layout collapses

---

### Issue 3: Center Panel VerticalBox Missing Sizing Hints

**Location:** `src/ui.slint`, lines 317-454

**Code:**
```slint
HorizontalBox {
    spacing: 10px;
    padding: 10px;
    
    // Left panel - explicitly 300px wide ✓
    VerticalBox {
        width: 300px;
        // ...
    }
    
    // Center panel - NO explicit width or height constraints ✗
    VerticalBox {
        spacing: 0px;
        // ...no width, no height...
    }
    
    // Right panel - explicitly 250px wide ✓
    VerticalBox {
        width: 250px;
        // ...
    }
}
```

**Problem:**
The center VerticalBox is missing explicit sizing. In a HorizontalBox with fixed-width siblings:
- Left: 300px (fixed)
- Right: 250px (fixed)
- Center: NO explicit width (relies on remaining space)

Without explicit sizing properties, the center panel's behavior becomes:
- Implicit growth to fill remaining space is unreliable
- When inner content conditionally changes, recalculation is unpredictable
- The layout engine may not properly allocate space

**Impact:**
- Center panel sizing is ambiguous
- View switching causes layout recalculation issues
- Space distribution becomes inconsistent

---

### Issue 4: Problematic Structure: Rectangle Wrapper Around Conditional VerticalBox

**Location:** `src/ui.slint`, lines 336-453

**Structure:**
```
VerticalBox (center panel)
  ├─ Rectangle (title bar, 35px height)
  └─ Rectangle (NO HEIGHT) ← PROBLEM
      ├─ if view == "gcode-editor": VerticalBox
      │   ├─ Rectangle (toolbar, 80px)
      │   └─ Rectangle (NO HEIGHT) 
      │       └─ ScrollView
      │           └─ TextEdit (NO HEIGHT)
      └─ if view == "device-console": VerticalBox
          └─ ScrollView
              └─ Text
```

**Problem:**
Using Rectangle to wrap conditional blocks breaks Slint's layout assumptions:
- Rectangle is designed for visual presentation, not layout flexibility
- Conditional `if current-view == "gcode-editor"` inside Rectangle has no explicit constraints
- When the condition changes, the Rectangle doesn't properly recalculate
- Nested Rectangle without height (lines 402-415) further complicates layout

**Impact:**
- Layout becomes fragile and view-dependent
- Switching between gcode-editor and device-console causes reflow issues
- Conditional content doesn't resize properly

---

## Why the Layout Oscillates Between Broken States

### State A: Content Visible, Layout Broken
**Sequence:**
1. File is loaded, `gcode-content` is populated with large text
2. TextEdit expands to fit all content (no height constraint)
3. ScrollView inside Rectangle also expands (Rectangle has no height)
4. Center panel VerticalBox expands
5. Right panel gets pushed off viewport or compressed
6. **Result:** User sees broken layout but gcode is editable

### State B: Layout Correct, Content Missing
**Sequence:**
1. Someone attempts to "fix" the layout by restructuring
2. This breaks the content binding or view switching logic
3. When opening files, gcode doesn't populate the TextEdit
4. Or the view doesn't switch to "gcode-editor"
5. **Result:** Layout appears correct with all panels visible, but no content shows

### Why This Cycles

The two states exist because:
- **Temporary Fix Attempts:** Each attempted fix changes the structure without understanding root causes
- **Conflicting Constraints:** Fixing one issue (layout) breaks another (content binding)
- **No Single Root Cause Addressed:** The missing height constraints remain in all iterations

---

## Why This Happens in Slint

Slint uses a constraints-based layout system similar to modern UI frameworks:

1. **VerticalBox** has implicit behavior:
   - Children grow to fill available height
   - If no height constraint, it sizes to content

2. **Rectangle** is different:
   - It's a visual primitive, not a layout container
   - It doesn't have built-in child distribution
   - Without explicit height, it calculates height from content

3. **Conditional Blocks** (`if` statements):
   - Treated as dynamic children
   - When condition changes, layout must recalculate
   - Unpredictable if parent has no height constraint

4. **TextEdit** specifics:
   - Expands by default to fit content (especially with `wrap: word-wrap`)
   - Needs explicit height or parent height constraint to work with ScrollView
   - Without constraints, it defeats ScrollView's purpose

---

## Solution Strategy

### Primary Fix: Add Explicit Heights to Layout Containers

The central panel needs to explicitly distribute its height:

```slint
// Center panel with explicit height distribution
VerticalBox {
    spacing: 0px;
    
    // Title - fixed height
    Rectangle {
        height: 35px;
        // title content
    }
    
    // Content area - fill remaining space
    Rectangle {
        height: 100%;  // ← CRITICAL FIX
        
        if current-view == "gcode-editor" : VerticalBox {
            spacing: 0px;
            
            Rectangle {
                height: 80px;  // toolbar fixed height
                // toolbar
            }
            
            Rectangle {
                height: 100%;  // ← CRITICAL FIX: TextEdit area gets remaining space
                
                ScrollView {
                    TextEdit {
                        text <=> root.gcode-content;
                        // other properties
                    }
                }
            }
        }
        
        if current-view == "device-console" : VerticalBox {
            // console content
        }
    }
}
```

### Secondary Fixes

1. **Ensure center panel VerticalBox grows to fill horizontal space:**
   ```slint
   VerticalBox {
       // Remove width constraint or make it flexible
       // Let it grow to fill remaining space
   }
   ```

2. **Verify TextEdit can properly constrain:**
   - TextEdit inside ScrollView should NOT have explicit `wrap: word-wrap` causing unwanted growth
   - Rely on ScrollView to handle scrolling

3. **Review conditional rendering:**
   - Ensure both branches (`if current-view == "gcode-editor"` and `"device-console"`) have similar height behavior

---

## Files Affected

- `src/ui.slint` - Main UI definition with layout issues
- `src/main.rs` - File loading and view switching logic (this part is correct, just set view and content)

---

## Testing Recommendations After Fix

1. **Load small gcode file:** Should display with correct layout, all panels visible
2. **Load large gcode file (1000+ lines):** Should display with ScrollView working, no layout distortion
3. **Switch between views:** Toggle between gcode-editor and device-console multiple times
4. **Edit content:** Make sure TextEdit allows editing in both scenarios
5. **Window resize:** Test layout recalculation when window is resized

---

## Conclusion

The g-code editor layout oscillates between broken states because:
1. **View container Rectangle missing height:** Causes unbounded expansion
2. **TextEdit missing height constraint:** Expands to full content size
3. **Nested containers lack explicit sizing:** Layout recalculation becomes unreliable
4. **No clear height distribution:** Panel sizes are ambiguous

The fix requires explicitly specifying heights for layout containers, particularly using `height: 100%` on the main content Rectangle and ensuring the central VerticalBox properly fills its allocated space.

The Slint version (1.14.1) is current and appropriate - this is not a version issue but a layout design issue.
