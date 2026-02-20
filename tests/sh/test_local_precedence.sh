#!/usr/bin/env bash
# Test D: Verify configuration precedence order.
#   defaults  →  config file  →  env vars  →  CLI flags
#
# Each layer should override the previous one.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

build_binary
setup_test_db_dir
backup_config

DB="${TEST_DB_DIR}/precedence-test.db"

print_header "Test D: Configuration Precedence"

# ── 1. Defaults (no file, no env, no CLI) ─────────────────────────
remove_config
run_cmd "$BIN" config show
assert_exit_ok    "defaults: config show exits 0"          "$RC"
assert_contains   "defaults: backend is local"             "$OUT" '"backend":"local"'
assert_contains   "defaults: path is default"              "$OUT" '~/.local/share/dunno/data.db'

# ── 2. Config file overrides defaults ─────────────────────────────
write_config "$(cat <<TOML
backend = "local"

[local]
path = "$DB"
TOML
)"
run_cmd "$BIN" config show
assert_exit_ok    "file: config show exits 0"              "$RC"
assert_contains   "file: path overrides default"           "$OUT" 'precedence-test.db'

# ── 3. Env var overrides config file ──────────────────────────────
run_cmd env DUNNO_LOCAL_PATH="/tmp/env-override.db" "$BIN" config show
assert_exit_ok    "env: config show exits 0"               "$RC"
assert_contains   "env: path overrides file"               "$OUT" '/tmp/env-override.db'

# ── 4. Config file sets cloud; env overrides to local ─────────────
write_config "$(cat <<TOML
backend = "cloud"

[cloud]
url = "wss://fake.example.com"
namespace = "dunno"
database = "dunno"
username = "user"
password = "pass"

[local]
path = "$DB"
TOML
)"
run_cmd "$BIN" config show
assert_contains   "file says cloud"                        "$OUT" '"backend":"cloud"'

run_cmd env DUNNO_BACKEND=local "$BIN" config show
assert_exit_ok    "env overrides file: exits 0"            "$RC"
assert_contains   "env overrides file to local"            "$OUT" '"backend":"local"'

# ── 5. CLI flag overrides everything ──────────────────────────────
# file=cloud, env=cloud, CLI=local → should resolve to local
run_cmd env DUNNO_BACKEND=cloud "$BIN" --backend local config show
assert_exit_ok    "cli overrides all: exits 0"             "$RC"
assert_contains   "cli overrides all to local"             "$OUT" '"backend":"local"'

# Verify it actually connects to local and works
run_cmd env DUNNO_BACKEND=cloud "$BIN" --backend local project create "PrecedenceProject" "CLI wins"
assert_exit_ok    "cli override: project create exits 0"   "$RC"
assert_contains   "cli override: project created"          "$OUT" '"name":"PrecedenceProject"'

# ── 6. Without CLI override, cloud should fail (fake URL) ─────────
run_cmd "$BIN" project list
assert_exit_nonzero "cloud with fake URL fails"            "$RC"
assert_contains   "cloud error is structured"              "$OUT" '"status":"error"'

# ── teardown ───────────────────────────────────────────────────────
restore_config
cleanup_test_db_dir
print_summary
