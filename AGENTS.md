# Agent Guidelines for gcodekit

## Technology Stack
- **Language**: Rust edition 2021 or greater
- **UI Framework**: slint

## Build Commands
- `cargo build` - Build debug binary, timeout is 600 seconds min
- `cargo build --release` - Build optimized release binary timeout is 600 seconds min
- `cargo check` - Check code without building
- Build only debug builds unless specificaly asked to perform a `release build`

## Test Commands
- `cargo test` - Run all tests
- `cargo test <test_function_name>` - Run specific test function
- `cargo test -- --nocapture` - Run tests with output visible
- `cargo test --lib` - Test library only (skip integration tests)
- **Test Timeout**: All test runs should have a 10-minute timeout to prevent hanging
  - Use `timeout 600 cargo test` on Unix/Linux
  - Use `cargo test --test-threads=1` for sequential execution if needed

### Test Organization
All tests **MUST** be located in the `tests/` folder organized by module hierarchy, NOT inline in source files:

- Use `#[test]` for sync tests and `#[tokio::test]` for async tests
- Import from the public `gcodekit4` crate (e.g., `use gcodekiti4::communication::GrblController;`)
- Be organized with related tests grouped together
- Follow naming convention: `test_<component>_<scenario>` (e.g., `test_jog_x_positive`)

## Lint & Format Commands
- `cargo clippy` - Run linter with clippy
- `cargo fmt` - Format code with rustfmt
- `cargo fmt --check` - Check formatting without changes

## Github access
- use "gh" to access all github repositories. 
- when asked to "push to remote", update the SPEC.md, README.md, STATS.md and CHANGELOG.md files with all recent activity and spec changes, construct a suitable commit message based on recent activity, commit all changes and push the changes to the remote repository.
- when asked to "push release to remote", update the release number, and then follow the "push to remote" process
- When initialising a new repo add BUG, FEATURE and CHANGE issue templates. 

## Changelog Management
- **CHANGELOG.md**: Maintain a changelog in the root directory documenting all changes before each push to remote.
- **Format**: Follow Keep a Changelog format (https://keepachangelog.com/)
- **Update Timing**: Update CHANGELOG.md before each push to remote with the latest changes, features, fixes, and improvements.
- **Version**: Use semantic versioning (major.minor.patch-prerelease) 

## Documentation standards 
-  For all functions create DOCBLOCK documentation comments above each function that describes the purpose of the function, and documents any arguments and return vaulues
-  For all modules place a DOCBLOCK at the top of the File that describes the purpose of the module, and any dependancies.
-  **Documentation Files**: All documentation markdown files (*.md) **MUST** be located in the `docs/` folder, except for `STATS.md`, `SPEC.md`, `AGENTS.md`, `README.md` , `PLAN.md` and `CHANGELOG.md` which remain in the project root. This includes: implementation guides, architecture documentation, feature specifications, task breakdowns, user guides, API references, and any other markdown documentation. Any future documentation should be created in the docs/ folder following this convention.
-  Do not create explainer documents or other .md files unless specificaly asked to.
-  **Test Organization**: All tests **MUST** be located in the `tests/` folder organized by module hierarchy, mirroring the `src/` directory structure, NOT inline in source files.


## Code Style Guidelines
- **Formatting**: 4 spaces, max 100 width, reorder_imports=true, Unix newlines
- **Naming**: snake_case for functions/variables, PascalCase for types/structs/enums
- **Imports**: Group std, external crates, then local modules; reorder automatically
- **Error Handling**: Use `Result<T, E>` with `?`, `anyhow::Result` for main, `thiserror` for custom errors
- **Types**: Prefer explicit types, use type aliases for complex types
- **Async**: Use `tokio` runtime, `async-trait` for trait methods
- **Logging**: Use `tracing` with structured logging, avoid `println!` in production
- **Documentation**: `//!` for crate docs, `///` for public APIs, `//` for internal comments
- **Linting**: No wildcard imports, cognitive complexity ≤30, warn on missing docs
- **Best Practices**: Read the best practices at https://www.djamware.com/post/68b2c7c451ce620c6f5efc56/rust-project-structure-and-best-practices-for-clean-scalable-code and apply to the project.

## Issue Handling Process
When dealing with issues in the remote repository:
1. **Analyze and Comment**: Place a comment in the issue that records your analysis of the issue and your proposed plan for fixing it
2. **Implement Fix**: After analysis, implement the proposed solution
3. **Wait for Confirmation**: Do not close the issue until the reporter has confirmed the fix is working 

## Work flow

1. When asked "whats next", present a list of the top 9 unimplemented tasks by task number, accept a task number and perform that task. 
2. Dont suggest features unless asked to 

## Versioning

1. During development the release number will have "-alpha" appended to the end as per semantic versioning standards. 

