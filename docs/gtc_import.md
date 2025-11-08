# GTC (Generic Tool Catalog) Import

The GTC Import feature allows you to import tool catalogs from tool suppliers in the industry-standard Generic Tool Catalog format.

## Overview

GTC is a standardized format for exchanging tool catalog data between CAM systems and tool suppliers. It enables easy import of complete tool libraries with specifications and cutting parameters.

## Supported Formats

### GTC Package (.zip)
- Complete tool catalog in a ZIP archive
- Contains `catalog.json`, `tools.json`, or `gtc.json`
- May include additional resources (images, documentation)

### GTC JSON (.json)
- Standalone JSON catalog file
- Direct import without ZIP extraction

## GTC Structure

### Catalog Format

```json
{
  "Version": "1.0",
  "Manufacturer": "Tool Company Name",
  "Tools": [
    {
      "ID": "TOOL-001",
      "Description": "Tool description",
      "Type": "End Mill",
      "Diameter": 6.0,
      "Length": 50.0,
      "FluteLength": 20.0,
      "ShankDiameter": 6.0,
      "NumberOfFlutes": 2,
      "Material": "Carbide",
      "Coating": "TiAlN",
      "Manufacturer": "Company",
      "PartNumber": "PART-001",
      "CuttingParameters": {
        "RPM": 18000,
        "FeedRate": 1200.0,
        "PlungeRate": 300.0,
        "Material": "Aluminum"
      }
    }
  ]
}
```

## Supported Tool Types

The GTC importer automatically maps tool types to the internal format:

| GTC Type | Mapped To |
|----------|-----------|
| "End Mill", "Endmill" | Flat End Mill |
| "Ball End Mill", "Ball Nose" | Ball Nose End Mill |
| "Corner Radius End Mill" | Corner Radius End Mill |
| "Drill", "Drill Bit" | Drill Bit |
| "Center Drill", "Spot Drill" | Spot Drill |
| "V-Bit", "V Bit" | V-Bit |
| "Engraving Bit", "Engraver" | Engraving Bit |
| "Chamfer" | Chamfer Tool |
| Other types | Specialty |

## Material Mapping

| GTC Material | Mapped To |
|--------------|-----------|
| "Carbide", "Solid Carbide" | Carbide |
| "Coated Carbide" | Coated Carbide |
| "HSS", "High Speed Steel" | HSS |
| "Diamond" | Diamond |
| Other | Carbide (default) |

## Coating Mapping

| GTC Coating | Mapped To |
|-------------|-----------|
| "TiAlN" | TiAlN |
| "TiN" | TiN |
| "DLC", "Diamond" | DLC |
| "AlOx", "Aluminum Oxide" | AlOx |
| Other | TiN (default) |

## Usage

### Backend API

```rust
use gcodekit4::ui::ToolsManagerBackend;

let mut backend = ToolsManagerBackend::new();

// Import from ZIP package
match backend.import_gtc_package("tools_catalog.zip") {
    Ok(result) => {
        println!("Imported {} of {} tools", 
                 result.imported_tools.len(), 
                 result.total_tools);
        
        if result.skipped_tools > 0 {
            println!("Skipped {} tools", result.skipped_tools);
            for error in &result.errors {
                println!("  Error: {}", error);
            }
        }
    }
    Err(e) => eprintln!("Import failed: {}", e),
}

// Import from JSON file
match backend.import_gtc_json("catalog.json") {
    Ok(result) => {
        println!("Successfully imported {} tools", result.imported_tools.len());
    }
    Err(e) => eprintln!("Import failed: {}", e),
}
```

### Import Result

The import operation returns a `GtcImportResult` with:

- **total_tools**: Total number of tools in the catalog
- **imported_tools**: Vector of successfully imported tools
- **skipped_tools**: Number of tools that failed to import
- **errors**: Vector of error messages for skipped tools

## Tool Number Assignment

- Imported tools are automatically assigned tool numbers
- Starting from the highest existing tool number + 1
- Prevents conflicts with existing tools

## Custom Flag

All imported tools are marked as `custom = true`:
- Included in persistence (saved to `custom_tools.json`)
- Can be deleted by user
- Distinguished from standard library tools

## Example Workflow

### 1. Obtain GTC Package

Download from tool manufacturer:
- Harvey Tool: `harvey_carbide_catalog.zip`
- Datron: `datron_tool_library.zip`
- Kodiak: `kodiak_cutting_tools.zip`

### 2. Import Package

```rust
let mut backend = ToolsManagerBackend::new();
let result = backend.import_gtc_package("harvey_carbide_catalog.zip")?;
```

### 3. Review Results

```rust
println!("Import Summary:");
println!("  Total: {}", result.total_tools);
println!("  Imported: {}", result.imported_tools.len());
println!("  Skipped: {}", result.skipped_tools);

// Show imported tools
for tool in &result.imported_tools {
    println!("  [{}] {} - Ø{}mm", 
             tool.number, 
             tool.name, 
             tool.diameter);
}

// Show errors
if !result.errors.is_empty() {
    println!("\nErrors:");
    for error in &result.errors {
        println!("  {}", error);
    }
}
```

### 4. Automatic Persistence

- Imported tools are automatically saved to disk
- Stored in `custom_tools.json`
- Available on next application startup

## Creating GTC Catalogs

### For Tool Suppliers

Tool manufacturers can create GTC packages for their customers:

1. **Create catalog.json** with tool specifications
2. **Add optional resources** (images, documentation)
3. **Package as ZIP** with catalog.json at root
4. **Distribute to customers**

### Example Structure

```
harvey_tools.zip
├── catalog.json          (required)
├── images/              (optional)
│   ├── EM-001.png
│   ├── EM-002.png
│   └── ...
├── docs/                (optional)
│   └── specifications.pdf
└── README.txt           (optional)
```

## Error Handling

The importer handles errors gracefully:

### Common Errors

- **File not found**: ZIP or JSON file doesn't exist
- **Invalid format**: JSON parsing errors
- **Missing catalog**: No catalog file in ZIP
- **Invalid tool data**: Missing required fields

### Partial Import

- Import continues even if some tools fail
- Successfully imported tools are retained
- Errors are collected and reported
- Changes are persisted to disk

## Validation

Tools are validated during import:

- **Required fields**: ID, Description, Type, Diameter, Length
- **Type mapping**: Must map to a known tool type
- **Diameter**: Must be > 0
- **Length**: Must be > 0

Invalid tools are skipped and errors reported.

## Integration

### With Materials Manager

- GTC cutting parameters can reference materials
- Material-specific parameters preserved
- Future: Auto-link to Materials Manager database

### With CAM Operations

- Imported tools immediately available for operations
- Cutting parameters can guide operation setup
- Tool selection enhanced with supplier catalogs

## Limitations

### Current Version

- Cutting parameters imported but not yet fully integrated
- Image/document resources in ZIP not extracted
- Single material per cutting parameter set
- Tool geometry limited to basic parameters

### Future Enhancements

1. **Enhanced Cutting Parameters**
   - Multiple material support
   - Chip load calculations
   - Surface speed recommendations

2. **Resource Extraction**
   - Extract and display tool images
   - Link to documentation PDFs
   - 3D tool models

3. **Advanced Tool Geometry**
   - Corner radius specifications
   - V-bit angles
   - Thread mill parameters

4. **Supplier Integration**
   - Direct download from suppliers
   - Automatic updates
   - Version tracking

## Tool Supplier Catalogs

### Compatible Suppliers

GTC format is supported by many major tool manufacturers:

- Harvey Tool
- Datron
- Kodiak Cutting Tools
- Kennametal
- Sandvik Coromant
- Many others

### Requesting GTC Catalogs

Contact your tool supplier and request:
- "Generic Tool Catalog format"
- "GTC package for CAM import"
- "Tool library in JSON format"

## See Also

- [Tools Manager](tools_manager.md)
- [Materials Manager](materials_manager.md)
- [Example GTC Catalog](gtc_example.json)

## Technical Reference

### File Locations

- **GTC Import Module**: `src/data/gtc_import.rs`
- **Backend Integration**: `src/ui/tools_manager_backend.rs`
- **Example Catalog**: `docs/gtc_example.json`

### Data Models

- `GtcCatalog`: Top-level catalog structure
- `GtcTool`: Individual tool definition
- `GtcCuttingParams`: Material-specific parameters
- `GtcImportResult`: Import operation result

### Error Types

Import operations return `Result<GtcImportResult, Box<dyn std::error::Error>>`:
- IO errors (file not found, permission denied)
- ZIP errors (corrupt archive, invalid format)
- JSON errors (parse failures, invalid structure)
