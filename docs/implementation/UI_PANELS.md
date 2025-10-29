# UI Panels Reference

This document provides a comprehensive overview of all UI panels in the gcodekit application.

## Panel List

### 1. Connection Panel
**File**: `src/ui/connection_panel.rs`  
**Task**: Task 68  
**Purpose**: UI for managing serial/network connections to CNC controllers

**Key Features**:
- Port selection and enumeration
- Baud rate configuration
- Network connection settings
- Connection status management

---

### 2. Controller State Panel (DRO)
**File**: `src/ui/dro_panel.rs`  
**Task**: Task 69  
**Purpose**: Digital Readout (DRO) for displaying machine and work coordinates

**Key Features**:
- Real-time coordinate display
- Machine position tracking
- Work coordinates visualization
- Unit conversion support

---

### 3. Jog Controller Panel
**File**: `src/ui/jog_controller.rs`  
**Task**: Task 70  
**Purpose**: Manual machine control with jog buttons and step size selection

**Key Features**:
- XYZ jog controls
- Adjustable step sizes
- Continuous jog support
- Quick position buttons

---

### 4. File Operations Panel
**File**: `src/ui/file_operations.rs`  
**Task**: Task 71  
**Purpose**: G-Code file browser, open dialog, and file information display

**Key Features**:
- File browser with filters
- Recent files list
- File information display
- Drag and drop support

---

### 5. G-Code Viewer/Editor Panel
**File**: `src/ui/gcode_viewer.rs`  
**Task**: Task 72  
**Purpose**: Text editor with syntax highlighting, line numbers, and search/replace

**Key Features**:
- Syntax highlighting for G-Code
- Line number display
- Search and replace functionality
- Real-time validation

---

### 6. Console/Output Panel
**File**: `src/ui/console_panel.rs`  
**Task**: Task 73  
**Purpose**: Display controller responses, command history, and message filtering

**Key Features**:
- Real-time message display
- Command history
- Message filtering and search
- Timestamp display

---

### 7. Control Buttons Panel
**File**: `src/ui/control_buttons.rs`  
**Task**: Task 74  
**Purpose**: Machine control buttons (Start, Pause, Stop, Home, Reset, Unlock, Alarm Clear)

**Key Features**:
- Program start/pause/stop controls
- Home command
- Machine reset
- Emergency stop (Unlock)
- Alarm clear function

---

### 8. Overrides Panel
**File**: `src/ui/overrides_panel.rs`  
**Task**: Task 75  
**Purpose**: Feed rate and spindle speed override controls

**Key Features**:
- Feed rate percentage adjustment
- Spindle speed override controls
- Real-time adjustment display
- Override range validation

---

### 9. Coordinate System Panel
**File**: `src/ui/coordinate_system.rs`  
**Task**: Task 76  
**Purpose**: Work coordinate system selection and offsets display

**Key Features**:
- G54-G59 work coordinate selection
- Offset value display
- Coordinate system switching
- Offset editing capabilities

---

### 10. Macros Panel
**File**: `src/ui/macros_panel.rs`  
**Task**: Task 77  
**Purpose**: G-Code macro button grid, editor, execution, variable substitution, and macro import/export

**Key Features**:
- Macro button grid
- Macro editor
- Variable substitution
- Import/export functionality
- Macro execution

---

### 11. Settings/Preferences Dialog
**File**: `src/ui/settings_dialog.rs`  
**Task**: Task 78  
**Purpose**: Application settings with categories for controller settings, UI preferences, file processing options, and keyboard shortcuts configuration

**Key Features**:
- Application preferences
- Controller configuration
- UI theme and appearance
- Keyboard shortcuts mapping
- File processing settings

---

### 12. Firmware Settings Panel
**File**: `src/ui/firmware_settings_panel.rs`  
**Task**: Task 79  
**Purpose**: Display and manage firmware-specific parameters with validation, descriptions, save/restore functionality

**Key Features**:
- Firmware parameter display
- Parameter validation
- Descriptions and help text
- Save/restore settings
- Firmware version detection

---

## Supporting Modules

### Main Components
- **Main Window** (`src/ui/main_window.rs`): Core application window with menu bar, toolbar, and status bar
- **Architecture** (`src/ui/architecture.rs`): UI Architecture Setup - defines Slint component hierarchy and communication patterns
- **State Management** (`src/ui/state.rs`): UI state management and data flow through components

### Utility Modules
- **Events** (`src/ui/events.rs`): Event system for inter-component communication
- **Components** (`src/ui/components.rs`): Reusable UI component definitions
- **File Management** (`src/ui/file_management.rs`): File I/O, recent files, processing pipeline, statistics, export, drag/drop, validation, comparison, backup, templates
- **UI Polish** (`src/ui/ui_polish.rs`): Progress indicators, status notifications, keyboard shortcuts, themes, i18n, responsive layout, help system

## Panel Organization

All panels follow a consistent structure:
- Located in `src/ui/` directory
- Named with `*_panel.rs` or specific component naming
- Implement Slint components
- Handle state and events for their domain
- Include documentation headers with task references

## Interaction Flow

The UI panels communicate through:
1. **Event System** - Event-based messaging for inter-component communication
2. **Shared State** - Centralized state management for application-wide data
3. **Message Passing** - Component-to-component communication through the event system
