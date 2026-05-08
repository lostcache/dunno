# dn

A cli tool to:
1) Manage codebase knowledge at file level to prevent fuzzy reads while vibe-coding in a new session.
2) Manages TODOs, Tasks, Plans, Skills, Persona and all other md files meant for Agents.
3) Just two binaries and nothing else.

# Why?
1) I hate when LLM used grep or rg to read codebase while working in a new session.
2) I hate to commit all the md files with the sourcecode as it's not sourcecode.
3) I wanted to solve the above problems in a way such that enables multiple people or agents could work at the same time and to sync the md files in real time without having to sync on github.

---

## Installation

### Build from Source

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

## Configuration

dn uses a layered config (highest priority first):

1. Local project config (`./dn.toml`)
2. Global user config (`~/.config/dn/dn.toml`)
3. Built-in defaults


```toml
backend    = "local"                        # local | local-server | cloud
local_path = "~/.local/share/dunno/data.db" # used when backend = "local"
url        = ""                             # used when backend = local-server (default: ws://127.0.0.1:8000/rpc) | cloud
namespace  = "dunno"                        # used when backend = local-server | cloud
database   = "dunno"                        # used when backend = local-server | cloud
username   = "root"                         # used when backend = local-server | cloud
password   = "root"                         # used when backend = local-server | cloud
auth_type  = "root"                         # used when backend = cloud
```

> Note: Only requires the SurrealDB binary for `local-server` backend as it spawns a local SurrealDB instance.

> Note: To use the Web UI and CLI concurrently must use `local-server` or `cloud` backend to avoid DB-file lock contention.
---

## CLI (`dn`)

A command-line tool for all CRUD operations. Outputs JSON.

```bash
dn --help
dn --version
```

## Web UI (`dn-server`)

An HTTP server that serves a browser UI and a REST API.

```bash
dn-server                   # starts on :7700, opens browser
dn-server --port 8080       # custom port
dn-server --no-open         # does not open browser tab.
```

The UI provides full CRUD for all entities and an interactive graph visualization.

> To use both `dn` CLI and `dn-server` concurrently against the same local database instance, install the `surreal` binary (Recommended):
>
> ```bash
> curl -sSf https://install.surrealdb.com | sh
> ```

