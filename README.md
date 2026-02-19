# lazydev

`lazydev` is a Rust CLI that captures coding knowledge and retrieves deterministic context for AI agents.

Unlike traditional natural language search, `lazydev` uses a strict **Project -> Module -> Task** hierarchy. Context (mistakes, style rules, and skills) is linked to nodes in this hierarchy and is inherited down the tree.

## Core Hierarchy

- **Project:** The top-level container for all code and knowledge.
- **Module:** A functional area or component within a project.
- **Task:** A specific unit of work or feature.
- **Todo:** A project-level work queue item that can map to a task.

## Retrieval Strategy

Retrieval is purely deterministic and graph-based:
1. Provide a `task_id`.
2. The system traverses up to the parent **Module** and **Project**.
3. All unique knowledge nodes (Mistakes, Style Rules, Skills) linked to the task, its module, or the project are aggregated and returned as JSON.

## Prerequisites

- Rust toolchain (stable) with `cargo`.
- A running SurrealDB endpoint at `ws://localhost:8000` (started with `file:data.db` for durability or `memory` for ephemeral).

## Build

```bash
cargo build --release
```

Binary path: `target/release/lazydev`

## Quick Start

### 1. Initialize your hierarchy

```bash
# Create a project
lazydev project create "My App" "A web application"
# Returns project:abc

# Create a module within the project
lazydev module create project:abc "Auth" "Authentication system"
# Returns module:def

# Create a task within the module
lazydev task create module:def "Implement JWT" "Add token support"
# Returns task:ghi
```

### 2. Link knowledge

```bash
# Add a global style rule to the project
lazydev add --category rust --type style --content "Use explicit error types" --link-to project:abc

# Add a task-specific mistake to avoid
lazydev add --category security --type mistake --content "Do not log raw passwords" --link-to task:ghi
```

### 3. Retrieve context for the task

```bash
lazydev context --task-id task:ghi
```

**Expected output:**
A JSON object containing both the global style rule (inherited) and the task-specific mistake.

## CLI Commands

### Knowledge Management
- `lazydev add`: Add a mistake, style rule, or skill. Optional `--link-to <ID>` for context mapping.
- `lazydev context --task-id <ID>`: Retrieve aggregated context for a task.

### Hierarchy Management
- `lazydev project create <NAME> <DESC>` / `list`
- `lazydev module create <PROJECT_ID> <NAME> <DESC>` / `list`
- `lazydev task create <MODULE_ID> <NAME> <DESC>` / `list`
- `lazydev task update <TASK_ID> [--name <NAME>] [--description <DESC>] [--status <not_started|started|finished>]`
- `lazydev task append-update <TASK_ID> <CONTENT>`
- `lazydev task update-entry <UPDATE_ID> <CONTENT>`
- `lazydev task list-updates <TASK_ID>`

### Work Queue
- `lazydev todo create <PROJECT_ID> <CONTENT>`
- `lazydev todo list <PROJECT_ID>`

## Output Contract

All commands return structured JSON for easy consumption by agents:

```json
{"status":"ok"}
```

```json
{"results": [...]}
```

Task statuses are strictly one of:

```json
["not_started", "started", "finished"]
```

Errors are also returned as JSON:

```json
{"status":"error","kind":"runtime_error","error":"Task not found: task:123"}
```

## Development

Run integration tests (requires SurrealDB at `ws://localhost:8000`):

```bash
cargo test
```
