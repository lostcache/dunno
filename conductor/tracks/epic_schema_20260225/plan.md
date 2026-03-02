# Track Plan: Epic Schema & Relations

## Phase 1: Schema & Relations (SurrealDB)

- [ ] Task: Add an `epic` table to the SurrealDB schema.
- [ ] Task: Add relation table / edge `has_epic` with `project -> has_epic -> epic`.
- [ ] Task: Add relation table / edge `belongs_to_project` support for `epic` (`epic -> belongs_to_project -> project`).
- [ ] Task: Add relation table / edge `has_user_story` support for `epic` (`epic -> has_user_story -> user_story`).
- [ ] Task: Add relation table / edge `belongs_to_epic` for `user_story` (`user_story -> belongs_to_epic -> epic`).
- [ ] Task: Add relation table / edge `has_task` support for `epic` (`epic -> has_task -> task`).
- [ ] Task: Add relation table / edge `belongs_to_epic` for `task` (`task -> belongs_to_epic -> epic`).

## Phase 2: Models & Rust Types

- [ ] Task: Add a Rust `Epic` struct (e.g., in `src/models.rs`) with fields:
  - `id: Option<String>`
  - `title: String`
  - `description: String`
- [ ] Task: Add serialization / deserialization unit tests for `Epic`.

## Phase 3: DB Layer (`src/db/surreal/`)

- [ ] Task: Implement `create_epic(project_id, title, description)`:
  - `CREATE epic SET ...`
  - `RELATE project -> has_epic -> epic`
  - `RELATE epic -> belongs_to_project -> project`
- [ ] Task: Implement helpers to:
  - List epics for a given project via `project -> has_epic -> epic`.
  - Link an existing `user_story` to an `epic` (create `epic -> has_user_story -> user_story` and `user_story -> belongs_to_epic -> epic`).
  - Link an existing `task` to an `epic` (create `epic -> has_task -> task` and `task -> belongs_to_epic -> epic`).
  - List user stories for a given epic via `epic -> has_user_story -> user_story`.
  - List tasks for a given epic via `epic -> has_task -> task`.

## Phase 4: CLI & Context Integration

- [ ] Task: Extend CLI args (e.g., `src/args.rs`) with:
  - `epic` command group (`create`, `list`, and optional linking commands).
  - Optional flags on `task` and `user-story` commands to attach to an epic (e.g., `--epic-id`).
- [ ] Task: Wire new commands into `src/main.rs`.
- [ ] Task: Extend context retrieval in `src/context.rs` to optionally traverse:
  - `project -> has_epic -> epic`
  - `epic -> has_user_story -> user_story`
  - `epic -> has_task -> task`
- [ ] Task: Decide and document how epic context participates in knowledge inheritance (e.g., whether knowledge linked to an epic is visible from its user stories and tasks, and vice versa).

## Phase 5: Tests, Docs, and Migration

- [ ] Task: Add unit and integration tests for:
  - Creating and listing epics.
  - Linking user stories and tasks to epics.
  - Context queries that include epics alongside user stories and tasks.
- [ ] Task: Update `README.md` and conductor docs (`conductor/`) to describe the Epic layer and example CLI flows.
- [ ] Task: If needed, add a migration/seed step to backfill existing user stories and tasks into epics or to create default epics per project.

## Notes

- Relations to introduce:
  - `project -> has_epic -> epic`
  - `epic -> belongs_to_project -> project`
  - `epic -> has_user_story -> user_story`
  - `user_story -> belongs_to_epic -> epic`
  - `epic -> has_task -> task`
  - `task -> belongs_to_epic -> epic`

