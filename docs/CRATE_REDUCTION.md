# Crate Reduction and Dependency Consolidation Plan

## Overview

This document outlines a plan to reduce redundancy, fix version mismatches, and consolidate dependencies across the GCodeKit4 workspace. By centralizing dependency management and removing redundant crates, we can improve build times, reduce binary size, and ensure consistent behavior.

**Estimated Reduction:**
- **Direct Dependencies:** We can eliminate ~5-10 redundant direct dependency declarations by moving them to the workspace level.
- **Transitive Dependencies:** By aligning versions (e.g., `slint`, `uuid`, `thiserror`), we can remove duplicate versions of the same crate from the dependency tree, potentially saving 10+ compiled crates.
- **Removed Crates:** We identified potential candidates for removal (`lazy_static`, `config` in UI).

## Analysis Findings

### 1. Version Mismatches
The following crates have conflicting versions across the workspace:

| Crate | Versions Found | Locations | Recommendation |
|-------|----------------|-----------|----------------|
| `slint` | 1.14.1, 1.8 | Root/UI vs `devicedb` | Upgrade `devicedb` to 1.14.1 |
| `uuid` | 1.6, 1.0 | Core/UI vs `devicedb` | Upgrade `devicedb` to 1.6 |
| `thiserror` | 2.0.17, 1.0 | Core/Visualizer vs `devicedb` | Upgrade `devicedb` to 2.0.17 |
| `toml` | 0.8, 0.9.8 | `settings` vs `ui` | Standardize on 0.8 (or latest stable) |

### 2. Redundant Functionality
- **`lazy_static`**: Used in `camtools`, `designer`, `visualizer`. Can be replaced by `std::sync::OnceLock` (Rust 1.70+) or `once_cell` to reduce dependencies.
- **`config`**: Used in `ui`. If `gcodekit4-settings` already handles configuration using `toml` and `serde`, the `config` crate might be unnecessary overhead in `ui`.
- **`ropey`**: Used in `ui`. `ui` should likely rely on `gcodekit4-gcodeeditor` for text manipulation rather than depending on `ropey` directly.

### 3. Workspace Management
Dependencies are currently defined in each individual `Cargo.toml`. This is the root cause of version mismatches. Moving them to `[workspace.dependencies]` in the root `Cargo.toml` is the primary fix.

## Implementation Plan

### Task 1: Centralize Dependencies in Workspace
Move all common dependencies to the root `Cargo.toml` under `[workspace.dependencies]`.

**Prompt:**
```text
Refactor the Cargo.toml files to use workspace dependencies.
1. Modify the root Cargo.toml to add a [workspace.dependencies] section containing:
   - slint = { version = "1.14.1", features = ["backend-winit", "image-default-formats", "raw-window-handle-06"] }
   - tokio = { version = "1.35", features = ["full"] }
   - tracing = "0.1"
   - anyhow = "1.0"
   - thiserror = "2.0.17"
   - serde = { version = "1.0", features = ["derive"] }
   - serde_json = "1.0"
   - uuid = { version = "1.6", features = ["v4", "serde"] }
   - chrono = { version = "0.4", features = ["serde"] }
   - image = "0.25"
   - svg = "0.18"
   - dxf = "0.4"
   - lyon = "1.0"
   - regex = "1.10"
   - lazy_static = "1.4"
   - ropey = "1.6"
   - dirs = "5.0"
   - rfd = "0.15.4"
   - toml = "0.8"

2. Update all sub-crate Cargo.toml files (core, communication, camtools, designer, gcodeeditor, visualizer, settings, devicedb, ui) to use `workspace = true` for these dependencies.
```

### Task 2: Fix `devicedb` Version Mismatches
Update `gcodekit4-devicedb` to use the modern versions of dependencies, aligning it with the rest of the project.

**Prompt:**
```text
Update crates/gcodekit4-devicedb/Cargo.toml to align with the workspace versions.
1. Ensure `slint`, `uuid`, and `thiserror` are using `workspace = true`.
2. Verify that the code in `gcodekit4-devicedb` compiles with the newer versions (slint 1.14, uuid 1.6). Fix any breaking changes if they occur.
```

### Task 3: Remove `lazy_static`
Replace `lazy_static` with `std::sync::OnceLock` or `once_cell` to remove the dependency.

**Prompt:**
```text
Remove the `lazy_static` dependency from `gcodekit4-camtools`, `gcodekit4-designer`, and `gcodekit4-visualizer`.
1. Identify usages of `lazy_static!` macro.
2. Refactor them to use `std::sync::OnceLock` (or `once_cell::sync::Lazy` if MSRV < 1.70, but prefer std).
3. Remove `lazy_static` from the Cargo.toml files.
```

### Task 4: Evaluate and Remove `config` from UI
Check if `config` crate usage in `ui` can be replaced by `gcodekit4-settings` logic.

**Prompt:**
```text
Analyze the usage of the `config` crate in `crates/gcodekit4-ui`.
1. If it is used for loading settings, refactor the code to use the `gcodekit4-settings` crate instead, which already handles TOML serialization/deserialization.
2. Remove `config` from `crates/gcodekit4-ui/Cargo.toml`.
```

### Task 5: Remove `ropey` from UI
Check if `ropey` usage in `ui` is redundant.

**Prompt:**
```text
Analyze the usage of `ropey` in `crates/gcodekit4-ui`.
1. If `ropey` is only used to interact with `gcodekit4-gcodeeditor`, ensure that `gcodeeditor` exposes the necessary high-level API so `ui` doesn't need to depend on `ropey` directly.
2. Remove `ropey` from `crates/gcodekit4-ui/Cargo.toml` if possible.
```
