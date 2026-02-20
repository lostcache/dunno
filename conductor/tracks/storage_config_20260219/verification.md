# Verification Notes: Dual Storage Backend & Config System

## Automated Verification

Executed:

```bash
cargo fmt
cargo test
cargo run -- config show
DUNNO_LOCAL_PATH=./db_data/local-test.db cargo run -- project list
```

Observed:
- Test suite passes, including new config precedence/validation tests and local embedded DB tests.
- `config show` returns resolved configuration as JSON with cloud password redacted.
- Local embedded backend starts and serves CLI read operations without requiring an external SurrealDB instance.

## Shell Tests — Local (End-to-End)

Five test suites in `tests/sh/` provide comprehensive end-to-end verification of the config system against a locally persistent embedded SurrealDB:

```bash
./tests/sh/run_all.sh        # runs all 5 suites (127 assertions total)
```

| Suite | What it verifies |
|-------|-----------------|
| `env` | Full CRUD + context flow using `DUNNO_BACKEND` + `DUNNO_LOCAL_PATH` env vars |
| `config` | Full CRUD + context flow using `~/.config/dunno/config.toml` only |
| `cli` | Full CRUD + context flow using `--backend local` CLI flag |
| `precedence` | Defaults -> config file -> env vars -> CLI flag override chain |
| `cross` | Data created via one config method is readable via any other |

## Shell Tests — Cloud (End-to-End)

Three cloud test suites verify the full Phase 6 CRUD + context flow against a live SurrealDB Cloud instance:

```bash
./tests/sh/run_cloud.sh          # runs all 3 cloud suites
./tests/sh/run_cloud.sh config   # run only the config-file suite
```

| Suite | File | What it verifies |
|-------|------|-----------------|
| `env` | `test_cloud_env_vars.sh` | Full flow using `DUNNO_CLOUD_*` env vars |
| `config` | `test_cloud_config_file.sh` | Full flow using `~/.config/dunno/config.toml` (no env vars needed) |
| `cli` | `test_cloud_cli_flags.sh` | Full flow using `--backend cloud` CLI flag + env var credentials |

Cloud config file test (32/32 assertions) verified against SurrealDB Cloud with:
- `auth_type = "namespace"` (non-root authentication)
- Custom namespace, database, username, and password from config

## Coverage Added in This Iteration

- Config parsing and merging from TOML.
- Missing config file fallback to defaults.
- Invalid backend value validation.
- Env override application.
- CLI override precedence over config file backend.
- Local embedded `surrealkv://` initialization via `DB::from_config`.
- Cloud backend required-field validation guardrails.
- Cloud `auth_type` field supporting `root`, `namespace`, and `database` signin scopes.
- TLS crypto provider (`rustls` + `aws-lc-rs`) installed at startup for `wss://` connections.
- End-to-end shell tests for all three config methods and cross-method persistence (local).
- End-to-end cloud shell tests verified against live SurrealDB Cloud.

## Remaining Validation Work

- Optional UX enhancement: print active backend when verbose mode is enabled.
