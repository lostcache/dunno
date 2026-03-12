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

# ── 1. add project ─────────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" project add "EnvProject" "Created via env vars"
assert_exit_ok    "project add exits 0"             "$RC"
assert_contains   "project has id"                     "$OUT" '"id":"project:'
assert_contains   "project has name"                   "$OUT" '"name":"EnvProject"'
PROJECT_ID=$(json_str "$OUT" "id")

# ── 2. add module ──────────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" module add --project-ids "$PROJECT_ID" "EnvModule" "Auth module"
assert_exit_ok    "module add exits 0"              "$RC"
assert_contains   "module has id"                      "$OUT" '"id":"module:'
MODULE_ID=$(json_str "$OUT" "id")

# ── 3. add task ────────────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" task add --module-ids "$MODULE_ID" --project-ids "$PROJECT_ID" "EnvTask" "Login flow"
assert_exit_ok    "task add exits 0"                "$RC"
assert_contains   "task status is not_started"         "$OUT" '"status":"not_started"'
TASK_ID=$(json_str "$OUT" "id")

# ── 4. update task status ─────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" task update "$TASK_ID" --status started
assert_exit_ok    "task update exits 0"                "$RC"
assert_contains   "task status is started"             "$OUT" '"status":"started"'

# ── 8. add todo ────────────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" todo add --project-ids "$PROJECT_ID" "Set up CI pipeline"
assert_exit_ok    "todo add exits 0"                "$RC"

# ── 9. ls todos ─────────────────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" todo ls --project-id "$PROJECT_ID"
assert_exit_ok    "todo ls exits 0"                  "$RC"
assert_contains   "todo ls has item"                 "$OUT" 'Set up CI pipeline'

# ── 10. add knowledge linked to task ──────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" add --field category --value rust --field type --value mistake --field content --value "Forgot to refresh OAuth token" --link-to "$TASK_ID"
assert_exit_ok    "add knowledge exits 0"              "$RC"
assert_contains   "add returns ok"                     "$OUT" '"status":"ok"'

# ── 11. retrieve task ctx ─────────────────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" ctx --task-id "$TASK_ID"
assert_exit_ok    "ctx exits 0"                    "$RC"
assert_contains   "ctx has results"                "$OUT" '"results":'
assert_contains   "ctx contains linked mistake"    "$OUT" 'Forgot to refresh OAuth token'

# ── 12. structured error on missing task ──────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" ctx --task-id "task:nonexistent"
assert_exit_nonzero "error exits nonzero"              "$RC"
assert_contains   "error is structured JSON"           "$OUT" '"status":"error"'
assert_contains   "error message is meaningful"        "$OUT" 'Task not found'

# ── persistence: re-read after all writes ─────────────────────────
run_cmd env DUNNO_BACKEND=local DUNNO_LOCAL_PATH="$DB" "$BIN" project ls
assert_exit_ok    "persistence: project ls exits 0"  "$RC"
assert_contains   "persistence: project still exists"  "$OUT" 'EnvProject'

# ── teardown ───────────────────────────────────────────────────────
restore_config
cleanup_test_db_dir
print_summary
