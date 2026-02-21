# Track Plan: Graph-Native Schema Redesign

## Phase 1: Model Rewrite (`src/models.rs`) ✅ DONE
- [x] Task: Remove FK fields from all structs (`Module.project_id`, `Submodule.module_id`, `File.module_id`/`submodule_id`, `Task.module_id`, `TaskUpdate.task_id`, `TodoItem.project_id`/`task_id`).
- [x] Task: Remove `KnowledgeEdge` struct entirely.
- [x] Task: Remove `CategoryTag` struct entirely.
- [x] Task: Remove `Skill` struct (deprecated, not in target ER).
- [x] Task: Add `Subtask` struct with fields: `id`, `name`, `description`, `status`.
- [x] Task: Add `SecurityDetail` struct with fields: `id`, `content`, `severity`, `category`, `tags`.
- [x] Task: Update all existing unit tests for new struct shapes.
- [x] Task: Add serialization unit tests for `Subtask` and `SecurityDetail`.

## Phase 2: DB Layer Rewrite (`src/db.rs`) ✅ DONE
- [x] Task: Rewrite `create_project` — `CREATE project SET ...` (no FK, no edge needed for top-level).
- [x] Task: Rewrite `create_module(name, desc, project_id)` — `CREATE module ... ; RELATE project -> contains -> module`.
- [x] Task: Rewrite `create_submodule(name, desc, module_id)` — `CREATE submodule ...; RELATE module -> contains -> submodule`.
- [x] Task: Rewrite `create_file(name, path, parent_id)` — `CREATE file ...; RELATE parent -> contains -> file` (parent is module or submodule).
- [x] Task: Rewrite `create_task(name, desc, module_id)` — `CREATE task ...; RELATE module -> contains -> task`.
- [x] Task: Add `create_subtask(name, desc, task_id)` — `CREATE subtask ...; RELATE task -> contains -> subtask`.
- [x] Task: Add `create_security_detail(content, severity, category, tags)` — `CREATE security_detail SET ...`.
- [x] Task: Rewrite `list_modules` to use `SELECT ->contains->module.* FROM $project_id`.
- [x] Task: Rewrite `list_submodules_by_module` to use `SELECT ->contains->submodule.* FROM $module_id`.
- [x] Task: Rewrite `list_files_by_module` / `list_files_by_submodule` to use `->contains->file.*`.
- [x] Task: Rewrite `list_tasks` to support filtering by module via `->contains->task.*`.
- [x] Task: Add `list_subtasks_by_task` via `SELECT ->contains->subtask.* FROM $task_id`.
- [x] Task: Replace `create_edge` with `link_context(from_id, to_id)` — `RELATE $from -> has_context -> $to`.
- [x] Task: Remove `get_edges_from`, `list_edges`, `fetch_knowledge_node_json`, `create_or_get_category_tag`, `list_category_tags`.
- [x] Task: Rewrite all DB unit tests for the new RELATE-based approach.

## Phase 3: Context Retrieval Rewrite (`src/context.rs`) ✅ DONE (code written, needs integration test validation)
- [x] Task: Rewrite `get_task_context` — single SurrealQL query traversing Task `<-contains<-` Module `<-contains<-` Project, collecting `->has_context->` at each level.
- [x] Task: Rewrite `get_file_context` — single SurrealQL query traversing File `<-contains<-` Submodule (optional) `<-contains<-` Module `<-contains<-` Project.
- [x] Task: Add `get_subtask_context` — single SurrealQL query traversing Subtask `<-contains<-` Task `<-contains<-` Module `<-contains<-` Project.
- [x] Task: Remove `get_linked_context` helper (absorbed into single queries).
- [x] Task: Remove `dedup_context_nodes` helper (dedup handled in SurrealQL via `array::distinct`).

## Phase 4: CLI Updates (`src/args.rs`, `src/main.rs`, `src/ingest.rs`) ✅ DONE
- [x] Task: Update `ModuleCommands::Create` — change positional `project_id` to `--project-id` named flag.
- [x] Task: Update `SubmoduleCommands::Create` — change positional `module_id` to `--module-id` named flag.
- [x] Task: Update `TaskCommands::Create` — change positional `module_id` to `--module-id` named flag.
- [x] Task: Update `FileCommands::Create` — change positional `module_id` to `--parent-id` named flag (accepts module or submodule ID).
- [x] Task: Add `SubtaskCommands` subcommand group: `create --task-id <TASK_ID> <NAME> <DESC>`, `list --task-id <TASK_ID>`, `update`.
- [x] Task: Add `--subtask-id` to the `Context` command (conflicts with `--task-id` and `--file-id`).
- [x] Task: Wire all new commands in `src/main.rs`.
- [x] Task: Add `"security"` match arm in `src/ingest.rs` that creates a `SecurityDetail` record.
- [x] Task: Remove `CategoryTag` edge creation from `src/ingest.rs` (lines 52-57 current).
- [x] Task: Update `link_to` logic in `src/ingest.rs` to call `db.link_context()` instead of `db.create_edge()`.

## Phase 5: Tests & Validation ✅ DONE
- [x] Task: All 27 unit tests pass (`cargo test --lib`).
- [x] Task: Rewrite `tests/integration_tests.rs` — all 5 tests updated for no-FK create patterns + RELATE-based context. Added subtask + SecurityDetail tests.
- [x] Task: **FIXED**: Context query issue - the SurrealDB 3.0 query index was correct (8 for task context, 12 for file, 11 for subtask) but the `flatten_context_result` function was looking at wrong nesting level. Fixed to navigate through `->has_context` key to find knowledge nodes.
- [x] Task: Run `cargo fmt --all --check` — passes.
- [x] Task: Run `cargo clippy --all-targets --all-features -- -D warnings` — passes (fixed pre-existing clippy warnings in config.rs and db.rs).

## Phase 6: Documentation Updates ✅ DONE
- [x] Task: Update `README.md` CLI command signatures to reflect named parent-ID flags.
- [x] Task: Remove references to `KnowledgeEdge` from all docs.
- [x] Task: Update conductor entity tables to mark everything as implemented.

## Notes
- Branch: `feat/graph-native-schema`
- Commit: `520cbfe` — phases 1-4 code complete, 27 unit tests pass
- Commit `HEAD` (this PR): Phase 5 complete - fixed context query flattening, all tests pass
- SurrealDB 3.0 renamed `type::thing()` to `type::record()` — fixed
- `Mistake` kept minimal (just `id` + `content`) per user direction — no `category`/`tags` fields
- `VectorDB` removed from `context.rs` and `main.rs` (was unused no-op)
