# dunno

`dunno` is a Rust CLI that captures coding knowledge and retrieves deterministic context for AI agents.

Currently it just supports SurrealDB as a backend, but the architecture is designed to allow adding more backends in the future (e.g. SQLite, Postgres, etc.) without changing the core logic.

---

## 📖 User Guide

### Installation

#### Option 1: Build from Source (Current Method)

```bash
# Clone the repository
git clone <repo-url>
cd dunno

# Build release binary
cargo build --release

# The binary is now available at:
./target/release/dunno

# Optional: Install to system PATH
sudo cp target/release/dunno /usr/local/bin/
```

#### Option 2: Install via Cargo

Coming soon - not yet published to crates.io.

#### Option 3: Download Binary

Coming soon - pre-built binaries not yet available in releases.

### Quick Start

#### 1. Initial Setup

```bash
# Verify installation
dunno --version

# Check current configuration
dunno config show
```

By default, dunno uses local embedded storage at `~/.local/share/dunno/data.db`. No configuration is required to get started.

#### 2. Project Setup (Recommended)

For project-specific settings, create a local config file:

```bash
# Create local config
cat > dunno.toml << 'EOF'
[local]
path = "./.dunno/data.db"
EOF

# Create data directory
mkdir -p .dunno
```

#### 3. Create Your First Project

```bash
# Create a project
dunno project create "My App" "A web application"
# Returns: {"id":"project:abc","name":"My App","description":"A web application"}

# Create a module
dunno module create --project-ids project:abc "Auth" "Authentication system"
# Returns: {"id":"module:def", ...}

# Create a task
dunno task create --module-ids module:def --project-ids project:abc "Implement login" "Add JWT authentication"
# Returns: {"id":"task:ghi", ...}
```

#### 4. Add Knowledge

```bash
# Add a coding mistake to remember
dunno add --field type --value mistake --field content --value "Don't use unwrap() in production" --link-to task:ghi

# Add a style rule
dunno add --field type --value style --field content --value "Use Result for error handling" --link-to module:def

# Add security note
dunno add --field type --value security --field content --value "Validate all JWT tokens" --link-to project:abc
```

#### 5. Retrieve Context

```bash
# Get context for a task
dunno context --task-id task:ghi
# Returns all linked knowledge as JSON
```

### Configuration

Dunno uses a layered configuration system on a **per-field basis** (highest to lowest priority):

1. **CLI flags** (`--backend`, `--pretty`)
2. **Local project config** (`./dunno.toml`)
3. **Global user config** (`~/.config/dunno/dunno.toml`)
4. **Environment variables**
5. **Built-in defaults**

#### Global CLI Flags

- `--backend <BACKEND>` - Override storage backend (`local` or `cloud`)
- `--pretty` - Format output with indentation for better readability (applies to **all** JSON output)

```bash
# View config in JSON format (default)
dunno config show

# View config in human-readable format
dunno config show --pretty

# Pretty output works with all commands
dunno project list --pretty
dunno task list --pretty
dunno context --task-id task:abc --pretty
```

#### Config File Locations

- **Local:** `./dunno.toml` (project-specific, not committed to git)
- **Global:** `~/.config/dunno/dunno.toml` (user-wide settings)

#### Example Configuration

**Global config** (`~/.config/dunno/dunno.toml`):
```toml
backend = "cloud"

[cloud]
url = "wss://my-instance.surrealdb.com"
namespace = "my-namespace"
database = "dunno"
username = "root"
password = "root"
auth_type = "root"
```

**Local config** (`./dunno.toml`):
```toml
# Override only the database path for this project
[local]
path = "./.dunno/data.db"
```

#### Environment Variables

All config fields can be set via environment:

- `DUNNO_BACKEND` - Backend type (`local` or `cloud`)
- `DUNNO_LOCAL_PATH` - Local database file path
- `DUNNO_CLOUD_URL` - Cloud instance URL
- `DUNNO_CLOUD_NS` - Namespace
- `DUNNO_CLOUD_DB` - Database name
- `DUNNO_CLOUD_USER` - Username
- `DUNNO_CLOUD_PASS` - Password
- `DUNNO_CLOUD_AUTH_TYPE` - Auth type (`root`, `namespace`, `database`)

### Core Concepts

#### Hierarchy

Dunno organizes work into two parallel paths:

**Code Structure:**
```
Project → Module → Submodule (optional) → File
```

**Work Tracking:**
```
Project → Module → Task → Subtask (optional)
```

**Optional Layers:**
- **Epics** - Large feature groups
- **User Stories** - User-centric feature descriptions

#### Knowledge Types

Link knowledge to any structural node:

- **Mistake** - Known pitfalls and errors
- **StyleRule** - Coding conventions
- **SecurityDetail** - Security constraints
- **Custom** - Any key-value pairs you need

### CLI Reference

#### Project Management
```bash
dunno project create "<name>" "<description>"  # Create project
dunno project list                              # List all projects
```

#### Module Management
```bash
dunno module create --project-ids <id> "<name>" "<description>"
dunno module list
```

#### Task Management
```bash
dunno task create --module-ids <id> --project-ids <id> "<name>" "<description>"
dunno task list
dunno task update <id> --status started
dunno task delete <id>   # Delete a task by ID
```

#### Knowledge Management
```bash
# Add knowledge with arbitrary fields
dunno add --field <key> --value <val> [--link-to <id> ...]

# Examples:
dunno add --field type --value mistake --field content --value "Avoid panic!" --link-to project:abc
dunno add --field type --value style --field language --value rust --field rule --value "Use ? operator" --link-to module:def
```

#### Context Retrieval
```bash
dunno context --task-id <id>
dunno context --file-id <id>
dunno context --subtask-id <id>
dunno context --epic-id <id>
```

#### Linking
```bash
dunno link --from-id <id> --edge <type> --to-ids <id> [<id> ...]
```

### Common Workflows

**1. Starting a New Feature:**
```bash
# Create epic for the feature
dunno epic create --project-id project:abc "User Authentication" "Complete auth system"

# Create user story
dunno user-story create --project-id project:abc --epic-ids epic:mno "As a user, I want to login" "Authentication feature"

# Create implementation task
dunno task create --module-ids module:def --project-ids project:abc --epic-ids epic:mno "Implement JWT" "Add token support"
```

**2. Recording Mistakes:**
```bash
# After fixing a bug, record it for future reference
dunno add --field type --value mistake \
  --field content --value "MutexGuard across await causes deadlock" \
  --field solution --value "Use tokio::sync::Mutex instead" \
  --field severity --value high \
  --link-to task:ghi
```

**3. Code Review Context:**
```bash
# Before reviewing, get all context for a task
dunno context --task-id task:ghi | jq '.[] | select(.fields.type == "mistake")'
```

**4. Cleaning Up:**
```bash
# List all tasks
dunno task list

# Delete a task that's no longer needed
dunno task delete task:abc123

# Verify deletion
dunno task list
```

---

## 🔧 Developer Guide

### Architecture Overview

Dunno is built on a graph database (SurrealDB) with a hierarchical structure. The codebase follows a modular architecture:

```
src/
├── main.rs          # Entry point and CLI dispatch
├── args.rs          # CLI argument definitions (clap)
├── config.rs        # Configuration management
├── context.rs       # Context retrieval logic
├── ingest.rs        # Knowledge ingestion
├── models.rs        # Data models/structs
└── db/
    ├── mod.rs       # DB module interface
    └── surreal/     # SurrealDB implementation
        ├── mod.rs   # Connection management
        ├── context.rs   # Context queries
        ├── ingest.rs    # Knowledge operations
        └── entities/    # Entity CRUD operations
            ├── projects.rs
            ├── modules.rs
            ├── tasks.rs
            └── ...
```

### Core Design Principles

1. **Graph-Based Hierarchy** - All data is stored as nodes and edges in a graph
2. **Deterministic Retrieval** - No search algorithms; exact graph traversal only
3. **Bidirectional Links** - Every link has a reverse edge for consistency
4. **Schemaless Knowledge** - Knowledge entries use arbitrary key-value fields
5. **Local-First** - Default embedded database; cloud is optional

### Technology Stack

- **Language:** Rust (Edition 2024)
- **CLI Framework:** clap v4.5 with derive macros
- **Database:** SurrealDB v3.0.0 (embedded or cloud)
- **Serialization:** serde with JSON/TOML support
- **Async Runtime:** tokio

### Database Schema

#### Entities (Nodes)

| Entity | Record ID Pattern | Description |
|--------|-------------------|-------------|
| Project | `project:<id>` | Top-level container |
| Module | `module:<id>` | Code organization unit |
| Submodule | `submodule:<id>` | Nested code unit |
| File | `file:<id>` | Source file reference |
| Task | `task:<id>` | Work item |
| Subtask | `subtask:<id>` | Sub-work item |
| Epic | `epic:<id>` | Large feature group |
| UserStory | `user_story:<id>` | User-centric feature |
| Context | `context:<id>` | Knowledge entry |
| Todo | `todo_item:<id>` | Work queue item |

#### Graph Relations (Edges)

| Edge | From | To | Purpose |
|------|------|-----|---------|
| `contains` | project, module, submodule | module, submodule, file | Structural containment |
| `has_task` | project, epic, user_story | task | Task assignment |
| `has_subtask` | task | subtask | Subtask grouping |
| `has_context` | *any structural* | context | Knowledge linking |
| `has_user_story` | project, epic | user_story | Story grouping |
| `has_epic` | project | epic | Epic grouping |
| `has_todo` | project | todo_item | Todo tracking |
| `belongs_to_project` | task, context, user_story, epic | project | Reverse link |
| `belongs_to_module` | task, context | module | Reverse link |
| `belongs_to_task` | subtask, context | task | Reverse link |
| `belongs_to_story` | task | user_story | Reverse link |
| `belongs_to_epic` | user_story, task | epic | Reverse link |

### Development Setup

#### Prerequisites

- Rust toolchain (stable) with `cargo`
- Optional: SurrealDB Cloud credentials for cloud backend testing

#### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run with cargo
cargo run -- --help
```

#### Testing

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test config
cargo test db::surreal

# Run with output
cargo test -- --nocapture
```

Tests use in-memory backends (`mem://`) and don't require a SurrealDB server.

#### Shell Tests

```bash
# Run local integration tests
./tests/sh/run_all.sh

# Run cloud integration tests (requires cloud credentials)
./tests/sh/run_cloud.sh
```

### Configuration Implementation

Configuration is loaded in priority order (lowest to highest):

1. **Defaults** - Hardcoded in `Config::default()`
2. **ENV vars** - Applied via `apply_env_overrides()`
3. **Global config** - `~/.config/dunno/dunno.toml`
4. **Local config** - `./dunno.toml`
5. **CLI args** - Overrides passed to `Config::load()`

Each source only overrides fields it explicitly defines (partial config support).

### Context Retrieval Implementation

Context queries are implemented in `src/context.rs` using SurrealQL:

```rust
// Task context query traverses:
// Task -> belongs_to_module -> Module -> belongs_to_project -> Project
// Collects has_context edges at each level
```

Results are:
1. Flattened from nested graph structure
2. Tagged with `node_type` for identification
3. Deduplicated by record ID

### Adding New Features

#### 1. New Entity Type

1. Add model to `src/models.rs`
2. Create entity module in `src/db/surreal/entities/`
3. Add CRUD operations
4. Add CLI commands to `src/args.rs`
5. Add dispatch in `src/main.rs`
6. Add tests

#### 2. New Knowledge Field

No code changes needed! Knowledge is schemaless. Users can add any fields:

```bash
dunno add --field my_custom_field --value "anything" --link-to task:abc
```

#### 3. New Config Option

1. Add field to `Config` struct in `src/config.rs`
2. Add to `PartialConfig` for file loading
3. Add env var in `apply_env_overrides()`
4. Add CLI arg in `src/args.rs` if needed
5. Update tests

### Contribution Guidelines

1. **Tests Required** - All new features must include tests
2. **Test Isolation** - Tests must be thread-safe and use unique temp files
3. **Error Handling** - Use `anyhow` for errors; provide meaningful messages
4. **CLI Consistency** - Follow existing command patterns
5. **Documentation** - Update README and inline docs

### Release Process

When ready to distribute:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite: `cargo test`
4. Build release: `cargo build --release`
5. Test binary: `./target/release/dunno --version`
6. Create git tag: `git tag vX.Y.Z`
7. Push tag: `git push origin vX.Y.Z`
8. Create GitHub Release with binary

### Distribution Setup (For Project Maintainers)

The following sections describe how to set up distribution channels:

#### Cargo (crates.io)

Publish to crates.io for `cargo install` support:

```bash
cargo publish --dry-run
cargo publish
```

#### Binary Releases

Build for multiple targets to create release binaries:

```bash
# macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```

#### Homebrew (macOS/Linux)

Create a Homebrew formula:

```ruby
class Dunno < Formula
  desc "Capture and retrieve coding knowledge"
  homepage "https://github.com/yourusername/dunno"
  url "https://github.com/yourusername/dunno/archive/v1.0.0.tar.gz"
  sha256 "..."
  
  depends_on "rust" => :build
  
  def install
    system "cargo", "install", *std_cargo_args
  end
end
```

### Troubleshooting

**Test failures due to environment:**
```bash
# Run single-threaded to avoid race conditions
cargo test -- --test-threads=1
```

**Database locked:**
```bash
# Kill any hanging dunno processes
pkill -f dunno
```

**Reset local database:**
```bash
rm -rf ~/.local/share/dunno/
# or for project-specific:
rm -rf ./.dunno/
```
