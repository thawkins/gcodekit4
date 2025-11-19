# Lyon Crate Integration Review

## Executive Summary
We recommend integrating the `lyon` crate ecosystem to replace the custom vector math and SVG path handling currently in `gcodekit4-camtools` and `gcodekit4-designer`. This will provide robust, industry-standard algorithms for path manipulation, flattening, and tessellation, which are prerequisites for implementing the requested "internal path hatching" feature.

## Current State Analysis

### 1. SVG Parsing & Path Storage
- **Current**: `VectorEngraver` (camtools) and `SvgImporter` (designer) both implement custom, partial SVG parsers.
- **Issues**: 
  - Duplicated logic between crates.
  - Manual string parsing of SVG paths is fragile.
  - Custom adaptive subdivision for Bezier curves (`adaptive_cubic_segments`) is reinventing the wheel and likely less efficient/accurate than `lyon`'s flattening algorithms.
  - Data is stored as `Vec<Vec<PathElement>>` (simple polylines), losing the semantic meaning of curves until they are baked.

### 2. Vector Operations
- **Current**: Basic point arithmetic and manual distance calculations.
- **Missing**: No robust support for:
  - Path offsetting (needed for "island" pocketing).
  - Point-in-polygon testing (needed for hatching).
  - Intersection calculations (needed for hatching).

## Proposed Architecture

### Dependencies
Add `lyon` (v1.0+) to `gcodekit4-camtools` and `gcodekit4-designer`.
- `lyon_path`: For robust path data structures (`Path`, `PathBuilder`).
- `lyon_algorithms`: For `walk`, `hit_test`, and `aabb`.
- `lyon_geom`: For low-level curve math.
- `lyon_tessellation`: While primarily for GPU mesh generation, its `FillTessellator` can be useful for visualizing the fill, but for *generating G-code toolpaths*, we need vector hatching, not triangle meshes.

### Hatching Implementation Strategy
To "hatch internal paths" (fill regions with lines), `lyon` alone doesn't provide a "hatch" function, but it provides the necessary primitives:

1.  **Path Flattening**: Use `lyon_algorithms::path::iterator::Flattened` to convert curves to line segments with configurable tolerance.
2.  **Scanline Algorithm**:
    - Calculate the AABB of the path.
    - Generate horizontal (or angled) scanlines across the AABB.
    - Compute intersections between scanlines and the flattened path segments.
    - Sort intersections by X-coordinate.
    - Apply "Non-Zero" or "Even-Odd" rule to determine filled segments (pairs of intersections).
3.  **G-code Generation**: Convert these segments into G1 moves.

## Task List

### Phase 1: Foundation & Refactoring
- [ ] Add `lyon` dependency to `gcodekit4-camtools` and `gcodekit4-designer`.
- [ ] Refactor `VectorEngraver` to use `lyon::path::Path` as the primary data structure instead of `Vec<Vec<PathElement>>`.
- [ ] Replace custom SVG path parsing in `VectorEngraver` with a builder that feeds `lyon::path::PathBuilder`.
- [ ] Replace custom `adaptive_cubic_segments` with `lyon`'s flattening iterators.

### Phase 2: Hatching Feature
- [ ] Implement `HatchGenerator` struct in `camtools`.
  - Input: `lyon::path::Path`, `angle`, `spacing`.
  - Output: `Vec<Vec<PathElement>>` (the hatch lines).
- [ ] Implement scanline intersection logic (Ray casting).
- [ ] Add "Hatch" option to `VectorEngravingParameters`.
- [ ] Update `generate_gcode` to include generated hatch paths.

### Phase 3: Designer Integration
- [ ] Update `gcodekit4-designer` to use `lyon` for its internal shape representation where appropriate.
- [ ] Expose hatching options in the UI (requires UI updates).

## Recommendation
Proceed with Phase 1 immediately to reduce technical debt in SVG parsing before building the complex Hatching feature on top of the legacy parser.
