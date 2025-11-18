# Vector Engraver Bug Fixes

## Issues Fixed

### 1. Multi-Pass Not Implemented
**Problem**: Vector engraver was not performing multiple passes as specified in `num_passes` parameter, only engraving once regardless of settings.

**Root Cause**: The `generate_gcode_with_progress` function only looped through paths once and did not implement the outer multi-pass loop.

**Solution**: 
- Wrapped the path loop with an outer pass loop
- Added pass counter and Z-axis depth adjustment between passes
- Z is decremented by `z_increment * pass_number` for each successive pass
- Progress tracking now accounts for total passes

**Code Changes** (crates/gcodekit4-parser/src/processing/vector_engraver.rs):
- Lines 1132-1190: Replaced single-pass loop with multi-pass implementation
- Now generates proper G-code comments indicating which pass is active
- Z-axis moves between passes when `multi_pass` is enabled

**Testing**: Added comprehensive tests in `tests/test_vector_engraver_multipass.rs`:
- `test_multipass_generation`: Verifies 3 passes are generated with proper Z increments
- `test_single_pass_no_multipass`: Confirms single pass behavior when disabled

### 2. Laser Stays Enabled at Path End (Creating Dots)
**Problem**: Laser was being left enabled while the machine moved between paths, creating burn dots at the end of each path.

**Root Cause**: The laser was engaged before moving to the first point of the path. The move to first point should be rapid (G0 without cutting), then engage laser.

**Solution**:
- Changed move to path start from G1 (cutting) to G0 (rapid) before laser engagement
- Laser now only engages immediately before cutting starts
- Laser is explicitly disabled (M5) before moving to the next path
- This prevents any dwell time with laser enabled during travel

**Testing**: Added `test_laser_disabled_at_path_end` test to verify:
- M5 (laser off) commands are >= M3 (laser on) commands
- Laser is disabled after each path before travel

## Technical Details

### Multi-Pass Implementation
```
For each pass:
  For each path:
    Rapid move to start (G0, laser off)
    Engage laser (M3 with power)
    Cut/engrave path (G1 moves)
    Disable laser (M5)
  If not last pass:
    Lower Z by z_increment
```

### Z-Axis Depth Control
- Pass 0: Z at 0
- Pass 1: Z = -z_increment
- Pass 2: Z = -2*z_increment
- etc.

## Test Results
All multi-pass tests pass:
- ✅ test_multipass_generation
- ✅ test_single_pass_no_multipass  
- ✅ test_laser_disabled_at_path_end

## Files Modified
- `crates/gcodekit4-parser/src/processing/vector_engraver.rs` - Core fix
- `crates/gcodekit4-camtools/src/vector_engraver.rs` - Synced fix
- `src/main.rs` - Fixed unused result warning
- `tests/test_vector_engraver_multipass.rs` - New comprehensive tests
