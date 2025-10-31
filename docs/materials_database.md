# Materials Database Guide

## Overview

The Materials Database is a comprehensive resource management system that provides CNC machine operators with material-specific machining parameters, properties, and best practices. It integrates with CAM tools to enable intelligent parameter selection and optimization.

## Architecture

### Core Components

#### 1. Material Categories
Materials are organized into seven primary categories:
- **Wood**: Natural hardwoods, softwoods, and specialty woods
- **Engineered Wood**: MDF, plywood, particleboard, and composite panels
- **Plastic**: Acrylics, engineering plastics, foams, and specialty polymers
- **Non-Ferrous Metals**: Aluminum, brass, copper, bronze, titanium
- **Ferrous Metals**: Steel, stainless steel, tool steel
- **Composite**: Carbon fiber, fiberglass, phenolic, aramids
- **Stone & Ceramic**: Marble, granite, tile, glass, corian

### Material Properties

Each material stores comprehensive information across four domains:

#### Physical Properties
- Density (kg/m³)
- Tensile strength (MPa)
- Melting point / Glass transition temperature (°C)
- Optional custom properties

#### Machining Characteristics
- **Chip Type**: Continuous, Segmented, Granular, or Small
- **Heat Sensitivity**: Low, Moderate, or High
- **Abrasiveness**: Low, Moderate, or High (tool wear factor)
- **Surface Finish**: Excellent, Good, Fair, or Rough
- **Machinability Rating**: 1-10 scale (higher = easier to machine)

#### Safety Information
- **Dust Hazard**: Health risk from dust (None, Minimal, Moderate, High)
- **Fume Hazard**: Health risk from fumes/off-gassing
- **PPE Requirements**: Eye protection, respiratory, hearing, gloves, apron
- **Coolant Requirements**: Whether coolant is needed

#### Cutting Parameters
Stored per tool type, includes:
- RPM range (minimum, maximum)
- Feed rate range (mm/min for roughing)
- Plunge rate (as % of feed rate)
- Maximum depth of cut (DOC)
- Stepover range (as % of tool diameter)
- Recommended coolant type
- Additional notes

### Data Model

```rust
pub struct Material {
    pub id: MaterialId,                    // Unique identifier
    pub name: String,                      // Display name
    pub category: MaterialCategory,        // Material category
    pub subcategory: String,               // E.g., "Red Oak" for hardwood
    
    // Physical properties
    pub density: f32,                      // kg/m³
    pub machinability_rating: u8,          // 1-10
    pub tensile_strength: Option<f32>,     // MPa
    pub melting_point: Option<f32>,        // °C
    
    // Machining characteristics
    pub chip_type: ChipType,
    pub heat_sensitivity: HeatSensitivity,
    pub abrasiveness: Abrasiveness,
    pub surface_finish: SurfaceFinishability,
    
    // Safety
    pub dust_hazard: HazardLevel,
    pub fume_hazard: HazardLevel,
    pub required_ppe: Vec<PPE>,
    pub coolant_required: bool,
    
    // Cutting parameters by tool type
    pub cutting_params: HashMap<String, CuttingParameters>,
    
    // Metadata
    pub custom: bool,                      // User-defined material
    pub notes: String,                     // Tips and tricks
}

pub struct CuttingParameters {
    pub rpm_range: (u32, u32),
    pub feed_rate_range: (f32, f32),      // mm/min
    pub plunge_rate_percent: f32,          // 0-100%
    pub max_doc: f32,                      // mm
    pub stepover_percent: (f32, f32),      // Min-max as % of tool diameter
    pub coolant_type: CoolantType,
    pub notes: String,
}
```

## Usage Examples

### Creating a Material Library

```rust
use gcodekit4::data::materials::*;

// Initialize with standard materials
let library = init_standard_library();

// Or create an empty library and add materials
let mut library = MaterialLibrary::new();
let mut oak = Material::new(
    MaterialId("wood_oak".to_string()),
    "Oak".to_string(),
    MaterialCategory::Wood,
    "Hardwood".to_string(),
);

library.add_material(oak);
```

### Querying Materials

```rust
// Search by name
let results = library.search_by_name("oak");

// Get by category
let woods = library.get_materials_by_category(MaterialCategory::Wood);

// Get specific material
let material = library.get_material(&MaterialId("wood_oak_red".to_string()));
```

### Working with Cutting Parameters

```rust
// Get cutting parameters for a tool type
if let Some(params) = material.get_cutting_params("endmill_flat") {
    println!("RPM: {:?}", params.rpm_range);
    println!("Feed: {:?} mm/min", params.feed_rate_range);
    println!("Max DOC: {} mm", params.max_doc);
}

// Set custom parameters
let mut params = CuttingParameters::default();
params.rpm_range = (16000, 20000);
params.feed_rate_range = (1200.0, 2000.0);
params.max_doc = 6.0;
material.set_cutting_params("endmill_flat".to_string(), params);
```

## Standard Materials Library

### Phase 1 Included Materials

#### Woods
- **Red Oak**: Density 750 kg/m³, Machinability 8/10
  - Good grain structure, moderate tool wear
  - Endmill parameters: 16,000-20,000 RPM, 1200-2000 mm/min feed

#### Non-Ferrous Metals
- **Aluminum 6061**: Density 2700 kg/m³, Machinability 9/10
  - Excellent machinability, requires coolant
  - Endmill parameters: 3,000-5,000 RPM, 1500-3000 mm/min feed with water-soluble coolant

#### Plastics
- **Acrylic (PMMA)**: Density 1190 kg/m³, Machinability 9/10
  - Excellent surface finish, high heat sensitivity
  - Keep speeds high, feed moderate, air cooling only
  - Endmill parameters: 18,000-24,000 RPM, 1000-1800 mm/min feed

## Integration with CAM Tools

### Workflow

1. **Material Selection**: User selects material in CAM design panel
2. **Tool Suggestion**: System recommends compatible tools based on material properties
3. **Parameter Auto-Fill**: Cutting parameters automatically populate from material definition
4. **Validation**: System warns if parameters are outside recommended ranges
5. **Execution**: Validated parameters passed to controller

### Benefits

- Eliminates guesswork in parameter selection
- Reduces tool breakage and material waste
- Improves surface finish quality
- Faster project setup
- Educational resource for CNC operators

## Safety Considerations

### PPE Requirements

Each material defines required Personal Protective Equipment:
- **Eye Protection**: Always required (safety glasses/face shield)
- **Respiratory**: For materials producing harmful dust or fumes
- **Hearing Protection**: For high-speed or extended operations
- **Gloves**: When appropriate
- **Apron**: For protective covering

### Material Safety Properties

- **Dust Hazard**: Indicates potential respiratory health risks
- **Fume Hazard**: Off-gassing or combustion risks
- **Coolant Requirements**: Necessary for safe operation

## Future Enhancements

### Phase 2: Properties & Parameters
- Expanded material library (50+ materials)
- Complete property definitions
- Tool-material compatibility matrix
- Material preview images/swatches

### Phase 3: Integration
- Bidirectional CAM Tools palette linking
- Smart parameter selection
- Override warnings
- Parameter validation

### Phase 4: Advanced Features
- Custom material creation and editing
- Community material sharing
- Cost tracking and inventory management
- Material testing and iteration tracking

### Phase 5: Documentation & Tools
- Material guides and best practices
- Feeds and speeds calculator
- Chip load calculator
- Tool life estimator
- Video tutorials

## Testing

The materials module includes comprehensive test coverage:

### Unit Tests (9 tests)
- Material creation and properties
- Machinability descriptions
- Cutting parameters storage
- Library operations

### Integration Tests (22 tests)
- Full material library operations
- Search and filtering
- Category-based retrieval
- Mutable access and modification
- Parameter management
- Safety requirements

### Running Tests

```bash
# Run all material tests
cargo test data::materials
cargo test test_materials_database

# Run specific test
cargo test test_aluminum_material_properties
```

## API Reference

### Material Functions

- `Material::new()` - Create new material
- `get_cutting_params()` - Retrieve parameters for tool type
- `set_cutting_params()` - Store parameters for tool type
- `machinability_desc()` - Get human-readable machinability level

### MaterialLibrary Functions

- `MaterialLibrary::new()` - Create empty library
- `add_material()` - Add material to library
- `get_material()` - Retrieve material by ID
- `get_material_mut()` - Mutable access to material
- `remove_material()` - Remove material from library
- `get_all_materials()` - Get all materials
- `get_materials_by_category()` - Filter by category
- `search_by_name()` - Search materials by name
- `len()` / `is_empty()` - Library size queries

### Initialization

- `init_standard_library()` - Load standard materials

## File Format

Materials can be saved and loaded from `.gk4materials` files (JSON/TOML format):

```json
{
  "materials": [
    {
      "id": "wood_oak_red",
      "name": "Red Oak",
      "category": "Wood",
      "subcategory": "Hardwood",
      "density": 750.0,
      "machinability_rating": 8,
      "chip_type": "Continuous",
      "heat_sensitivity": "Low",
      "abrasiveness": "Low",
      "surface_finish": "Good",
      "dust_hazard": "Minimal",
      "fume_hazard": "None",
      "coolant_required": false,
      "custom": false,
      "cutting_params": {
        "endmill_flat": {
          "rpm_range": [16000, 20000],
          "feed_rate_range": [1200.0, 2000.0],
          "plunge_rate_percent": 50.0,
          "max_doc": 6.0,
          "stepover_percent": [40.0, 60.0],
          "coolant_type": "None"
        }
      }
    }
  ]
}
```

## Contributing Custom Materials

Users can extend the materials library with custom materials:

1. Create new Material instance
2. Define all properties and safety information
3. Add cutting parameters for each tool type
4. Set `custom: true` flag
5. Add to library

Custom materials can be saved separately and shared with the community.
