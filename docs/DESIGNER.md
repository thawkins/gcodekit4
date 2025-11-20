# Designer Module Analysis and Plan

## Current State Analysis

The `gcodekit4-designer` crate provides basic 2D drawing and G-code generation capabilities. However, the current implementation has several limitations and inefficiencies:

1.  **Shape Management Inefficiency**:
    *   Shapes are stored as `Box<dyn Shape>` within `DrawingObject`.
    *   Transformations (move, resize) involve creating entirely new shape instances and re-boxing them, which is inefficient and verbose.
    *   The `Shape` trait lacks methods for in-place mutation (e.g., `translate`, `scale`).

2.  **G-code Generation Limitations**:
    *   The `generate_gcode` function approximates complex shapes (Polygons, RoundRectangles) as simple Rectangles. This is a significant logical error.
    *   It implicitly treats all shapes as "Profile" cuts (following the contour).
    *   There is no support for "Pocket" operations (clearing the area inside a shape).

3.  **Missing Features**:
    *   **Pocketing**: No ability to define a shape as a pocket or specify a pocket depth.
    *   **Text**: No support for adding text objects.

4.  **Pocketing Implementation**:
    *   `pocket_operations.rs` exists but is limited to Rectangles and Circles and uses a simple inset strategy. It does not support arbitrary polygons or complex paths.

## Plan for Renewal

To address these issues and add the requested features, the following plan is proposed:

### 1. Refactor Data Structures

*   **Enhance `DrawingObject`**:
    *   Add an `operation` field: `enum OperationType { Profile, Pocket }`.
    *   Add a `cut_depth` field: `Option<f64>` (to override global depth).
    *   Add a `pocket_depth` field (specifically for pocket operations).
*   **Update `Shape` Trait**:
    *   Add `translate(&mut self, dx: f64, dy: f64)` method.
    *   Add `resize(&mut self, handle: usize, dx: f64, dy: f64)` method.
    *   This will simplify `Canvas` logic and reduce memory allocations.

### 2. Implement Text Shape

*   **Dependencies**: Add `ab_glyph` (or similar) to `Cargo.toml` to handle font parsing.
*   **Font Loading**: Load the "Fira Mono" font (from `assets/fonts/fira-code/FiraCode-Regular.ttf`).
*   **`TextShape` Struct**:
    *   Fields: `text: String`, `position: Point`, `font_size: f64`, `font_data: Vec<u8>`.
    *   Implement `Shape` trait.
    *   **Rendering**: Convert glyphs to `lyon::path::Path` for rendering and G-code generation.

### 3. Implement Pocketing

*   **UI Integration**: Allow users to select "Pocket" operation and set "Pocket Depth" for selected shapes.
*   **G-code Generation**:
    *   Update `generate_gcode` to check `OperationType`.
    *   If `Pocket`: Use an enhanced `PocketGenerator`.
    *   **Enhanced Pocketing**:
        *   Extend `PocketGenerator` to handle arbitrary paths (using `lyon` or a scanline fill algorithm).
        *   Ensure it respects the `pocket_depth` attribute.

### 4. Fix G-code Generation Logic

*   **Remove Approximations**: Ensure `generate_gcode` uses the actual geometry of Polygons and RoundRectangles, not their bounding boxes.
*   **Path Iteration**: Use `lyon`'s path iteration to generate precise G-code for all shape types.

### 5. UI Updates (Future Work)

*   Update the Designer UI to expose the new attributes (Operation, Depth) in the properties panel.
*   Add a "Text" tool to the toolbar.

## Summary of New Attributes

Each shape/object will effectively have:
*   **Geometry**: (Existing)
*   **Operation**: `Profile` (default) or `Pocket`.
*   **Pocket Depth**: `f64` (if Pocket).
*   **Cut Depth**: `f64` (optional override).

## Text Implementation Details

*   Use `ab_glyph` to load the TTF font.
*   Generate outlines for each character.
*   Combine outlines into a single `PathShape` or keep as a specialized `TextShape` that generates the path on demand.

## Implementation Progress (Feature Branch)

### Data Model
- [x] Added `OperationType` enum (Profile, Pocket).
- [x] Added `pocket_depth` to `DrawingObject` and `ShapeData`.
- [x] Added `Text` variant to `ShapeType`.
- [x] Added `TextShape` struct.
- [x] Updated serialization to handle new fields and shapes.

### UI
- [x] Added "Text" tool button to toolbar.
- [x] Added "Pocket Operation" checkbox and "Pocket Depth" input to properties panel.
- [x] Added "Content" and "Font Size" inputs for Text shapes.
- [x] Updated `DesignerPanel` and `MainWindow` to bind new properties and callbacks.

### Backend Integration
- [x] Updated `DesignerState` to handle property updates for pocket and text.
- [x] Updated `main.rs` to wire up UI callbacks to state methods.
- [x] Added `rusttype` dependency for future text rendering.

### Next Steps
- [x] Implement font loading (Fira Mono).
- [x] Implement `TextShape::bounding_box` using actual font metrics.
- [x] Implement text rendering in `renderer.rs` and `svg_renderer.rs`.
- [x] Implement toolpath generation for Pocket operations (raster/offset).
- [x] Implement toolpath generation for Text (glyph to path).
- [x] Remove RoundRectangle shape and tool.
