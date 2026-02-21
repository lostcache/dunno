# Track Plan: Hierarchy Update (Submodule and File)

## Phase 1: Model Additions
- [x] Task: Add `Submodule` and `File` structs to `src/models.rs`.
- [x] Task: Add JSON serialization logic tests.

## Phase 2: DB Operations
- [x] Task: Add `create_submodule`, `get_submodule`, and `list_submodules` methods in `src/db.rs`.
- [x] Task: Add `create_file`, `get_file`, and `list_files` methods in `src/db.rs`.

## Phase 3: CLI Commands
- [x] Task: Update `src/args.rs` to include `Submodule` and `File` commands subtrees.
- [x] Task: Update command interpreter in `src/main.rs` to execute operations handling the new structurally explicit entities.

## Phase 4: Validations
- [x] Task: Ensure tests pass natively and formatting logic is respected.
- [x] Task: Run compilation and test paths successfully.
