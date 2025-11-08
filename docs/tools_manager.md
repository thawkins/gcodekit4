# CNC Tools Manager

The CNC Tools Manager provides a comprehensive database for managing CNC cutting tools including end mills, drills, taps, reamers, and other cutting tools with their specifications and cutting parameters.

## Features

### Tool Types Supported

- **End Mills**: Flat, ball nose, corner radius
- **Drills**: Standard twist drills, center drills, spot drills
- **Engraving Tools**: V-bits, engraving bits
- **Specialty Tools**: Chamfer tools, thread mills, custom tools

### Tool Properties

#### Basic Information
- **Tool ID**: Unique identifier
- **Tool Number**: Machine tool table number
- **Name**: Descriptive name
- **Type**: Tool type classification
- **Material**: HSS, Carbide, Coated Carbide, Diamond

#### Geometry
- **Diameter**: Tool diameter in mm
- **Flute Length**: Cutting length in mm
- **Overall Length**: Total tool length in mm
- **Shaft Diameter**: Shank diameter in mm
- **Flute Count**: Number of flutes/cutting edges

#### Cutting Parameters (Per Material)
- **RPM Range**: Minimum and maximum spindle speeds
- **Feed Rate Range**: Minimum and maximum feed rates (mm/min)
- **Depth of Cut (DOC)**: Maximum recommended depth
- **Width of Cut (WOC)**: Stepover percentage
- **Plunge Rate**: Percentage of feed rate
- **Coolant**: Recommended coolant type

#### Additional Data
- **Manufacturer**: Tool manufacturer name
- **Part Number**: Manufacturer part number
- **Coating**: Tool coating (TiN, TiAlN, etc.)
- **Purchase Information**: URL and cost
- **Usage Tracking**: Total runtime and last used date
- **Notes**: Custom notes and recommendations

## Persistence

Custom tools are automatically saved to disk and restored when the application starts:

- **Storage Location**: `~/.config/gcodekit4/custom_tools.json` (Linux/macOS) or `%APPDATA%\gcodekit4\custom_tools.json` (Windows)
- **Auto-Save**: Tools are saved automatically when created, updated, or deleted
- **Format**: JSON format for easy backup and manual editing if needed
- **Standard Library**: Built-in standard tools are always available and cannot be deleted

**Note**: Only custom tools (those created by users) are saved to disk. Standard library tools are loaded from code on each startup.

## Backend Integration

### ToolsManagerBackend

The backend provides helper functions for managing the tools database with automatic persistence:

```rust
use gcodekit4::ui::ToolsManagerBackend;
use gcodekit4::data::tools::{Tool, ToolId, ToolType, ToolMaterial};

// Create a backend instance
let mut backend = ToolsManagerBackend::new();

// Get all tools
let tools = backend.get_all_tools();

// Search tools
let results = backend.search_tools("end mill");

// Filter by type
let end_mills = backend.filter_by_type(ToolType::EndMillFlat);

// Filter by diameter (6.35mm ± 0.1mm tolerance)
let tools_6mm = backend.filter_by_diameter(6.35, 0.1);

// Add a tool
let tool = Tool::new(
    ToolId("custom_em_8mm".to_string()),
    101, // tool number
    "8mm Carbide End Mill".to_string(),
    ToolType::EndMillFlat,
    8.0,  // diameter
    50.0, // length
);
backend.add_tool(tool);

// Remove a tool
backend.remove_tool(&ToolId("custom_em_8mm".to_string()));

// Import tools from GTC package
let result = backend.import_gtc_package("supplier_catalog.zip")?;
println!("Imported {} of {} tools", result.imported_tools.len(), result.total_tools);

// Import from GTC JSON file
let result = backend.import_gtc_json("catalog.json")?;
```

### Helper Functions

The backend provides conversion functions for UI values:

- `string_to_tool_type()`: Convert type name to `ToolType` enum
- `string_to_tool_material()`: Convert material name to `ToolMaterial` enum

## Standard Tools Library

The system comes with a standard library of common CNC tools:

- **3.175mm (1/8") Carbide End Mill**: 2-flute, general purpose
- **6.35mm (1/4") Carbide End Mill**: 2-flute, versatile
- **6mm HSS Drill**: Standard twist drill
- **1/4" Ball Nose**: For 3D contours
- **90° V-Bit**: For engraving and chamfering

## Tool Calculations

### Surface Speed (SFM)

```rust
let tool = backend.get_tool(&tool_id).unwrap();
let sfm = tool.surface_speed_sfm(18000); // at 18,000 RPM
```

### Chip Load Per Tooth

```rust
let chip_load = tool.chip_load(1000.0, 18000); // feed rate, RPM
```

## Usage Tracking

The system automatically tracks:
- **Total Runtime**: Cumulative machining time
- **Last Used Date**: When the tool was last used
- **Tool Wear**: (Future feature) Track tool condition

## GTC Import

Import complete tool catalogs from suppliers using the Generic Tool Catalog (GTC) format:

```rust
// Import from ZIP package
let result = backend.import_gtc_package("harvey_tools.zip")?;

// Import from JSON file  
let result = backend.import_gtc_json("catalog.json")?;

// Check results
println!("Imported: {}/{}", result.imported_tools.len(), result.total_tools);
if result.skipped_tools > 0 {
    println!("Errors: {:?}", result.errors);
}
```

See [GTC Import Guide](gtc_import.md) for detailed information.

## File Locations

- **Backend Logic**: `src/ui/tools_manager_backend.rs`
- **Data Models**: `src/data/tools.rs`
- **GTC Import**: `src/data/gtc_import.rs`
- **Storage**: `~/.config/gcodekit4/custom_tools.json`

## Future Enhancements

Planned improvements include:

1. **Tool Life Management**: Track tool wear and replacement schedules
2. **Recommended Parameters**: Auto-suggest cutting parameters based on material
3. **Tool Library Import/Export**: Share tool libraries between machines
4. **Tool Presets**: Quick selection from common tool configurations
5. **Cost Tracking**: Monitor tool costs and usage economics
6. **3D Tool Visualization**: Visual representation of tool geometry
7. **Feeds & Speeds Calculator**: Integrated calculation of optimal parameters

## Integration with CAM Operations

The Tools Manager integrates with CAM operations to:

- Provide available tools for operation planning
- Supply recommended cutting parameters based on tool and material
- Track tool usage across multiple jobs
- Warn about tool availability and condition

## Example JSON Storage

```json
[
  {
    "id": "custom_em_10mm",
    "number": 10,
    "name": "10mm 4-Flute Carbide End Mill",
    "tool_type": "EndMillFlat",
    "diameter": 10.0,
    "length": 75.0,
    "shaft_diameter": 10.0,
    "flutes": 4,
    "material": "Carbide",
    "coating": "TiAlN",
    "manufacturer": "Harvey Tool",
    "part_number": "HT-935060",
    "cutting_params": {
      "Aluminum": {
        "rpm_min": 12000,
        "rpm_max": 18000,
        "feed_rate_min": 1500.0,
        "feed_rate_max": 2500.0,
        "doc_max": 5.0,
        "woc_percent_min": 40.0,
        "woc_percent_max": 50.0,
        "plunge_rate_percent": 25.0,
        "coolant": "Flood or mist"
      }
    },
    "custom": true,
    "total_runtime_minutes": 125.5,
    "notes": "Excellent for aluminum profiling"
  }
]
```

## See Also

- [GTC Import Guide](gtc_import.md)
- [Materials Manager](materials_manager.md)
- [CAM Operations](cam_operations.md)
- [Feeds & Speeds Guide](feeds_speeds.md)
