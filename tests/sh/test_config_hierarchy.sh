#!/usr/bin/env bash
# Test: Verify configuration hierarchy and precedence.
#   defaults  →  global config file  →  local config file
#
# Each layer should override the previous one.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

build_binary
setup_test_db_dir
backup_config

DB="${TEST_DB_DIR}/hierarchy-test.db"

print_header "Test: Configuration Hierarchy"

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
assert_contains   "file: path overrides default"           "$OUT" 'hierarchy-test.db'

# ── 3. Config file sets cloud; fake URL should fail ───────────────
write_config "$(cat <<TOML
backend = "cloud"
url = "wss://fake.example.com"
namespace = "dunno"
database = "dunno"
username = "user"
password = "pass"
TOML
)"
run_cmd "$BIN" config show
assert_exit_ok    "cloud file: config show exits 0"        "$RC"
assert_contains   "file says cloud"                        "$OUT" '"backend":"cloud"'

run_cmd "$BIN" project ls
assert_exit_nonzero "cloud with fake URL fails"            "$RC"
assert_contains   "cloud error is structured"              "$OUT" '"status":"error"'

# ── teardown ───────────────────────────────────────────────────────
restore_config
cleanup_test_db_dir
print_summary
