# Technology Stack

## Core Language
- **Rust:** Chosen for its performance, safety, and single-binary deployment capabilities.

## CLI Framework
- **clap (Command Line Argument Parser):** The de facto standard for building CLIs in Rust, ensuring robust and ergonomic argument parsing.

## Database & Persistence
- **Graph Database: SurrealDB (Primary):** The sole knowledge engine for the MVP. Knowledge is strictly hierarchical: `Project -> Module -> Task`. Context nodes (`mistake`, `style_rule`, `skill`) are linked to these structural nodes.
- **Vector Database:** Removed for MVP. Retrieval is purely deterministic based on graph structure.

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
- **Configuration: toml:** For parsing TOML configuration files.

## Development Tools
- **Cargo:** The standard build system and package manager for Rust.
