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

Context can be linked to **any** structural node: project, module, submodule, task, or subtask. Retrieval returns only the context directly linked to the requested node.

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

# Create a module within the project (single project; flag is repeatable for multiple)
dunno module create --project-ids project:abc "Auth" "Authentication system"
# Returns: {"id":"module:def", ...}

# (Optional) Create a submodule within the module
dunno submodule create --module-ids module:def "OAuth" "OAuth2 providers"

# Register a file under the module (or submodule)
dunno file create --parent-ids module:def "oauth.rs" "src/auth/oauth.rs"

# Create a task within the module
dunno task create --module-ids module:def --project-ids project:abc "Implement JWT" "Add token support"
# Returns: {"id":"task:ghi", ...}

# (Optional) Create a subtask within the task
dunno subtask create --task-ids task:ghi "Write tests" "Add unit tests"
```

### Epics (Optional)

Epics provide a higher-level grouping above user stories for agile workflow management:

```bash
# Create an epic linked to a project
dunno epic create --project-id project:abc "Authentication Epic" "Complete auth system"
# Returns: {"id":"epic:mno","title":"Authentication Epic","description":"Complete auth system"}

# List epics for a project
dunno epic list --project-id project:abc

# Create a user story linked to an epic
dunno user-story create --project-id project:abc --epic-ids epic:mno "As a user, I want login" "Authentication feature"

# Create a task linked to an epic
dunno task create --module-ids module:def --project-ids project:abc --epic-ids epic:mno "Implement login" "Add JWT auth"

# Link existing user story to epic
dunno link --from-id epic:mno --edge has_user_story --to-ids user_story:jkl
dunno link --from-id user_story:jkl --edge belongs_to_epic --to-ids epic:mno

# Link existing task to epic
dunno link --from-id epic:mno --edge has_task --to-ids task:ghi
dunno link --from-id task:ghi --edge belongs_to_epic --to-ids epic:mno
```

### User Stories (Optional)

User stories provide an additional layer between projects and tasks for agile workflow management:

```bash
# Create a user story linked to a project
dunno user-story create --project-id project:abc "As a user, I want login" "Authentication feature"
# Returns: {"id":"user_story:jkl", ...}

# List user stories for a project
dunno user-story list --project-id project:abc

# Create a task linked to a user story
dunno task create --module-ids module:def --project-ids project:abc --user-story-ids user_story:jkl "Implement login" "Add JWT auth"

# Link existing task to user story (using generic link command)
dunno link --from-id user_story:jkl --edge has_task --to-ids task:ghi
dunno link --from-id task:ghi --edge belongs_to_story --to-ids user_story:jkl
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

# Link to submodule (e.g. submodule:xyz from submodule create)
dunno add --type style --content "Submodule convention" --link-to submodule:xyz

# Link to subtask (e.g. subtask:stu from subtask create)
dunno add --type mistake --content "Subtask pitfall" --link-to subtask:stu
```

### 4. Retrieve context

```bash
# By task (direct-only)
dunno context --task-id task:ghi

# By file (direct-only)
dunno context --file-id file:456

# By subtask
dunno context --subtask-id subtask:stu

# By epic
dunno context --epic-id epic:mno
```

Returns a JSON array of all linked context items directly linked to the requested node.

#### Context at every level

You can link context at project, module, submodule, task, or subtask. Context retrieval returns only what is directly linked to the requested node.

## CLI Reference

### Knowledge Management
- `dunno add --type <mistake|style|security> --content <CONTENT> [--link-to <ID> ...]` — `--link-to` is **repeatable**; each `<ID>` can be `project:...`, `module:...`, `submodule:...`, `task:...`, or `subtask:...`.
- `dunno context --task-id <ID>` — context for a task (direct-only).
- `dunno context --file-id <ID>` — context for a file (direct-only).
- `dunno context --subtask-id <ID>` — context for a subtask (direct-only).

### Hierarchy Management
- `dunno project create <NAME> <DESC>` / `list`
- `dunno module create --project-ids <ID> [--project-ids <ID> ...] <NAME> <DESC>` / `list`
- `dunno submodule create --module-ids <ID> [--module-ids <ID> ...] <NAME> <DESC>` / `list [--module-id <ID>]`
- `dunno file create --parent-ids <ID> [--parent-ids <ID> ...] <NAME> <PATH>` / `list [--module-id <ID>] [--submodule-id <ID>]`
- `dunno task create [--module-ids <ID> --project-ids <ID>] [--user-story-ids <ID> ...] <NAME> <DESC>` / `list`
- `dunno task update <TASK_ID> [--name <NAME>] [--description <DESC>] [--status <not_started|started|finished>]`
- `dunno subtask create --task-ids <ID> [--task-ids <ID> ...] <NAME> <DESC>` / `list --task-id <ID>`

### User Stories
- `dunno user-story create --project-id <ID> [--epic-ids <ID> ...] <TITLE> <DESC>` — create linked to project, optionally to epics.
- `dunno user-story list [--project-id <ID>] [--epic-id <ID>]` — list all or filter by project or epic.

### Epics
- `dunno epic create --project-id <ID> <TITLE> <DESC>` — create linked to project.
- `dunno epic list [--project-id <ID>]` — list all or filter by project.

### Work Queue
- `dunno todo create --project-ids <ID> [--project-ids <ID> ...] <CONTENT>` / `list --project-id <ID>`

### Generic Linking
- `dunno link --from <ID> --edge <EDGE> --to <ID> [--to <ID> ...]`
  - `--from` / `--to` are record IDs like `project:abc`, `module:def`, `task:ghi`.
  - `--edge` must be one of: `contains`, `has_task`, `has_subtask`, `has_todo`, `has_context`, `has_user_story`, `has_module`, `has_submodule`, `has_epic`, `belongs_to_project`, `belongs_to_module`, `belongs_to_task`, `belongs_to_story`, `belongs_to_user_story`, `belongs_to_epic`.

### Recommended Patterns (AI Agent)

- **Primitive operations**:
  - **Create**: use the typed `create` commands to create a freestanding node (no link flags) or to attach it to one or more parents (repeat the appropriate `--*-ids` flags).
  - **Link**: use `dunno link` (or `--link-to` for `dunno add`) to add relationships between existing nodes.
- **Task invariants**:
  - For `dunno task create`, either:
    - Omit `--module-ids` / `--project-ids` entirely to create a **freestanding** task, or
    - Provide **exactly one** `--module-ids` and **exactly one** `--project-ids` to create a fully linked task.
  - Any other combination is rejected to avoid ambiguous hierarchies.
- **Preferred flow**:
  - Use typed `create` commands with link IDs for standard hierarchy (`project → module → submodule → file`, `project → module → task → subtask`).
  - Use `dunno link` as a low-level escape hatch when you need to wire non-standard or additional relationships between existing nodes.

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
| `contains` | project, module, submodule | module, submodule, file |
| `has_task` | project, user_story, epic | task |
| `belongs_to_project` | task, context, user_story, epic | project |
| `belongs_to_module` | task, context | module |
| `has_subtask` | task | subtask |
| `belongs_to_task` | subtask, context | task |
| `has_context` | project, task, module, submodule, subtask | context |
| `has_todo` | project | todo_item |
| `has_user_story` | project, epic | user_story |
| `belongs_to_story` | task | user_story |
| `has_module` | user_story | module |
| `has_submodule` | user_story | submodule |
| `belongs_to_user_story` | module, submodule | user_story |
| `has_epic` | project | epic |
| `belongs_to_epic` | user_story, task | epic |

## Development

### Context retrieval (implementation)

Task, file, subtask, and epic context are implemented in `src/context.rs` as single SurrealQL queries:

- **Task context:** Task `->belongs_to_module->` Module, Task `->belongs_to_project->` Project; collect `->has_context->` at each level.
- **File context:** File `<-contains<-` Submodule (if any) `<-contains<-` Module `<-contains<-` Project; collect `->has_context->` at each level.
- **Subtask context:** Subtask `->belongs_to_task->` Task `->belongs_to_module->` Module `->belongs_to_project->` Project; collect `->has_context->` at each level.
- **Epic context:** Epic `->belongs_to_project->` Project; collect `->has_context->` at epic and project levels.

Results are flattened, tagged with `node_type`, and deduplicated by id.

Knowledge links are **bidirectional**: when you link a mistake, style rule, or security detail to a structural node, the graph stores the forward edge (e.g. `task -> has_mistake -> mistake`) and reverse edges using the same relation names as tasks: `belongs_to_project`, `belongs_to_module`, and `belongs_to_task` (e.g. `mistake -> belongs_to_project -> project`, `mistake -> belongs_to_module -> module`, `mistake -> belongs_to_task -> task`). This keeps “what belongs to what” explicit and consistent with the task hierarchy.

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
