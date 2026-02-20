# Track Plan: Dual Storage Backend & Config System

## Phase 1: Config Module Rewrite
- [x] Task: Add `dirs` crate dependency and define config directory constants (`~/.config/dunno`, `~/.local/share/dunno`).
- [x] Task: Rewrite `Config` struct to model dual backends (`backend`, `local`, `cloud` sections) with serde defaults.
- [x] Task: Implement config loading: file read from `~/.config/dunno/config.toml` -> env var overrides -> CLI flag overrides. Missing file = defaults.
- [x] Task: Write unit tests for config loading (file parsing, env overrides, missing file defaults, invalid values).
- [x] Task: Conductor - User Manual Verification 'Phase 1: Config Module Rewrite' (Protocol in workflow.md)

## Phase 2: Local Embedded Storage
- [x] Task: Update SurrealDB connection logic to support `surrealkv://` (embedded, file-backed) using the path from config.
- [x] Task: Auto-create the data directory (`~/.local/share/dunno/`) on first run if it doesn't exist.
- [x] Task: Migrate existing `db.rs` initialization to use the new config-driven connection (remove hardcoded `ws://localhost:8000`).
- [x] Task: Write integration tests for local embedded storage (CRUD lifecycle with file-backed DB).
- [x] Task: Conductor - User Manual Verification 'Phase 2: Local Embedded Storage' (Protocol in workflow.md)

## Phase 3: Cloud Storage Backend
- [x] Task: Implement cloud connection path: `wss://` endpoint with namespace, database, and sign-in from config/env vars.
- [x] Task: Add credential validation (fail fast with a clear error if cloud is selected but URL/credentials are missing).
- [x] Task: Add configurable `auth_type` field (`root` | `namespace` | `database`) to `CloudConfig` for flexible SurrealDB signin scopes.
- [x] Task: Install `rustls` crypto provider (`aws-lc-rs`) at startup for TLS on `wss://` connections.
- [x] Task: Write end-to-end cloud shell tests (`test_cloud_env_vars.sh`, `test_cloud_config_file.sh`, `test_cloud_cli_flags.sh`) verified against live SurrealDB Cloud.
- [x] Task: Conductor - User Manual Verification 'Phase 3: Cloud Storage Backend' — all 32 assertions pass against SurrealDB Cloud.

## Phase 4: CLI Wiring & Polish
- [x] Task: Add `--backend` CLI flag override (optional, takes precedence over config file).
- [ ] Task: Print active backend on `--verbose` / debug output (never print credentials).
- [x] Task: Add `dunno config show` subcommand that prints resolved config (redacting secrets).
- [x] Task: Update README and help text to document config file location, backend options, and `auth_type`.
- [x] Task: Conductor - User Manual Verification 'Phase 4: CLI Wiring & Polish' (Protocol in workflow.md)
