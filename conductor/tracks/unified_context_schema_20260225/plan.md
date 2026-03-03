# Track Plan: Unified Context Schema (single `has_context` → `context` with type field)

Replace dedicated `mistake`, `style_rule`, and `security_detail` nodes and their edges (`has_mistake`, `has_style`, `has_security_detail`) with a single `context` table and a single `has_context` edge. Each context record has a string `type` field (e.g. `mistake`, `style_rule`, `security_detail`, `code_styleguide`, `skill`, or any other value) and the same structural relations as the current knowledge nodes.

## Phase 1: Schema (SurrealDB)

- [X] Task: Add a `context` table to the SurrealDB schema (replacing or alongside legacy tables for migration).
- [X] Task: Define a single relation `has_context` with `IN project|task|module|submodule|subtask OUT context`.
- [X] Task: Update `belongs_to_project`, `belongs_to_module`, `belongs_to_task` so that `IN` includes `context` (instead of `mistake|style_rule|security_detail`).
- [X] Task: Remove or deprecate relation tables `has_mistake`, `has_style`, `has_security_detail` and tables `mistake`, `style_rule`, `security_detail` once migration is done.
- [X] Task: Update `TABLES` in `schema.rs`: add `context`, remove `mistake`, `style_rule`, `security_detail` when switching over.

## Phase 2: Models & Rust types

- [X] Task: Add a unified `Context` struct (e.g. in `src/models.rs`) with:
  - `id: Option<String>`
  - `type_: String` (or `context_type`) — a free-form string (well-known values include `mistake`, `style_rule`, `security_detail`, `code_styleguide`, `skill`)
  - Shared/optional fields to cover current Mistake (content), StyleRule (description, example), SecurityDetail (content, severity, category, tags); e.g. `content`, `description`, `example`, `severity`, `category`, `tags` with `Option` where not used for that type.
- [X] Task: Add serialization/deserialization that respects the `type` field and optional fields; add unit tests.
- [X] Task: Keep or remove legacy `Mistake`, `StyleRule`, `SecurityDetail` structs depending on migration strategy (e.g. remove after migration, or keep as view types over `Context`).

## Phase 3: DB layer (`src/db/surreal/`)

- [X] Task: Implement `create_context(context: &Context)` that creates a `context` record with the `type` field set.
- [X] Task: Implement `get_context(id)`, `list_contexts()` (and optionally `list_contexts_by_type(type_)`).
- [X] Task: Change `link_context(from_id, to_id)` to use only the `has_context` edge (to_id is always `context:*`). Create reverse `belongs_to_project`, `belongs_to_module`, `belongs_to_task` from the context record to the structural hierarchy, same as today.
- [X] Task: Update `get_belongs_to_targets(knowledge_record_id)` to work for `context:*` (same logic: follow belongs_to_* edges).
- [X] Task: Remove or refactor `create_mistake`, `create_style_rule`, `create_security_detail`, `get_mistake`, etc., in favor of `create_context` / `get_context` (or thin wrappers that set `type` and call these).

## Phase 4: Context queries and flattening

- [X] Task: Update context retrieval in `src/db/surreal/entities/tasks.rs` (and files/tasks that fetch knowledge): use a single traversal `->has_context->context.*` instead of separate `->has_mistake->mistake.*`, `->has_style->style_rule.*`, `->has_security_detail->security_detail.*`.
- [X] Task: Update `flatten_context_result` in `src/db/surreal/flatten_context.rs` to recognize a single `has_context` shape and derive `node_type` from the context record’s `type` field (instead of separate edge keys).
- [X] Task: Update public context API (`get_task_context`, `get_file_context`, `get_subtask_context`) so returned context exposes a unified list of context items (with `node_type` or equivalent from `context.type`); keep backward compatibility for CLI/consumers (e.g. still return mistakes, style_rules, security_details as lists built from filtering by type).

## Phase 5: CLI and `dunno add`

- [X] Task: Update `dunno add --type mistake|style|security` to create a `context` record with the corresponding `type` and link it via `has_context` (and belongs_to_*). Preserve existing CLI flags and behavior where possible.
- [X] Task: Update any CLI that lists or displays mistakes/style_rules/security_details to read from the unified context API (filter by type if needed).

## Phase 6: Tests, docs, and migration

- [X] Task: Add unit and integration tests for: creating context by type, linking via `has_context`, belongs_to_* targets, and context retrieval (task/file/subtask) returning the correct unified context.
- [X] Task: Optional: add a one-off migration or seed script to copy existing `mistake`, `style_rule`, `security_detail` records into `context` with the right `type` and recreate `has_context` / belongs_to_* edges; then remove old tables and edges.
- [X] Task: Update README and conductor docs (and the dunno-knowledge-db skill) to describe the unified context model: single `has_context` → `context` with `type` field, same relations as before.

## Notes

- **Relations for context (same as current knowledge):**
  - `project|task|module|submodule|subtask -> has_context -> context`
  - `context -> belongs_to_project -> project`
  - `context -> belongs_to_module -> module`
  - `context -> belongs_to_task -> task`
- **Type semantics:** `type` is an open string field. Three well-known types (`mistake`, `style_rule`, `security_detail`) get first-class handling in the API/CLI, and any other value (e.g. `code_styleguide`, `skill`, or arbitrary labels) is still allowed and will be surfaced as-is from context queries.
- Branch suggestion: `feat/unified-context-schema` when starting implementation.
