# Track Plan: Epic Schema & Relations

## Phase 1: Schema & Relations (SurrealDB)

- [X] Task: Add an `epic` table to the SurrealDB schema.
- [X] Task: Add relation table / edge `has_epic` with `project -> has_epic -> epic`.
- [X] Task: Add relation table / edge `belongs_to_project` support for `epic` (`epic -> belongs_to_project -> project`).
- [X] Task: Add relation table / edge `has_user_story` support for `epic` (`epic -> has_user_story -> user_story`).
- [X] Task: Add relation table / edge `belongs_to_epic` for `user_story` (`user_story -> belongs_to_epic -> epic`).
- [X] Task: Add relation table / edge `has_task` support for `epic` (`epic -> has_task -> task`).
- [X] Task: Add relation table / edge `belongs_to_epic` for `task` (`task -> belongs_to_epic -> epic`).

## Phase 2: Models & Rust Types

- [X] Task: Add a Rust `Epic` struct (e.g., in `src/models.rs`) with fields:
  - `id: Option<String>`
  - `title: String`
  - `description: String`
- [X] Task: Add serialization / deserialization unit tests for `Epic`.

## Phase 3: DB Layer (`src/db/surreal/`)

- [X] Task: Implement `create_epic(project_id, title, description)`:
  - `CREATE epic SET ...`
  - `RELATE project -> has_epic -> epic`
  - `RELATE epic -> belongs_to_project -> project`
- [X] Task: Implement helpers to:
  - List epics for a given project via `project -> has_epic -> epic`.
  - Link an existing `user_story` to an `epic` (create `epic -> has_user_story -> user_story` and `user_story -> belongs_to_epic -> epic`).
  - Link an existing `task` to an `epic` (create `epic -> has_task -> task` and `task -> belongs_to_epic -> epic`).
  - List user stories for a given epic via `epic -> has_user_story -> user_story`.
  - List tasks for a given epic via `epic -> has_task -> task`.

## Phase 4: CLI & Context Integration

- [X] Task: Extend CLI args (e.g., `src/args.rs`) with:
  - `epic` command group (`create`, `list`, and optional linking commands).
  - Optional flags on `task` and `user-story` commands to attach to an epic (e.g., `--epic-id`).
- [X] Task: Wire new commands into `src/main.rs`.
- [X] Task: Extend context retrieval in `src/context.rs` to support epic context queries.
- [X] Task: Document epic context in README and product docs.

## Phase 5: Tests, Docs, and Migration

- [X] Task: Add unit and integration tests for:
  - Creating and listing epics.
  - Linking user stories and tasks to epics.
  - Context queries that include epics alongside user stories and tasks.
- [X] Task: Update `README.md` and conductor docs (`conductor/`) to describe the Epic layer and example CLI flows.
- [X] Task: Mark track as complete in `conductor/tracks.md`.

## Implementation Summary

All phases completed successfully:

1. **Schema** (`src/db/surreal/schema.rs`): Added `epic` table and 6 new relation types:
   - `has_epic` (project -> epic)
   - Updated `belongs_to_project` to include epic
   - Updated `has_user_story` to include epic
   - `belongs_to_epic` (user_story, task -> epic)
   - Updated `has_task` to include epic

2. **Models** (`src/models.rs`): Added `Epic` struct with serialization and unit tests.

3. **DB Layer** (`src/db/surreal/entities/epics.rs`): Implemented:
   - `create_epic()` - creates epic with bidirectional project links
   - `list_epics()`, `list_epics_by_project()`
   - `link_user_story_to_epic()`, `link_task_to_epic()`
   - `list_user_stories_by_epic()`, `list_tasks_by_epic()`
   - `list_epics_by_user_story()`, `list_epics_by_task()`
   - `get_epic_context_json()` - context retrieval for epics

4. **CLI** (`src/args.rs`, `src/main.rs`):
   - Added `Epic` command group with `create` and `list` subcommands
   - Added `--epic-ids` flag to `task create` command
   - Added `--epic-ids` flag to `user-story create` command
   - Added `--epic-id` flag to `user-story list` command
   - Added `--epic-id` flag to `context` command
   - Updated allowed edges list to include `has_epic` and `belongs_to_epic`

5. **Documentation**:
   - Updated `README.md` with Epic CLI examples
   - Updated Graph Schema table with epic relations
   - Updated `conductor/product.md` with Epic entity
   - Updated `conductor/tracks.md` to mark track complete

## Notes

- Relations introduced:
  - `project -> has_epic -> epic`
  - `epic -> belongs_to_project -> project`
  - `epic -> has_user_story -> user_story`
  - `user_story -> belongs_to_epic -> epic`
  - `epic -> has_task -> task`
  - `task -> belongs_to_epic -> epic`

- All 116 unit tests pass
- Code compiles with no errors (1 warning about unused function unrelated to this track)
