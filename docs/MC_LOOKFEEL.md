# Machine Control Aesthetics Analysis & Recommendations

## Current State Analysis

The current Machine Control Panel (`MachineControlPanel`) uses a legacy aesthetic that is inconsistent with the new Designer and Visualizer standards.

*   **Theme**: Uses a light theme (`#ecf0f1` backgrounds, `#333` text) which clashes with the application's move to a dark theme (`#2c3e50` panels, `#1e1e1e` backgrounds).
*   **Layout**: Uses a vertical stack of panels (Connection, DRO, Jog Controls) in a scroll view. The new standard uses a **Left Sidebar** for controls and a main area for visualization/interaction.
*   **Components**:
    *   **Jog Buttons**: Custom `Rectangle` implementation with light background.
    *   **DRO**: Uses `LineEdit` widgets for display, which look like input fields rather than digital readouts.
    *   **Connection Bar**: A horizontal bar at the top, which takes up vertical space.

## Recommendations

To unify the application's look and feel, the Machine Control Panel should be refactored to match the "Designer Standard".

### 1. Layout Restructuring

Adopt the **Left Sidebar + Main Area** pattern.

*   **Left Sidebar (250px)**:
    *   **Connection Controls**: Port selection, Connect/Disconnect.
    *   **Machine State**: Status indicator (Idle, Run, Alarm).
    *   **Work Coordinate Systems**: G54-G59 buttons.
    *   **Homing/Unlock**: Home All, Unlock buttons.
*   **Main Area**:
    *   **Digital Readout (DRO)**: Large, high-contrast display of coordinates.
    *   **Jog Control Pad**: A visual, directional pad for machine movement.

### 2. Color Palette Harmonization

*   **Panel Background**: `#2c3e50` (Sidebar).
*   **Main Area Background**: `#1e1e1e` (Dark grey for contrast).
*   **Text Color**: `#ecf0f1` (White/Light Grey).
*   **Accent Color**: `#3498db` (Blue) for active elements, `#e74c3c` (Red) for stop/alarm.

### 3. Component Modernization

*   **DRO Display**:
    *   Use large, monospace fonts (e.g., "Fira Code", 32px) for coordinate values.
    *   Green (`#2ecc71`) for active axes, Grey for inactive.
    *   Label axes clearly (X, Y, Z, A).
*   **Jog Controls**:
    *   Replace the grid of rectangular buttons with a **Directional Pad** layout.
    *   Use icon-based buttons (Arrows) styled like `VisualizerToolButton`.
    *   **Step Size Selector**: Segmented control or toggle buttons instead of a dropdown.

### 4. Specific Features

*   **Connection Panel**: Move to the top of the Left Sidebar.
*   **Zero Buttons**: Place "Zero X/Y/Z" buttons next to the corresponding DRO readout for intuitive access.

## Implementation Plan

1.  **Refactor `MachineControlPanel`**:
    *   Split into `HorizontalLayout`.
    *   **Left**: `Rectangle { width: 250px; background: #2c3e50; ... }`
    *   **Right**: `Rectangle { background: #1e1e1e; ... }`
2.  **Implement New DRO**:
    *   Create a `DROAxis` component: Label + Value + Zero Button.
    *   Stack them vertically in the Main Area.
3.  **Implement Jog Pad**:
    *   Create a visual layout for X/Y arrows (Cross shape) and Z arrows (Vertical stack).
    *   Place below or beside the DRO in the Main Area.
