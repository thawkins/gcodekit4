# Designer SVG Import Feature

## Overview

The Designer tool now includes full SVG (Scalable Vector Graphics) import functionality, allowing users to import vector designs from external graphics applications directly into the Designer canvas.

## Supported SVG Elements

The SVG importer supports the following SVG elements:

### Basic Shapes
- **Rectangle** (`<rect>`) - Converted to Designer Rectangle
- **Circle** (`<circle>`) - Converted to Designer Circle
- **Ellipse** (`<ellipse>`) - Converted to Designer Ellipse
- **Line** (`<line>`) - Converted to Designer Line

### Complex Shapes
- **Polyline** (`<polyline>`) - Converted to Designer Polygon
- **Polygon** (`<polygon>`) - Converted to Designer Polygon
- **Path** (`<path>`) - Parsed and converted to appropriate shapes

### Path Commands Supported
- **M/m** - MoveTo (absolute/relative)
- **L/l** - LineTo (absolute/relative)
- **H/h** - Horizontal LineTo (absolute/relative)
- **V/v** - Vertical LineTo (absolute/relative)
- **Z/z** - ClosePath

## Usage

### From the UI

1. Click the **SVG** button in the Designer left sidebar
2. Select an SVG file from the file dialog
3. The shapes will be imported and displayed on the canvas
4. Status message shows the number of shapes imported

### Programmatic Usage

```rust
use gcodekit4::designer::SvgImporter;

// Create importer with scale and offset
let importer = SvgImporter::new(
    1.0,    // scale factor
    0.0,    // x offset
    0.0     // y offset
);

// Import from file
let design = importer.import_file("path/to/file.svg")?;

// Or import from string
let svg_content = std::fs::read_to_string("file.svg")?;
let design = importer.import_string(&svg_content)?;

// Access imported shapes
for shape in design.shapes {
    // Process each shape
}
```

## Transformation Options

The `SvgImporter` supports the following transformations:

- **Scale**: Multiply all dimensions by a scale factor
- **Offset X**: Add an X offset to all coordinates
- **Offset Y**: Add an Y offset to all coordinates

Example with transformations:
```rust
// Double the size and offset by (10, 20)
let importer = SvgImporter::new(2.0, 10.0, 20.0);
```

## Coordinate System

- SVG coordinates are in pixels by default
- The importer preserves the coordinate system from the SVG
- Origin (0,0) is at the top-left corner
- Positive X is right, positive Y is down

## Limitations

Current limitations of the SVG importer:

1. **No curve support yet**: Bezier curves (C, Q, S, T commands) are not yet implemented
2. **No arc support**: Arc commands (A) are not yet implemented
3. **No transforms**: SVG transform attributes are not processed
4. **No groups**: Group hierarchy is flattened
5. **No styles**: Fill, stroke, and other styling attributes are ignored
6. **No text**: Text elements are not imported

## File Format Requirements

- Valid XML structure
- UTF-8 encoding recommended
- Standard SVG namespace: `http://www.w3.org/2000/svg`

## Example SVG File

```xml
<?xml version="1.0" encoding="UTF-8"?>
<svg width="200" height="200" xmlns="http://www.w3.org/2000/svg">
  <rect x="10" y="10" width="50" height="30"/>
  <circle cx="100" cy="50" r="20"/>
  <line x1="10" y1="100" x2="80" y2="100"/>
  <ellipse cx="150" cy="150" rx="30" ry="20"/>
  <path d="M 120 120 L 140 120 L 140 140 Z"/>
</svg>
```

## Error Handling

The importer returns `Result<ImportedDesign, anyhow::Error>` and will fail with descriptive errors for:

- Invalid XML syntax
- Missing required attributes
- File I/O errors
- Malformed path data

## Future Enhancements

Planned improvements for future versions:

1. Bezier curve support with adaptive tessellation
2. Arc command support
3. SVG transform parsing and application
4. Layer support via groups
5. Text-to-path conversion
6. Style preservation for toolpath planning

## Testing

The module includes comprehensive unit tests:

```bash
cargo test --lib import::
```

Test SVG file available at: `target/temp/test_import.svg`

## Related Modules

- `designer::import` - Main import module
- `designer::shapes` - Shape definitions
- `designer::dxf_parser` - DXF import functionality
- `designer_state` - Designer UI integration
