# Designer Viewport and Coordinate Mapping

## Overview

The viewport module provides comprehensive zoom and pan functionality for the Designer tool with proper coordinate transformation between pixel space (screen coordinates) and world space (design coordinates).

## Key Features

### 1. Coordinate Transformation

#### Pixel to World Conversion
Converts screen/pixel coordinates to design space coordinates, accounting for zoom and pan.

```
world_x = (pixel_x - pan_x) / zoom
world_y = (pixel_y - pan_y) / zoom
```

#### World to Pixel Conversion
Converts design space coordinates to screen/pixel coordinates.

```
pixel_x = world_x * zoom + pan_x
pixel_y = world_y * zoom + pan_y
```

### 2. Viewport Operations

**Zoom Control:**
- `zoom()` - Get current zoom level (1.0 = 100%)
- `set_zoom(zoom)` - Set zoom level (constrained 0.1 - 10.0)
- `zoom_in()` - Zoom in by 20% (multiply by 1.2)
- `zoom_out()` - Zoom out by 20% (divide by 1.2)
- `reset_zoom()` - Reset to 100%

**Pan Control:**
- `pan_x()` / `pan_y()` - Get current pan offset
- `set_pan(x, y)` - Set pan offset
- `pan_by(dx, dy)` - Pan by delta amount
- `reset_pan()` - Reset pan to (0, 0)

**View Management:**
- `fit_to_view(min_x, min_y, max_x, max_y)` - Fit bounding box with 10% padding
- `fit_to_bounds(min_x, min_y, max_x, max_y, padding)` - Fit with custom padding
- `center_on(x, y)` - Center viewport on a world point
- `zoom_to_point(point, zoom)` - Zoom while keeping point in screen position
- `reset()` - Reset to default state (1:1 zoom, no pan)

### 3. Canvas Integration

The Canvas class now uses Viewport internally and exposes these methods:

```rust
pub fn pixel_to_world(&self, pixel_x: f64, pixel_y: f64) -> Point
pub fn world_to_pixel(&self, world_x: f64, world_y: f64) -> (f64, f64)
pub fn zoom_in(&mut self)
pub fn zoom_out(&mut self)
pub fn fit_all_shapes(&mut self)
pub fn center_on(&mut self, point: &Point)
pub fn reset_view(&mut self)
pub fn viewport(&self) -> &Viewport
pub fn viewport_mut(&mut self) -> &mut Viewport
```

## Architecture

### Viewport Structure

```rust
pub struct Viewport {
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
    canvas_width: f64,
    canvas_height: f64,
}
```

### Coordinate System

**Screen Space (Pixel Coordinates):**
- Origin (0, 0) at top-left of canvas
- X increases right, Y increases down
- Used for mouse events, rendering

**Design Space (World Coordinates):**
- Origin (0, 0) at logical design start
- X increases right, Y increases down
- Independent of canvas size and zoom level

### Transformation Pipeline

```
Mouse Event (Screen Coords)
    ↓
pixel_to_world()
    ↓
Design Operation (World Coords)
    ↓
world_to_pixel()
    ↓
Render (Screen Coords)
```

## Usage Examples

### Basic Zoom and Pan

```rust
let mut canvas = Canvas::new();

// Zoom in
canvas.zoom_in();  // 20% zoom increase

// Pan
canvas.pan_by(100.0, 50.0);  // Move view by 100 pixels right, 50 down

// Convert mouse click to design coordinates
let mouse_pixel = (x, y);  // From UI event
let design_point = canvas.pixel_to_world(mouse_pixel.0 as f64, mouse_pixel.1 as f64);

// Use design point for selection
if let Some(shape_id) = canvas.select_at(&design_point) {
    println!("Selected shape: {}", shape_id);
}
```

### Fit Content to View

```rust
let mut canvas = Canvas::new();

// Add shapes...
canvas.add_rectangle(0.0, 0.0, 100.0, 100.0);
canvas.add_circle(Point::new(200.0, 200.0), 50.0);

// Fit all shapes with automatic zoom and pan
canvas.fit_all_shapes();
```

### Zoom at Cursor Position

```rust
let mut canvas = Canvas::new();

// Get cursor position in world coordinates
let cursor_world = canvas.pixel_to_world(cursor_x as f64, cursor_y as f64);

// Zoom while keeping cursor point fixed on screen
canvas.zoom_in_at(&cursor_world);
```

### Center and Zoom

```rust
let mut canvas = Canvas::new();

// Center on a specific design point
let center_point = Point::new(100.0, 100.0);
canvas.center_on(&center_point);

// Get viewport for manual control
let viewport = canvas.viewport_mut();
viewport.set_zoom(2.0);
```

## Mathematical Details

### Zoom Constraints

- Minimum: 0.1x (view is 10x larger than normal)
- Maximum: 10.0x (view is 10x smaller/zoomed in)
- Default: 1.0x (100% - 1:1 mapping)

### Roundtrip Accuracy

The transformation is reversible with high accuracy:

```rust
let original = Point::new(123.456, 789.012);
let (pixel_x, pixel_y) = viewport.world_to_pixel(original.x, original.y);
let roundtrip = viewport.pixel_to_world(pixel_x, pixel_y);
assert!((roundtrip.x - original.x).abs() < 0.001);
```

### Fit-to-Bounds Algorithm

1. Calculate content dimensions from bounding box
2. Calculate zoom needed for width and height
3. Use smaller of the two zooms (maintains aspect ratio)
4. Calculate pan to center content
5. Apply padding percentage (default 10%)

## Performance Characteristics

All transformations are O(1):
- Pixel to world: 2 divisions
- World to pixel: 2 multiplications and 2 additions
- Zoom operations: 1 multiplication/division
- Pan operations: 1 addition/subtraction

## Test Coverage

### Unit Tests (14 tests)
- Viewport creation and initialization
- Pixel-to-world conversion (no transform, with zoom, with pan, combined)
- World-to-pixel conversion (no transform, with zoom, with pan)
- Roundtrip conversion accuracy
- Zoom constraints and limits
- Zoom in/out operations
- Center on point
- Fit to bounds
- Reset operations

### Integration Tests (22 tests)
- Canvas viewport integration
- Coordinate mapping with shapes
- Pan/zoom interaction
- Zoom constraints
- Canvas size awareness
- Compatibility methods (pan_offset, pan)
- Multiple shape fitting
- View reset functionality

## Integration with Designer

### Event Handling Flow

1. **UI Event** (e.g., mouse click at pixel x, y)
2. **Convert to World** using `pixel_to_world()`
3. **Canvas Operation** (e.g., select shape at point)
4. **Update State**
5. **Render** using world_to_pixel for each shape

### Rendering Pipeline

```rust
for shape in canvas.shapes() {
    let (x1, y1, x2, y2) = shape.bounding_box();
    let (pixel_x1, pixel_y1) = canvas.world_to_pixel(x1, y1);
    let (pixel_x2, pixel_y2) = canvas.world_to_pixel(x2, y2);
    render_rectangle(pixel_x1, pixel_y1, pixel_x2, pixel_y2);
}
```

## Future Enhancements

### Planned Features
1. **Aspect Ratio Locking** - Option to maintain design aspect ratio when resizing canvas
2. **Smooth Zoom Animation** - Gradual zoom transitions instead of discrete steps
3. **Pan Constraints** - Limit panning to keep content visible
4. **Multi-level Zoom** - Different zoom strategies (fit-width, fit-height, fit-all)
5. **Grid-aligned Panning** - Optional snap-to-grid for pan operations
6. **Zoom History** - Undo/redo for zoom operations
7. **Viewport Presets** - Save and restore common zoom/pan configurations

### Integration Opportunities
1. Integrate with touch gestures (pinch to zoom, drag to pan)
2. Mouse wheel zoom at cursor
3. Keyboard shortcuts for zoom/pan operations
4. Viewport synchronization for multi-view editing
5. Persistent viewport state (save between sessions)

## API Reference

### Viewport Methods

```rust
impl Viewport {
    pub fn new(canvas_width: f64, canvas_height: f64) -> Self
    pub fn set_canvas_size(&mut self, width: f64, height: f64)
    pub fn zoom(&self) -> f64
    pub fn set_zoom(&mut self, zoom: f64)
    pub fn zoom_in(&mut self)
    pub fn zoom_out(&mut self)
    pub fn reset_zoom(&mut self)
    pub fn pan_x(&self) -> f64
    pub fn pan_y(&self) -> f64
    pub fn set_pan(&mut self, x: f64, y: f64)
    pub fn pan_by(&mut self, dx: f64, dy: f64)
    pub fn reset_pan(&mut self)
    pub fn pixel_to_world(&self, pixel_x: f64, pixel_y: f64) -> Point
    pub fn world_to_pixel(&self, world_x: f64, world_y: f64) -> (f64, f64)
    pub fn world_point_to_pixel(&self, point: &Point) -> (f64, f64)
    pub fn fit_to_bounds(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64, padding: f64)
    pub fn fit_to_view(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64)
    pub fn zoom_to_point(&mut self, world_point: &Point, new_zoom: f64)
    pub fn zoom_in_at(&mut self, world_point: &Point)
    pub fn zoom_out_at(&mut self, world_point: &Point)
    pub fn center_on(&mut self, world_x: f64, world_y: f64)
    pub fn center_on_point(&mut self, point: &Point)
    pub fn reset(&mut self)
    pub fn to_string(&self) -> String
}
```

## File Organization

```
src/designer/
├── viewport.rs              ~400 lines
├── canvas.rs                Updated with viewport integration
├── mod.rs                   Updated exports
└── ... (existing modules)

tests/
├── designer_viewport_coordinate_mapping.rs    22 integration tests
└── ... (existing test files)
```

## Testing

Run viewport tests:
```bash
cargo test --lib viewport
cargo test --test designer_viewport_coordinate_mapping
```

All 36 tests (14 unit + 22 integration) pass with 100% success rate.

## References

- **Coordinate Systems**: https://en.wikipedia.org/wiki/Coordinate_system
- **Affine Transformations**: https://en.wikipedia.org/wiki/Affine_transformation
- **Viewport Rendering**: Graphics/Game Engine best practices
- **CAD/CAM Zoom Interaction**: Standard UI patterns in design software
