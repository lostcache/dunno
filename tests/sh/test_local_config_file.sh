#!/usr/bin/env bash
# Test B: Local persistence configured entirely via ~/.config/dunno/config.toml.
# No environment variables or CLI flags are used for backend/path selection.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

build_binary
setup_test_db_dir
backup_config

DB="${TEST_DB_DIR}/config-test.db"

write_config "$(cat <<TOML
backend = "local"

[local]
path = "$DB"
TOML
)"

print_header "Test B: Config File"

# ── config show ────────────────────────────────────────────────────
run_cmd "$BIN" config show
assert_exit_ok    "config show exits 0"                "$RC"
assert_contains   "backend is local"                   "$OUT" '"backend":"local"'
assert_contains   "local path from config file"        "$OUT" 'config-test.db'

# ── 1. create project ─────────────────────────────────────────────
run_cmd "$BIN" project create "ConfigProject" "Created via config file"
assert_exit_ok    "project create exits 0"             "$RC"
assert_contains   "project has id"                     "$OUT" '"id":"project:'
assert_contains   "project has name"                   "$OUT" '"name":"ConfigProject"'
PROJECT_ID=$(json_str "$OUT" "id")

# ── 2. create module ──────────────────────────────────────────────
run_cmd "$BIN" module create "$PROJECT_ID" "ConfigModule" "Auth module"
assert_exit_ok    "module create exits 0"              "$RC"
assert_contains   "module has id"                      "$OUT" '"id":"module:'
MODULE_ID=$(json_str "$OUT" "id")

# ── 3. create task ────────────────────────────────────────────────
run_cmd "$BIN" task create "$MODULE_ID" "ConfigTask" "Login flow"
assert_exit_ok    "task create exits 0"                "$RC"
assert_contains   "task status is not_started"         "$OUT" '"status":"not_started"'
TASK_ID=$(json_str "$OUT" "id")

# ── 4. update task status ─────────────────────────────────────────
run_cmd "$BIN" task update "$TASK_ID" --status started
assert_exit_ok    "task update exits 0"                "$RC"
assert_contains   "task status is started"             "$OUT" '"status":"started"'

# ── 5. append task update ─────────────────────────────────────────
run_cmd "$BIN" task append-update "$TASK_ID" "Discovered expiry issue"
assert_exit_ok    "append-update exits 0"              "$RC"
assert_contains   "has created_at_ms"                  "$OUT" '"created_at_ms":'
UPDATE_ID=$(json_str "$OUT" "id")

# ── 6. edit task update ───────────────────────────────────────────
run_cmd "$BIN" task update-entry "$UPDATE_ID" "Expiry issue - need proactive refresh"
assert_exit_ok    "update-entry exits 0"               "$RC"
assert_contains   "has updated_at_ms"                  "$OUT" '"updated_at_ms":'

# ── 7. list task updates ──────────────────────────────────────────
run_cmd "$BIN" task list-updates "$TASK_ID"
assert_exit_ok    "list-updates exits 0"               "$RC"
assert_contains   "returns the update"                 "$OUT" 'proactive refresh'

# ── 8. create todo ────────────────────────────────────────────────
run_cmd "$BIN" todo create "$PROJECT_ID" "Write tests"
assert_exit_ok    "todo create exits 0"                "$RC"
assert_contains   "todo status is pending"             "$OUT" '"status":"pending"'

# ── 9. list todos ─────────────────────────────────────────────────
run_cmd "$BIN" todo list "$PROJECT_ID"
assert_exit_ok    "todo list exits 0"                  "$RC"
assert_contains   "todo list has item"                 "$OUT" 'Write tests'

# ── 10. add knowledge ─────────────────────────────────────────────
run_cmd "$BIN" add --category rust --type mistake -C "Config file mistake" --link-to "$TASK_ID"
assert_exit_ok    "add knowledge exits 0"              "$RC"
assert_contains   "add returns ok"                     "$OUT" '"status":"ok"'

# ── 11. retrieve context ──────────────────────────────────────────
run_cmd "$BIN" context --task-id "$TASK_ID"
assert_exit_ok    "context exits 0"                    "$RC"
assert_contains   "context contains mistake"           "$OUT" 'Config file mistake'

# ── 12. structured error ──────────────────────────────────────────
run_cmd "$BIN" context --task-id "task:nonexistent"
assert_exit_nonzero "error exits nonzero"              "$RC"
assert_contains   "error is structured JSON"           "$OUT" '"status":"error"'

# ── persistence check ─────────────────────────────────────────────
run_cmd "$BIN" project list
assert_exit_ok    "persistence: project list exits 0"  "$RC"
assert_contains   "persistence: project still exists"  "$OUT" 'ConfigProject'

# ── teardown ───────────────────────────────────────────────────────
restore_config
cleanup_test_db_dir
print_summary
