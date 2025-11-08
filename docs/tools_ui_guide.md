# CNC Tools Manager UI Guide

Complete guide to using the CNC Tools Manager interface.

## Overview

The CNC Tools Manager provides a comprehensive interface for managing your CNC tool library with full CRUD (Create, Read, Update, Delete) capabilities and GTC catalog import.

## Accessing the Tools Manager

1. Launch GCodeKit4
2. Click the **🔩 CNC Tools** tab in the top navigation bar
3. The Tools Manager interface will open

## Interface Layout

```
┌─────────────────────────────────────────────────────────────┐
│ CNC Tools Manager    [Search...]  [➕ New Tool] [📦 Import] │
├──────────────┬──────────────────────────────────────────────┤
│              │                                              │
│  Tool List   │         Details / Edit Form                  │
│  (Left)      │              (Right)                         │
│              │                                              │
└──────────────┴──────────────────────────────────────────────┘
```

### Left Panel: Tool List

Displays all tools as cards showing:
- **T#** - Tool number
- **Name** - Tool name
- **Type** - Tool type (End Mill, Drill, etc.)
- **Specs** - Diameter × Length
- **Material** - HSS, Carbide, etc.
- **Coating** - TiN, TiAlN, etc. (if applicable)
- **Custom badge** - For user-created tools

**Interactions:**
- **Single-click** - Select tool
- **Double-click** - Open for editing

### Right Panel: Details/Edit Form

Shows either:
- Empty state with instructions (no tool selected)
- Edit form (when creating/editing a tool)

## Creating a New Tool

1. Click **➕ New Tool** button
2. Fill in the form:

   **Basic Information:**
   - Tool Number (T#)
   - Tool Name
   - Tool Type (dropdown)
   - Material (dropdown)
   - Coating (dropdown)

   **Tool Geometry:**
   - Diameter (mm)
   - Overall Length (mm)
   - Flute Length (mm)
   - Shaft Diameter (mm)
   - Number of Flutes

   **Manufacturer Information:**
   - Manufacturer name
   - Part Number
   - Description

   **Notes:**
   - Any additional information

3. Click **💾 Save** to create the tool
4. The tool appears in the list and is automatically saved to disk

## Editing a Tool

1. **Double-click** a tool in the list
2. Modify any fields in the edit form
3. Click **💾 Save** to update
4. Click **❌ Cancel** to discard changes

**Note:** Standard library tools (non-custom) can be viewed but cannot be deleted.

## Deleting a Tool

1. Select a custom tool (has "Custom" badge)
2. Double-click to open for editing
3. Click **🗑️ Delete** button
4. The tool is removed from the list and storage

**Note:** Only custom tools can be deleted. Standard library tools are protected.

## Searching Tools

1. Type in the **Search** box at the top
2. Results filter automatically as you type
3. Search looks in tool names
4. Clear the search box to see all tools

## Filtering by Type

1. Click the **Filter by Type** dropdown
2. Select a tool type:
   - All Types
   - Flat End Mill
   - Ball End Mill
   - Corner Radius End Mill
   - V-Bit
   - Drill Bit
   - Spot Drill
   - Engraving Bit
   - Chamfer Tool
   - Specialty

3. List shows only tools of selected type
4. Select "All Types" to see everything

## Importing GTC Catalogs

The **📦 Import GTC** button allows you to import tool catalogs from suppliers.

### Import Process

1. Click **📦 Import GTC** button
2. A file dialog opens
3. Select one of:
   - **GTC Package** (.zip file)
   - **GTC JSON** (.json file)
4. Click **Open**
5. Import process runs automatically
6. Results are logged:
   - Number of tools imported
   - Number of tools skipped
   - Any errors encountered
7. Tool list refreshes with newly imported tools

### Import Results

After import:
- ✅ **Successful** - New tools appear in the list
- ⚠️ **Partial** - Some tools imported, some skipped (check logs)
- ❌ **Failed** - No tools imported (check error message)

### Supported File Formats

**ZIP Packages:**
- Standard GTC distribution format
- Contains `catalog.json`, `tools.json`, or `gtc.json`
- May include additional resources (ignored)
- Example: `harvey_tools.zip`

**JSON Files:**
- Direct JSON catalog
- Faster for single-file catalogs
- Example: `catalog.json`

### Tool Number Assignment

Imported tools are automatically assigned numbers:
- Starting from highest existing number + 1
- Prevents conflicts with existing tools
- Example: If you have T1-T10, imports start at T11

### Custom Flag

All imported tools are marked as custom:
- Show "Custom" badge
- Saved to persistent storage
- Can be edited and deleted

### Where to Get GTC Catalogs

**Major Suppliers:**
- Harvey Tool
- Datron
- Kodiak Cutting Tools
- Kennametal
- Sandvik Coromant

**How to Request:**
Ask your tool supplier for:
- "Generic Tool Catalog format"
- "GTC package for CAM import"
- "Tool library in JSON format"

### Sample Catalog

A sample GTC catalog is included:
- **Location:** `docs/gtc_example.json`
- **Contents:** 4 example tools
- **Use:** Test import functionality

To test:
1. Click **📦 Import GTC**
2. Navigate to `docs/gtc_example.json`
3. Click Open
4. Verify 4 tools are imported

## Tool Properties

### Basic Information
- **Tool Number** - Unique identifier (T1, T2, etc.)
- **Tool Name** - Descriptive name
- **Tool Type** - Category of tool
- **Material** - Tool construction material
- **Coating** - Surface coating (optional)

### Geometry
- **Diameter** - Cutting diameter in mm
- **Overall Length** - Total tool length in mm
- **Flute Length** - Length of cutting flutes in mm
- **Shaft Diameter** - Shank/shaft diameter in mm
- **Number of Flutes** - Flute count (typically 1-12)

### Manufacturer Info
- **Manufacturer** - Tool maker
- **Part Number** - Manufacturer's part number
- **Description** - Detailed description

### Additional
- **Notes** - User notes and observations
- **Custom Flag** - Whether tool is user-defined

## Persistence

### Automatic Saving

Tools are automatically saved when:
- Creating a new tool
- Updating an existing tool
- Deleting a tool
- Importing from GTC

### Storage Location

Custom tools are stored in:
- **Linux:** `~/.config/gcodekit4/custom_tools.json`
- **macOS:** `~/Library/Application Support/gcodekit4/custom_tools.json`
- **Windows:** `%APPDATA%\gcodekit4\custom_tools.json`

### Backup Recommendation

Periodically backup your custom tools file:
```bash
# Linux/macOS
cp ~/.config/gcodekit4/custom_tools.json ~/Backups/

# Windows
copy %APPDATA%\gcodekit4\custom_tools.json %USERPROFILE%\Backups\
```

## Standard Library Tools

GCodeKit4 includes standard tools:

1. **T1** - 3.175mm (1/8") Carbide End Mill - 2-flute
2. **T2** - 6.35mm (1/4") Carbide End Mill - 2-flute
3. **T3** - 6mm HSS Drill - Standard twist drill
4. **T4** - 6.35mm (1/4") Ball Nose End Mill
5. **T5** - 90° V-Bit - For engraving

**Properties:**
- Cannot be deleted
- Can be duplicated and modified
- Always available
- Not saved to custom_tools.json

## Keyboard Shortcuts

Currently supported:
- **Double-click** - Edit tool
- **Enter** in search - Execute search
- **Escape** in edit - Cancel editing (future)

## Tips and Best Practices

### Organizing Tools

1. **Use consistent naming:**
   - Include diameter and type
   - Example: "6mm 2-Flute End Mill"

2. **Add manufacturer info:**
   - Helps with reordering
   - Useful for warranty claims

3. **Document in notes:**
   - Specific uses
   - Material restrictions
   - Feed rate recommendations

### Import Strategy

1. **Start with supplier catalogs:**
   - Import GTC from your primary supplier
   - Builds library quickly

2. **Add custom tools as needed:**
   - Specialty tools
   - Modified tools
   - Custom ground tools

3. **Keep notes updated:**
   - Document tool performance
   - Record any issues

### Tool Numbering

1. **Reserve ranges:**
   - T1-T10: Standard tools
   - T11-T100: End mills
   - T101-T200: Drills
   - T201-T300: Specialty

2. **Import considerations:**
   - Imports use next available number
   - Can manually edit after import

## Troubleshooting

### GTC Import Fails

**Problem:** Import button does nothing
- **Solution:** Check file permissions, try different location

**Problem:** Some tools skipped
- **Solution:** Check logs for errors, verify JSON format

**Problem:** Wrong tool types
- **Solution:** Tool type mapping issue, edit after import

### Tool Not Appearing

**Problem:** Created tool doesn't show in list
- **Solution:** Click "All Types" filter, check search is clear

**Problem:** Tool disappeared after restart
- **Solution:** Check storage location exists, permissions OK

### Cannot Delete Tool

**Problem:** Delete button is grayed out
- **Solution:** Standard library tools cannot be deleted

**Problem:** Delete fails silently
- **Solution:** Check file permissions on storage location

## Technical Details

### File Format

Custom tools are stored as JSON:
```json
{
  "custom_tools": [
    {
      "id": "custom_1",
      "number": 101,
      "name": "Custom 8mm End Mill",
      "tool_type": "EndMillFlat",
      ...
    }
  ]
}
```

### GTC Format

See [GTC Import Guide](gtc_import.md) for detailed format specification.

## See Also

- [GTC Import Guide](gtc_import.md) - Detailed GTC information
- [Tools Manager Backend](tools_manager.md) - Technical documentation
- [Example GTC Catalog](gtc_example.json) - Sample catalog

## Support

For issues or questions:
1. Check logs in terminal/console
2. Verify file permissions
3. Test with example GTC catalog
4. Report issues with error details
