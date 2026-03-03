# Track Plan: User Story Schema & Relations

## Phase 1: Schema & Models (SurrealDB + `src/models.rs`)

- [x] Task: Add a `user_story` table to the SurrealDB schema.
- [x] Task: Add relation table `has_user_story` with edges `project -> has_user_story -> user_story`.
- [x] Task: Add relation table `belongs_to_project` support for `user_story` (`user_story -> belongs_to_project -> project`).
- [x] Task: Add Rust `UserStory` struct in `src/models.rs` with fields:
  - `id: Option<String>`
  - `title: String`
  - `description: String`
- [x] Task: Add serialization unit tests for `UserStory`.

## Phase 2: DB Layer (`src/db/surreal/`)

- [x] Task: Implement `create_user_story(project_id, title, description)`:
  - `CREATE user_story SET ...`
  - `RELATE project -> has_user_story -> user_story`
  - `RELATE user_story -> belongs_to_project -> project`
- [x] Task: Implement `list_user_stories_by_project(project_id)` using `SELECT ->has_user_story->user_story.* FROM $project_id`.
- [x] Task: Implement `get_user_story(id)` to fetch a single story.

## Phase 3: Story ↔ Task relations

- [x] Task: Add relation table `has_task` support for `user_story` (`user_story -> has_task -> task`).
- [x] Task: Add relation table `belongs_to_story` (`task -> belongs_to_story -> user_story`).
- [x] Task: Update task creation flow to optionally associate a task with a `user_story`:
  - When linking a task to a story, create `has_task` and `belongs_to_story` edges.
- [x] Task: Add DB helpers to:
  - List tasks for a given user story via `->has_task->task.*`.
  - List user stories for a given task via `->belongs_to_story->user_story.*`.

## Phase 4: Story ↔ Module/Submodule relations

- [x] Task: Add relation tables `has_module` and `has_submodule` with edges `user_story -> has_module|has_submodule -> module|submodule`.
- [x] Task: Add relation table `belongs_to_user_story` for `module`/`submodule` (`module|submodule -> belongs_to_user_story -> user_story`).
- [x] Task: Implement DB helpers to:
  - Link an existing module or submodule to a user story.
  - List modules/submodules for a given user story.
  - List user stories that reference a given module or submodule.

## Phase 5: CLI & Context Integration

- [x] Task: Extend CLI args (`src/args.rs`) with:
  - `user-story` command group (`create`, `list`).
  - Optional flags on `task` commands to attach to a user story (`--user-story-ids`).
- [x] Task: Wire new commands into `src/main.rs`.
- [x] Task: Handle task creation with user story linking.
- [x] Task: Update allowed edges list for `link` command.

## Phase 6: Tests, Docs, and Migration

- [x] Task: Add unit and integration tests for:
  - Creating/listing user stories.
  - Linking tasks and modules/submodules to user stories.
- [x] Task: Update README.md with User Story documentation.

## Summary

All phases completed successfully! The User Story entity is now fully integrated into the dunno knowledge graph.

### New CLI Commands

```bash
# Create a user story linked to a project
dunno user-story create --project-id project:abc "As a user..." "Story description"

# List all user stories
dunno user-story list

# List user stories for a specific project  
dunno user-story list --project-id project:abc

# Create a task linked to a user story
dunno task create --module-ids module:def --project-ids project:abc \
  --user-story-ids user_story:ghi "Task name" "Task description"
```

### Relations Added

- `project -> has_user_story -> user_story`
- `user_story -> belongs_to_project -> project`
- `user_story -> has_task -> task`
- `task -> belongs_to_story -> user_story`
- `user_story -> has_module -> module`
- `user_story -> has_submodule -> submodule`
- `module/submodule -> belongs_to_user_story -> user_story`

### Files Changed

- `src/models.rs` - Added UserStory model
- `src/db/surreal/schema.rs` - Added schema definitions
- `src/db/surreal/entities/user_stories.rs` - New file with DB operations
- `src/db/surreal/entities/mod.rs` - Added user_stories module
- `src/db/surreal/tests.rs` - Added integration tests
- `src/args.rs` - Added CLI arguments
- `src/main.rs` - Added command handlers
- `README.md` - Updated documentation
- `conductor/tracks.md` - Marked track as complete
