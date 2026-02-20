# Technology Stack

## Core Language
- **Rust:** Chosen for its performance, safety, and single-binary deployment capabilities.

## CLI Framework
- **clap (Command Line Argument Parser):** The de facto standard for building CLIs in Rust, ensuring robust and ergonomic argument parsing.

## Database & Persistence
- **Graph Database: SurrealDB** — the sole knowledge engine. Knowledge is strictly hierarchical: `Project -> Module -> Task`. Context nodes (`mistake`, `style_rule`, `skill`) are linked to these structural nodes.
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
- **Primary Retrieval:** Deterministic graph traversal starting from a `task_id`. The system traverses up to the parent `Module` and `Project` to aggregate all relevant context.
- **Inheritance Rules:** Context is additive. A task inherits all constraints and guides from its containing module and project.
- **Output Contract:** CLI responses are JSON-structured and deterministic.

## Graph Model (Agent-Centric)
- **Hierarchy:** `project -> contains -> module -> contains -> task`
- **Work Queue:** `project -> has_todo -> todo_item`, with `todo_item -> maps_to -> task`
- **Context Links:** 
    - `task -> has_context -> mistake/style_rule/skill`
    - `module -> has_context -> mistake/style_rule/skill`
    - `project -> has_context -> mistake/style_rule/skill`
- **Runtime Learning:** Agents append new context nodes to the current task/module.

## Update Semantics
- **Append-Only Task Updates:** Post-task learnings are persisted as `task_update` records and linked with `task -> has_note -> task_update`.
- **Dynamic Mistake Capture:** Agent-reported mistakes are append-only, tagged by source (`agent_runtime`/manual) and type (`code`/`logic`).

## Migration Note (2026-02-18)
- The MVP track pivots from vector-first retrieval to graph-first retrieval to prioritize explicit, auditable knowledge relationships and deterministic context assembly.

## Data Handling
- **Serialization: serde (serde_json):** The standard framework for serializing and deserializing Rust data structures efficiently, especially for JSON output.
- **Configuration: toml:** For parsing TOML configuration files from `~/.config/dunno/config.toml`.
- **Path Resolution: dirs:** For resolving platform-appropriate config/data directories (`~/.config/dunno`, `~/.local/share/dunno`).

## Development Tools
- **Cargo:** The standard build system and package manager for Rust.
