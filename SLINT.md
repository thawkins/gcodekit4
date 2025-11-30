# Slint UI Architecture

## Overview
The UI is built using the Slint framework, organized into a modular structure with a centralized theme and shared component library.

## Directory Structure
```
crates/gcodekit4-ui/
├── ui/
│   ├── theme.slint              # Centralized color palette and sizing constants
│   ├── ui_components/
│   │   └── shared.slint         # Reusable widgets (Buttons, Inputs, Sidebars)
│   └── images/                  # Assets
├── ui_panels/                   # Individual Tab Views
│   ├── gcode_editor.slint
│   ├── machine_control.slint
│   ├── device_console.slint
│   ├── device_info.slint
│   ├── config_settings.slint
│   ├── cam_tools.slint
│   ├── materials_manager.slint
│   └── tools_manager.slint
└── ui.slint                     # Main Window composition
```

## Theme System
The `Theme` global singleton in `theme.slint` defines the application's visual style (Dark Mode).
- **Colors**: `primary`, `secondary`, `background`, `surface`, `text-primary`, `text-secondary`, etc.
- **Sizes**: `padding`, `spacing`, `border-radius`, `sidebar-width`.

## Shared Components
Common UI elements are defined in `shared.slint` to ensure consistency:
- **StandardButton**: Primary, secondary, and destructive styles.
- **StandardInput**: Text input fields.
- **StandardCheckBox**: Toggle controls.
- **StandardSpinBox** / **StandardFloatSpinBox**: Numeric inputs.
- **StandardSidebar**: Layout container for left-side navigation panels (fixed 250px width).

## Best Practices
1.  **Import Theme**: Always import `Theme` from `../ui/theme.slint` instead of hardcoding colors.
2.  **Use Shared Components**: Prefer `StandardButton` over raw `Button` or `Rectangle`.
3.  **Layouts**: Use `StandardSidebar` for panel layouts that require a left navigation pane.
4.  **Consistency**: Follow the established patterns for headers, spacing, and alignment.

## Designer Features
- **Array Tools**:
    - **Linear Array**: Creates copies in a grid pattern defined by X/Y counts and spacing.
    - **Circular Array**: Creates copies arranged in a circle defined by center, radius, start angle, and total count.
    - **Grid Array**: Alias for Linear Array with specific grid terminology (Columns/Rows).
    - **Grouping**: All array operations automatically group the resulting shapes (including the original) into a single group for easy manipulation.
    - **Dialogs**: Dedicated modal dialogs (`ArrayLinearDialog`, `ArrayCircularDialog`, `ArrayGridDialog`) for parameter input.
- **Context Menu**:
    - **Right-Click**: Opens a context menu for the selected shape(s).
    - **Operations**:
        - Copy/Paste
        - Group/Ungroup
        - Align (Horizontal/Vertical)
        - Convert To (Rectangle, Path)
        - Array (Linear, Circular, Grid)
        - Delete

## Troubleshooting
- **Layout Issues**: Check `horizontal-stretch` and `vertical-stretch` properties.
- **Focus**: Ensure `FocusScope` is used correctly for keyboard input.
- **Z-Index**: Slint renders in order of declaration; later elements are on top.
- **Z-Index Limitation**: The `z` property must be a number literal and cannot be bound to an expression.
- **Z-Index Best Practice**: Use moderate z-index values (e.g., 10, 50, 100) to avoid creating unintended stacking contexts. Very high values (1000+) can cause layout issues.
- **Tooltip Z-Index**: Tooltips use `z: 100` to appear above buttons within the same layout context. This works for most use cases without breaking layout flow.
- **Brace Balance**: Slint files must have balanced braces. Removing or adding braces breaks compilation.
- **Brace Debugging**: When fixing layout issues caused by misplaced braces, MOVE braces rather than adding/removing them. Use `awk` to count braces in sections to find misplaced ones.
- **Blank Lines**: Blank lines have NO effect on Slint syntax - focus only on braces, not whitespace.
