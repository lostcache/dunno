# Track Plan: Test Coverage & Stateful Test Layout

## Phase 1: Document Conventions

- [x] Task: Add a **Testing** section to `conductor/code_styleguides/rust.md` that specifies:
  - Unit tests live in the same `.rs` file as the code they test, inside `#[cfg(test)] mod tests`.
  - Multistep, stateful SurrealDB tests live exclusively in `src/db/surreal/tests.rs` using `#[tokio::test]` and `DB::new("mem://")`.
  - No DB/stateful tests belong under `tests/` (that directory is reserved for future explicit end-to-end CLI tests).

## Phase 2: Clean Up Legacy Integration Tests

- [x] Task: Remove `tests/integration_tests.rs` from the repository; treat it as a legacy, fake integration test module.
- [x] Task: Verify that equivalent multistep flows (context hierarchies, `belongs_to` reverse edges, file/subtask context, freestanding + link-after-create) are already covered in `src/db/surreal/tests.rs`.

## Phase 3: Audit & Backfill Unit Tests

- [x] Task: Audit `src/*.rs` and `src/db/surreal/**/*.rs` for files that lack a `#[cfg(test)] mod tests` section.
- [x] Task: For modules with non-trivial logic and no tests, add co-located unit tests that exercise their pure logic (no DB state).

## Phase 4: Conductor Wiring

- [X] Task: Add this track to `conductor/tracks.md` as \"Test Coverage & Stateful Test Layout\" with a link to `conductor/tracks/test_coverage_20260301/`.
- [X] Task: Conductor - User Manual Verification 'Test Coverage & Stateful Test Layout' (Protocol in `conductor/workflow.md`), including:
  - `cargo test --all` passes.
  - No DB/stateful tests live under `tests/`.
  - New or modified modules have co-located unit tests where appropriate.

