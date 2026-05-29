# Why?

![](static/dunno_tried.jpg)

1. I want LLMs to know the high-level structure and intent of files in codebase before using grep like tools to investigate the details.
2. I don't like having to commit all the md files with the sourcecode as it's not sourcecode.
3. I want to solve the problem in a way that enables multiple people or agents fetch the latest updated context without having to sync using github.

---

# dn

A cli tool to:

1. Manage codebase knowledge at file level to avoid fuzzy reads while vibe-coding.
2. Manages TODOs, Tasks, Plans, Skills, Persona and all other md files meant for Agents.

---

## Vibe-coding responsibly with dn

https://github.com/user-attachments/assets/f94c6514-aba1-498d-aa5d-4a7007550108

## Using dn for an issue

https://github.com/user-attachments/assets/a967057b-b16d-46c5-a485-21cb70d329e8

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
backend    = "local"                        # embedded | local | cloud
local_path = "~/.local/share/dunno/data.db" # used when backend = embedded
url        = ""                             # used when backend = local (default: ws://127.0.0.1:8000/rpc) | cloud
namespace  = "dunno"                        # used when backend = local | cloud
database   = "dunno"                        # used when backend = local | cloud
username   = "root"                         # used when backend = local | cloud
password   = "root"                         # used when backend = local | cloud
auth_type  = "database"                     # used when backend = cloud
```

> Note: Only requires the SurrealDB binary for `local` backend as it spawns a local SurrealDB instance.

> Note: To use the Web UI and CLI concurrently must use `local` or `cloud` backend to avoid DB-file lock contention.

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
> [https://github.com/surrealdb/surrealdb](https://github.com/surrealdb/surrealdb)

---
