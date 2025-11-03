# Canvas-Based Visualizer Rendering

## Overview
The G-Code visualizer now supports two rendering modes:
1. **Canvas Rendering** (default) - Uses Slint Path elements for vector graphics
2. **Image Rendering** (fallback) - Renders to PNG images

## Implementation Details

### Canvas Renderer (`src/visualizer/canvas_renderer.rs`)
New module that converts G-Code commands to SVG path data:
- `render_toolpath_to_path()` - Converts toolpath to SVG path commands
- `render_grid_to_path()` - Generates grid lines as SVG paths

### UI Changes

#### Slint UI (`src/ui_panels/gcode_visualizer.slint`)
- Added `Path` elements for vector rendering
- Dual rendering paths: canvas (default) or image (fallback)
- Properties:
  - `visualization-path-data` - SVG path commands for toolpath
  - `visualization-grid-data` - SVG path commands for grid
  - `use-canvas-rendering` - Toggle between modes

#### Main Window (`src/ui.slint`)
Exposed new properties:
- `visualizer-use-canvas-rendering: true` (default)
- `visualization-path-data: ""`
- `visualization-grid-data: ""`

### Rust Integration (`src/main.rs`)
Modified visualization rendering to:
1. Generate canvas path data using `render_toolpath_to_path()`
2. Generate grid data using `render_grid_to_path()`
3. Send path data through message channel
4. Set path data in Slint UI properties
5. Fallback to image rendering if canvas mode disabled

## Benefits

### Performance
- **Reduced Memory** - Vector paths use less memory than raster images
- **GPU Acceleration** - Slint's Path elements are GPU-accelerated
- **Scalability** - Vector graphics scale without quality loss

### Rendering Quality
- **Smooth Lines** - Anti-aliased vector rendering
- **No Pixelation** - Resolution-independent
- **Better Zoom** - Clean rendering at all zoom levels

### Future Enhancements
Vector-based rendering enables:
- Interactive element selection
- Real-time path editing
- Animation and highlighting
- Efficient partial updates

## Usage

### Default Behavior
Canvas rendering is enabled by default:
```rust
window.set_visualizer_use_canvas_rendering(true);
```

### Switching to Image Mode
To use legacy image rendering:
```rust
window.set_visualizer_use_canvas_rendering(false);
```

### Example G-Code Rendering
```rust
use gcodekit4::visualizer::{Visualizer2D, render_toolpath_to_path, render_grid_to_path};

let mut visualizer = Visualizer2D::new();
visualizer.parse_gcode(gcode_content);

// Generate vector paths
let path_data = render_toolpath_to_path(&visualizer, 800, 600);
let grid_data = render_grid_to_path(&visualizer, 800, 600);

// Set in UI
window.set_visualization_path_data(slint::SharedString::from(path_data));
window.set_visualization_grid_data(slint::SharedString::from(grid_data));
```

## Testing

All tests pass:
```bash
cargo test visualizer::canvas_renderer
```

Test coverage:
- Empty visualizer rendering
- Simple line rendering
- Grid visibility based on zoom level
- Arc rendering
- Coordinate transformation

## Architecture

```
┌─────────────────────────────────────┐
│   G-Code Input                      │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   Visualizer2D::parse_gcode()       │
│   - Parse commands                  │
│   - Calculate bounds                │
└──────────────┬──────────────────────┘
               │
               ├──────────────┬────────────────┐
               ▼              ▼                ▼
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │ Canvas Path  │  │ Grid Path    │  │ Image (PNG)  │
    │ Generator    │  │ Generator    │  │ Renderer     │
    └──────┬───────┘  └──────┬───────┘  └──────┬───────┘
           │                 │                  │
           ▼                 ▼                  ▼
    ┌──────────────────────────────────────────────┐
    │   Slint UI                                   │
    │   - Path elements (canvas mode)              │
    │   - Image element (fallback)                 │
    └──────────────────────────────────────────────┘
```

## SVG Path Command Format

### Linear Moves
```
M x1 y1 L x2 y2 L x3 y3 ...
```

### Arc Moves
```
M x1 y1 A rx ry 0 large-arc sweep x2 y2
```
- `rx`, `ry`: Arc radius
- `large-arc`: 0 (small arc) or 1 (large arc)
- `sweep`: 0 (counter-clockwise) or 1 (clockwise)

### Grid Lines
```
M x1 y1 L x2 y2 M x3 y3 L x4 y4 ...
```
Multiple line segments for horizontal and vertical grid

## Known Limitations

1. **Rapid Moves** - Currently filtered out in canvas rendering
2. **Arc Accuracy** - Approximated using SVG arc commands
3. **Markers** - Start/end point markers not yet implemented in canvas mode
4. **Origin Cross** - Not rendered in canvas mode

## Future Work

- [ ] Add start/end point markers to canvas rendering
- [ ] Implement origin cross in canvas mode
- [ ] Support rapid move visualization
- [ ] Add interactive element selection
- [ ] Implement path animation for simulation
- [ ] Add color coding by feedrate/speed
- [ ] Support multiple toolpaths in layers

## Related Files

- `src/visualizer/canvas_renderer.rs` - Canvas rendering implementation
- `src/visualizer/visualizer_2d.rs` - Core visualizer logic
- `src/ui_panels/gcode_visualizer.slint` - UI definition
- `src/ui.slint` - Main window properties
- `src/main.rs` - Rust integration code

## Issue Reference

Implemented for issue **gcodekit4-83**: "Convert visualizer from slint image element to canvas element"
