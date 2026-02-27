# Track Plan: DB & CLI Flexible Create/Link APIs

## Goals

- **DB API**: Each structural node type (`project`, `module`, `submodule`, `file`, `task`, `subtask`, `todo_item`, `context`) can be created as a free-standing record, and optionally linked to one or more related nodes (single or multiple) when IDs are provided.
- **CLI**: Provide three clear patterns for the AI agent: (1) create-only commands that make freestanding nodes, (2) a generic `link` command that supports single and multiple links, and (3) `create` commands that accept optional single or multiple link IDs.
- **Safety**: Preserve existing graph semantics (`contains`, `has_task`, `has_subtask`, `has_todo`, `has_context`, `belongs_to_*`) and avoid breaking current tests/flows.

## High-level design

### DB layer pattern

- Introduce internal helpers in the SurrealDB layer that create a bare record for each node type without adding relationships, e.g. `create_project_record`, `create_module_record`, `create_task_record`, etc., in the relevant entity files like `src/db/surreal/entities/projects.rs`, `modules.rs`, `tasks.rs`, `files.rs`, `todos.rs`, and `knowledge.rs`.
- Refactor existing `create_*` functions so they:
  - Call the corresponding `*_record` helper to insert the row.
  - **Optionally** wire up relationships when parent IDs are provided (converted to `Option<String>` or separate overloads).
- Keep the relationship wiring logic explicit per type to preserve current edge semantics, e.g.:
  - `module` ↔ `project` via `contains`.
  - `submodule` ↔ `module` via `contains`.
  - `file` ↔ `module|submodule` via `contains`.
  - `task` ↔ `project` via `has_task` / `belongs_to_project`, and ↔ `module` via `belongs_to_module`.
  - `subtask` ↔ `task` via `has_subtask` / `belongs_to_task`.
  - `todo_item` ↔ `project` via `has_todo`.
  - `context` ↔ structural nodes via `has_context` and reverse `belongs_to_*`.

### DB API surface changes

- Update public `DB` methods that are used by the CLI in `src/db/surreal/entities/*.rs` to support optional linking:
  - `create_module(&self, name, description, project_id: Option<&str>)` and only call `self.relate(project_id, "contains", module_id)` when `Some`.
  - `create_submodule(&self, name, description, module_id: Option<&str>)` with conditional `contains` relation.
  - `create_file(&self, name, path, parent_id: Option<&str>)` with conditional `contains` relation.
  - `create_task(&self, name, description, module_id: Option<&str>, project_id: Option<&str>)` with conditional `has_task`/`belongs_to_*` edges (if both provided) and a defined behavior if only one is present (either reject or support partial linking; likely reject from DB layer and surface as CLI validation for now).
  - `create_subtask(&self, name, description, task_id: Option<&str>)` with conditional `has_subtask`/`belongs_to_task`.
  - `create_todo(&self, content, project_id: Option<&str>)` with optional `has_todo` relation.
- For `Context`, keep `create_context` as the pure record-creation API and leave `link_context` for relations; treat this as the canonical pattern the other entities now follow.
- Where necessary, keep or add non-optional wrappers to preserve current internal callsites (e.g. test helpers) while moving the core logic to the new generalized functions.

### CLI create commands (freestanding vs linked)

- In `src/args.rs`, relax required link parameters into **optional, repeatable** ones where appropriate so each `create` command supports:
  - **Freestanding mode**: call without any link flags to create an isolated node.
  - **Linked mode (single or multiple)**: supply one or more IDs to establish multiple relationships in one go (e.g. repeatable `--link-to` or `--parent-id` flags).
- Concretely, model these as `Vec<String>` fields in the clap definitions, for example:
  - `ModuleCommands::Create { project_ids: Vec<String>, name, description }`.
  - `SubmoduleCommands::Create { module_ids: Vec<String>, name, description }`.
  - `FileCommands::Create { parent_ids: Vec<String>, name, path }`.
  - `TaskCommands::Create { module_ids: Vec<String>, project_ids: Vec<String>, name, description }` (with validation to avoid ambiguous combinations).
  - `SubtaskCommands::Create { task_ids: Vec<String>, name, description }`.
  - `TodoCommands::Create { project_ids: Vec<String>, content }`.
- In `src/main.rs`, update the command handling to:
  - Call the updated DB APIs once to create the freestanding record.
  - Then iterate over the provided IDs (if any) and call the appropriate linking helpers (`relate`, `link_context`, etc.) so that a single `create` invocation can attach multiple relationships.
  - Optionally perform light validation where a particular invariant is required for useful behavior, e.g. for `TaskCommands::Create` restrict to a single `(project_id, module_id)` pair or return a clear error if the combination is ambiguous.
- Keep `dunno add` behavior as the pattern reference: it already creates a freestanding `Context` and only calls `db.link_context` when one or more `--link-to` flags are present.

### New generic CLI link command (single and multiple)

- Extend `Commands` in `src/args.rs` with a new `Link` variant that supports one or many targets, for example:
  - `Link { from_id: String, edge: String, to_ids: Vec<String> }`.
- In `src/main.rs`:
  - Add a new match arm that calls a small helper in a loop, e.g. for each `to_id` do `db.relate(&from_id, &edge, &to_id).await?`, and then return a simple `{"status":"ok"}` JSON.
  - Optionally, add a thin validation layer restricting `edge` to known relationships (`contains`, `has_task`, `has_subtask`, `has_todo`, `has_context`, `belongs_to_project`, `belongs_to_module`, `belongs_to_task`) to avoid corrupting the graph.
- Document this in `README.md` as the low-level escape hatch that the AI agent can use to wire up arbitrary relationships between existing nodes, including multiple links from one source in a single command.

### Tests and invariants

- Update existing integration tests in `src/db/surreal/tests.rs` and `tests/integration_tests.rs` to:
  - Use the new `*_record` or optional-ID variants where they previously assumed auto-linking.
  - Add new tests that explicitly cover:
    - Creating each node type without any link IDs and asserting that the record exists and no graph edges are present.
    - Creating each node type with valid link IDs and asserting the correct edges (e.g. `contains`, `has_task`, `belongs_to_*`) are present and that downstream helpers like `get_task_hierarchy` still work.
    - The `Link` CLI command: create two freestanding nodes via CLI, call `dunno link ...`, and verify the relationship via an appropriate list/get helper.
- Keep the behavior of higher-level context APIs (`get_task_context`, file context flattening in `src/db/surreal/flatten_context.rs`) unchanged for nodes that *are* properly linked; they should simply yield empty or partial results for truly freestanding nodes.

### Documentation for AI agent use

- Update `README.md` to describe:
  - The new optional link flags on `create` commands and how to use them to create either free-standing or structured nodes.
  - The generic `dunno link` command, with examples showing common patterns (e.g. link an existing file to a submodule, link a task to a project and module, attach additional contexts).
  - Any invariants or recommended patterns for the AI agent (e.g. always prefer typed `create` commands with link IDs where possible, and fall back to `link` only for non-standard relationships).

## Tasks (mirroring .cursor plan todos)

- [ ] Review all existing DB entity create and link functions for projects, modules, submodules, files, tasks, subtasks, todos, and contexts to capture current relationship semantics.
- [ ] Introduce internal `*_record` helper functions per entity type to create bare records without relationships and refactor existing `create_*` methods to use them.
- [ ] Update DB create methods to accept optional parent IDs and conditionally establish relationships while preserving invariants.
- [ ] Adjust CLI `create` subcommands to make linking IDs optional/repeatable and pass them through to the updated DB APIs with any necessary validation.
- [ ] Add a new generic `dunno link` CLI command that connects existing nodes via named edges, backed by DB `relate`.
- [ ] Update and extend SurrealDB integration tests and CLI tests to cover both freestanding and linked creation flows plus the new link command.
- [ ] Document the new behaviors and recommended usage patterns for the AI agent in `README.md`.

## Notes / trade-offs

- **Backward compatibility**: Some CLI flags becoming optional is technically a behavior change, but existing usage that supplies them continues to work exactly as before. Tests will ensure we don't regress core flows.
- **Graph consistency**: Freestanding nodes are allowed by design; code that relies on traversing from projects/modules/tasks must handle the case that edges are missing. Existing helper methods already behave reasonably (returning empty results) in such scenarios.
- **Extensibility**: By centralizing bare-record creation logic in `*_record` helpers and using a generic `Link` CLI, it becomes easier to add new node types or relationship patterns later without further expanding the public surface area dramatically.

