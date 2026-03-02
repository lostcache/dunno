# Track Plan: User Story Schema & Relations

## Phase 1: Schema & Models (SurrealDB + `src/models.rs`)

- [ ] Task: Add a `user_story` table to the SurrealDB schema.
- [ ] Task: Add relation table `has_user_story` with edges `project -> has_user_story -> user_story`.
- [ ] Task: Add relation table `belongs_to_project` support for `user_story` (`user_story -> belongs_to_project -> project`).
- [ ] Task: Add Rust `UserStory` struct in `src/models.rs` with fields:
  - `id: Option<String>`
  - `title: String`
  - `description: String`
- [ ] Task: Add serialization unit tests for `UserStory`.

## Phase 2: DB Layer (`src/db/surreal/`)

- [ ] Task: Implement `create_user_story(project_id, title, description)`:
  - `CREATE user_story SET ...`
  - `RELATE project -> has_user_story -> user_story`
  - `RELATE user_story -> belongs_to_project -> project`
- [ ] Task: Implement `list_user_stories_by_project(project_id)` using `SELECT ->has_user_story->user_story.* FROM $project_id`.
- [ ] Task: Implement `get_user_story(id)` to fetch a single story.

## Phase 3: Story ↔ Task relations

- [ ] Task: Add relation table `has_task` support for `user_story` (`user_story -> has_task -> task`).
- [ ] Task: Add relation table `belongs_to_story` (`task -> belongs_to_story -> user_story`).
- [ ] Task: Update task creation flow to optionally associate a task with a `user_story`:
  - When linking a task to a story, create `has_task` and `belongs_to_story` edges.
- [ ] Task: Add DB helpers to:
  - List tasks for a given user story via `->has_task->task.*`.
  - List user stories for a given task via `<-belongs_to_story<-user_story.*`.

## Phase 4: Story ↔ Module/Submodule relations

- [ ] Task: Add relation table `has_user_story_module` (or reuse a generic name) with edges `user_story -> has_module|has_submodule -> module|submodule`.
- [ ] Task: Add relation table `belongs_to_user_story` for `module`/`submodule` (`module|submodule -> belongs_to_user_story -> user_story`).
- [ ] Task: Implement DB helpers to:
  - Link an existing module or submodule to a user story.
  - List modules/submodules for a given user story.
  - List user stories that reference a given module or submodule.

## Phase 5: CLI & Context Integration

- [ ] Task: Extend CLI args (`src/args.rs`) with:
  - `user-story` command group (e.g., `create`, `list`).
  - Optional flags on `task` commands to attach to a user story (e.g., `--user-story-id`).
- [ ] Task: Wire new commands into `src/main.rs`.
- [ ] Task: Extend context retrieval in `src/context.rs` to optionally traverse:
  - `project -> has_user_story -> user_story`
  - `user_story -> has_task -> task`
  - `user_story -> has_module/submodule -> module/submodule`
- [ ] Task: Decide and document how user story context participates in knowledge inheritance (e.g., whether knowledge linked to a user story is visible from its tasks/modules/files).

## Phase 6: Tests, Docs, and Migration

- [ ] Task: Add unit and integration tests for:
  - Creating/listing user stories.
  - Linking tasks and modules/submodules to user stories.
  - Context queries that include user stories.
- [ ] Task: Update `README.md` and any conductor docs (`conductor/`) to describe the User Story layer and example CLI flows.
- [ ] Task: If needed, add a migration/seed step to backfill existing tasks/modules into user stories or to create a default story per project.

## Notes

- Branch: `feat/user-story-schema` (to be created when implementation starts).
- Relations to introduce:
  - `project -> has_user_story -> user_story`
  - `user_story -> belongs_to_project -> project`
  - `user_story -> has_task -> task`
  - `task -> belongs_to_story -> user_story`
  - `user_story -> has_module/submodule -> module/submodule`
  - `module/submodule -> belongs_to_user_story -> user_story`

