# Track Specification: Dual Storage Backend & Config System

## Overview
Replace the hardcoded SurrealDB connection with a config-driven storage layer that supports two backends: local embedded SurrealDB (zero-dependency default) and remote SurrealDB Cloud. Configuration lives in `~/.config/dunno/config.toml`.

## Objectives
1. **Config System:** Load, validate, and merge settings from `~/.config/dunno/config.toml` with CLI flag and environment variable overrides.
2. **Local Storage Backend:** Connect via embedded SurrealDB (`surrealkv://` engine) with data persisted to `~/.local/share/dunno/data.db` by default.
3. **Cloud Storage Backend:** Connect to a remote SurrealDB instance over `wss://` with namespace, database, and credential support.
4. **Unified Client:** Application code uses a single SurrealDB client interface regardless of backend — backend selection is purely a config concern.
5. **Zero-Config First Run:** The CLI must work immediately after install with no config file, defaulting to local storage.

## User Stories
- **As a developer**, I want `dunno` to work out of the box with local storage so I don't need to run a separate database process.
- **As a developer**, I want to switch to SurrealDB Cloud by editing a config file so I can sync knowledge across machines.
- **As a developer**, I want to override config values with environment variables (`DUNNO_BACKEND`, `DUNNO_CLOUD_URL`, etc.) so I can configure the tool in CI or ephemeral environments without a config file.
- **As a developer**, I want the CLI to tell me which backend it's connected to (on verbose/debug output) so I can verify my config is correct.

## Technical Requirements
- **Language:** Rust (Edition 2024)
- **Crates:**
    - `surrealdb` — unified client for both embedded and remote connections.
    - `toml` — config file parsing.
    - `dirs` (or `directories`) — resolve `~/.config/dunno` and `~/.local/share/dunno` cross-platform.
    - `serde`, `serde_json` — config struct deserialization.
    - `thiserror` — typed config/connection errors.

## Constraints
- The config file is optional — missing file = local defaults.
- Cloud credentials must never be logged or included in JSON output.
- The embedded SurrealDB data directory must be created automatically on first run.
- Existing integration tests that use `mem://` must continue to work (test mode bypasses config).

## Config Reference

```toml
# ~/.config/dunno/config.toml

# "local" | "cloud"
backend = "local"

[local]
path = "~/.local/share/dunno/data.db"

[cloud]
url = "wss://YOUR_INSTANCE.surrealdb.com"
namespace = "dunno"
database = "dunno"
username = ""
password = ""
```

## Environment Variable Overrides
| Variable | Overrides |
|---|---|
| `DUNNO_BACKEND` | `backend` |
| `DUNNO_LOCAL_PATH` | `local.path` |
| `DUNNO_CLOUD_URL` | `cloud.url` |
| `DUNNO_CLOUD_NS` | `cloud.namespace` |
| `DUNNO_CLOUD_DB` | `cloud.database` |
| `DUNNO_CLOUD_USER` | `cloud.username` |
| `DUNNO_CLOUD_PASS` | `cloud.password` |
