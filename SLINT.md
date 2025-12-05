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
- **ErrorDialog**: Modal dialog for error messages.
- **SuccessDialog**: Modal dialog for success messages.

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

## File Dialogs and Callbacks
- **Pattern**: Slint cannot open native file dialogs directly.
- **Implementation**:
    1.  Define a callback in the Slint component (e.g., `callback browse-path(string)`).
    2.  Invoke the callback from the UI (e.g., on button click).
    3.  Implement the callback in Rust (`main_window.on_browse_path(...)`).
    4.  Use `rfd` crate in Rust to open the native dialog.
    5.  Update the Slint property with the result (e.g., `controller.update_setting(...)`).
    6.  Refresh the UI if necessary.

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

## Window Management
- **Maximization on Windows**: Use `ShowWindow(hwnd, SW_MAXIMIZE)` via `windows-sys` crate in a platform-specific initialization function. This ensures the window opens maximized correctly.
  ```rust
  // In platform.rs
  ShowWindow(hwnd, SW_MAXIMIZE);
  ```

## File Dialogs on Windows
- **Z-Order Issue**: Native file dialogs (`rfd`) may appear behind the main window on Windows if the parent window is not set.
- **Full Screen Issue**: Passing the `slint::Window` handle directly to `rfd` can cause dialogs to open in full-screen mode on some systems.
- **Solution**: Use `crate::platform::pick_file_with_parent` wrappers which manually extract the HWND and wrap it in a clean `Win32ParentHandle`. This ensures the dialog has a parent (fixing Z-order) without inheriting problematic window styles (fixing full-screen).
- **Implementation**: Uses `raw-window-handle` to extract the HWND and creates a custom `HasWindowHandle` implementation.

## Numeric Inputs and Unit Conversion
When dealing with numeric inputs that require unit conversion (e.g., switching between mm and inches) or specific formatting (e.g., keeping trailing zeros), it is often better to bind the UI property to a `string` rather than a `float`.
- **Problem**: Binding a `LineEdit` text directly to a `float` property can cause issues with formatting (e.g., precision loss, unwanted rounding) and makes it difficult to handle unit labels or fractional inputs (like "1 1/2").
- **Solution**: Use `string` properties in the `.slint` file for the display value. Perform parsing and validation in the Rust backend.
    - In Slint: `property <string> my-value: "0.0";`
    - In Rust: Use `slint::SharedString` to pass values. Parse the string using a helper (like `units::parse_from_string`) that handles the current unit system.
    - This allows the UI to display exactly what the user types or a formatted representation (e.g., "10.000") without fighting the automatic float-to-string conversion.

## Tabbed Box Generator
- **Path Grouping**: When implementing layout optimization (packing), paths must be grouped (e.g., a wall and its internal slots) to ensure they move together. The `TabbedBoxMaker` uses a `path_groups` structure to maintain these relationships during packing.
- **Slots**: Divider slots are generated as separate paths (holes) inside the wall panels. Without grouping, the packing algorithm treats them as separate items and moves them away from the wall.

## CAM Tool Pattern
When implementing CAM tools that generate G-code:
1.  Use `invoke_load_editor_text` to load G-code into the editor. This handles:
    *   Loading text into `EditorBridge`.
    *   Resetting cursor and scroll position.
    *   Updating UI properties (`gcode_content`, `can_undo`, etc.).
    *   Updating visible lines.
2.  Set the filename and switch view:
    ```rust
    w.set_gcode_filename(slint::SharedString::from("filename.gcode"));
    w.set_current_view(slint::SharedString::from("gcode-editor"));
    w.set_gcode_focus_trigger(w.get_gcode_focus_trigger() + 1);
    ```
3.  Show a success dialog.
4.  Close the tool dialog using `d.hide().ok()` inside the success block.
5.  Do NOT put `d.show()` at the end of the callback, as it will re-open the dialog immediately.

## Release v0.68.2-alpha.0 (December 2025)
- **Build Fix**: Fixed Windows build failure by adding missing `raw_window_handle` imports in `src/platform.rs`.

## Release v0.68.1-alpha.0 (December 2025)
- **Build Fix**: Resolved duplicate `MachineControlPanel` definition issue by removing the deprecated `src/ui_panels` directory.
- **Assets**: Updated `eStop.png` with improved design and text layout.

