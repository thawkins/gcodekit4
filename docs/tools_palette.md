# CAM Tools Palette Guide

## Overview

The CAM Tools Palette is a comprehensive tool library management system that provides CNC machine operators with detailed tool specifications, cutting parameters, and material compatibility information. It integrates with the Materials Database and Designer to enable intelligent tool selection and parameter optimization.

## Architecture

### Core Components

#### 1. Tool Types
Tools are classified into nine primary types:
- **End Mill Flat**: General purpose cutting, flat bottom
- **End Mill Ball**: Curved surfaces, contouring
- **End Mill Corner Radius**: Smooth edges with radius
- **V-Bit**: Engraving, 2D lettering, angled cuts
- **Drill Bit**: Hole drilling
- **Spot Drill**: Center drilling, countersinking
- **Engraving Bit**: Fine detail work, small diameter
- **Chamfer Tool**: Edge beveling
- **Specialty**: Custom or uncommon tools

#### 2. Tool Materials
- **HSS** (High Speed Steel): General purpose, affordable
- **Carbide**: High-speed, longer life, more brittle
- **Coated Carbide**: Improved performance with coatings
- **Diamond**: Hardest, used for difficult materials

#### 3. Tool Coatings
- **TiN** (Titanium Nitride): Golden color, improved life
- **TiAlN** (Titanium Aluminum Nitride): Better high-temp performance
- **DLC** (Diamond-Like Carbon): Superior performance
- **Al2O3** (Aluminum Oxide): Ceramic coating

### Tool Properties

Each tool stores comprehensive specifications:

#### Geometry
- Cutting diameter (mm)
- Shaft diameter (mm)
- Overall length (mm)
- Flute length (mm)
- Number of flutes (1-4+)
- Corner radius (for radius end mills)
- Tip angle (for v-bits, drills)

#### Material Specifications
- Tool material composition
- Coating type
- Shank type (straight, tapered, collet)

#### Cutting Parameters
- Recommended RPM
- RPM range (min, max)
- Default feed rate (mm/min)
- Default plunge rate (mm/min)
- Stepover percentage
- Depth per pass (mm)

#### Metadata
- Tool number (for reference)
- Manufacturer name
- Part number
- Cost per unit
- Notes and tips
- Custom tool flag

### Data Model

```rust
pub struct Tool {
    pub id: ToolId,                    // Unique identifier
    pub number: u32,                   // Tool number for reference
    pub name: String,                  // Display name
    pub description: String,           // Detailed description
    pub tool_type: ToolType,
    
    // Geometry
    pub diameter: f32,                 // mm
    pub shaft_diameter: Option<f32>,
    pub length: f32,                   // mm
    pub flute_length: f32,             // mm
    pub flutes: u32,
    pub corner_radius: Option<f32>,    // mm
    pub tip_angle: Option<f32>,        // degrees
    
    // Material specs
    pub material: ToolMaterial,
    pub coating: Option<ToolCoating>,
    pub shank: ShankType,
    
    // Parameters
    pub params: ToolCuttingParams,
    
    // Metadata
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
    pub cost: Option<f32>,
    pub notes: String,
    pub custom: bool,
}

pub struct ToolCuttingParams {
    pub rpm: u32,
    pub rpm_range: (u32, u32),
    pub feed_rate: f32,        // mm/min
    pub plunge_rate: f32,      // mm/min
    pub stepover_percent: f32,  // % of diameter
    pub depth_per_pass: f32,    // mm
}
```

## Usage Examples

### Creating a Tool Library

```rust
use gcodekit4::data::tools::*;

// Initialize with standard tools
let library = init_standard_library();

// Or create an empty library and add tools
let mut library = ToolLibrary::new();
let mut tool = Tool::new(
    ToolId("custom_tool".to_string()),
    10,
    "Custom Tool".to_string(),
    ToolType::EndMillFlat,
    6.35,
    50.0,
);

library.add_tool(tool);
```

### Querying Tools

```rust
// Search by name
let flat_mills = library.search_by_name("flat");

// Get by type
let vbits = library.get_tools_by_type(ToolType::VBit);

// Search by diameter range
let small_tools = library.search_by_diameter(0.0, 4.0);

// Get specific tool
let tool = library.get_tool(&ToolId("tool_1_4_flat".to_string()));
```

### Working with Tool Parameters

```rust
// Get cutting parameters
if let Some(tool) = library.get_tool(&tool_id) {
    println!("RPM: {:?}", tool.params.rpm_range);
    println!("Feed: {} mm/min", tool.params.feed_rate);
    println!("Plunge: {} mm/min", tool.params.plunge_rate);
}

// Modify parameters
if let Some(tool) = library.get_tool_mut(&tool_id) {
    tool.params.rpm = 20000;
    tool.params.feed_rate = 2000.0;
}
```

## Standard Tools Library

### Phase 1 Included Tools

#### End Mills
- **1/4" Flat End Mill** (Tool #1)
  - 2 flutes, carbide with TiN coating
  - RPM: 12,000-24,000 (recommended 18,000)
  - Feed: 1500 mm/min

- **1/8" Flat End Mill** (Tool #2)
  - 2 flutes, carbide with TiN coating
  - RPM: 18,000-30,000 (recommended 24,000)
  - Feed: 1000 mm/min

- **1/8" Ball End Mill** (Tool #5)
  - 2 flutes, carbide with TiAlN coating
  - RPM: 16,000-28,000 (recommended 22,000)
  - Feed: 1200 mm/min
  - Stepover: 20% for smooth curves

#### V-Bits
- **90° V-Bit** (Tool #3)
  - 1 flute, carbide with TiN coating
  - RPM: 15,000-25,000 (recommended 20,000)
  - Feed: 1200 mm/min
  - Depth per pass: 2mm (for engraving)

#### Drill Bits
- **1/4" Drill Bit** (Tool #4)
  - 2 flutes, HSS (High Speed Steel)
  - RPM: 2,000-4,000 (recommended 3,000)
  - Feed: 300 mm/min (pecking recommended)
  - Tip angle: 118°

## Integration with Materials Database

### Workflow

1. **Tool Selection**: User selects a tool from the palette
2. **Material Selection**: User specifies material being cut
3. **Parameter Lookup**: System retrieves cutting parameters for tool+material combination
4. **Validation**: Parameters are validated against safety limits
5. **Execution**: Validated parameters used in G-code generation

### Tool-Material Compatibility

Tools can be marked as suitable/unsuitable for specific materials:
- Carbide end mills: Good for wood, plastics, soft metals
- HSS drill bits: General purpose, metals
- Diamond tools: Hard materials, ceramics, composites
- Specialized tools: Specific applications

## File Format

Tools can be saved and loaded from `.gk4tools` files (JSON/TOML format):

```json
{
  "tools": [
    {
      "id": "tool_1_4_flat",
      "number": 1,
      "name": "1/4\" Flat End Mill",
      "tool_type": "EndMillFlat",
      "diameter": 6.35,
      "length": 50.0,
      "flute_length": 40.0,
      "flutes": 2,
      "material": "Carbide",
      "coating": "TiN",
      "params": {
        "rpm": 18000,
        "rpm_range": [12000, 24000],
        "feed_rate": 1500.0,
        "plunge_rate": 750.0,
        "stepover_percent": 50.0,
        "depth_per_pass": 3.0
      },
      "manufacturer": "Generic",
      "custom": false
    }
  ]
}
```

## API Reference

### Tool Functions

- `Tool::new()` - Create new tool with basic properties
- `description_short()` - Get brief tool description
- `is_suitable_for_material()` - Check material compatibility

### ToolLibrary Functions

- `ToolLibrary::new()` - Create empty library
- `add_tool()` - Add tool to library
- `get_tool()` - Retrieve tool by ID
- `get_tool_mut()` - Mutable access to tool
- `remove_tool()` - Remove tool from library
- `get_all_tools()` - Get all tools
- `get_tools_by_type()` - Filter by tool type
- `search_by_name()` - Search tools by name
- `search_by_diameter()` - Search by diameter range
- `len()` / `is_empty()` - Library size queries
- `next_tool_number()` - Get next available tool number

### Initialization

- `init_standard_library()` - Load standard tools

## Testing

The tools module includes comprehensive test coverage:

### Unit Tests (16 tests)
- Tool creation and properties
- Tool type and material enums
- Cutting parameters
- Library operations

### Integration Tests (27 tests)
- Full tool library operations
- Search and filtering
- Type-based retrieval
- Diameter range search
- Mutable access and modification
- Parameter management
- Material and coating compatibility
- Manufacturer and cost tracking

### Running Tests

```bash
# Run all tool tests
cargo test data::tools
cargo test test_tools_palette

# Run specific test
cargo test test_tool_properties
```

## Future Enhancements

### Phase 2: Parameters & Materials
- Material-specific tool parameters
- Parameter validation rules
- Feed/speed calculators
- Tool life estimation
- Tool preview rendering

### Phase 3: Organization & Search
- Advanced search and filtering
- Tool collections/favorites
- Category organization
- Import/export functionality
- Bulk operations

### Phase 4: Integration
- Designer tool integration
- G-code generation integration
- Visualizer tool display
- Automatic parameter population
- Tool change command generation

### Phase 5: Advanced Features
- Tool life tracking
- Presets and templates
- Community tool libraries
- Usage analytics
- Cost tracking

## Best Practices

### Tool Selection
1. Choose appropriate tool type for operation
2. Select carbide for production, HSS for flexibility
3. Consider material hardness and abrasiveness
4. Check tool diameter for minimum feature size

### Parameter Optimization
1. Start with recommended parameters
2. Reduce feed rate for poor surface finish
3. Increase speed if tool chatter occurs
4. Use coated tools for extended life
5. Regular tool inspection for wear

### Safety
1. Never exceed RPM limits
2. Plunge rate should be 40-60% of feed rate
3. Depth per pass should not exceed diameter
4. Ensure adequate chip evacuation
5. Use appropriate cooling/lubrication

## Contributing Custom Tools

Users can extend the tool library with custom tools:

1. Create new Tool instance
2. Define all geometric and material properties
3. Set cutting parameters for typical materials
4. Set `custom: true` flag
5. Add to library
6. Optional: Export and share with community
