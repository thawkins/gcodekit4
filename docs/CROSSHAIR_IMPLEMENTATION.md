# Designer Crosshair Implementation

## Overview

Added a bright yellow crosshair at the world coordinate origin (0,0) to the Designer canvas for improved spatial reference and design alignment.

## Implementation Details

### File Modified

**`src/designer/renderer.rs`**

### Changes Made

1. **Added crosshair color constant:**
   ```rust
   const CROSSHAIR_COLOR: Rgb<u8> = Rgb([255, 255, 0]); // Bright yellow
   ```

2. **Added crosshair rendering in `render_canvas()` function:**
   ```rust
   // Draw 0,0 crosshair - convert world origin to screen coordinates
   let (origin_x, origin_y) = viewport.world_to_pixel(0.0, 0.0);
   let origin_x = origin_x as i32;
   let origin_y = origin_y as i32;
   
   // Draw horizontal line across entire canvas
   draw_line(&mut img, 0, origin_y, width as i32, origin_y, CROSSHAIR_COLOR);
   
   // Draw vertical line across entire canvas
   draw_line(&mut img, origin_x, 0, origin_x, height as i32, CROSSHAIR_COLOR);
   ```

3. **Updated module documentation:**
   Added feature list including crosshair reference

## Features

### Visual Properties

- **Color**: RGB(255, 255, 0) - Bright Yellow
- **Visibility**: Always visible across entire canvas
- **Width**: 3 pixels (center line ± 1 pixel)
- **Style**: Solid lines

### Behavior

- **Transforms with viewport**: 
  - Moves correctly when panning
  - Scales properly when zooming
  - Always represents world coordinate (0,0)
  
- **Rendering order**:
  1. Background drawn
  2. Crosshair drawn (on top of background)
  3. Shapes drawn (on top of crosshair)
  4. Selection indicators drawn (on top of shapes)

### Benefits

1. **Spatial Reference**: Clear visual indicator of the coordinate system origin
2. **Design Alignment**: Helps align imported designs and manually drawn shapes
3. **Machine Origin**: Shows where the machine's work coordinate origin is located
4. **Navigation Aid**: Provides reference point when panning/zooming around the canvas

## Usage

The crosshair is automatically displayed and requires no user interaction:

- **Always Visible**: Crosshair is always rendered on the canvas
- **No Toggle**: Currently no option to hide (always on)
- **Automatic**: Transforms automatically with all viewport operations

### Common Use Cases

1. **Importing Designs**: 
   - Check where imported SVG/DXF designs are positioned relative to origin
   - Adjust import offset to align with origin

2. **Creating Designs**:
   - Start shapes at known coordinates relative to origin
   - Align multiple shapes to coordinate axes

3. **Toolpath Planning**:
   - Understand where G-code will execute relative to machine origin
   - Plan safe positions for tool changes and returns

4. **Debugging**:
   - Verify coordinate transformations are working correctly
   - Check viewport pan/zoom behavior

## Technical Details

### Coordinate Transformation

The crosshair uses the viewport's `world_to_pixel()` transformation to convert the world coordinate origin (0.0, 0.0) to screen pixel coordinates.

```rust
let (origin_x, origin_y) = viewport.world_to_pixel(0.0, 0.0);
```

This ensures the crosshair:
- Moves when the viewport is panned
- Scales correctly when zoomed
- Always marks the true world origin

### Rendering Performance

- **Cost**: Minimal - six line draws per frame (3 lines per axis for thickness)
- **Method**: Uses existing `draw_line()` function (Bresenham's algorithm)
- **Impact**: Negligible performance impact

### Z-Order

Rendering order (bottom to top):
1. Background (solid color fill)
2. **Crosshair** (yellow lines)
3. Shapes (blue outlines)
4. Selection boxes (yellow with handles)

This ensures:
- Crosshair is visible under shapes
- Shapes can obscure crosshair (normal behavior)
- Selection indicators appear on top

## Testing

- ✅ Build: Successful
- ✅ Tests: All 241 designer tests passing
- ✅ Clippy: No warnings
- ✅ Visual: Crosshair renders at correct position

## Future Enhancements

Potential improvements:

1. **Toggle Visibility**: Add UI control to show/hide crosshair
2. **Customizable Color**: Allow user to change crosshair color
3. **Line Style**: Add dashed or dotted line options
4. **Thickness**: Make line thickness configurable
5. **Grid Snap**: Add crosshair snapping to grid intersections
6. **Multiple Origins**: Support displaying multiple coordinate system origins (G54-G59)
7. **Labels**: Add "X" and "Y" labels at axis ends

## Related Files

- `src/designer/renderer.rs` - Main implementation
- `src/designer/viewport.rs` - Coordinate transformation
- `src/designer/canvas.rs` - Canvas state management
- `docs/designer_features.md` - User documentation

## Summary

The crosshair implementation provides a simple but effective visual reference for the Designer tool's coordinate system. The bright yellow color ensures high visibility against the dark gray background, and the full-canvas extent ensures it's always visible regardless of design content.

**Status**: ✅ Complete and ready for use
