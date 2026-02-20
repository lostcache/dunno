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
- Optional: SurrealDB Cloud credentials if using the cloud backend. Supports `root`, `namespace`, and `database` authentication scopes via the `auth_type` config field.

## Build

```bash
cargo build --release
```

Binary path: `target/release/lazydev`

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
lazydev project list

# Force local mode for one command
lazydev --backend local project list

# Cloud mode via env vars
DUNNO_BACKEND=cloud \
DUNNO_CLOUD_URL="wss://YOUR_INSTANCE.surrealdb.com" \
DUNNO_CLOUD_NS="dunno" \
DUNNO_CLOUD_DB="dunno" \
DUNNO_CLOUD_USER="YOUR_USER" \
DUNNO_CLOUD_PASS="YOUR_PASS" \
lazydev project list

# Inspect fully resolved config (password redacted)
lazydev config show
```

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

### Config
- `lazydev config show`: Print resolved config with secrets redacted.
- `lazydev --backend <local|cloud> ...`: Override backend for a single command run.

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
