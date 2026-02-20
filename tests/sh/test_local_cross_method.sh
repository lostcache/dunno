#!/usr/bin/env bash
# Test E: Cross-method persistence.
# Data written via one config method should be readable via any other method
# that points to the same DB path.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

build_binary
setup_test_db_dir
backup_config
remove_config

DB="${TEST_DB_DIR}/cross-test.db"

print_header "Test E: Cross-Method Persistence"

# ── 1. Create data via ENV VARS ───────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" project create "CrossProject" "Created via env vars"
assert_exit_ok    "env: project create exits 0"            "$RC"
assert_contains   "env: project created"                   "$OUT" '"name":"CrossProject"'
PROJECT_ID=$(json_str "$OUT" "id")

run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" module create "$PROJECT_ID" "EnvModule" "Module via env"
assert_exit_ok    "env: module create exits 0"             "$RC"

# ── 2. Read data via CONFIG FILE ──────────────────────────────────
write_config "$(cat <<TOML
backend = "local"

[local]
path = "$DB"
TOML
)"

run_cmd "$BIN" project list
assert_exit_ok    "file: project list exits 0"             "$RC"
assert_contains   "file: reads env-created project"        "$OUT" 'CrossProject'

run_cmd "$BIN" module list
assert_exit_ok    "file: module list exits 0"              "$RC"
assert_contains   "file: reads env-created module"         "$OUT" 'EnvModule'

# ── 3. Create more data via CONFIG FILE ───────────────────────────
MODULE_ID=$(json_str "$OUT" "id")
run_cmd "$BIN" task create "$MODULE_ID" "ConfigTask" "Task via config"
assert_exit_ok    "file: task create exits 0"              "$RC"
assert_contains   "file: task created"                     "$OUT" '"name":"ConfigTask"'
TASK_ID=$(json_str "$OUT" "id")

run_cmd "$BIN" add --category rust --type skill -C "Cross-method skill" --link-to "$TASK_ID"
assert_exit_ok    "file: add knowledge exits 0"            "$RC"

# ── 4. Read everything via CLI FLAG ───────────────────────────────
remove_config

run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local project list
assert_exit_ok    "cli: project list exits 0"              "$RC"
assert_contains   "cli: reads env-created project"         "$OUT" 'CrossProject'

run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local module list
assert_exit_ok    "cli: module list exits 0"               "$RC"
assert_contains   "cli: reads env-created module"          "$OUT" 'EnvModule'

run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local task list
assert_exit_ok    "cli: task list exits 0"                 "$RC"
assert_contains   "cli: reads config-created task"         "$OUT" 'ConfigTask'

run_cmd env DUNNO_LOCAL_PATH="$DB" "$BIN" --backend local context --task-id "$TASK_ID"
assert_exit_ok    "cli: context exits 0"                   "$RC"
assert_contains   "cli: reads config-created knowledge"    "$OUT" 'Cross-method skill'

# ── teardown ───────────────────────────────────────────────────────
restore_config
cleanup_test_db_dir
print_summary
