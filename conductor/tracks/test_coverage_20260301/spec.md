# Track Specification: Test Coverage & Stateful Test Layout

## Overview

This track defines where tests live and how they are structured, so that coverage stays high and test intent is obvious:

- **Unit tests** live in the same file as the code they test.
- **Multistep, stateful SurrealDB tests** live in a single, well-known module: `src/db/surreal/tests.rs`.
- Legacy crate-level DB integration tests in `tests/integration_tests.rs` are removed.

The goal is to make it trivial for both humans and AI agents to know where to add tests and how to exercise the data layer.

## Objectives

1. **Co-located unit tests:** Every Rust module with non-trivial logic has a `#[cfg(test)] mod tests` block in the same file.
2. **Centralized stateful tests:** All multistep, stateful SurrealDB tests (CRUD, hierarchy, linking, freestanding creation, purge, config) live in `src/db/surreal/tests.rs`.
3. **Clean integration surface:** Remove `tests/integration_tests.rs` so there is a single source of truth for SurrealDB multistep tests.
4. **Documented conventions:** Update conductor docs and the Rust style guide so contributors and AI agents follow the same testing layout.

## Current State

- Co-located unit tests already exist in:
  - `src/models.rs`
  - `src/config.rs`
  - `src/ingest.rs`
  - `src/vector_db.rs`
  - `src/db/surreal/flatten_context.rs`
- Multistep SurrealDB tests currently live in:
  - `src/db/surreal/tests.rs` (authoritative)
  - `tests/integration_tests.rs` (legacy, overlapping coverage)

This track removes the ambiguity by making `src/db/surreal/tests.rs` the only home for stateful SurrealDB tests.

## Constraints

- **Coverage target:** New code should maintain or improve overall coverage, aiming for >80% as defined in `conductor/workflow.md`.
- **Single source of truth:** No new DB/stateful tests may be added under `tests/` — that directory is reserved for future, explicit end-to-end CLI tests if needed.
- **Non-disruptive:** Removing `tests/integration_tests.rs` must not remove any unique coverage; equivalent flows must exist in `src/db/surreal/tests.rs`.

## Acceptance Criteria

- `tests/integration_tests.rs` is removed from the repository.
- `src/db/surreal/tests.rs` contains the multistep SurrealDB tests for:
  - CRUD
  - project/module/submodule/file/task/subtask/todo hierarchy
  - `link_context` and `get_belongs_to_targets`
  - freestanding + link-after-create flows
  - `purge_database`
  - config-driven DB initialization
- Each new or modified Rust module with logic has a `#[cfg(test)] mod tests` section with meaningful unit tests.
- `conductor/code_styleguides/rust.md` has a **Testing** section describing:
  - co-located unit tests
  - centralized SurrealDB stateful tests in `src/db/surreal/tests.rs`
  - the fact that `tests/integration_tests.rs` was removed and that DB tests should not be added under `tests/`.
