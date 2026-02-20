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

## Shell Tests (End-to-End)

Five test suites in `tests/sh/` provide comprehensive end-to-end verification of the config system against a locally persistent embedded SurrealDB:

```bash
./tests/sh/run_all.sh        # runs all 5 suites (127 assertions total)
```

| Suite | What it verifies |
|-------|-----------------|
| `env` | Full CRUD + context flow using `DUNNO_BACKEND` + `DUNNO_LOCAL_PATH` env vars |
| `config` | Full CRUD + context flow using `~/.config/dunno/config.toml` only |
| `cli` | Full CRUD + context flow using `--backend local` CLI flag |
| `precedence` | Defaults → config file → env vars → CLI flag override chain |
| `cross` | Data created via one config method is readable via any other |

## Coverage Added in This Iteration

- Config parsing and merging from TOML.
- Missing config file fallback to defaults.
- Invalid backend value validation.
- Env override application.
- CLI override precedence over config file backend.
- Local embedded `surrealkv://` initialization via `DB::from_config`.
- Cloud backend required-field validation guardrails.
- End-to-end shell tests for all three config methods and cross-method persistence.

## Remaining Validation Work

- End-to-end cloud integration test against a real `wss://` SurrealDB instance (gated by secure CI secrets).
- Optional UX enhancement: print active backend when verbose mode is enabled.
