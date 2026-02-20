#!/usr/bin/env bash
# Test F: Cloud backend configured entirely via environment variables.
# Runs the full Phase-6 verification flow against SurrealDB Cloud.
#
# Required env var: DUNNO_CLOUD_URL
# Namespace/database/credentials are hardcoded for the dunno cloud setup.
# Skips gracefully if DUNNO_CLOUD_URL is not set.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/helpers.sh"

require_cloud_env
build_binary
backup_config
remove_config

PREFIX="EnvCloud_${CLOUD_TS}"

cloud_run() {
    run_cmd env \
        DUNNO_BACKEND=cloud \
        DUNNO_CLOUD_URL="$DUNNO_CLOUD_URL" \
        DUNNO_CLOUD_NS=dunno \
        DUNNO_CLOUD_DB=dunno \
        DUNNO_CLOUD_USER=dunno \
        DUNNO_CLOUD_PASS=dunnodev \
        DUNNO_CLOUD_AUTH_TYPE=namespace \
        "$BIN" "$@"
}

print_header "Test F: Cloud Backend (env vars)"

# ── config show ────────────────────────────────────────────────────
cloud_run config show
assert_exit_ok    "config show exits 0"                "$RC"
assert_contains   "backend is cloud"                   "$OUT" '"backend":"cloud"'
assert_contains   "cloud URL set"                      "$OUT" "$DUNNO_CLOUD_URL"
assert_contains   "namespace is dunno"                 "$OUT" '"namespace":"dunno"'
assert_contains   "username is dunno"                  "$OUT" '"username":"dunno"'

# ── 1. create project ─────────────────────────────────────────────
cloud_run project create "${PREFIX}_Project" "Cloud project via env vars"
assert_exit_ok    "project create exits 0"             "$RC"
assert_contains   "project has id"                     "$OUT" '"id":"project:'
assert_contains   "project has name"                   "$OUT" "\"name\":\"${PREFIX}_Project\""
PROJECT_ID=$(json_str "$OUT" "id")

# ── 2. create module ──────────────────────────────────────────────
cloud_run module create "$PROJECT_ID" "${PREFIX}_Module" "Auth module"
assert_exit_ok    "module create exits 0"              "$RC"
assert_contains   "module has id"                      "$OUT" '"id":"module:'
assert_contains   "module has project_id"              "$OUT" "\"project_id\":\"$PROJECT_ID\""
MODULE_ID=$(json_str "$OUT" "id")

# ── 3. create task ────────────────────────────────────────────────
cloud_run task create "$MODULE_ID" "${PREFIX}_Task" "Login flow"
assert_exit_ok    "task create exits 0"                "$RC"
assert_contains   "task status is not_started"         "$OUT" '"status":"not_started"'
TASK_ID=$(json_str "$OUT" "id")

# ── 4. update task status ─────────────────────────────────────────
cloud_run task update "$TASK_ID" --status started
assert_exit_ok    "task update exits 0"                "$RC"
assert_contains   "task status is started"             "$OUT" '"status":"started"'

# ── 5. append task update ─────────────────────────────────────────
cloud_run task append-update "$TASK_ID" "OAuth tokens expire after 1h"
assert_exit_ok    "append-update exits 0"              "$RC"
assert_contains   "has content"                        "$OUT" 'OAuth tokens expire'
assert_contains   "has created_at_ms"                  "$OUT" '"created_at_ms":'
UPDATE_ID=$(json_str "$OUT" "id")

# ── 6. edit task update ───────────────────────────────────────────
cloud_run task update-entry "$UPDATE_ID" "OAuth tokens expire - must refresh proactively"
assert_exit_ok    "update-entry exits 0"               "$RC"
assert_contains   "content is updated"                 "$OUT" 'must refresh proactively'
assert_contains   "has updated_at_ms"                  "$OUT" '"updated_at_ms":'

# ── 7. list task updates ──────────────────────────────────────────
cloud_run task list-updates "$TASK_ID"
assert_exit_ok    "list-updates exits 0"               "$RC"
assert_contains   "list contains edited update"        "$OUT" 'must refresh proactively'

# ── 8. create todo ────────────────────────────────────────────────
cloud_run todo create "$PROJECT_ID" "${PREFIX} Set up CI pipeline"
assert_exit_ok    "todo create exits 0"                "$RC"
assert_contains   "todo status is pending"             "$OUT" '"status":"pending"'

# ── 9. list todos ─────────────────────────────────────────────────
cloud_run todo list "$PROJECT_ID"
assert_exit_ok    "todo list exits 0"                  "$RC"
assert_contains   "todo list has item"                 "$OUT" 'Set up CI pipeline'

# ── 10. add knowledge linked to task ──────────────────────────────
cloud_run add --category rust --type mistake -C "${PREFIX} Forgot to refresh OAuth token" --link-to "$TASK_ID"
assert_exit_ok    "add knowledge exits 0"              "$RC"
assert_contains   "add returns ok"                     "$OUT" '"status":"ok"'

# ── 11. retrieve task context ─────────────────────────────────────
cloud_run context --task-id "$TASK_ID"
assert_exit_ok    "context exits 0"                    "$RC"
assert_contains   "context has results"                "$OUT" '"results":'
assert_contains   "context contains linked mistake"    "$OUT" 'Forgot to refresh OAuth token'

# ── 12. structured error on missing task ──────────────────────────
cloud_run context --task-id "task:nonexistent_${CLOUD_TS}"
assert_exit_nonzero "error exits nonzero"              "$RC"
assert_contains   "error is structured JSON"           "$OUT" '"status":"error"'
assert_contains   "error message is meaningful"        "$OUT" 'Task not found'

# ── persistence: re-read after all writes ─────────────────────────
cloud_run project list
assert_exit_ok    "persistence: project list exits 0"  "$RC"
assert_contains   "persistence: project still exists"  "$OUT" "${PREFIX}_Project"

# ── teardown ───────────────────────────────────────────────────────
restore_config
print_summary
