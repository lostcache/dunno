# Product Guide

## Target Users
- **AI Coding Assistants:** The primary consumer, designed to integrate with tools like Cursor, Windsurf, and Copilot.
- **Human Developers:** A secondary audience, using the tool for reference and onboarding to project standards.

## Data Storage & Architecture
- **Knowledge Base:** The system uses a strict graph hierarchy powered by SurrealDB.
- **Why:** Agents need deterministic, auditable retrieval for task execution, best supported by explicit hierarchy and relation edges. Natural language search is replaced by precise structural traversal.
- **Storage Modes:**
    - **Local (default):** Embedded SurrealDB persisted to disk. Zero-config, works offline, single-binary experience. Ideal for individual developers.
    - **Cloud:** Remote SurrealDB instance (e.g., SurrealDB Cloud). Enables cross-machine sync and team-shared knowledge bases.
- **Configuration:** All storage settings live in `~/.config/dunno/config.toml`. The CLI works out of the box with no config file (local mode with defaults). Backend switching is a single config toggle.

## Entity-Relationship Model

The knowledge graph database is built around core entities connected by native SurrealDB graph edges (via `RELATE`). No foreign key fields are stored on structs — all relationships are expressed as graph edges.

### Entities (Nodes)

| # | Entity | Table | Description |
|---|--------|-------|-------------|
| 1 | **Task** | `task` | A unit of work. Has name, description, status. |
| 2 | **Subtask** | `subtask` | A child of a Task. Has name, description, status. |
| 3 | **Module** | `module` | A functional area or component within a Project. |
| 4 | **Submodule** | `submodule` | An optional grouping within a Module. |
| 5 | **File** | `file` | A source file mapped by name and path. |
| 6 | **StyleRule** | `style_rule` | Coding style rules (description + example). |
| 7 | **Mistake** | `mistake` | Known pitfalls (content + category + tags). |
| 8 | **SecurityDetail** | `security_detail` | Security constraints (content + severity + category + tags). |

Supporting entities: `Project` (top-level container), `TaskUpdate` (append-only task log), `TodoItem` (work queue).

### Relations (Graph Edges via RELATE)

All relationships are expressed as native SurrealDB graph edges, not FK fields.

**Structural hierarchy** (via `contains` edge table):
- `project -> contains -> module`
- `module -> contains -> submodule` (optional)
- `module -> contains -> file` (or `submodule -> contains -> file`)
- `module -> contains -> task`
- `task -> contains -> subtask` (optional)

**Knowledge links** (via `has_context` edge table):
- Any structural node can link to any knowledge node:
  - `node -> has_context -> mistake`
  - `node -> has_context -> style_rule`
  - `node -> has_context -> security_detail`

**Work queue** (via `has_todo` edge table):
- `project -> has_todo -> todo_item`

**Task log** (via `has_update` edge table):
- `task -> has_update -> task_update`

### Structural Hierarchies

Two parallel traversal paths through the graph:

**Code Structure Path** (for file-level context):
```
Project -> contains -> Module -> contains -> Submodule (optional) -> contains -> File
```

**Work Tracking Path** (for task-level context):
```
Project -> contains -> Module -> contains -> Task -> contains -> Subtask (optional)
```

### Context Inheritance

Context retrieval traverses **upward** through `<-contains<-` edges, collecting all `->has_context->` knowledge nodes at each ancestor level. Each context path resolves in a **single SurrealQL query**.

- **Task context:** Task `<-contains<-` Module `<-contains<-` Project
- **File context:** File `<-contains<-` Submodule (if any) `<-contains<-` Module `<-contains<-` Project
- **Subtask context:** Subtask `<-contains<-` Task `<-contains<-` Module `<-contains<-` Project

## Core Functionality
- **Retrieval Interface:** The interaction model is ID-based graph traversal via the CLI.
    - Task context: `dunno context --task-id <id>`
    - File context: `dunno context --file-id <id>`
    - Subtask context: `dunno context --subtask-id <id>`
    - Output: JSON containing knowledge nodes (mistakes, style rules, security details) aggregated from the queried node and all its ancestors.
- **Knowledge Management:**
    - **Hierarchy Management:** CLI supports creating projects, modules, submodules, files, tasks, and subtasks. Parent relationships are created as `RELATE` graph edges.
    - **Todo Queue:** CLI supports project-level todo management. Agents claim todo items which map to specific tasks.
    - **Context Linking:** Mistakes, style rules, and security details are linked to any structural node via `RELATE node -> has_context -> knowledge_node`.
    - **Runtime Learning:** Agents can append new mistakes, rules, or security notes to the current task/module for future reference.

## Compatibility
- **Initial Targets:**
    - Cursor (via context features)
    - VS Code Copilot Chat
    - Standalone LLM CLIs
