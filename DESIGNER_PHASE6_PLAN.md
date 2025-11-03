# Designer Phase 6 Implementation Plan

**Start Date**: November 3, 2025  
**Status**: 🔄 PLANNING  
**Focus**: Layer Management & Advanced CAM Features

## Overview

Phase 6 builds upon the completed Phase 5 work by adding professional CAD/CAM features focused on layer management, advanced toolpath strategies, and enhanced workflow capabilities.

## Goals

1. Implement comprehensive layer management system
2. Add advanced CAM toolpath strategies
3. Enhance multi-tool workflow support
4. Improve design organization and productivity
5. Add industry-standard CAM features

## Task Breakdown

## Issue Tracking

All Phase 6 tasks are tracked in the bd issue system:

- **Phase 6.1**: gcodekit4-79 - Layer Management System
- **Phase 6.2**: gcodekit4-84 - Layer Panel UI  
- **Phase 6.3**: gcodekit4-85 - Advanced Toolpath Strategies
- **Phase 6.4**: gcodekit4-86 - Multi-Tool Support
- **Phase 6.5**: gcodekit4-87 - Tabbed Operations
- **Phase 6.6**: gcodekit4-88 - Design Validation & Analysis
- **Phase 6.7**: gcodekit4-89 - Nesting & Array Operations Enhancement
- **Phase 6.8**: gcodekit4-90 - Design File Operations (Open, Save, Save As)

Use `bd show <issue-id>` for detailed information on each task.
Use `bd ready` to see which tasks are ready to work on.

### Phase 6.1: Layer Management System ✅ Priority
**Issue**: gcodekit4-79  
**Estimated Effort**: 3-4 hours  
**Status**: Open

#### Features
- Add layer field to `DrawingObject` structure
- Implement `LayerManager` for layer operations
- Layer properties: name, visible, locked, color
- Default layer creation ("0" or "Default")
- Layer persistence in save files

#### Technical Implementation
```rust
// In canvas.rs
pub struct DrawingObject {
    pub id: u64,
    pub shape: Box<dyn Shape>,
    pub selected: bool,
    pub layer: String,      // NEW
    pub visible: bool,       // NEW
}

pub struct LayerInfo {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub color: Option<(u8, u8, u8)>,
}

impl Canvas {
    pub fn add_layer(&mut self, name: String)
    pub fn remove_layer(&mut self, name: &str)
    pub fn set_layer_visible(&mut self, name: &str, visible: bool)
    pub fn set_layer_locked(&mut self, name: &str, locked: bool)
    pub fn set_active_layer(&mut self, name: &str)
    pub fn get_layers(&self) -> Vec<&LayerInfo>
}
```

#### Acceptance Criteria
- [x] DrawingObject includes layer name field
- [ ] Default layer created automatically
- [ ] DXF import preserves layer names
- [ ] Canvas methods for layer visibility
- [ ] Hidden layers not rendered
- [ ] Layer panel UI with visibility toggles
- [ ] Active layer selection affects new shapes
- [ ] Tests verify layer preservation

#### Files to Modify
- `src/designer/canvas.rs` - DrawingObject and layer management
- `src/designer/import.rs` - Preserve layers on import
- `src/ui_panels/designer.slint` - Layer panel UI
- `src/main.rs` - Layer management callbacks

---

### Phase 6.2: Layer Panel UI
**Estimated Effort**: 2-3 hours  
**Dependencies**: Phase 6.1

#### Features
- Scrollable layer list showing all layers
- Eye icon for visibility toggle
- Lock icon for edit protection
- Active layer indicator (highlighted)
- Layer color indicators
- Right-click context menu (rename, delete, merge)
- Add/remove layer buttons
- Layer reordering (drag & drop)

#### UI Layout
```
┌─────────────────────────────┐
│ Layers                  [+] │
├─────────────────────────────┤
│ ● 👁 🔓 Layer 1        ✓   │  <- Active
│ ● 👁 🔓 Layer 2            │
│ ● 👁 🔒 Construction       │  <- Locked
│ ● 👁 🔓 Toolpaths          │
│ ● 🔍 🔓 Hidden Layer       │  <- Hidden
└─────────────────────────────┘
```

#### Callbacks
- `on_layer_add(name: string)`
- `on_layer_remove(layer: string)`
- `on_layer_toggle_visible(layer: string)`
- `on_layer_toggle_locked(layer: string)`
- `on_layer_set_active(layer: string)`
- `on_layer_rename(old_name: string, new_name: string)`

---

### Phase 6.3: Advanced Toolpath Strategies
**Estimated Effort**: 4-5 hours  
**Status**: Not Started

#### Features

##### Ramping and Lead-in/Lead-out
- Helical ramping for Z-axis entry
- Arc lead-in for smooth entry
- Arc lead-out for smooth exit
- Configurable ramp angle and arc radius
- Prevent tool marks at entry points

##### Finishing Passes
- Offset finishing pass (leave stock for finish cut)
- Spring pass (retrace final contour)
- Configurable finish allowance (0.1-1.0mm)
- Separate finish feed rate

##### Island Detection
- Automatic detection of closed shapes within contours
- Skip islands during pocket operations
- Optional island clearing strategies
- Handle nested islands

#### Technical Implementation
```rust
pub enum LeadInStrategy {
    None,
    Line { length: f64 },
    Arc { radius: f64, angle: f64 },
    Helical { radius: f64, pitch: f64 },
}

pub enum LeadOutStrategy {
    None,
    Line { length: f64 },
    Arc { radius: f64, angle: f64 },
}

pub struct FinishingPass {
    pub enabled: bool,
    pub allowance: f64,      // Stock to leave (mm)
    pub feed_rate: f64,      // Finish feed rate
    pub spring_passes: u32,  // Number of spring passes
}

pub struct ToolpathParams {
    pub lead_in: LeadInStrategy,
    pub lead_out: LeadOutStrategy,
    pub finishing: FinishingPass,
    pub detect_islands: bool,
}
```

---

### Phase 6.4: Multi-Tool Support
**Estimated Effort**: 3-4 hours  
**Status**: Not Started

#### Features
- Tool library management
- Store tool properties (diameter, flutes, material, speeds/feeds)
- Assign tools to toolpaths
- Generate tool change commands (M6)
- Tool length offset (TLO) support
- Automatic speed/feed calculation per tool
- Tool wear tracking

#### Tool Library Structure
```rust
pub struct Tool {
    pub id: u32,
    pub name: String,
    pub diameter: f64,
    pub flutes: u32,
    pub material: ToolMaterial,
    pub max_rpm: u32,
    pub recommended_feed: f64,
    pub length: f64,
    pub notes: String,
}

pub enum ToolMaterial {
    HSS,
    Carbide,
    CoatedCarbide,
    Diamond,
}

pub struct ToolLibrary {
    tools: Vec<Tool>,
}

impl ToolLibrary {
    pub fn add_tool(&mut self, tool: Tool)
    pub fn remove_tool(&mut self, id: u32)
    pub fn get_tool(&self, id: u32) -> Option<&Tool>
    pub fn save_to_file(&self, path: &Path)
    pub fn load_from_file(path: &Path)
}
```

#### UI Components
- Tool library panel
- Tool selection dropdown in toolpath params
- Tool editor dialog
- Tool change preview in G-code
- Tool usage statistics

---

### Phase 6.5: Tabbed Operations
**Estimated Effort**: 3-4 hours  
**Status**: Not Started

#### Features
- Define tab locations on contours
- Tab width and height configuration
- Automatic tab distribution
- Manual tab placement
- Tab ramping for clean breaks
- Preview tabs before generating G-code

#### Tab Configuration
```rust
pub struct Tab {
    pub position: Point,     // Location on contour
    pub width: f64,          // Tab width (mm)
    pub height: f64,         // Tab thickness (mm)
}

pub enum TabPlacement {
    Manual(Vec<Tab>),
    Auto {
        count: u32,
        width: f64,
        height: f64,
    },
}

impl ToolpathGenerator {
    pub fn add_tabs(&mut self, placement: TabPlacement)
    pub fn preview_tabs(&self) -> Vec<Tab>
}
```

---

### Phase 6.6: Design Validation & Analysis
**Estimated Effort**: 2-3 hours  
**Status**: Not Started

#### Features
- Geometry validation (self-intersecting paths, open contours)
- Toolpath collision detection
- Cut time estimation
- Material usage calculation
- Recommended speeds and feeds analysis
- Design optimization suggestions

#### Validation Types
```rust
pub enum ValidationIssue {
    SelfIntersectingPath { shape_id: u64 },
    OpenContour { shape_id: u64 },
    TooSmallRadius { shape_id: u64, radius: f64, min_radius: f64 },
    ToolTooLarge { shape_id: u64, tool_diameter: f64 },
    InsufficientStock { depth: f64, material_thickness: f64 },
}

pub struct DesignAnalysis {
    pub issues: Vec<ValidationIssue>,
    pub estimated_time: Duration,
    pub total_distance: f64,
    pub material_removed: f64,
}

impl Canvas {
    pub fn validate_design(&self, params: &ToolpathParams) -> DesignAnalysis
}
```

---

### Phase 6.7: Nesting & Array Operations Enhancement
**Estimated Effort**: 3-4 hours  
**Status**: Not Started

#### Features
- Automatic nesting of shapes to minimize material waste
- Rectangular array with rotation
- Circular array around point
- Path array along curve
- Mirror operations (X, Y, XY)
- Spacing optimization
- Material boundary definition

#### Array Types
```rust
pub enum ArrayType {
    Rectangular {
        rows: u32,
        cols: u32,
        x_spacing: f64,
        y_spacing: f64,
        rotation: f64,
    },
    Circular {
        center: Point,
        count: u32,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    },
    Path {
        curve: Vec<Point>,
        count: u32,
        align_to_path: bool,
    },
}

impl Canvas {
    pub fn create_array(&mut self, shape_ids: Vec<u64>, array_type: ArrayType)
    pub fn nest_shapes(&mut self, material_bounds: Rect)
}
```

---

### Phase 6.8: Design File Operations (Open, Save, Save As)
**Issue**: gcodekit4-90  
**Estimated Effort**: 3-4 hours  
**Status**: Open

#### Features
- Native design file format (.gck4 - JSON based)
- Save complete design state (shapes, layers, viewport, settings)
- Open operation restores full design
- Save As prompts for new filename
- Recent files tracking
- Unsaved changes detection
- Modified indicator in UI
- Confirmation prompt on close with unsaved changes
- Optional auto-save/recovery

#### File Format
```json
{
  "version": "1.0",
  "metadata": {
    "name": "My Design",
    "created": "2025-11-03T19:00:00Z",
    "modified": "2025-11-03T19:30:00Z",
    "author": "User"
  },
  "viewport": {
    "zoom": 1.0,
    "pan_x": 0.0,
    "pan_y": 0.0
  },
  "layers": [
    {"name": "Layer 1", "visible": true, "locked": false, "color": [255, 0, 0]}
  ],
  "shapes": [
    {"id": 1, "type": "rectangle", "x": 0, "y": 0, "width": 100, "height": 50, "layer": "Layer 1"}
  ],
  "toolpath_params": {
    "feed_rate": 1000,
    "spindle_speed": 3000,
    "tool_diameter": 3.175,
    "cut_depth": -5.0
  }
}
```

#### State Management
```rust
pub struct DesignFileState {
    pub current_file: Option<PathBuf>,
    pub is_modified: bool,
    pub recent_files: Vec<PathBuf>,
}

impl DesignerState {
    pub fn save_to_file(&self, path: &Path) -> Result<()>
    pub fn load_from_file(&mut self, path: &Path) -> Result<()>
    pub fn mark_modified(&mut self)
    pub fn add_recent_file(&mut self, path: PathBuf)
}
```

#### UI Components
- File menu in designer panel (New, Open, Save, Save As, Recent Files)
- File picker dialog for Open/Save As
- Modified indicator (* in title bar or status)
- Unsaved changes confirmation dialog
- Recent files submenu (last 10 files)

#### Callbacks
- `on_file_new()` - Clear canvas, reset state
- `on_file_open()` - Show file picker, load design
- `on_file_save()` - Save to current path or prompt if new
- `on_file_save_as()` - Always prompt for new path
- `on_before_close()` - Check for unsaved changes

#### Files to Modify
- `src/designer/serialization.rs` (NEW) - JSON serialization/deserialization
- `src/designer_state.rs` - Add file state tracking
- `src/ui_panels/designer.slint` - File menu UI
- `src/main.rs` - File operation callbacks
- `src/utils/file_io.rs` - Recent files management (already exists)

---

## Implementation Order

### Week 1: Core Layer System
1. Phase 6.1: Layer Management System (Day 1-2)
2. Phase 6.2: Layer Panel UI (Day 2-3)
3. Testing and bug fixes (Day 3)

### Week 2: Advanced CAM Features
4. Phase 6.3: Advanced Toolpath Strategies (Day 1-2)
5. Phase 6.4: Multi-Tool Support (Day 3-4)

### Week 3: Production Features
6. Phase 6.5: Tabbed Operations (Day 1-2)
7. Phase 6.6: Design Validation (Day 2-3)
8. Phase 6.7: Nesting & Arrays (Day 3-4)
9. Phase 6.8: File Operations (Day 4)

### Week 4: Integration & Polish
- Integration testing
- Documentation updates
- Bug fixes and refinement
- Performance optimization

## Success Metrics

### Functionality
- [ ] All layer operations work correctly
- [ ] Layer visibility affects rendering and selection
- [ ] Imports preserve layer information
- [ ] Advanced toolpaths generate valid G-code
- [ ] Multi-tool operations produce correct M6 commands
- [ ] Tabs prevent part movement during cutting
- [ ] Validation catches common design errors

### Performance
- [ ] Layer operations < 10ms for 1000 shapes
- [ ] UI remains responsive with 10+ layers
- [ ] Toolpath generation < 5s for complex designs
- [ ] Nesting algorithm < 30s for 100 shapes

### User Experience
- [ ] Layer panel is intuitive and responsive
- [ ] Tool library is easy to manage
- [ ] Validation provides actionable feedback
- [ ] Advanced features don't complicate basic workflows

## Testing Requirements

### Unit Tests
- Layer management operations
- Layer visibility and locking
- Advanced toolpath generation
- Tool library CRUD operations
- Array and nesting algorithms
- Validation logic

### Integration Tests
- Import/export with layers
- Multi-tool G-code generation
- Tab generation in toolpaths
- End-to-end workflow tests

### UI Tests
- Layer panel interactions
- Tool library UI
- Context menu operations
- Validation feedback display

## Documentation Updates

### User Documentation
- Layer management guide
- Advanced toolpath strategies
- Multi-tool workflow tutorial
- Design validation best practices
- Nesting and arrays guide

### Technical Documentation
- Layer system architecture
- Toolpath algorithm descriptions
- Tool library format specification
- API documentation for new features

## Dependencies

### External
- None (all internal features)

### Internal
- Phases 1-5 must be complete (✅)
- SVG/DXF import functionality (✅)
- Toolpath generation system (✅)
- Canvas rendering system (✅)

## Risks & Mitigation

### Risk: Layer system complexity
**Mitigation**: Start with minimal layer support, expand incrementally

### Risk: Performance with many layers
**Mitigation**: Use existing spatial index, batch render operations

### Risk: UI clutter with new features
**Mitigation**: Use collapsible panels, sane defaults, optional advanced features

### Risk: Breaking existing designs
**Mitigation**: Add layer field with default value, maintain backward compatibility

## Future Considerations (Phase 7+)

- 3D toolpath preview
- CNC simulation with collision detection
- Post-processor system for different controllers
- Cloud-based design library
- Collaborative design features
- CAM optimization with AI/ML
- Material database integration
- Tool wear simulation

## Notes

This phase focuses on bringing the Designer tool to professional CAD/CAM standards while maintaining the simplicity and usability established in previous phases. The layer system is the foundational feature that enables better organization for all other advanced features.

Priority is on implementing features that have immediate user value and solve real workflow problems. Each feature should be independently useful and not require other Phase 6 features to function.

## Sign-off

**Phase Owner**: Development Team  
**Review Date**: TBD  
**Approval**: Pending Implementation

---

**Last Updated**: November 3, 2025  
**Next Review**: After Phase 6.1 completion
