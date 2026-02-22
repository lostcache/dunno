# dunno

`dunno` is a Rust CLI that captures coding knowledge and retrieves deterministic context for AI agents.

Unlike traditional natural language search, `dunno` uses a strict graph hierarchy. Context (mistakes, style rules, and security details) is linked to nodes in this hierarchy and is inherited down the tree.

## Core Hierarchy

The knowledge graph has two parallel structural paths:

**Code Structure Path:**
- **Project** -> **Module** -> **Submodule** (optional) -> **File** (path)

**Work Tracking Path:**
- **Project** -> **Module** -> **Task** -> **Subtask** (optional)

### Knowledge Entities (linkable to any structural node)
- **Mistake:** Known pitfalls and errors to avoid.
- **StyleRule:** Coding style rules with examples.
- **SecurityDetail:** Security constraints and audit notes.

### Supporting Entities
- **Todo:** A project-level work queue item.

## Retrieval Strategy

Retrieval is purely deterministic and graph-based:
1. Provide a `task_id`, `file_id`, or `subtask_id`.
2. **Task path:** Traverses Task -> Module -> Project, collecting all linked knowledge at each level.
3. **File path:** Traverses File -> Submodule (if any) -> Module -> Project, collecting all linked knowledge at each level.
4. All unique knowledge nodes (Mistakes, Style Rules, Security Details) are deduplicated and returned as JSON.

## Prerequisites

- Rust toolchain (stable) with `cargo`.
- Optional: SurrealDB Cloud credentials if using the cloud backend.

## Build

```bash
cargo build --release
```

Binary path: `target/release/dunno`

## Configuration

The CLI resolves configuration from (highest to lowest precedence):
1. CLI flags (`--backend`)
2. Environment variables
3. `~/.config/dunno/config.toml`
4. Built-in defaults

By default, no config file is required. The app uses local embedded storage at `~/.local/share/dunno/data.db`.

### Example config file

```toml
backend = "local" # "local" | "cloud"

[local]
path = "~/.local/share/dunno/data.db"

[cloud]
url = "wss://YOUR_INSTANCE.surrealdb.com"
namespace = ""
database = ""
username = "root"
password = "root"
auth_type = "root" # "root" | "namespace" | "database"
```

### Environment overrides

- `DUNNO_BACKEND`
- `DUNNO_LOCAL_PATH`
- `DUNNO_CLOUD_URL`
- `DUNNO_CLOUD_NS`
- `DUNNO_CLOUD_DB`
- `DUNNO_CLOUD_USER`
- `DUNNO_CLOUD_PASS`
- `DUNNO_CLOUD_AUTH_TYPE`

## Quick Start

### 1. Initialize your hierarchy

```bash
# Create a project
dunno project create "My App" "A web application"
# Returns: {"id":"project:abc","name":"My App","description":"A web application"}

# Create a module within the project
dunno module create --project-id project:abc "Auth" "Authentication system"
# Returns: {"id":"module:def", ...}

# (Optional) Create a submodule within the module
dunno submodule create --module-id module:def "OAuth" "OAuth2 providers"

# Register a file under the module (or submodule)
dunno file create --parent-id module:def "oauth.rs" "src/auth/oauth.rs"

# Create a task within the module
dunno task create --module-id module:def --project-id project:abc "Implement JWT" "Add token support"
# Returns: {"id":"task:ghi", ...}

# (Optional) Create a subtask within the task
dunno subtask create --task-id task:ghi "Write tests" "Add unit tests"
```

### 2. Track task progress

```bash
# Update a task's status
dunno task update task:ghi --status started

# Edit a task in-place
dunno task update task:ghi --description "Add JWT token support with refresh"
```

### 3. Link knowledge

```bash
# Add a style rule to the project
dunno add --type style --content "Use explicit error types" --link-to project:abc

# Add a mistake to a task
dunno add --type mistake --content "Do not log raw passwords" --link-to task:ghi

# Add a security note to a module
dunno add --type security --content "Validate all user inputs" --link-to module:def
```

### 4. Retrieve context

```bash
# By task (traverses Task -> Module -> Project)
dunno context --task-id task:ghi

# By file (traverses File -> Submodule -> Module -> Project)
dunno context --file-id file:456

# By subtask
dunno context --subtask-id subtask:stu
```

Returns a JSON object containing all linked knowledge (style rules, mistakes, security details) aggregated from the node and its ancestors.

## CLI Reference

### Knowledge Management
- `dunno add --type <mistake|style|security> --content <CONTENT> [--link-to <ID>]`
- `dunno context --task-id <ID>`
- `dunno context --file-id <ID>`
- `dunno context --subtask-id <ID>`

### Hierarchy Management
- `dunno project create <NAME> <DESC>` / `list`
- `dunno module create --project-id <ID> <NAME> <DESC>` / `list`
- `dunno submodule create --module-id <ID> <NAME> <DESC>` / `list [--module-id <ID>]`
- `dunno file create --parent-id <ID> <NAME> <PATH>` / `list [--module-id <ID>] [--submodule-id <ID>]`
- `dunno task create --module-id <ID> --project-id <ID> <NAME> <DESC>` / `list`
- `dunno task update <TASK_ID> [--name <NAME>] [--description <DESC>] [--status <not_started|started|finished>]`
- `dunno subtask create --task-id <ID> <NAME> <DESC>` / `list --task-id <ID>`

### Work Queue
- `dunno todo create --project-id <ID> <CONTENT>` / `list --project-id <ID>`

### Config
- `dunno config show`
- `dunno --backend <local|cloud> ...`

## Output Contract

All commands return structured JSON. Successful operations return the created/updated object or a list. Errors return:

```json
{"status":"error","kind":"runtime_error","error":"Task not found: task:123"}
```

Task statuses are strictly one of: `not_started`, `started`, `finished`.

## Graph Schema

The database uses SurrealDB with explicit graph relation types for visualization in Surrealist:

| Edge | From | To |
|------|------|----|
| `contains` | project, module, submodule, task | module, submodule, file, task, subtask |
| `has_task` | project | task |
| `belongs_to_project` | task | project |
| `belongs_to_module` | task | module |
| `has_context` | task, module | mistake, style_rule, security_detail |
| `has_todo` | project | todo_item |

## Development

### Tests

```bash
cargo test
```

Tests run against in-memory backends (`mem://`) and do not require a separate SurrealDB server.

### Shell Tests

```bash
# Run all local suites
./tests/sh/run_all.sh

# Run all cloud suites
./tests/sh/run_cloud.sh
```
