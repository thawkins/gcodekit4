# Test Migration Status - Issue gcodekit4-4

**Last Updated**: 2025-10-28  
**Commit**: 6aef56b  
**Progress**: 1/67 files (1.5%)

## Summary

Began systematic migration of all inline `#[cfg(test)]` tests from `src/` to `tests/` directory following AGENTS.md guidelines and the successful visualizer migration pattern (gcodekit4-1).

## Completed Files ✅

### Firmware Module (1/21)
1. **firmware/capabilities.rs** → tests/firmware/capabilities.rs
   - 3 tests migrated: `test_default_capabilities`, `test_axis_support`, `test_set_capability`
   - All tests passing
   - Public API testing only

## Remaining Work

### Phase 1: Firmware - HIGH Priority (20/21 remaining)
#### GRBL (6 files)
- firmware/grbl/capabilities.rs
- firmware/grbl/command_creator.rs
- firmware/grbl/response_parser.rs
- firmware/grbl/status_parser.rs
- firmware/grbl/utils.rs
- (4 already migrated: communicator, controller, override_manager, settings)

#### Other Firmware (14 files)
- firmware/connection_watch.rs
- firmware/file_service.rs
- firmware/override_manager.rs
- firmware/settings.rs
- firmware/fluidnc/* (3 files)
- firmware/g2core/capabilities.rs
- firmware/smoothieware/* (3 files)

### Phase 2: Processing - MEDIUM Priority (8 files)
- processing/advanced_features.rs
- processing/arc_expander.rs
- processing/comment_processor.rs
- processing/core_infrastructure.rs
- processing/optimizer.rs
- processing/stats.rs
- processing/toolpath.rs
- processing/validator.rs

### Phase 3: UI - LOW Priority (28 files)
- All ui/*.rs files (may need API rethinking)

### Phase 4: Utils - MEDIUM Priority (10 files)
- utils/advanced.rs
- utils/export.rs
- utils/file_io.rs
- utils/phase6_extended.rs
- utils/phase7.rs
- utils/processing.rs

## Migration Pattern

```rust
// tests/firmware/module_name.rs
use gcodekit4::firmware::module_name::*;

#[test]
fn test_public_api_feature() {
    // Test using public API only
    assert!(condition);
}
```

## Key Changes Per File

1. Extract `#[cfg(test)]` module from source
2. Create `tests/{module}/{file}.rs`
3. Change `use super::*` to `use gcodekit4::{module}::*`
4. Remove private implementation tests
5. Delete inline test module from source
6. Update `tests/{module}/mod.rs` to include new file
7. Run `cargo test` to verify
8. Commit with descriptive message

## Statistics

- **Total files to migrate**: 67
- **Files migrated**: 1
- **Tests migrated**: 3
- **Tests passing**: 3
- **Estimated remaining**: 66 files, ~200-300 tests

## Documentation

- **Migration Plan**: docs/test_migration_plan.md
- **Status**: docs/test_migration_status.md (this file)
- **Reference**: Visualizer migration (gcodekit4-1): 28 → 102 tests

## Next Session Goals

1. Complete GRBL firmware tests (6 files)
2. Complete other firmware tests (14 files)
3. Target: Phase 1 complete (21/21 files)
4. Update progress tracking

## Notes

- Focus on public API testing
- Remove tests of private implementation details
- UI tests may require significant rework
- Some tests might need to become integration tests
- Follow visualizer pattern as model
- Commit frequently to track progress
