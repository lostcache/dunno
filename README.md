# dn

`dn` is a Rust CLI and web UI that captures coding knowledge (mistakes, style guides, security details, custom fields) in a graph database and retrieves deterministic context for AI agents. Unlike natural language search, dn uses a strict graph hierarchy where knowledge is linked to nodes and inherited down the tree.

---

## For Users

### Installation

#### Build from Source

```bash
git clone <repo-url>
cd dunno

# Build everything (UI then Rust binaries)
make build

# The binaries are now at:
./target/release/dn
./target/release/dn-server

# Optional: install to PATH
sudo cp target/release/dn /usr/local/bin/
sudo cp target/release/dn-server /usr/local/bin/
```

**Prerequisites for `make build`:** Node.js (for the UI build) and Rust stable toolchain.

> `cargo publish` / pre-built binaries: not yet available.

---

### Interfaces

dn has two interfaces that share the same database.

#### CLI (`dn`)

A command-line tool for all CRUD operations and context retrieval. Outputs JSON.

```bash
dn --version
dn --help
```

#### Web UI (`dn-server`)

An HTTP server that serves a browser UI and a REST API. Launches a browser tab automatically.

```bash
dn-server                   # starts on :7700, opens browser
dn-server --port 8080       # custom port
dn-server --no-open         # suppress auto-open
dn-server --backend cloud   # override backend
```

The UI provides full CRUD for all entities and an interactive graph visualization of the knowledge hierarchy.

---

### Quick Start

#### 1. Start the server (recommended for new users)

```bash
dn-server
```

This opens the web UI at `http://127.0.0.1:7700`. If you have the `surreal` binary installed, `dn-server` manages SurrealDB automatically for the `local` backend.

> To use both `dn` CLI and `dn-server` concurrently against the same local database, install the `surreal` binary:
>
> ```bash
> curl -sSf https://install.surrealdb.com | sh
> ```

#### 2. Or use the CLI directly

```bash
# Verify installation
dn --version

# Check current configuration
dn config show
```

By default, dn uses local embedded storage at `~/.local/share/dn/data.db`. No configuration is required.

#### 3. Create a project

```bash
dn project add "My App" "A web application"
# Returns: {"id":"project:abc","name":"My App","description":"A web application"}
```

#### 4. Add structure

```bash
# Create a module (using project ID)
dn module add --pids project:abc "Auth" "Authentication system"

# Or using project name
dn module add -p "My App" "Auth" "Authentication system"

# Create a child module (nested under a parent module)
dn module add --pids project:abc --parent-module-id module:def "JWT" "JWT handling"

# Create a task
dn task add --mids module:def -p "My App" "Implement login" "Add JWT authentication"
```

#### 5. Add knowledge

```bash
# Link a mistake to a task
dn add -f type -v mistake -f content -v "Don't use unwrap() in production" --ln task:ghi

# Link a style rule to a module
dn add -f type -v style -f content -v "Use Result for error handling" --ln module:def

# Add a security note to the project
dn add -f type -v security -f content -v "Validate all JWT tokens" --ln project:abc
```

#### 6. Retrieve context

```bash
# Context for a task (task + directly linked knowledge)
dn ctx --tid task:ghi

# Full inherited context (includes project and module rules)
dn ctx --tid task:ghi --full
```

---

### Configuration

dn uses a layered config (highest priority first):

1. CLI flags (`--backend`, `--pretty`)
2. Local project config (`./dn.toml`)
3. Global user config (`~/.config/dn/dn.toml`)
4. Environment variables
5. Built-in defaults

#### Config Files

- **Local:** `./dn.toml` — project-specific, typically not committed
- **Global:** `~/.config/dn/dn.toml` — user-wide settings

#### Example Configurations

**Local embedded (default):**

```toml
backend = "local"
local_path = "./.dn/data.db"
```

**Local server** (connecting to a running SurrealDB instance):

```toml
backend = "local-server"
url = "ws://127.0.0.1:8000/rpc"
namespace = "dunno"
database = "dunno"
username = "root"
password = "root"
```

**Cloud:**

```toml
backend = "cloud"
url = "wss://my-instance.surrealdb.com"
namespace = "my-namespace"
database = "dn"
username = "root"
password = "root"
auth_type = "root"
```

#### Environment Variables

| Variable           | Description                                    |
| ------------------ | ---------------------------------------------- |
| `DUNNO_BACKEND`    | Backend type: `local`, `local-server`, `cloud` |
| `DUNNO_LOCAL_PATH` | Local database file path                       |
| `DUNNO_URL`        | SurrealDB instance URL                         |
| `DUNNO_NS`         | Namespace                                      |
| `DUNNO_DB`         | Database name                                  |
| `DUNNO_USER`       | Username                                       |
| `DUNNO_PASS`       | Password                                       |
| `DUNNO_AUTH_TYPE`  | Auth type: `root`, `namespace`, `database`     |

#### Global CLI Flags

```bash
dn [FLAGS] <COMMAND>

--backend, --b <BACKEND>   Override storage backend
--pretty, --pp             Pretty-print JSON output
-i, --ignore-case          Case-insensitive project name matching
```

---

### Core Concepts

#### Hierarchy

```
Project → Module → Module → ... → File
Project → Module → Task
Project → Epic → User Story → Task
```

Modules nest recursively to any depth. Use `--parent-module-id` to create a child module.

Knowledge can be attached to any node. When retrieving context with `--full`, knowledge is inherited from all ancestors.

#### Knowledge Types

Knowledge entries are schemaless key-value maps. Common conventions:

| `type` value   | Purpose                   |
| -------------- | ------------------------- |
| `mistake`      | Known pitfalls and errors |
| `style`        | Coding conventions        |
| `security`     | Security constraints      |
| _(any string)_ | Custom knowledge types    |

---

### CLI Reference

#### Project

```bash
dn project add "<name>" "<description>"
dn project ls
```

#### Module

```bash
dn module add {--pids|--project-ids} <pid> "<name>" "<desc>" [--notes <notes>]
dn module add {-p|--project} "<project_name>" "<name>" "<desc>"
dn module add --pids <pid> --parent-module-id <mid> "<name>" "<desc>"   # child module
dn module ls
dn module ls {--pid|--project-id} <id>
dn module ls {-p|--project} "<project_name>"
dn module ls {--mid|--module-id} <id>   # list child modules of a module
```

#### Task

```bash
dn task add --project-id <pid> "<name>" "<desc>"
dn task add -p "<project_name>" "<name>" "<desc>"
dn task add --mids <mid> --project-id <pid> "<name>" "<desc>"
dn task add --mids <mid> -p "<project_name>" "<name>" "<desc>"
dn task ls
dn task ls {--pid|--project-id} <id>
dn task update <id> --status started
dn task rm <id>
```

#### File

```bash
dn file add --pids <pid> "<name>" "<path>" ["<desc>"] [--notes <notes>]
dn file add --pids <pid> --parent-ids <module_id> "<name>" "<path>"
dn file ls
dn file ls {--pid|--project-id} <id>
dn file ls {--mid|--module-id} <id>
```

#### User Story

```bash
dn user-story add {--pid|--project-id} <id> "<title>" "<desc>"
dn user-story add -p "<project_name>" "<title>" "<desc>"
dn user-story ls {--pid|--project-id} <id>
```

#### Epic

```bash
dn epic add {--pid|--project-id} <id> "<title>" "<desc>"
dn epic add -p "<project_name>" "<title>" "<desc>"
dn epic ls {--pid|--project-id} <id>
```

#### Persona

```bash
dn persona add --pids <pid> "<name>" "<content>"
dn persona ls
dn persona rm <id>
```

#### Workflow

```bash
dn workflow add --pids <pid> "<name>" "<content>"
dn workflow ls
dn workflow rm <id>
```

#### Todo

```bash
dn todo add --pids <pid> "<content>"
dn todo ls {--pid|--project-id} <id>
dn todo rm <id>
```

#### Issue

```bash
dn issue add "<title>" "<description>"
dn issue add --task-id <task_id> "<title>" "<description>"
dn issue add --task-id <task_id> --plan "<plan>" "<title>" "<description>"
dn issue update <id> [--title <title>] [--description <desc>] [--plan <plan>] [--status <status>]
dn issue ls
dn issue ls --task-id <task_id>
dn issue rm <id> [<id> ...]
```

Issue status values: `pending` (default), `active`, `completed`.

#### Knowledge

```bash
dn add {-f|--field} <key> {-v|--value} <val> [{--ln|--link-to} <id> ...]

# Example with multiple fields
dn add -f type -v mistake \
  -f content -v "MutexGuard across await causes deadlock" \
  -f solution -v "Use tokio::sync::Mutex instead" \
  -f severity -v high \
  --ln task:ghi
```

#### Context

```bash
dn ctx {--tid|--task-id} <id> [--full]
dn ctx {--fid|--file-id} <id> [--full]
dn ctx {--eid|--epic-id} <id> [--full]
```

#### Link

```bash
dn link {-f|--from-id} <id> {-e|--edge} <type> {-t|--to-ids} <id> [<id> ...]

# Example
dn link -f file:abc -e belongs_to_task -t task:ghi
```

#### Short Flags & Aliases

| Long               | Short     |
| ------------------ | --------- |
| `--backend`        | `--b`     |
| `--pretty`         | `--pp`    |
| `--ignore-case`    | `-i`      |
| `--field`          | `-f`      |
| `--value`          | `-v`      |
| `--link-to`        | `--ln`    |
| `--project`        | `-p`      |
| `--project-id`     | `--pid`   |
| `--project-ids`    | `--pids`  |
| `--module-id`      | `--mid`   |
| `--module-ids`     | `--mids`  |
| `--parent-module-id` | `--pmid` |
| `--task-id`        | `--tid`   |
| `--file-id`        | `--fid`   |
| `--epic-id`        | `--eid`   |
| `--epic-ids`       | `--eids`  |
| `--user-story-ids` | `--usids` |
| `--from-id`        | `-f`      |
| `--edge`           | `-e`      |
| `--to-ids`         | `-t`      |

| Command      | Aliases         |
| ------------ | --------------- |
| `project`    | `proj`, `prj`   |
| `module`     | `mod`, `mdl`    |
| `file`       | `f`, `fi`       |
| `task`       | `t`, `tk`       |
| `user-story` | `us`, `story`   |
| `epic`       | `ep`, `e`       |
| `todo`       | `td`, `to`      |
| `persona`    | `per`           |
| `workflow`   | `wf`            |
| `config`     | `cfg`, `conf`   |
| `link`       | `ln`            |
| `context`    | `ctx`           |

---

### Common Workflows

**Starting a new feature:**

```bash
dn epic add -p "My App" "User Authentication" "Complete auth system"
dn user-story add -p "My App" --eids epic:mno "As a user, I want to login" "Auth feature"
dn task add --mids module:def -p "My App" --eids epic:mno "Implement JWT" "Add token support"
```

**Recording a mistake after a bug fix:**

```bash
dn add -f type -v mistake \
  -f content -v "MutexGuard across await causes deadlock" \
  -f solution -v "Use tokio::sync::Mutex instead" \
  --ln task:ghi
```

**Working on an issue:**

```bash
# 1. List open issues (optionally filter by task)
dn issue ls --task-id task:abc

# 2. Create an issue and record a resolution plan
dn issue add --task-id task:abc --plan "Investigate token expiry logic in auth module" "Token expiry bug" "Tokens expire 10 min too early"

# 3. Mark it active when you start
dn issue update issue:xyz --status active

# 4. Update the plan as you learn more
dn issue update issue:xyz --plan "Root cause: clock skew between services. Fix: use UTC everywhere."

# 5. Mark it completed when resolved
dn issue update issue:xyz --status completed
```

**Getting context before a code review:**

```bash
dn ctx --tid task:ghi --full | jq '.[] | select(.fields.type == "mistake")'
```

**Resetting local database:**

```bash
rm -rf ~/.local/share/dn/
# or for project-specific:
rm -rf ./.dn/
```

---

## For Developers

### Architecture Overview

dn is a Cargo workspace with three crates:

```
dunno/
├── dn-core/        # Core library: DB abstraction, models, config, context queries
├── dn/             # CLI binary (dn) — clap-based, calls dn-core
├── dn-server/      # HTTP server binary (dn-server) — axum REST API + embedded Svelte UI
├── ui/             # Svelte 5 frontend (built into static/dist/, embedded in dn-server)
├── static/         # Build output directory (generated by make ui-build)
├── tests/          # Shell integration tests
└── Makefile        # Build orchestration
```

#### `dn-core`

The shared library consumed by both `dn` and `dn-server`. Contains:

- `config.rs` — Layered config loading (defaults → env → global → local → CLI)
- `models.rs` — All entity types (Project, Module, File, Task, Epic, UserStory, Context, Todo, Persona, Workflow)
- `db/surreal/` — SurrealDB implementation: connection, CRUD per entity, context queries, graph queries
- `context.rs` — Context retrieval orchestration
- `ingest.rs` — Schemaless knowledge ingestion

#### `dn-server`

An axum HTTP server that:

- Serves a REST API under `/api/*` (full CRUD for all entities, context queries, graph endpoints)
- Embeds the built Svelte UI via `rust-embed` (served as SPA with client-side routing fallback)
- For the `local` backend, auto-spawns a `surreal` server process on a random port and tears it down on exit
- Defaults to port `7700`, auto-opens browser unless `--no-open` is passed

#### `ui`

A Svelte 5 SPA built with Vite. Communicates with `dn-server` via the REST API. Includes a graph visualization view powered by Cytoscape.js.

---

### Tech Stack

| Layer         | Technology                                     |
| ------------- | ---------------------------------------------- |
| Language      | Rust (Edition 2024)                            |
| CLI           | clap v4.5 (derive macros)                      |
| HTTP server   | axum 0.7 + tower-http                          |
| Database      | SurrealDB v3.0.0 (embedded kv-surrealkv or WS) |
| Async         | tokio                                          |
| Serialization | serde (JSON + TOML)                            |
| UI framework  | Svelte 5 + Vite 8                              |
| UI components | bits-ui (shadcn-svelte) + Tailwind CSS v4      |
| Graph viz     | Cytoscape.js                                   |
| UI embedding  | rust-embed                                     |

---

### Development Setup

#### Prerequisites

- Rust stable toolchain
- Node.js (for UI development)
- `surreal` binary (required for `dn-server` local backend with concurrent CLI access)

#### Build

```bash
# Full production build (UI then both Rust binaries)
make build

# Build only the CLI
cargo build --release --bin dn

# Build only the server (needs static/dist/ to already exist)
cargo build --release --bin dn-server

# Build only the UI
make ui-build
```

#### Development Workflow

Run the backend and UI dev server separately for hot-reload:

```bash
# Terminal 0: start SurrealDB (required for local-server backend)
surreal start --bind 127.0.0.1:8000 --username root --password root surrealkv://~/.local/share/dunno/data.db

# Set local-server backend in dunno.toml:
# backend = "local-server"

# Terminal 1: Rust server (no browser auto-open)
cargo run --bin dn-server -- --no-open

# Terminal 2: Vite dev server with HMR (proxies /api to :7700)
make dev
```

The Vite dev server runs on `:5173` and proxies `/api` requests to the Rust server at `:7700`.

---

### Testing

```bash
# Run all unit tests
cargo test

# Run with output
cargo test -- --nocapture

# Run single-threaded (avoids file lock races with config tests)
cargo test -- --test-threads=1

# Specific module
cargo test db::surreal

# Shell integration tests (local backend)
./tests/sh/run_all.sh

# Shell integration tests (cloud backend — requires credentials)
./tests/sh/run_cloud.sh
```

Unit tests use in-memory SurrealDB (`mem://`) and don't require a running server.

---

### Database Schema

#### Entities

| Entity    | Record ID Pattern | Description                 |
| --------- | ----------------- | --------------------------- |
| Project   | `project:<id>`    | Top-level container         |
| Module    | `module:<id>`     | Code organization unit (nests recursively) |
| File      | `file:<id>`       | Source file reference       |
| Task      | `task:<id>`       | Work item                   |
| Epic      | `epic:<id>`       | Large feature group         |
| UserStory | `user_story:<id>` | User-centric feature        |
| Context   | `context:<id>`    | Schemaless knowledge entry  |
| Todo      | `todo_item:<id>`  | Work queue item             |
| Persona   | `persona:<id>`    | AI agent persona definition |
| Workflow  | `workflow:<id>`   | Workflow definition         |
| Issue     | `issue:<id>`      | Bug or problem linked to a task |

#### Graph Relations

| Edge                    | From                                                     | To          | Purpose                |
| ----------------------- | -------------------------------------------------------- | ----------- | ---------------------- |
| `contains`              | project, module                                          | module, file | Structural containment |
| `has_task`              | project, epic, user_story                                | task        | Task assignment        |
| `has_context`           | project, task, module, epic, file                        | context     | Knowledge linking      |
| `has_user_story`        | project, epic                                            | user_story  | Story grouping         |
| `has_epic`              | project                                                  | epic        | Epic grouping          |
| `has_todo`              | project                                                  | todo_item   | Todo tracking          |
| `has_persona`           | project                                                  | persona     | Persona grouping       |
| `has_workflow`          | project                                                  | workflow    | Workflow grouping      |
| `belongs_to_project`    | task, context, user_story, epic, file, module, persona, workflow | project | Reverse link      |
| `belongs_to_module`     | task, context, file, module                              | module      | Reverse link (module→parent for child modules) |
| `belongs_to_story`      | task                                                     | user_story  | Reverse link           |
| `belongs_to_user_story` | module                                                   | user_story  | Reverse link           |
| `belongs_to_epic`       | user_story, task                                         | epic        | Reverse link           |
| `has_issue`             | task                                                     | issue       | Issue tracking         |
| `belongs_to_task`       | file, context, issue                                     | task        | Reverse link           |

---

### Adding New Features

#### New Entity

1. Add model to `dn-core/src/models.rs`
2. Create entity file in `dn-core/src/db/surreal/entities/`
3. Expose via `DB` in `dn-core/src/db/surreal/mod.rs`
4. Add CLI commands to `dn/src/args.rs` and dispatch in `dn/src/main.rs`
5. Add REST handlers and routes to `dn-server/src/main.rs`
6. Add tests

#### New Config Option

1. Add field to `Config` in `dn-core/src/config.rs`
2. Add to `PartialConfig` for file loading
3. Add env var in `apply_env_overrides()`
4. Add CLI arg in `dn/src/args.rs` if user-facing

#### New Knowledge Field

No code changes needed. Knowledge is schemaless:

```bash
dn add -f my_custom_field -v "anything" --ln task:abc
```

---

### Contribution Guidelines

- Tests required for all new features
- Tests must be thread-safe; use unique temp files or `mem://` backends
- Use `anyhow` for errors with meaningful messages
- Follow existing CLI command patterns (names, flag aliases, output format)
- Update this README when adding user-facing features

---

### Troubleshooting

**Database locked / port conflict:**

```bash
pkill -f dn-server
pkill -f surreal
```

**`dn-server` says "UI not built":**

```bash
make ui-build
```

**Test failures due to concurrency:**

```bash
cargo test -- --test-threads=1
```
