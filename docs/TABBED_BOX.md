# Tabbed Box Generator Analysis

## Current Implementation Review

The current implementation in `crates/gcodekit4-camtools/src/tabbed_box.rs` and `crates/gcodekit4-ui/ui_panels/tabbed_box_dialog.slint` has several critical issues and missing features compared to the reference implementation [TabbedBoxMaker](https://github.com/paulh-rnd/TabbedBoxMaker).

### Critical Bugs

1.  **Protrusion Length Accuracy**:
    -   The code currently hardcodes the finger protrusion length to the material thickness `t`.
    -   The `FingerJointSettings` struct contains an `extra_length` field, but it is **completely ignored** in the `draw_finger_edge` function.
    -   **Fix**: The `draw_finger_edge` function should incorporate `extra_length` into the protrusion calculation.

2.  **"No Top" Box Behavior**:
    -   **Inverted Logic**: The current code skips the *Bottom* panel when `BoxType::NoTop` is selected, instead of the *Top* panel.
        ```rust
        // Bottom: x × y with all fingers in
        if self.params.box_type == BoxType::FullBox {
            self.paths.push(self.draw_rectangular_wall(x, y, "ffff", x_offset, y_offset));
        }
        ```
    -   **Incorrect Edge Style**: When "No Top" is selected, the top edges of the side walls (Walls 1, 2, 3, 4) are still drawn with finger joints ("F" or "f"). They should be drawn as flat edges ('e').
    -   **Fix**:
        -   Skip the "Top" panel generation if `BoxType::NoTop`.
        -   Change the edge definition strings for walls to use 'e' for the top edge (index 2) when `BoxType::NoTop` is active.

3.  **Kerf/Burn Compensation**:
    -   The `BoxParameters` struct has a `burn` field (intended for kerf compensation), but it is **never used** in the geometry generation.
    -   **Fix**: Apply `burn` / 2.0 offset to the path coordinates to account for the cutter width.

### Missing Features (vs TabbedBoxMaker)

1.  **Dividers**: The current implementation does not support internal dividers (length or width wise).
2.  **Dogbone/T-Bone Fillets**: Essential for CNC milling to allow corners to fit together. The current implementation only supports "Rectangular", "Springs", "Barbs", "Snap" (though only Rectangular seems fully implemented logic-wise in the drawing).
3.  **Layout Optimization**: The current layout is a simple linear arrangement. It lacks "Compact" or "3-Piece" layout options to save material.
4.  **Inside vs Outside Dimensions**: The code has an `outside` boolean, but the implementation `adjust_size` simply subtracts `2 * thickness`. This is a simplified view and might not be accurate for all box types or internal dimensions.

## Proposed Plan

### Phase 1: Fix Critical Bugs (Completed)

1.  **Fix "No Top" Logic**:
    -   ✅ Correctly skip the Top panel.
    -   ✅ Dynamically generate edge strings based on `BoxType`.
2.  **Implement `extra_length`**:
    -   ✅ Update `draw_finger_edge` to use `settings.extra_length`.
    -   ✅ Expose this setting in the UI.
3.  **Implement `burn` (Kerf)**:
    -   ✅ Apply kerf compensation to the generated paths.

### Phase 2: Feature Parity (Completed)

1.  **Add Dividers**:
    -   ✅ Added `dividers_x` and `dividers_y` parameters.
    -   ✅ Implemented generation of divider panels.
    -   ⚠️ Slots in main walls for dividers are not yet implemented.
2.  **CNC Support**:
    -   ✅ Added `FingerStyle::Dogbone`.
    -   ✅ Implemented dogbone overcuts in `draw_finger_edge`.
    -   ✅ Added `Tool Diameter` (using `burn` parameter) support.
3.  **Layout Options**:
    -   ✅ Added `optimize_layout` parameter.
    -   ✅ Implemented `pack_paths` using a shelf packing algorithm.
    -   ✅ Added UI checkbox for layout optimization.

## UI Updates

-   ✅ Added `LineEdit` for `extra_length` (Protrusion Offset).
-   ✅ Updated `burn` label to "Burn / Tool Dia".
-   ✅ Added inputs for `Dividers X` and `Dividers Y`.
-   ✅ Added `Dogbone` to Finger Style dropdown.
-   ✅ Added `Optimize Layout` checkbox.

### Phase 3: Feature Parity with Python BoxMaker
Analysis of `boxmaker.py` and related files reveals the following missing features in the Rust implementation:

### Missing Box Types
The Python implementation supports 6 box types, while Rust only supports `FullBox` and `NoTop`.
- [x] `NoBottom`
- [x] `NoSides` (Top and Bottom only)
- [x] `NoFrontBack` (Top, Bottom, Left, Right)
- [x] `NoLeftRight` (Top, Bottom, Front, Back)

### Tab Features
- [x] `Dimple` support: The Python code supports adding "dimples" to tabs (friction fit bumps).
  - Requires `dimple_height` and `dimple_length` parameters.
  - Logic in `dimple_str` needs to be ported.

### Dividers
- [x] `KeyDividerType`: Support for keying dividers into walls/floor.
  - `WallsAndFloor`
  - `WallsOnly`
  - `FloorOnly`
  - `None`
- [x] Cross-divider slots (halving joints).

### Optimization
- [x] "Optimizing panel-sides to a closed path" is already implicitly handled by the Rust implementation's `draw_rectangular_wall` which produces a single continuous path per panel, whereas the Python code produces 4 separate paths.

## Plan
1.  Update `BoxType` enum in `tabbed_box.rs` to include missing types.
2.  Update `generate` logic in `tabbed_box.rs` to handle new box types (controlling which walls are generated and their edge styles).
3.  Add `dimple_height` and `dimple_length` to `FingerJointSettings`.
4.  Implement dimple drawing logic in `draw_finger_edge`.
