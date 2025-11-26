# Keybindings

This document tracks the currently assigned keybindings in GCodeKit4, organized by functional area.

## Global / Machine Control

These shortcuts are handled by the global keyboard manager and are generally available when the application has focus, particularly in the Machine Control view.

### File Operations
| Key Combination | Action |
|----------------|--------|
| `Ctrl + o` | Open File |
| `Ctrl + s` | Save File |
| `Ctrl + q` | Exit Application |

### Machine Control
| Key Combination | Action |
|----------------|--------|
| `Ctrl + h` | Home All Axes |
| `Ctrl + r` | Soft Reset |

### Streaming Control
| Key Combination | Action |
|----------------|--------|
| `Space` | Pause/Resume Stream |
| `Escape` | Stop Stream |

### Jogging (WASD / QZ)
| Key Combination | Action |
|----------------|--------|
| `d` | Jog +X (Right) |
| `a` | Jog -X (Left) |
| `w` | Jog +Y (Up) |
| `s` | Jog -Y (Down) |
| `q` | Jog +Z (Up) |
| `z` | Jog -Z (Down) |

### Jogging (Numpad / Arrow Keys)
*Note: These are defined in the Jog Controller logic.*
| Key Combination | Action |
|----------------|--------|
| `6` | Jog +X (Right) |
| `4` | Jog -X (Left) |
| `8` | Jog +Y (Up) |
| `2` | Jog -Y (Down) |
| `9` | Jog +Z (Page Up) |
| `3` | Jog -Z (Page Down) |

### View Controls
| Key Combination | Action |
|----------------|--------|
| `0` | Fit View |

---

## Designer

These shortcuts are active when the Designer tab is focused.

### Selection & Tools
| Key Combination | Action |
|----------------|--------|
| `Escape` | Deselect all objects |
| `Delete` | Delete selected objects |
| `Shift` (Hold) | Multi-selection |

### Edit Operations
| Key Combination | Action |
|----------------|--------|
| `Ctrl + Z` | Undo last action |
| `Ctrl + Shift + Z` | Redo last action |
| `Ctrl + Y` | Redo last action |

### Navigation
| Key Combination | Action |
|----------------|--------|
| `+` / `=` | Zoom In |
| `-` | Zoom Out |
| `Mouse Wheel` | Zoom In/Out |
| `Space + Left Drag` | Pan Canvas |

---

## G-Code Editor

These shortcuts are active when the G-Code Editor is focused.

### Navigation
| Key Combination | Action |
|----------------|--------|
| `Ctrl + Home` | Jump to beginning of file |
| `Ctrl + End` | Jump to end of file |
| `End` | Jump to end of line |

*Note: Standard text editing keys (Arrow keys, Backspace, Enter, etc.) are also supported.*
