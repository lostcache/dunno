#!/usr/bin/env bash
# Test A: Local persistence configured entirely via environment variables.
# Runs the full Phase-6 verification flow using DUNNO_BACKEND + DUNNO_LOCAL_PATH.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

build_binary
setup_test_db_dir
backup_config
remove_config

DB="${TEST_DB_DIR}/env-test.db"

print_header "Test A: Environment Variables"

# ── config show ────────────────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" config show
assert_exit_ok    "config show exits 0"               "$RC"
assert_contains   "backend is local"                   "$OUT" '"backend":"local"'
assert_contains   "local path reflects env var"        "$OUT" 'env-test.db'

# ── 1. create project ─────────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" project create "EnvProject" "Created via env vars"
assert_exit_ok    "project create exits 0"             "$RC"
assert_contains   "project has id"                     "$OUT" '"id":"project:'
assert_contains   "project has name"                   "$OUT" '"name":"EnvProject"'
PROJECT_ID=$(json_str "$OUT" "id")

# ── 2. create module ──────────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" module create "$PROJECT_ID" "EnvModule" "Auth module"
assert_exit_ok    "module create exits 0"              "$RC"
assert_contains   "module has id"                      "$OUT" '"id":"module:'
assert_contains   "module has project_id"              "$OUT" "\"project_id\":\"$PROJECT_ID\""
MODULE_ID=$(json_str "$OUT" "id")

# ── 3. create task ────────────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" task create "$MODULE_ID" "EnvTask" "Login flow"
assert_exit_ok    "task create exits 0"                "$RC"
assert_contains   "task status is not_started"         "$OUT" '"status":"not_started"'
TASK_ID=$(json_str "$OUT" "id")

# ── 4. update task status ─────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" task update "$TASK_ID" --status started
assert_exit_ok    "task update exits 0"                "$RC"
assert_contains   "task status is started"             "$OUT" '"status":"started"'

# ── 5. append task update ─────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" task append-update "$TASK_ID" "OAuth tokens expire after 1h"
assert_exit_ok    "append-update exits 0"              "$RC"
assert_contains   "has content"                        "$OUT" 'OAuth tokens expire'
assert_contains   "has created_at_ms"                  "$OUT" '"created_at_ms":'
UPDATE_ID=$(json_str "$OUT" "id")

# ── 6. edit task update ───────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" task update-entry "$UPDATE_ID" "OAuth tokens expire - must refresh proactively"
assert_exit_ok    "update-entry exits 0"               "$RC"
assert_contains   "content is updated"                 "$OUT" 'must refresh proactively'
assert_contains   "has updated_at_ms"                  "$OUT" '"updated_at_ms":'

# ── 7. list task updates ──────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" task list-updates "$TASK_ID"
assert_exit_ok    "list-updates exits 0"               "$RC"
assert_contains   "list contains edited update"        "$OUT" 'must refresh proactively'

# ── 8. create todo ────────────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" todo create "$PROJECT_ID" "Set up CI pipeline"
assert_exit_ok    "todo create exits 0"                "$RC"
assert_contains   "todo status is pending"             "$OUT" '"status":"pending"'

# ── 9. list todos ─────────────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" todo list "$PROJECT_ID"
assert_exit_ok    "todo list exits 0"                  "$RC"
assert_contains   "todo list has item"                 "$OUT" 'Set up CI pipeline'

# ── 10. add knowledge linked to task ──────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" add --category rust --type mistake -C "Forgot to refresh OAuth token" --link-to "$TASK_ID"
assert_exit_ok    "add knowledge exits 0"              "$RC"
assert_contains   "add returns ok"                     "$OUT" '"status":"ok"'

# ── 11. retrieve task context ─────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" context --task-id "$TASK_ID"
assert_exit_ok    "context exits 0"                    "$RC"
assert_contains   "context has results"                "$OUT" '"results":'
assert_contains   "context contains linked mistake"    "$OUT" 'Forgot to refresh OAuth token'

# ── 12. structured error on missing task ──────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" context --task-id "task:nonexistent"
assert_exit_nonzero "error exits nonzero"              "$RC"
assert_contains   "error is structured JSON"           "$OUT" '"status":"error"'
assert_contains   "error message is meaningful"        "$OUT" 'Task not found'

# ── persistence: re-read after all writes ─────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" project list
assert_exit_ok    "persistence: project list exits 0"  "$RC"
assert_contains   "persistence: project still exists"  "$OUT" 'EnvProject'

# ── teardown ───────────────────────────────────────────────────────
restore_config
cleanup_test_db_dir
print_summary
