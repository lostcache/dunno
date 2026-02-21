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
- **Todo:** A project-level work queue item that can map to a task.
- **TaskUpdate:** Append-only log entries on a task.

## Retrieval Strategy

Retrieval is purely deterministic and graph-based:
1. Provide a `task_id` or `file_id`.
2. **Task path:** Traverses Task -> Module -> Project, collecting all linked knowledge at each level.
3. **File path:** Traverses File -> Submodule (if any) -> Module -> Project, collecting all linked knowledge at each level.
4. All unique knowledge nodes (Mistakes, Style Rules, Security Details) are deduplicated and returned as JSON.

## Prerequisites

- Rust toolchain (stable) with `cargo`.
- Optional: SurrealDB Cloud credentials if using the cloud backend. Supports `root`, `namespace`, and `database` authentication scopes via the `auth_type` config field.

## Build

```bash
cargo build --release
```

Binary path: `target/release/dunno`

## Configuration

The CLI resolves configuration from:
1. CLI flags
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

### Backend examples

```bash
# Default local embedded mode (no config file needed)
dunno project list

# Force local mode for one command
dunno --backend local project list

# Cloud mode via env vars
DUNNO_BACKEND=cloud \
DUNNO_CLOUD_URL="wss://YOUR_INSTANCE.surrealdb.com" \
DUNNO_CLOUD_NS="dunno" \
DUNNO_CLOUD_DB="dunno" \
DUNNO_CLOUD_USER="YOUR_USER" \
DUNNO_CLOUD_PASS="YOUR_PASS" \
dunno project list

# Inspect fully resolved config (password redacted)
dunno config show
```

## Quick Start

### 1. Initialize your hierarchy

```bash
# Create a project
dunno project create "My App" "A web application"
# Returns project:abc

# Create a module within the project
dunno module create --project-id project:abc "Auth" "Authentication system"
# Returns module:def

# (Optional) Create a submodule within the module
dunno submodule create --module-id module:def "OAuth" "OAuth2 providers"
# Returns submodule:xyz

# Register a file under the module (or submodule)
dunno file create --parent-id module:def "oauth.rs" "src/auth/oauth.rs"
# Or use submodule as parent:
dunno file create --parent-id submodule:xyz "oauth.rs" "src/auth/oauth.rs"
# Returns file:456

# Create a task within the module
dunno task create --module-id module:def "Implement JWT" "Add token support"
# Returns task:ghi

# (Optional) Create a subtask within the task
dunno subtask create --task-id task:ghi "Write tests" "Add unit tests"
# Returns subtask:stu
```

### 2. Link knowledge

```bash
# Add a style rule to the project
dunno add --type style --content "Use explicit error types" --link-to project:abc

# Add a mistake to a task
dunno add --type mistake --content "Do not log raw passwords" --link-to task:ghi

# Add a security note to a module
dunno add --type security --content "Validate all user inputs" --link-to module:def
```

### 3. Retrieve context

```bash
# By task (traverses Task -> Module -> Project)
dunno context --task-id task:ghi

# By file (traverses File -> Submodule -> Module -> Project)
dunno context --file-id file:456
```

**Expected output:**
A JSON object containing all linked knowledge (style rules, mistakes, security details) aggregated from the node and its ancestors.

## CLI Commands

### Knowledge Management
- `dunno add --type <mistake|style|security> --content <CONTENT> [--link-to <ID>]`: Add a knowledge entry and optionally link it to a structural node.
- `dunno context --task-id <ID>`: Retrieve aggregated context for a task.
- `dunno context --file-id <ID>`: Retrieve aggregated context for a file.
- `dunno context --subtask-id <ID>`: Retrieve aggregated context for a subtask.

### Hierarchy Management
- `dunno project create <NAME> <DESC>` / `list`
- `dunno module create --project-id <ID> <NAME> <DESC>` / `list`
- `dunno submodule create --module-id <ID> <NAME> <DESC>` / `list`
- `dunno file create --parent-id <ID> <NAME> <PATH>` / `list [--module-id <ID>] [--submodule-id <ID>]`
- `dunno task create --module-id <ID> <NAME> <DESC>` / `list`
- `dunno task update <TASK_ID> [--name <NAME>] [--description <DESC>] [--status <not_started|started|finished>]`
- `dunno task append-update <TASK_ID> <CONTENT>`
- `dunno task update-entry <UPDATE_ID> <CONTENT>`
- `dunno task list-updates <TASK_ID>`
- `dunno subtask create --task-id <ID> <NAME> <DESC>` / `list --task-id <ID>`

### Work Queue
- `dunno todo create --project-id <ID> <CONTENT>` / `list --project-id <ID>`

### Config
- `dunno config show`: Print resolved config with secrets redacted.
- `dunno --backend <local|cloud> ...`: Override backend for a single command run.

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

### Unit & Integration Tests

```bash
cargo test
```

Unit and integration tests run against in-memory backends (`mem://`) and do not require a separate SurrealDB server or config file.

### Shell Tests (Local Persistence)

End-to-end shell tests verify the full CLI against a locally persistent embedded SurrealDB instance. They cover all three configuration methods.

```bash
# Run all suites
./tests/sh/run_all.sh

# Run specific suites
./tests/sh/run_all.sh env config cli

# Run a single suite directly
./tests/sh/test_local_env_vars.sh
```

| Suite | File | What it tests |
|-------|------|---------------|
| `env` | `test_local_env_vars.sh` | Full Phase-6 flow using `DUNNO_BACKEND` + `DUNNO_LOCAL_PATH` |
| `config` | `test_local_config_file.sh` | Full Phase-6 flow using `~/.config/dunno/config.toml` |
| `cli` | `test_local_cli_flags.sh` | Full Phase-6 flow using `--backend local` CLI flag |
| `precedence` | `test_local_precedence.sh` | Config precedence: defaults → file → env → CLI |
| `cross` | `test_local_cross_method.sh` | Data created via one config method readable via another |

Each script is self-contained: builds the binary, creates isolated test DBs, backs up and restores any existing config file, and cleans up on exit.

### Shell Tests (Cloud)

Cloud end-to-end tests verify the full CLI against a live SurrealDB Cloud instance. They require a valid `~/.config/dunno/config.toml` with `backend = "cloud"` (for the config test) or `DUNNO_CLOUD_URL` env var (for env/cli tests).

```bash
# Run all cloud suites
./tests/sh/run_cloud.sh

# Run a specific cloud suite
./tests/sh/run_cloud.sh config
```

| Suite | File | What it tests |
|-------|------|---------------|
| `env` | `test_cloud_env_vars.sh` | Full Phase-6 flow using `DUNNO_CLOUD_*` env vars |
| `config` | `test_cloud_config_file.sh` | Full Phase-6 flow using `~/.config/dunno/config.toml` only |
| `cli` | `test_cloud_cli_flags.sh` | Full Phase-6 flow using `--backend cloud` CLI flag |
