# Device Info Aesthetics Analysis & Recommendations

## Current State Analysis

The current Device Info Panel (`DeviceInfoPanel`) uses a light theme that stands in stark contrast to the dark, professional aesthetic of the rest of the application (Designer, Visualizer, Machine Control).

*   **Theme**: Light backgrounds (`#ecf0f1`, `#ffffff`) with dark text (`#2c3e50`).
*   **Layout**: A simple vertical stack of cards. It does not utilize the screen width effectively on larger displays.
*   **Consistency**: It feels like a separate application compared to the dark-themed modules.

## Recommendations

To unify the application's look and feel, the Device Info view should adopt the "Designer Standard" aesthetics.

### 1. Layout Restructuring

Adopt the **Left Sidebar + Main Area** pattern.

*   **Left Sidebar (250px)**:
    *   **Device Summary**: Prominently display the Device Name, Firmware Type, and Version.
    *   **Status Icon**: A large, dynamic icon representing the connection state/firmware type.
    *   **Actions**: Buttons for "Refresh Info" or "Copy Configuration".
*   **Main Area**:
    *   **Capabilities Grid**: A detailed, scrollable list of firmware capabilities.
    *   **Configuration**: (Future) Read-only view of critical firmware settings (e.g., buffer sizes, max feed rates).

### 2. Color Palette Harmonization

*   **Sidebar Background**: `#2c3e50`.
*   **Main Background**: `#1e1e1e`.
*   **Card/Row Backgrounds**: `#252526` (Dark Grey) with `#2d2d30` for alternating rows.
*   **Text Color**: `#ecf0f1` (Primary), `#bdc3c7` (Secondary).
*   **Status Colors**:
    *   Enabled/Connected: `#2ecc71` (Bright Green).
    *   Disabled/Disconnected: `#e74c3c` (Red).
    *   Info/Version: `#3498db` (Blue).

### 3. Component Modernization

*   **Capability Rows**:
    *   Replace the simple text list with a structured row component.
    *   Use a "Pill" or "Badge" style for the "Enabled/Disabled" status.
*   **Typography**:
    *   Use "Fira Code" for version numbers and technical details.
    *   Increase font weight for headers.

## Implementation Plan

1.  **Refactor `DeviceInfoPanel`**:
    *   Change inheritance to `Rectangle` to manage the main background.
    *   Implement `HorizontalLayout` for Sidebar + Main Content.
2.  **Sidebar**:
    *   Create a "Device Card" layout for the summary info.
    *   Add `MCToolButton` for actions.
3.  **Capabilities Area**:
    *   Style the list with dark theme colors.
    *   Implement alternating row colors for readability.
