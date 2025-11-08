# Materials Manager

The Materials Manager provides a comprehensive CRUD (Create, Read, Update, Delete) interface for managing the materials database in GCodeKit4.

## Features

### Material Properties

The Materials Manager allows you to manage the following material properties:

#### Basic Information
- **ID**: Unique material identifier (e.g., `wood_oak_red`, `metal_al_6061`)
- **Name**: Display name of the material
- **Category**: Material category (Wood, Engineered Wood, Plastic, Non-Ferrous Metal, Ferrous Metal, Composite, Stone & Ceramic)
- **Subcategory**: More specific classification (e.g., "Hardwood", "Alloy")
- **Description**: Brief description of the material

#### Physical Properties
- **Density**: Material density in kg/m³
- **Machinability Rating**: Ease of machining on a scale of 1-10
- **Tensile Strength**: Optional tensile strength in MPa
- **Melting Point**: Optional melting or glass transition temperature in °C

#### Machining Characteristics
- **Chip Type**: Type of chips formed during cutting (Continuous, Segmented, Granular, Small)
- **Heat Sensitivity**: Sensitivity to heat during machining (Low, Moderate, High)
- **Abrasiveness**: Tool wear factor (Low, Moderate, High)
- **Surface Finish**: Achievable surface finish quality (Excellent, Good, Fair, Rough)

#### Safety Information
- **Dust Hazard**: Level of dust hazard (None, Minimal, Moderate, High)
- **Fume Hazard**: Level of fume hazard (None, Minimal, Moderate, High)
- **Coolant Required**: Whether coolant is required for machining
- **Required PPE**: Personal protective equipment needed

#### Additional Notes
- **Notes**: Custom notes, tips, and recommendations for working with the material

## Accessing the Materials Manager

The Materials Manager is accessible from:

1. **Tab Bar**: Click the "📦 Materials" tab in the main tab bar
2. **Menu Bar**: View → Materials
3. **Keyboard**: Use the menu shortcut to switch to Materials view

## User Interface

### Left Panel - Materials List

The left panel displays all materials in the library with:
- Material name and category
- Machinability rating with color coding:
  - Green (7-10): Easy to machine
  - Orange (5-6): Moderate difficulty
  - Red (1-4): Difficult to machine
- "Custom" badge for user-defined materials

#### Search and Filter
- **Search Box**: Search materials by name or subcategory
- **Category Filter**: Filter materials by category
- **Refresh Button**: Reload materials from the database

### Right Panel - Material Editor

The right panel provides a tabbed interface for editing material properties:

#### Tabs
1. **Basic Info**: Name, category, subcategory, and description
2. **Properties**: Physical properties (density, machinability, tensile strength, melting point)
3. **Machining**: Machining characteristics (chip type, heat sensitivity, abrasiveness, surface finish)
4. **Safety**: Safety information (dust/fume hazards, coolant requirements, PPE)
5. **Notes**: Additional notes and machining tips

### CRUD Operations

#### Create a New Material
1. Click the "➕ New Material" button
2. Fill in the material ID (required for new materials)
3. Enter material properties across the tabs
4. Click "💾 Save" to add the material to the library

#### Read/View Material
1. Click on any material in the list
2. Properties are displayed in the tabbed interface
3. Navigate between tabs to view different property groups

#### Update a Material
1. Select a material from the list
2. Modify any properties in the editor
3. Click "💾 Save" to update the material

#### Delete a Material
1. Select a custom material from the list
2. Click "🗑️ Delete" to remove it
3. Note: Standard library materials cannot be deleted

## Persistence

Custom materials are automatically saved to disk and restored when the application starts:

- **Storage Location**: `~/.config/gcodekit4/custom_materials.json` (Linux/macOS) or `%APPDATA%\gcodekit4\custom_materials.json` (Windows)
- **Auto-Save**: Materials are saved automatically when created, updated, or deleted
- **Format**: JSON format for easy backup and manual editing if needed
- **Standard Library**: Built-in materials (Red Oak, Aluminum 6061, Acrylic) are always available and cannot be deleted

**Note**: Only custom materials (those created by users) are saved to disk. Standard library materials are loaded from code on each startup.

## Backend Integration

### MaterialsManagerBackend

The backend provides helper functions for managing the materials database with automatic persistence:

```rust
use gcodekit4::ui::MaterialsManagerBackend;
use gcodekit4::data::materials::{Material, MaterialId, MaterialCategory};

// Create a backend instance
let mut backend = MaterialsManagerBackend::new();

// Get all materials
let materials = backend.get_all_materials();

// Search materials
let results = backend.search_materials("aluminum");

// Filter by category
let metals = backend.filter_by_category(MaterialCategory::NonFerrousMetal);

// Add a material
let material = Material::new(
    MaterialId("custom_material".to_string()),
    "Custom Material".to_string(),
    MaterialCategory::Wood,
    "Custom".to_string(),
);
backend.add_material(material);

// Remove a material
backend.remove_material(&MaterialId("custom_material".to_string()));
```

### Helper Functions

The backend provides conversion functions for UI values:

- `string_to_category()`: Convert category name to `MaterialCategory` enum
- `string_to_chip_type()`: Convert chip type name to `ChipType` enum
- `string_to_heat_sensitivity()`: Convert sensitivity level to `HeatSensitivity` enum
- `string_to_abrasiveness()`: Convert abrasiveness level to `Abrasiveness` enum
- `string_to_surface_finish()`: Convert finish quality to `SurfaceFinishability` enum
- `string_to_hazard_level()`: Convert hazard level to `HazardLevel` enum

## Standard Materials Library

The system comes with a standard library of common materials:

- **Red Oak** (Hardwood): Dense American hardwood, good for general CNC work
- **Aluminum 6061** (Alloy): Common aluminum alloy, excellent machinability
- **Acrylic** (PMMA): Clear plastic, good for engraving and cutting

## File Locations

- **UI Component**: `src/ui_panels/materials_manager.slint`
- **Backend Logic**: `src/ui/materials_manager_backend.rs`
- **Data Models**: `src/data/materials.rs`

## Future Enhancements

Planned improvements include:

1. **PPE Management**: Detailed PPE requirement tracking and display
2. **Cutting Parameters**: Per-tool cutting parameter recommendations
3. **Material Database Import/Export**: JSON/CSV import and export
4. **Material Templates**: Quick creation from templates
5. **Supplier Information**: Track material suppliers and pricing
6. **Usage Statistics**: Track which materials are used most frequently

## Integration with Designer

The Materials Manager integrates with the Designer tool to:

- Provide material selection for design projects
- Supply recommended cutting parameters based on material properties
- Display safety warnings for selected materials
- Calculate estimated machining time based on material machinability

## See Also

- [Tool Library Management](tool_library.md)
- [Designer Tool](designer.md)
- [Material Setup Configuration](material_setup.md)
