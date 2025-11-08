# Materials Manager Integration

## Overview

The Materials Manager has been successfully integrated into the main GCodeKit4 UI as a dedicated tab view.

## Integration Points

### 1. Main UI Tab Bar

Added a new "📦 Materials" tab button in the main tab bar alongside existing tabs:
- G-Code Editor
- Device Console
- Designer
- Laser Tools
- CAM Tools
- **Materials** ← NEW

Location: `src/ui.slint` lines 593-613

### 2. Menu Bar Integration

Added Materials menu item under the **View** menu:
- View → Materials

The menu item shows a checkmark (✓) when the Materials view is active.

Location: `src/ui.slint` lines 327-336

### 3. View Rendering

The MaterialsManager component is conditionally rendered when `current-view == "materials"`:

```slint
if current-view == "materials" : MaterialsManager {
}
```

Location: `src/ui.slint` lines 960-962

### 4. Callback Registration

Added callback for view switching:
```slint
callback menu-view-materials();
```

Location: `src/ui.slint` line 181

## UI Layout

```
┌─────────────────────────────────────────────────────────────┐
│ File  Edit  View  Help                                      │
│                            View Menu:                        │
│                            - G-Code Editor                   │
│                            - Machine                         │
│                            - Device Console                  │
│                            - Visualizer                      │
│                            - Designer                        │
│                            - Materials  ← NEW                │
├─────────────────────────────────────────────────────────────┤
│ [G-Code] [Console] [Designer] [🔥Laser] [🔧CAM] [📦Materials]│
│                                                      ↑ NEW   │
├──────────────────┬──────────────────────────────────────────┤
│ Materials (3)    │                                           │
│ ┌──────────────┐ │  Select a material to view details       │
│ │ Red Oak      │ │  or create a new material                │
│ │ Hardwood     │ │                                           │
│ │ Rating: 8/10 │ │                                           │
│ └──────────────┘ │                                           │
│ ┌──────────────┐ │                                           │
│ │ Aluminum 6061│ │  When material selected:                 │
│ │ Alloy        │ │  ┌──────────────────────────────────┐   │
│ │ Rating: 9/10 │ │  │ [Basic] [Properties] [Machining]│   │
│ └──────────────┘ │  │  [Safety] [Notes]                │   │
│                  │  │                                   │   │
│ [➕New Material] │  │  Material properties editor       │   │
│ [🔄Refresh]      │  │  with tabbed interface            │   │
│                  │  │                                   │   │
│                  │  │  [💾Save] [❌Cancel] [🗑️Delete]  │   │
│                  │  └──────────────────────────────────┘   │
└──────────────────┴──────────────────────────────────────────┘
```

## Key Features Available in Tab View

1. **Search & Filter**
   - Search materials by name
   - Filter by category
   - Real-time results

2. **Material List**
   - Displays all materials with color-coded machinability ratings
   - Shows category and subcategory
   - Indicates custom vs. standard materials

3. **Material Editor**
   - 5 organized tabs for different property groups
   - Two-way data binding
   - Validation and type checking

4. **CRUD Operations**
   - Create new materials
   - View/read material properties
   - Update existing materials
   - Delete custom materials

## Navigation

Users can access the Materials Manager through:

1. **Clicking the 📦 Materials tab** in the tab bar
2. **Using the View menu** → Materials
3. **Programmatically** via `current-view = "materials"`

## View State Management

The view state is managed through the `current-view` property:
- Current view indicator in tab bar (highlighted tab)
- Checkmark in menu when active
- Proper component mounting/unmounting

## File Changes

### Modified Files
- `src/ui.slint` - Added Materials tab, view, and menu integration
- `src/ui/mod.rs` - Exported MaterialsManagerBackend module

### New Files
- `src/ui_panels/materials_manager.slint` - Materials Manager UI component
- `src/ui/materials_manager_backend.rs` - Backend logic and helpers
- `docs/materials_manager.md` - User documentation
- `docs/materials_manager_integration.md` - This integration guide

## Testing the Integration

To test the Materials Manager integration:

1. **Build the project:**
   ```bash
   cargo build
   ```

2. **Run the application:**
   ```bash
   cargo run
   ```

3. **Navigate to Materials:**
   - Click the "📦 Materials" tab, or
   - Use View → Materials from the menu

4. **Test functionality:**
   - View the standard materials library (Red Oak, Aluminum 6061, Acrylic)
   - Click on a material to view its properties
   - Try creating a new custom material
   - Test search and filter functionality
   - Update material properties and save

## Next Steps

Potential enhancements for the Materials Manager integration:

1. **Keyboard Shortcuts**: Add Ctrl+M or similar shortcut to switch to Materials view
2. **Context Menu**: Right-click materials for quick actions
3. **Drag & Drop**: Drag materials to Designer or other views
4. **Material Import/Export**: Import materials from JSON/CSV files
5. **Material Selection Dialog**: Popup material selector for other tools
6. **Integration with Designer**: Auto-populate cutting parameters based on material selection
7. **Material Usage Tracking**: Show which projects use which materials

## Backend Integration Example

Other modules can access the materials backend:

```rust
use gcodekit4::ui::MaterialsManagerBackend;

// Initialize backend
let backend = MaterialsManagerBackend::new();

// Get materials
let materials = backend.get_all_materials();

// Search
let aluminum_materials = backend.search_materials("aluminum");

// Filter
use gcodekit4::data::materials::MaterialCategory;
let metals = backend.filter_by_category(MaterialCategory::NonFerrousMetal);
```

## See Also

- [Materials Manager Documentation](materials_manager.md)
- [Materials Data Model](../src/data/materials.rs)
- [UI Architecture](../src/ui/architecture.rs)
