# Test Migration Plan - Issue gcodekit4-4

## Objective
Migrate all inline `#[cfg(test)]` tests from `src/` to `tests/` directory, organized by module hierarchy.

## Current Status
- **Total files with inline tests**: 67
- **Already migrated**: visualizer module (gcodekit4-1)
- **Test count**: ~530 tests currently

## Migration Strategy

### Phase 1: Firmware Modules (21 files)
Priority: HIGH - Core functionality

#### GRBL (10 files)
- [ ] src/firmware/grbl/capabilities.rs → tests/firmware/grbl/capabilities.rs
- [ ] src/firmware/grbl/command_creator.rs → tests/firmware/grbl/command_creator.rs
- [ ] src/firmware/grbl/communicator.rs → (already migrated)
- [ ] src/firmware/grbl/controller.rs → (already migrated)
- [ ] src/firmware/grbl/override_manager.rs → (already migrated)
- [ ] src/firmware/grbl/response_parser.rs → tests/firmware/grbl/response_parser.rs
- [ ] src/firmware/grbl/settings.rs → (already migrated)
- [ ] src/firmware/grbl/status_parser.rs → tests/firmware/grbl/status_parser.rs
- [ ] src/firmware/grbl/utils.rs → tests/firmware/grbl/utils.rs

#### Other Firmware (11 files)
- [ ] src/firmware/capabilities.rs → tests/firmware/capabilities.rs
- [ ] src/firmware/connection_watch.rs → tests/firmware/connection_watch.rs
- [ ] src/firmware/file_service.rs → tests/firmware/file_service.rs
- [ ] src/firmware/override_manager.rs → tests/firmware/override_manager.rs
- [ ] src/firmware/settings.rs → tests/firmware/settings.rs
- [ ] src/firmware/fluidnc/capabilities.rs → tests/firmware/fluidnc/capabilities.rs
- [ ] src/firmware/fluidnc/command_creator.rs → tests/firmware/fluidnc/command_creator.rs
- [ ] src/firmware/fluidnc/response_parser.rs → tests/firmware/fluidnc/response_parser.rs
- [ ] src/firmware/g2core/capabilities.rs → tests/firmware/g2core/capabilities.rs
- [ ] src/firmware/smoothieware/capabilities.rs → tests/firmware/smoothieware/capabilities.rs
- [ ] src/firmware/smoothieware/command_creator.rs → tests/firmware/smoothieware/command_creator.rs
- [ ] src/firmware/smoothieware/response_parser.rs → tests/firmware/smoothieware/response_parser.rs

### Phase 2: Processing Modules (8 files)
Priority: MEDIUM

- [ ] src/processing/advanced_features.rs → tests/processing/advanced_features.rs
- [ ] src/processing/arc_expander.rs → tests/processing/arc_expander.rs
- [ ] src/processing/comment_processor.rs → tests/processing/comment_processor.rs
- [ ] src/processing/core_infrastructure.rs → tests/processing/core_infrastructure.rs
- [ ] src/processing/optimizer.rs → tests/processing/optimizer.rs
- [ ] src/processing/stats.rs → tests/processing/stats.rs
- [ ] src/processing/toolpath.rs → tests/processing/toolpath.rs
- [ ] src/processing/validator.rs → tests/processing/validator.rs

### Phase 3: UI Modules (28 files)
Priority: LOW - Most tests may need rethinking

- [ ] src/ui/*.rs → tests/ui/*.rs (28 files)

### Phase 4: Utils Modules (10 files)
Priority: MEDIUM

- [ ] src/utils/advanced.rs → tests/utils/advanced.rs
- [ ] src/utils/export.rs → tests/utils/export.rs
- [ ] src/utils/file_io.rs → tests/utils/file_io.rs
- [ ] src/utils/phase6_extended.rs → tests/utils/phase6_extended.rs
- [ ] src/utils/phase7.rs → tests/utils/phase7.rs
- [ ] src/utils/processing.rs → tests/utils/processing.rs

## Migration Checklist Per File

For each file:
1. ✅ Extract `#[cfg(test)]` module from source file
2. ✅ Create corresponding test file in `tests/` directory
3. ✅ Add necessary imports (use `gcodekit4::` crate path)
4. ✅ Convert tests to test public API only
5. ✅ Remove inline test module from source file
6. ✅ Run tests to verify: `cargo test`
7. ✅ Commit changes

## Test Template

```rust
use gcodekit4::firmware::grbl::capabilities::*;

#[test]
fn test_feature_name() {
    // Test implementation using public API only
}
```

## Progress Tracking

- **Phase 1**: 0/21 complete
- **Phase 2**: 0/8 complete
- **Phase 3**: 0/28 complete
- **Phase 4**: 0/10 complete
- **Total**: 0/67 complete (0%)

## Notes

- Focus on public API testing
- Remove any tests that test private implementation details
- Add integration tests for complex interactions
- Follow visualizer module pattern (gcodekit4-1) as reference
