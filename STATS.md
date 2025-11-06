# Project Statistics

## Overview
- **Version:** 0.25.4-alpha
- **Last Updated:** 2025-11-06

## Code Metrics
- **Rust Code:** ~56,500 lines
- **Slint UI:** ~5,100 lines
- **Total:** ~61,600 lines
- **Test Coverage:** 694 comprehensive tests
- **Test Files:** 91 test modules

## Recent Changes (v0.25.4)
- **Real-Time Status Tracking**: Machine state parsing (Idle/Run/Hold/Alarm), 3-6 axis positions, feed/spindle
- **Fixed Serial Port Conflict**: Status polling now uses shared connection instead of duplicate port open
- **Enhanced Status Parser**: Full GRBL status parsing with machine state extraction
- **UI Improvements**: Reduced menu/tab spacing, better layout

## Module Breakdown
- **visualizer:** 2297 lines
- **utils:** 6006 lines
- **ui/ui_components:** 840 lines
- **ui_panels:** 2957 lines
- **ui:** 12984 lines
- **testing:** 501 lines
- **processing:** 1801 lines
- **gcode:** 2134 lines
- **firmware/tinyg:** 1510 lines
- **firmware/smoothieware:** 519 lines
- **firmware/grbl:** 3512 lines
- **firmware/g2core:** 1499 lines
- **firmware/fluidnc:** 552 lines
- **firmware:** 3120 lines
- **designer:** 10475 lines
- **data:** 1875 lines
- **core:** 784 lines
- **communication:** 1502 lines
