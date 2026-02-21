# Track Specification: Graph-Native Schema Redesign

## Overview

Rewrite the entire data layer so that SurrealDB is used as a **graph database**, not a document store with hand-rolled edges. The current schema stores parent references as string FK fields on structs (`module.project_id`, `task.module_id`, etc.), uses a hand-rolled `knowledge_edge` table for graph links, and requires 6-11 sequential queries for a single context retrieval. SurrealDB v3 has first-class `RELATE` edges and `->` / `<-` arrow traversal that eliminates all of this.

This track also adds the two missing ER entities (Subtask, SecurityDetail) and removes deprecated infrastructure (CategoryTag, KnowledgeEdge).

## Problems With Current Schema

### 1. Hand-Rolled Edge Table
The `KnowledgeEdge` struct stores `from_id`, `to_id`, `relation` as plain strings in a `knowledge_edge` table. SurrealDB's native `RELATE` statement creates typed edge records with `in` and `out` fields and enables `->` / `<-` traversal operators. The current approach prevents using any of these features.

### 2. Relational FK Fields in a Graph Database
Every structural model embeds its parent as a string field (`Module.project_id`, `Task.module_id`, `File.module_id + submodule_id`). In a graph database, these relationships should be expressed as graph edges so the hierarchy is traversable in both directions.

### 3. N+1 Query Explosion
`get_task_context` fires at minimum 6 sequential queries (fetch task, fetch module, fetch project, then for each level: fetch edges + fetch each knowledge node). With native graph traversal, this collapses to 1 SurrealQL query.

### 4. String-Prefix Type Dispatch
`fetch_knowledge_node_json` uses `id.starts_with("mistake:")` to decide which table to query. Every new knowledge type requires a new `if` branch. Native graph traversal (`->has_context->(mistake, style_rule, security_detail)`) eliminates this entirely.

### 5. Full-Table Scan for Deduplication
`create_edge` calls `list_edges()` (loads ALL edges) to check for duplicates before inserting. This is O(n) in total edge count.

### 6. Redundant Categorization
`Mistake.category` field, `CategoryTag` table, and `KnowledgeEdge` linking mistakes to tags — triple encoding of the same concept.

## Design Decisions

### D1: All relationships become RELATE edges
No FK fields on structs. Structural hierarchy and knowledge links are all expressed as graph edges.

- Structural: `RELATE project:abc -> contains -> module:def`
- Knowledge: `RELATE task:ghi -> has_context -> mistake:m1`
- Work queue: `RELATE project:abc -> has_todo -> todo_item:t1`

### D2: Keep separate tables per knowledge type
Separate `mistake`, `style_rule`, `security_detail` tables. SurrealQL's arrow syntax naturally handles multi-table targets: `->has_context->(mistake, style_rule, security_detail)`.

### D3: Remove KnowledgeEdge and CategoryTag
The hand-rolled `knowledge_edge` table is fully replaced by native `has_context` and `contains` edge tables. `CategoryTag` is removed; tags remain as `Vec<String>` on knowledge nodes.

### D4: Deprecate Skill
Not in the target ER. Remove from active code paths.

### D5: Atomic create-and-relate
When creating a module under a project, a single multi-statement SurrealQL query creates the node AND the RELATE edge together.

### D6: Single-query context retrieval
Each context path (task, file, subtask) is resolved by a single SurrealQL query that walks the `<-contains<-` chain upward and collects `->has_context->` at each level.

## Target Entities

| # | Entity | Table | Description |
|---|--------|-------|-------------|
| 1 | Project | `project` | Top-level container |
| 2 | Module | `module` | Functional area within a project |
| 3 | Submodule | `submodule` | Optional grouping within a module |
| 4 | File | `file` | Source file mapped by path |
| 5 | Task | `task` | Unit of work within a module |
| 6 | Subtask | `subtask` | **NEW** — child of a task |
| 7 | Mistake | `mistake` | Known pitfall (content, category, tags) |
| 8 | StyleRule | `style_rule` | Coding style rule (description, example) |
| 9 | SecurityDetail | `security_detail` | **NEW** — security constraint (content, severity, category, tags) |
| 10 | TaskUpdate | `task_update` | Append-only task log entry |
| 11 | TodoItem | `todo_item` | Project-level work queue item |

## Target Edge Tables (created via RELATE)

| Edge Table | Meaning | Example |
|------------|---------|---------|
| `contains` | Structural hierarchy | `project:abc -> contains -> module:def` |
| `has_context` | Knowledge link | `task:ghi -> has_context -> mistake:m1` |
| `has_todo` | Work queue link | `project:abc -> has_todo -> todo_item:t1` |
| `has_update` | Task log link | `task:ghi -> has_update -> task_update:u1` |

## Target Graph Traversal Queries

### Task context (replaces 11 queries):
```sql
LET $t = type::thing('task', $tid);
LET $m = (SELECT VALUE in FROM contains WHERE out = $t AND in.tb() = 'module' LIMIT 1);
LET $p = (SELECT VALUE in FROM contains WHERE out = $m AND in.tb() = 'project' LIMIT 1);

RETURN array::group([
    (SELECT VALUE out FROM has_context WHERE in = $t),
    (SELECT VALUE out FROM has_context WHERE in = $m),
    (SELECT VALUE out FROM has_context WHERE in = $p)
]);
```

### File context (handles optional submodule):
```sql
LET $f = type::thing('file', $fid);
LET $sub = (SELECT VALUE in FROM contains WHERE out = $f AND in.tb() = 'submodule' LIMIT 1);
LET $mod = IF $sub IS NOT NONE THEN
    (SELECT VALUE in FROM contains WHERE out = $sub AND in.tb() = 'module' LIMIT 1)
ELSE
    (SELECT VALUE in FROM contains WHERE out = $f AND in.tb() = 'module' LIMIT 1)
END;
LET $proj = (SELECT VALUE in FROM contains WHERE out = $mod AND in.tb() = 'project' LIMIT 1);

RETURN array::group([
    (SELECT VALUE out FROM has_context WHERE in = $f),
    IF $sub IS NOT NONE THEN (SELECT VALUE out FROM has_context WHERE in = $sub) ELSE [] END,
    (SELECT VALUE out FROM has_context WHERE in = $mod),
    (SELECT VALUE out FROM has_context WHERE in = $proj)
]);
```

## Constraints
- No FK fields on any Rust struct. All parent/child relationships expressed as RELATE edges.
- Context retrieval must be a single SurrealQL query per path.
- Pre-MVP: clean-slate rewrite, no data migration needed.
- All existing shell tests will need CLI signature updates (parent IDs become named flags).
