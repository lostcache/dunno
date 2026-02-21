# Track Plan: Graph-Native Schema Redesign

## Phase 1: Model Rewrite (`src/models.rs`)
- [ ] Task: Remove FK fields from all structs (`Module.project_id`, `Submodule.module_id`, `File.module_id`/`submodule_id`, `Task.module_id`, `TaskUpdate.task_id`, `TodoItem.project_id`/`task_id`).
- [ ] Task: Remove `KnowledgeEdge` struct entirely.
- [ ] Task: Remove `CategoryTag` struct entirely.
- [ ] Task: Remove `Skill` struct (deprecated, not in target ER).
- [ ] Task: Add `Subtask` struct with fields: `id`, `name`, `description`, `status`.
- [ ] Task: Add `SecurityDetail` struct with fields: `id`, `content`, `severity`, `category`, `tags`.
- [ ] Task: Update all existing unit tests for new struct shapes.
- [ ] Task: Add serialization unit tests for `Subtask` and `SecurityDetail`.

## Phase 2: DB Layer Rewrite (`src/db.rs`)
- [ ] Task: Rewrite `create_project` — `CREATE project SET ...` (no FK, no edge needed for top-level).
- [ ] Task: Rewrite `create_module(name, desc, project_id)` — `CREATE module ... ; RELATE project -> contains -> module`.
- [ ] Task: Rewrite `create_submodule(name, desc, module_id)` — `CREATE submodule ...; RELATE module -> contains -> submodule`.
- [ ] Task: Rewrite `create_file(name, path, parent_id)` — `CREATE file ...; RELATE parent -> contains -> file` (parent is module or submodule).
- [ ] Task: Rewrite `create_task(name, desc, module_id)` — `CREATE task ...; RELATE module -> contains -> task`.
- [ ] Task: Add `create_subtask(name, desc, task_id)` — `CREATE subtask ...; RELATE task -> contains -> subtask`.
- [ ] Task: Add `create_security_detail(content, severity, category, tags)` — `CREATE security_detail SET ...`.
- [ ] Task: Rewrite `list_modules` to use `SELECT ->contains->module.* FROM $project_id`.
- [ ] Task: Rewrite `list_submodules_by_module` to use `SELECT ->contains->submodule.* FROM $module_id`.
- [ ] Task: Rewrite `list_files_by_module` / `list_files_by_submodule` to use `->contains->file.*`.
- [ ] Task: Rewrite `list_tasks` to support filtering by module via `->contains->task.*`.
- [ ] Task: Add `list_subtasks_by_task` via `SELECT ->contains->subtask.* FROM $task_id`.
- [ ] Task: Replace `create_edge` with `link_context(from_id, to_id)` — `RELATE $from -> has_context -> $to`.
- [ ] Task: Remove `get_edges_from`, `list_edges`, `fetch_knowledge_node_json`, `create_or_get_category_tag`, `list_category_tags`.
- [ ] Task: Rewrite all DB unit tests for the new RELATE-based approach.

## Phase 3: Context Retrieval Rewrite (`src/context.rs`)
- [ ] Task: Rewrite `get_task_context` — single SurrealQL query traversing Task `<-contains<-` Module `<-contains<-` Project, collecting `->has_context->` at each level.
- [ ] Task: Rewrite `get_file_context` — single SurrealQL query traversing File `<-contains<-` Submodule (optional) `<-contains<-` Module `<-contains<-` Project.
- [ ] Task: Add `get_subtask_context` — single SurrealQL query traversing Subtask `<-contains<-` Task `<-contains<-` Module `<-contains<-` Project.
- [ ] Task: Remove `get_linked_context` helper (absorbed into single queries).
- [ ] Task: Remove `dedup_context_nodes` helper (dedup handled in SurrealQL via `array::distinct`).

## Phase 4: CLI Updates (`src/args.rs`, `src/main.rs`, `src/ingest.rs`)
- [ ] Task: Update `ModuleCommands::Create` — change positional `project_id` to `--project-id` named flag.
- [ ] Task: Update `SubmoduleCommands::Create` — change positional `module_id` to `--module-id` named flag.
- [ ] Task: Update `TaskCommands::Create` — change positional `module_id` to `--module-id` named flag.
- [ ] Task: Update `FileCommands::Create` — change positional `module_id` to `--parent-id` named flag (accepts module or submodule ID).
- [ ] Task: Add `SubtaskCommands` subcommand group: `create --task-id <TASK_ID> <NAME> <DESC>`, `list --task-id <TASK_ID>`, `update`.
- [ ] Task: Add `--subtask-id` to the `Context` command (conflicts with `--task-id` and `--file-id`).
- [ ] Task: Wire all new commands in `src/main.rs`.
- [ ] Task: Add `"security"` match arm in `src/ingest.rs` that creates a `SecurityDetail` record.
- [ ] Task: Remove `CategoryTag` edge creation from `src/ingest.rs` (lines 52-57 current).
- [ ] Task: Update `link_to` logic in `src/ingest.rs` to call `db.link_context()` instead of `db.create_edge()`.

## Phase 5: Tests & Validation
- [ ] Task: Rewrite `tests/integration_tests.rs` — all 3 existing tests updated for no-FK create patterns + RELATE-based context.
- [ ] Task: Add integration test for subtask context traversal (4-level chain: subtask -> task -> module -> project).
- [ ] Task: Add integration test for SecurityDetail in context results.
- [ ] Task: Run `cargo test --all` — all pass.
- [ ] Task: Run `cargo fmt --all --check` — clean.
- [ ] Task: Run `cargo clippy --all-targets --all-features -- -D warnings` — no warnings.

## Phase 6: Documentation Updates
- [ ] Task: Update `README.md` CLI command signatures to reflect named parent-ID flags.
- [ ] Task: Remove references to `KnowledgeEdge` from all docs.
- [ ] Task: Update conductor entity tables to mark everything as implemented.
