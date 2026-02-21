# Track Verification: Hierarchy Update

## Phase 1 Verification: Test Compilation
- Run `cargo fmt` and `cargo check`.
- Validate that models construct adequately.

## Phase 2 Verification: Integration Checks
- Run `cargo test` and ensure models safely map their surrogate components to SurrealDB.
- CLI must not crash and reject badly typed subcommands.
