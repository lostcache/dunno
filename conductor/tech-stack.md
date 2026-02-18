# Technology Stack

## Core Language
- **Rust:** Chosen for its performance, safety, and single-binary deployment capabilities.

## CLI Framework
- **clap (Command Line Argument Parser):** The de facto standard for building CLIs in Rust, ensuring robust and ergonomic argument parsing.

## Database & Persistence
- **Graph Database: SurrealDB (Primary):** The core knowledge engine. Knowledge is stored as typed nodes (`project`, `module`, `task`, `todo_item`, `mistake`, `style_rule`, `skill`, `task_update`) and explicit relation edges for traversal-first retrieval.
- **Vector Database: Qdrant (Optional/Future):** Kept for later semantic expansion, but not required for the graph-first MVP retrieval path.

## Retrieval Strategy
- **Primary Retrieval:** Graph traversal in SurrealDB seeded by `task_id` (and optionally project/module filters), expanding across explicit edges with bounded hop depth.
- **Inheritance Rules:** Retrieval includes task-local guidance plus inherited project-global guidance (`global_mistakes`, `global_style_rules`).
- **Output Contract:** CLI responses are JSON-structured and deterministic for agent consumption and automation.

## Graph Model (Agent-Centric)
- **Hierarchy:** `project -> contains -> module -> contains -> task`
- **Work Queue:** `project -> has_todo -> todo_item`, with optional `todo_item -> maps_to -> task`
- **Guidance Links:** `task -> must_avoid -> mistake`, `task -> should_follow -> style_rule`, `task -> requires_skill -> skill`
- **Global Baseline:** `project -> global_must_avoid -> mistake`, `project -> global_should_follow -> style_rule`
- **Runtime Learning:** Agents can append mistakes dynamically (code/logical), scoped to project/module/task, then link them into context graph edges.

## Update Semantics
- **Append-Only Task Updates:** Post-task learnings are persisted as `task_update` records and linked with `task -> has_note -> task_update`.
- **Dynamic Mistake Capture:** Agent-reported mistakes are append-only, tagged by source (`agent_runtime`/manual) and type (`code`/`logic`).

## Migration Note (2026-02-18)
- The MVP track pivots from vector-first retrieval to graph-first retrieval to prioritize explicit, auditable knowledge relationships and deterministic context assembly.

## Data Handling
- **Serialization: serde (serde_json):** The standard framework for serializing and deserializing Rust data structures efficiently, especially for JSON output.
- **Configuration: toml:** For parsing TOML configuration files.

## Development Tools
- **Cargo:** The standard build system and package manager for Rust.
