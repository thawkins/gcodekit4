# Designer Tool Features

## Overview

The Designer tool is a 2D CAD/CAM application integrated into GCodeKit4, allowing users to create designs and generate G-code toolpaths directly within the application.

## Visual Features

### Origin Crosshair

A **bright yellow crosshair** marks the world coordinate origin (0,0) on the canvas:

- **Color**: RGB(255, 255, 0) - Bright Yellow
- **Horizontal Line**: Extends across the entire canvas width
- **Vertical Line**: Extends across the entire canvas height
- **Behavior**: 
  - Transforms correctly with zoom and pan operations
  - Always visible regardless of where you navigate
  - Provides clear spatial reference for design work
  - Helps align imported designs and drawn shapes

### Canvas

- **Size**: 1600x1200 pixels
- **Background**: Dark gray (#34495e)
- **Coordinate System**: Standard Cartesian (0,0 at origin, +X right, +Y up)
- **Zoom**: 0.1x to 10x
- **Pan**: Full canvas navigation with mouse drag

## Drawing Tools

### Select Mode (S)
- Click to select shapes
- **Group Selection**: Clicking anywhere inside a group's bounding box selects the entire group
- Drag to move shapes
- Resize handles on selection
- Delete with Delete key

### Shape Tools

1. **Rectangle (R)** - Draw rectangular shapes
2. **Circle (C)** - Draw circular shapes
3. **Ellipse (E)** - Draw elliptical shapes
4. **Line (L)** - Draw straight lines
5. **Polygon (P)** - Draw regular polygons
6. **Round Rectangle (U)** - Draw rectangles with rounded corners

## Import Features

### SVG Import
- Click **SVG** button in left sidebar
- Supports: rectangles, circles, ellipses, lines, polylines, polygons, paths
- See [designer_svg_import.md](designer_svg_import.md) for details

### DXF Import
- Click **DXF** button in left sidebar
- Supports: lines, circles, arcs, polylines
- Automatic layer detection
- Scale and offset transformations

## Toolpath Generation

### Tool Setup

Configure cutting parameters in the right sidebar:

- **Feed Rate**: Cutting feed rate (mm/min)
- **Spindle Speed**: Spindle RPM
- **Tool Diameter**: Cutting tool diameter (mm)
- **Cut Depth**: Cutting depth (negative value in mm)

### Generate G-Code

1. Set tool parameters
2. Click **Generate** button
3. G-code appears in the G-Code Editor
4. Export or send directly to machine

## View Controls

Located in the left sidebar:

- **+ (Plus)**: Zoom in
- **- (Minus)**: Zoom out
- **⊡ (Fit)**: Zoom to fit all shapes
- **⌂ (Home)**: Reset view to default

## Edit Controls

- **✗ (Delete)**: Delete selected shape
- **⊕ (Clear)**: Clear entire canvas

## Keyboard Shortcuts

- **S**: Select mode
- **R**: Rectangle mode
- **C**: Circle mode
- **L**: Line mode
- **E**: Ellipse mode
- **P**: Polygon mode
- **U**: Round rectangle mode
- **Delete**: Delete selected shape
- **Escape**: Deselect all

## Coordinate Display

Top info bar shows:

- **Scale**: Current zoom percentage
- **X/Y**: Current pan offset
- **Shape**: Selected shape position and dimensions (X, Y, W, H)

## Design Workflow

### Basic Workflow

1. **Create/Import Design**
   - Draw shapes with tools OR
   - Import SVG/DXF files
   
2. **Edit Design**
   - Select and move shapes
   - Resize with handles
   - Delete unwanted elements
   
3. **Configure Toolpath**
   - Set feed rate, spindle speed
   - Set tool diameter and cut depth
   
4. **Generate G-Code**
   - Click Generate button
   - Review in G-Code Editor
   
5. **Execute**
   - Visualize in 3D viewer
   - Send to machine

### Tips

- **Use the crosshair** to align your design with the machine origin
- **Import designs** from Inkscape, Adobe Illustrator, or CAD software
- **Start with simple shapes** to test your toolpaths
- **Check dimensions** in the info bar before generating G-code
- **Zoom to fit** after importing to see the full design

## Color Reference

- **Background**: Dark gray (#34495e)
- **Shapes**: Blue (#3498db)
- **Selection**: Yellow (#ffeb3b)
- **Crosshair**: Bright yellow (#ffff00)

## Technical Details

- **Canvas Resolution**: 1600x1200 pixels
- **Units**: Millimeters (mm)
- **Scale**: 1 pixel = 1mm at 1x zoom
- **Coordinate System**: Right-hand Cartesian
- **Origin**: Center or custom offset

## Related Documentation

- [SVG Import Guide](designer_svg_import.md)
- [SVG Import Implementation](SVG_IMPORT_IMPLEMENTATION.md)
