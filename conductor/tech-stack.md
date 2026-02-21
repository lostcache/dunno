# Technology Stack

## Core Language
- **Rust:** Chosen for its performance, safety, and single-binary deployment capabilities.

## CLI Framework
- **clap (Command Line Argument Parser):** The de facto standard for building CLIs in Rust, ensuring robust and ergonomic argument parsing.

## Database & Persistence
- **Graph Database: SurrealDB v3** — the sole knowledge engine, used as a native graph database. All relationships are expressed as graph edges via SurrealDB's `RELATE` statement. No FK fields are stored on records.
    - **Structural hierarchy:** `project -> contains -> module -> contains -> [task | submodule -> contains -> file]`
    - **Knowledge links:** `node -> has_context -> [mistake | style_rule | security_detail]`
    - **Traversal:** Context retrieval uses SurrealQL's `<-contains<-` and `->has_context->` arrow syntax for single-query resolution.
- **Vector Database:** Removed for MVP. Retrieval is purely deterministic based on graph structure.

### Storage Backends
The CLI supports two storage backends, selected via configuration:

1. **Local (default):** Embedded SurrealDB using the `surrealkv://` engine. Data persists to a local directory (default: `~/.local/share/dunno/data.db`). Zero external dependencies — the binary is fully self-contained.
2. **Cloud:** Remote SurrealDB instance (e.g., SurrealDB Cloud). Connects over `wss://` with namespace, database, and credential fields from config. Supports configurable authentication types (`root`, `namespace`, `database`) via `auth_type`. Enables cross-machine sync and team sharing.

TLS for `wss://` connections is handled by `rustls` with the `aws-lc-rs` crypto provider, installed at process startup.

Backend selection is a single config toggle; the application code uses a unified SurrealDB client regardless of backend.

## Configuration
- **Location:** `~/.config/dunno/config.toml`
- **Format:** TOML (parsed via the `toml` crate).
- **Precedence:** CLI flags > environment variables > config file > built-in defaults.
- **Defaults:** If no config file exists, the CLI operates with local storage and sensible defaults (no setup required for first run).

### Reference Config
```toml
# ~/.config/dunno/config.toml

# "local" | "cloud"
backend = "local"

[local]
# Path to the embedded SurrealDB data directory.
path = "~/.local/share/dunno/data.db"

[cloud]
# SurrealDB Cloud (or any remote SurrealDB) endpoint.
url = "wss://YOUR_INSTANCE.surrealdb.com"
namespace = ""
database = ""
# Credentials — prefer env vars (DUNNO_CLOUD_USER / DUNNO_CLOUD_PASS) over plaintext here.
username = "root"
password = "root"
# "root" | "namespace" | "database" — determines the SurrealDB signin scope.
auth_type = "root"
```

## Retrieval Strategy
- **Primary Retrieval:** Deterministic graph traversal starting from a `task_id`, `file_id`, or `subtask_id`. Each path resolves in a **single SurrealQL query** that walks `<-contains<-` upward and collects `->has_context->` at each level.
    - **Task path:** Task `<-contains<-` Module `<-contains<-` Project
    - **File path:** File `<-contains<-` Submodule (optional) `<-contains<-` Module `<-contains<-` Project
    - **Subtask path:** Subtask `<-contains<-` Task `<-contains<-` Module `<-contains<-` Project
- **Inheritance Rules:** Context is additive. A node inherits all knowledge from every ancestor in its hierarchy chain.
- **Output Contract:** CLI responses are JSON-structured and deterministic.

## Graph Model (Agent-Centric)

All relationships use SurrealDB's native `RELATE` statement, creating typed edge records in dedicated edge tables. No FK fields exist on node records.

### Edge Tables

| Edge Table | Purpose | Example |
|------------|---------|---------|
| `contains` | Structural hierarchy | `RELATE project:abc -> contains -> module:def` |
| `has_context` | Knowledge links | `RELATE task:ghi -> has_context -> mistake:m1` |
| `has_todo` | Work queue | `RELATE project:abc -> has_todo -> todo_item:t1` |
| `has_update` | Task log | `RELATE task:ghi -> has_update -> task_update:u1` |

### Structural Hierarchies (via `contains`)
- **Code Structure:** `project -> contains -> module -> contains -> submodule (optional) -> contains -> file`
- **Work Tracking:** `project -> contains -> module -> contains -> task -> contains -> subtask (optional)`
- **Work Queue:** `project -> has_todo -> todo_item`

### Knowledge Context Links (via `has_context`)
Any structural node (project, module, submodule, file, task, subtask) can link to:
- `node -> has_context -> mistake`
- `node -> has_context -> style_rule`
- `node -> has_context -> security_detail`

### Node Tables

| Entity | Table | Description |
|--------|-------|-------------|
| Project | `project` | Top-level container |
| Module | `module` | Functional area within a project |
| Submodule | `submodule` | Optional grouping within a module |
| File | `file` | Source file (name + path) |
| Task | `task` | Unit of work (name, description, status) |
| Subtask | `subtask` | Child of task (name, description, status) |
| Mistake | `mistake` | Known pitfall (content, category, tags) |
| StyleRule | `style_rule` | Style rule (description, example) |
| SecurityDetail | `security_detail` | Security constraint (content, severity, category, tags) |
| TaskUpdate | `task_update` | Append-only task log entry |
| TodoItem | `todo_item` | Project work queue item |

### Removed Entities
- **`KnowledgeEdge`** — replaced by native `has_context` edge table via `RELATE`
- **`CategoryTag`** — removed; tags are `Vec<String>` on knowledge nodes
- **`Skill`** — deprecated; not in target ER

- **Runtime Learning:** Agents append new context nodes to the current task/module via `RELATE`.

## Update Semantics
- **Append-Only Task Updates:** Post-task learnings are persisted as `task_update` records and linked with `task -> has_note -> task_update`.
- **Dynamic Mistake Capture:** Agent-reported mistakes are append-only, tagged by source (`agent_runtime`/manual) and type (`code`/`logic`).

## Migration Notes

### 2026-02-19: Graph-Native Schema Redesign
- The data layer is being rewritten to use SurrealDB as a native graph database. FK fields (`project_id`, `module_id`, etc.) are removed from all structs. All relationships are expressed as `RELATE` graph edges. Context retrieval collapses from N+1 sequential queries to a single SurrealQL query per path. The `KnowledgeEdge` table, `CategoryTag` table, and `Skill` entity are removed. `Subtask` and `SecurityDetail` entities are added.
- Track: `conductor/tracks/graph_native_schema_20260219/`

### 2026-02-18: Vector to Graph Pivot
- The MVP track pivots from vector-first retrieval to graph-first retrieval to prioritize explicit, auditable knowledge relationships and deterministic context assembly.

## Data Handling
- **Serialization: serde (serde_json):** The standard framework for serializing and deserializing Rust data structures efficiently, especially for JSON output.
- **Configuration: toml:** For parsing TOML configuration files from `~/.config/dunno/config.toml`.
- **Path Resolution: dirs:** For resolving platform-appropriate config/data directories (`~/.config/dunno`, `~/.local/share/dunno`).

## Development Tools
- **Cargo:** The standard build system and package manager for Rust.
