# Track Plan: ER Model Completion (Subtask & SecurityDetail)

## Phase 1: Model Additions
- [ ] Task: Add `Subtask` struct to `src/models.rs` with `task_id`, `name`, `description`, `status` fields.
- [ ] Task: Add `SecurityDetail` struct to `src/models.rs` with `content`, `severity`, `category`, `tags` fields.
- [ ] Task: Add serialization unit tests for both new models.

## Phase 2: DB Operations
- [ ] Task: Add `create_subtask`, `get_subtask`, `list_subtasks`, `list_subtasks_by_task` methods in `src/db.rs`.
- [ ] Task: Add `create_security_detail`, `get_security_detail`, `list_security_details` methods in `src/db.rs`.
- [ ] Task: Update `fetch_knowledge_node_json` in `src/db.rs` to handle `security_detail:` prefix.
- [ ] Task: Add DB integration tests for both new entities.

## Phase 3: CLI Commands
- [ ] Task: Add `Subtask` subcommand group to `src/args.rs` (create, list, update).
- [ ] Task: Update `src/main.rs` command interpreter to handle Subtask CRUD operations.
- [ ] Task: Extend `dunno add --type security` to create SecurityDetail records in `src/ingest.rs`.

## Phase 4: Context Retrieval Updates
- [ ] Task: Add `get_subtask_context` function in `src/context.rs` (traverses Subtask -> Task -> Module -> Project).
- [ ] Task: Update `src/args.rs` to accept `--subtask-id` in the Context command.
- [ ] Task: Wire `--subtask-id` context path in `src/main.rs`.

## Phase 5: Validation
- [ ] Task: Run full test suite (`cargo test --all`).
- [ ] Task: Verify `cargo fmt --all --check` and `cargo clippy` pass.
- [ ] Task: Update integration tests in `tests/integration_tests.rs` for the new entities.
