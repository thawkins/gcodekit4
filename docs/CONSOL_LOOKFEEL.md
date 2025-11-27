# Device Console Aesthetics Analysis & Recommendations

## Current State Analysis

The current Device Console Panel (`DeviceConsolePanel`) uses a basic, functional layout that defaults to a light theme, inconsistent with the modern dark theme of the Designer and Visualizer.

*   **Theme**: Hardcoded `#000000` text implies a light background. Standard widgets (`Button`, `LineEdit`) are used without custom styling.
*   **Layout**: A simple vertical stack: Toolbar (Top) -> Output (Middle) -> Input (Bottom).
*   **Typography**: Uses default variable-width fonts, which is suboptimal for reading log output and G-code.

## Recommendations

To unify the application's look and feel, the Device Console should adopt the "Designer Standard" aesthetics.

### 1. Layout Restructuring

Adopt the **Left Sidebar + Main Area** pattern.

*   **Left Sidebar (200px)**:
    *   **Actions**: Clear Console, Copy Output.
    *   **Filters** (Future): Checkboxes for "Show Commands", "Show Responses", "Show Errors".
*   **Main Area**:
    *   **Console Log**: Large, scrollable area taking up most space.
    *   **Command Bar**: Fixed at the bottom of the main area.

### 2. Color Palette Harmonization

*   **Sidebar Background**: `#2c3e50`.
*   **Console Background**: `#1e1e1e` (Terminal black).
*   **Text Color**: `#ecf0f1` (Light Grey) for standard output.
    *   *Suggestion*: Use colors for different message types (e.g., `#2ecc71` for sent commands, `#e74c3c` for errors, `#f1c40f` for warnings).
*   **Input Field**: Dark background (`#252526`) with light text.

### 3. Component Modernization

*   **Buttons**: Replace standard `Button` with `MCToolButton` (icon + text) in the sidebar.
*   **Typography**: Use "Fira Code" or a monospace font for the console output to align columns and improve readability.
*   **Input Area**:
    *   Style the `LineEdit` to blend with the dark theme.
    *   Use an icon-based "Send" button (e.g., Paper Plane or Arrow).

## Implementation Plan

1.  **Refactor `DeviceConsolePanel`**:
    *   Change inheritance from `VerticalBox` to `Rectangle` (to manage background).
    *   Implement `HorizontalLayout` for Sidebar + Main Content.
2.  **Sidebar**:
    *   Add `MCToolButton` for "Clear" (Trash icon) and "Copy" (Clipboard icon).
3.  **Console Area**:
    *   Set background to `#1e1e1e`.
    *   Update `Text` element to use Monospace font and white color.
4.  **Command Bar**:
    *   Style the input row to sit flush at the bottom.
    *   Update the "Send" button to match the new style.
