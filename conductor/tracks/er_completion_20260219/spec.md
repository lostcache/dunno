# Track Specification: ER Model Completion (Subtask & SecurityDetail)

## Overview
Complete the agent knowledge graph ER model by adding the two missing entities: **Subtask** and **SecurityDetail**. These are required to match the target ER schema defined in `product.md`.

## Background
The target ER defines 8 entities:

| # | Entity | Status |
|---|--------|--------|
| 1 | Task | ✅ Implemented |
| 2 | Subtask | ❌ Missing |
| 3 | Module | ✅ Implemented |
| 4 | Submodule | ✅ Implemented |
| 5 | File (path) | ✅ Implemented |
| 6 | code style_guide | ✅ Implemented (`StyleRule`) |
| 7 | mistakes | ✅ Implemented (`Mistake`) |
| 8 | security details | ❌ Missing |

## Objectives

### 1. Subtask Entity
Add a `Subtask` struct that represents a child unit of work under a parent `Task`.

**Fields:**
- `id: Option<String>` — SurrealDB record ID
- `task_id: String` — FK to parent Task
- `name: String`
- `description: String`
- `status: TaskStatus` — reuses the existing `not_started | started | finished` enum

**Relations:**
- `Task` has many `Subtask` (one-to-many via `task_id`)
- A Subtask inherits context from its parent Task (Task -> Module -> Project)

### 2. SecurityDetail Entity
Add a `SecurityDetail` struct that represents security constraints, policies, or audit notes.

**Fields:**
- `id: Option<String>` — SurrealDB record ID
- `content: String` — the security note/constraint
- `severity: String` — e.g., "critical", "high", "medium", "low"
- `category: String` — e.g., "auth", "data", "network", "crypto"
- `tags: Vec<String>`

**Relations:**
- Linkable to any structural node (Project, Module, Submodule, File, Task) via `KnowledgeEdge`
- Context retrieval should include security details when traversing the graph

## Constraints
- Subtask must belong to exactly one Task.
- SecurityDetail follows the same linking pattern as Mistake and StyleRule (via `KnowledgeEdge`).
- Context retrieval (`get_task_context`, `get_file_context`) must include SecurityDetail nodes.
- The `lazydev add` command must accept `--type security` to create SecurityDetail records.
- Subtask context retrieval should traverse: Subtask -> Task -> Module -> Project.
