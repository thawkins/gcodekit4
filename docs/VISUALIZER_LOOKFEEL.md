# Visualizer Look & Feel Analysis

## Overview
This document analyzes the current aesthetics of the G-Code Visualizer and provides recommendations to align it with the Designer's dark-themed, modern UI.

## Current State vs. Designer Standard

| Feature | Visualizer (Current) | Designer (Target) |
|---------|----------------------|-------------------|
| **Theme** | Mixed (Light toolbar, Black canvas) | **Dark Mode** (`#2c3e50` panels, `#34495e` canvas) |
| **Layout** | Top Toolbar + Status Bar + Canvas | Left/Right Sidebars + Canvas |
| **Controls** | Standard `Button`, `CheckBox` | Custom `ToolButton`, `CompactSpinBox`, `DarkCheckBox` |
| **Canvas BG** | `black` | `#34495e` (Dark Blue-Grey) |
| **Status Info** | Static text bar above canvas | **Floating** semi-transparent pill overlay |
| **View Controls** | Standard buttons in toolbar | **Floating** bottom-right overlay |
| **Typography** | Dark text on light BG | Light text (`#ecf0f1`) on dark BG |

## Recommendations

### 1. Adopt the Dark Theme
The Visualizer should adopt the global dark theme used by the Designer.
- **Panel Background:** Change from default (white/transparent) to `#2c3e50`.
- **Text Color:** Change from dark (`#2c3e50`) to light (`#ecf0f1`).
- **Canvas Background:** Change from `black` to `#34495e` to maintain visual continuity between tabs.

### 2. Floating UI Overlays
Move status information and view controls from the static top bar to floating overlays on the canvas.
- **Status Bar:** Create a floating pill at the bottom-left showing Zoom, X/Y offsets, and Grid size.
- **Zoom Controls:** Create a floating pill at the bottom-right with `+`, `-`, and `Fit` buttons.
- **Implementation:** Copy the `Rectangle { ... status-layout ... }` and Zoom control blocks from `designer.slint`.

### 3. Modernize the Toolbar
The current top toolbar uses standard buttons which look out of place.
- **Option A (Sidebar):** Move controls to a vertical sidebar on the left, matching the Designer's tool palette.
- **Option B (Styled Top Bar):** Keep the top bar but style it with `#2c3e50` background and use icon-based `ToolButton`s instead of text buttons.
- **Recommendation:** **Option B** is likely easier for the Visualizer as it has fewer tools than the Designer, but **Option A** offers better consistency.

### 4. Component Standardization
Replace standard widgets with the custom dark-themed versions found in `designer.slint`.
- Replace `CheckBox` with `DarkCheckBox`.
- Replace standard `Button`s with `ToolButton` or styled equivalents.

### 5. Path Styling
- **Stroke Width:** Ensure stroke width is consistent (1px) and scale-independent (already implemented).
- **Colors:**
    - Rapid Moves (G0): Cyan (`#00FFFF80`) - *Keep, works well on dark blue-grey.*
    - Linear (G1): Yellow (`#FFFF00`) - *Keep.*
    - Arcs (G2/G3): Green/Red - *Keep.*
    - **Grid:** Ensure grid lines are subtle (`#808080` or lighter opacity) to not compete with the toolpath.

## Implementation Plan

1.  **Refactor Layout:** Remove the top `HorizontalBox` toolbar and the status `HorizontalBox`.
2.  **Apply Theme:** Set `GcodeVisualizerPanel` background to `#2c3e50`.
3.  **Update Canvas:**
    - Change background to `#34495e`.
    - Add `FocusScope` for keyboard handling (if not already present).
    - Add floating Status and Zoom overlays.
4.  **Migrate Controls:**
    - Move "Refresh", "Show Moves", "Show Grid" to a unified control area (either a floating top-left panel or a sidebar).
    - Use icons for "Refresh" instead of text.

## Code Reference (Designer Styling)

```slint
// Floating Status Bar
Rectangle {
    x: 10px;
    y: parent.height - 40px;
    height: 30px;
    background: #2c3e50;
    border-radius: 15px;
    opacity: 0.9;
    border-width: 1px;
    border-color: #34495e;
    // ... content ...
}

// Dark CheckBox
component DarkCheckBox inherits HorizontalLayout {
    // ... implementation from designer.slint ...
}
```
